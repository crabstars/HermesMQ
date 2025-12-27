mod client;
mod message;

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

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let active_streams: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    ping_listeners(Arc::clone(&active_streams));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let streams = active_streams.clone();
        std::thread::spawn(move || {
            if let Err(e) = client::handle_client(stream.try_clone().unwrap(), streams) {
                // TODO: add specific error message
                eprintln!("client handle error {e}");
                let _ = stream.write_all(b"Error occured while connecting");
            }
        });
    }
    Ok(())
}

fn ping_listeners(listener: Arc<Mutex<Vec<TcpStream>>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(4));
            let mut listener_guard = listener.lock().unwrap();

            println!("Currently {} active listener", listener_guard.len());
            // iterate backwards because if we remove the first one then we get an error for the second one
            for i in (0..listener_guard.len()).rev() {
                let res = listener_guard[i].write_all(b"ping\n");
                match res {
                    Ok(_) => continue,
                    Err(_) => {
                        listener_guard.remove(i);
                        continue;
                    }
                }
            }
        }
    });
}
