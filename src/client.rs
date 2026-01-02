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
    stream: &mut TcpStream,
    active_streams: Arc<Mutex<Vec<TcpStream>>>,
    channel_topic_map: Arc<Mutex<TopicMap>>,
) -> std::io::Result<()> {
    println!("Incomming stream");
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let mut message = String::new();
    reader.read_line(&mut message)?;
    let m: Message = serde_json::from_str(&message)?;
    println!("{:?}", m);
    match m.command {
        Command::Sub => {
            let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
            {
                match channel_topic_map.lock() {
                    Ok(mut topic_map) => topic_map.entry(m.topic.clone()).or_default().push(tx),
                    Err(_) => {
                        eprintln!(
                            "Error while locking ChannelTopicMap for adding Sender. Closing connection."
                        );
                        return Ok(());
                    }
                }
            }

            println!("Adding Sub");
            match active_streams.lock() {
                Ok(mut streams) => streams.push(stream.try_clone()?),
                Err(_) => {
                    eprintln!(
                        "Error while locking ActiveStreams when adding new stream. Closing connection."
                    )
                }
            }
            serve_sub(stream, rx)?;
        }
        Command::Pub => {
            println!("Adding Pub");
            serve_pub(reader, channel_topic_map.clone(), m.topic)?;
        }
        _ => {
            stream.write_all(b"command not valid right now\n")?;
            return Ok(());
        }
    }
    Ok(())
}

fn serve_sub(stream: &mut TcpStream, receiver: Receiver<String>) -> std::io::Result<()> {
    loop {
        let res = receiver.recv();
        match res {
            Ok(msg) => stream.write_all(msg.as_bytes())?,
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
            Err(poisoned) => poisoned.into_inner(),
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
