#[derive(serde::Deserialize, Debug)]
pub struct Message {
    pub command: Command,
    pub version: usize,
    pub topic: String,
    pub payload: Option<String>,
}

#[derive(serde::Deserialize, PartialEq, Debug)]
pub enum Command {
    Pub,
    Sub,
    Ack,
}
