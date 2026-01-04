use std::collections::HashMap;

pub mod client;
pub mod message;

pub type TopicMap = HashMap<String, Vec<std::sync::mpsc::Sender<String>>>;

pub use client::handle_client;
