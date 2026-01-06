use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};

/*
* Learnings
* we need to decide to keep open or close,
* start where we keep everything open
* mostly if we get a message we also have a \n at the end
*
* Usage
* 1. Open a connection for pub or sub
* 2. Currently Server sends ping to check if the connection is open -> replace with keepalive?
* 3.1 Pub: can send payload which the server accepts
* 3.2 Sub: can receive payload from the server
*
* Todo
* use socket2 for keepalive
*
*
* Terminal usage:
* nc 127.0.0.1 8080
* {"command":"Sub","version":1,"topic":"chat.general","payload":"hello"}
* hello
*/

use HermesMQ::{self, TopicMap, client};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let active_streams: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>> = Arc::new(Mutex::new(Vec::new()));
    let channel_topic_map: Arc<Mutex<TopicMap>> = Arc::new(Mutex::new(TopicMap::new()));
    ping_listeners(Arc::clone(&active_streams));

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error from incomming tcp stream: {}", e);
                continue;
            }
        };
        let streams = active_streams.clone();
        let map = channel_topic_map.clone();
        let client_stream = match stream.try_clone() {
            Ok(s) => s,
            Err(_) => {
                continue;
            }
        };
        std::thread::spawn(move || {
            if let Err(e) = client::handle_client(client_stream, streams, map) {
                // TODO: add specific error message
                eprintln!("client handle error {e}");
                let _ = stream.write_all(b"Error occured while connecting");
            }
        });
    }
    Ok(())
}

fn ping_listeners(listener: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(4));
            let mut listener_guard = match listener.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    eprintln!("Skip ping, because error while locking listener");
                    continue;
                }
            };

            println!("Currently {} active listener", listener_guard.len());
            // iterate backwards because if we remove the first one then we get an error for the second one
            for i in (0..listener_guard.len()).rev() {
                let stream_arc = Arc::clone(&listener_guard[i]);
                let mut stream = match stream_arc.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        continue;
                    }
                };
                if stream.write_all(b"ping\n").is_err() {
                    listener_guard.remove(i);
                }
            }
        }
    });
}
