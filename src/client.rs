use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::message::{Command, Message};

pub fn handle_client(
    mut stream: TcpStream,
    active_streams: Arc<Mutex<Vec<TcpStream>>>,
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
            active_streams.lock().unwrap().push(stream.try_clone()?);
            serve_sub(reader)?;
        }
        Command::Pub => {
            serve_pub(reader)?;
        }
        _ => {
            stream.write_all(b"command not valid right now\n")?;
            return Ok(());
        }
    }
    Ok(())
}

// TODO: probably use channels for sending
fn serve_sub(mut reader: BufReader<TcpStream>) -> std::io::Result<()> {
    loop {
        let mut new_msg: String = String::new();
        let n = reader.read_line(&mut new_msg)?;
        if n == 0 {
            println!("Client disconected");
            break;
        }
        println!("Msg: {}", new_msg);
    }
    Ok(())
}

fn serve_pub(mut reader: BufReader<TcpStream>) -> std::io::Result<()> {
    loop {
        let mut new_msg: String = String::new();
        let n = reader.read_line(&mut new_msg)?;
        if n == 0 {
            println!("Client disconected");
            break;
        }
        println!("Msg: {}", new_msg);
    }
    Ok(())
}
