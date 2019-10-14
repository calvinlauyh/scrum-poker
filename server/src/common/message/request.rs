use actix::prelude::*;
use serde::Deserialize;

#[derive(Message, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RequestMessage {
    CreateRoom(CreateRoomParams),
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomParams {
    pub private: bool,
    pub passphrase: Option<String>, // TODO: Use SecStr
    pub card_set: Vec<String>,
}
