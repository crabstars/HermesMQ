use HermesMQ::{TopicMap, handle_client};
use std::{
    io::{BufRead, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[test]
fn handle_client_rejects_ack_command_in_integration_test() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to ephemeral port");
    let addr = listener.local_addr().expect("listener has no address");

    let server = thread::spawn({
        let active_streams = Arc::new(Mutex::new(Vec::new()));
        let channel_topic_map = Arc::new(Mutex::new(TopicMap::new()));
        move || {
            let (mut stream, _) = listener.accept().expect("failed to accept connection");
            handle_client(&mut stream, active_streams, channel_topic_map)
                .expect("handle_client should not error for Ack");
        }
    });

    let mut client =
        TcpStream::connect(addr).expect("client failed to connect to server under test");
    client
        .write_all(br#"{"command":"Ack","version":1,"topic":"test","payload":null}"#)
        .expect("failed to send ack command");
    client.write_all(b"\n").expect("failed to send delimiter");

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .expect("failed to read response");
    assert_eq!(response, "command not valid right now\n");

    drop(client);
    server.join().expect("server thread panicked");
}

#[test]
fn handle_client_pub_and_sub_to_server_test() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to ephemeral port");
    let addr = listener.local_addr().expect("listener has no address");
    let active_streams = Arc::new(Mutex::new(Vec::new()));
    let channel_topic_map = Arc::new(Mutex::new(TopicMap::new()));

    let server = thread::spawn({
        let active_streams = active_streams.clone();
        let channel_topic_map = channel_topic_map.clone();
        move || {
            let (mut stream, _) = listener.accept().expect("failed to accept connection");
            handle_client(&mut stream, active_streams, channel_topic_map)
                .expect("handle_client should not error for Ack");
        }
    });

    let mut client_sub = TcpStream::connect(addr).expect("client failed to connect to server");
    client_sub
        .write_all(br#"{"command":"Sub","version":1,"topic":"chat.general","payload":"hello"}"#)
        .expect("Failed Sub command");
    client_sub
        .write_all(b"\n")
        .expect("failed to send delimiter for sub command");

    thread::sleep(Duration::from_millis(100));
    let mut client_pub = TcpStream::connect(addr).expect("client failed to connect to server");
    client_pub
        .write_all(
            b"{\"command\":\"Pub\",\"version\":1,\"topic\":\"chat.general\",\"payload\":\"hello\"}",
        )
        .expect("Failed Pub command");
    client_pub
        .write_all(b"\n")
        .expect("failed to send delimiter for pub command");
    client_pub
        .write_all(b"hello\n")
        .expect("failed to send message");

    let mut response = String::new();
    let mut reader = std::io::BufReader::new(client_sub.try_clone().expect("clone sub"));

    reader
        .read_line(&mut response)
        .expect("failed to read response");
    assert!(response.contains("hello"));

    drop(client_pub);
    drop(client_sub);
    server.join().expect("server thread panicked")
}
