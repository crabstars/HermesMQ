use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
};

use crate::{
    TopicMap,
    message::{Command, Message},
};

pub fn handle_client(
    stream: TcpStream,
    active_streams: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>,
    channel_topic_map: Arc<Mutex<TopicMap>>,
) -> std::io::Result<()> {
    println!("Incomming stream");
    let mut reader = BufReader::new(stream);
    let mut message = String::new();
    reader.read_line(&mut message)?;
    let m: Message = serde_json::from_str(&message)?;
    println!("{:?}", m);
    match m.command {
        Command::Sub => {
            let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
            {
                let mut topic_map = match channel_topic_map.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        eprintln!("Error locking ChannelTopicMap; closing connection.");
                        return Ok(());
                    }
                };
                topic_map.entry(m.topic.clone()).or_default().push(tx);
            }

            println!("Adding Sub");

            let stream = reader.into_inner();
            let shared_stream = Arc::new(Mutex::new(stream));

            match active_streams.lock() {
                Ok(mut streams) => streams.push(Arc::clone(&shared_stream)),
                Err(_) => eprintln!(
                    "Error while locking ActiveStreams when adding new stream. Closing connection."
                ),
            }
            serve_sub(shared_stream, rx)?;
        }
        Command::Pub => {
            println!("Adding Pub");
            serve_pub(reader, channel_topic_map.clone(), m.topic)?;
        }
        _ => {
            let stream = reader.get_mut();
            stream.write_all(b"command not valid right now\n")?;
            return Ok(());
        }
    }
    Ok(())
}

fn serve_sub(stream: Arc<Mutex<TcpStream>>, receiver: Receiver<String>) -> std::io::Result<()> {
    loop {
        match receiver.recv() {
            Ok(msg) => {
                let mut guard = match stream.lock() {
                    Ok(g) => g,
                    Err(_) => {
                        eprintln!("Sub: stream mutex poisoned");
                        break;
                    }
                };
                guard.write_all(msg.as_bytes())?;
            }
            Err(_) => {
                eprintln!("Error while receiving");
                break;
            }
        };
    }
    Ok(())
}

fn serve_pub(
    mut reader: BufReader<TcpStream>,
    channel_topic_map: Arc<Mutex<TopicMap>>,
    topic: String,
) -> std::io::Result<()> {
    loop {
        let mut msg: String = String::new();
        let n = reader.read_line(&mut msg)?;
        if n == 0 {
            println!("Client disconected");
            break;
        }
        let mut map = match channel_topic_map.lock() {
            Ok(guard) => guard,
            Err(_) => {
                continue;
            }
        };

        let senders = map.entry(topic.clone()).or_default();
        for i in (0..senders.len()).rev() {
            match senders[i].send(msg.clone()) {
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("Can't send to client, removing from list. Error: {}", e);
                    senders.remove(i);
                }
            }
        }
        println!("Msg: {}", msg);
    }
    Ok(())
}
