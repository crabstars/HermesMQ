use std::{
    io::{Read, Write},
    net::TcpListener,
    usize,
};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let mut message = String::new();
    let mut buffer = [0; 10];

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_nonblocking(true)?;
        let mut stop = false;
        loop {
            let read_count = match stream.read(&mut buffer) {
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    stop = true;
                    0
                }
                Err(_) => panic!("weird error"),
            };
            println!("{:?}", &read_count);
            if stop {
                println!("finished reading");
                println!("{}", message);
                break;
            }

            let buffer_str = str::from_utf8(&buffer[..read_count]);
            match buffer_str {
                Ok(val) => message.push_str(val),
                Err(_) => stream.write_all(b"Could not read message")?,
            }
        }
    }
    Ok(())
}
