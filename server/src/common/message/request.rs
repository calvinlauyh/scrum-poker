use actix::prelude::*;
use serde::Deserialize;

use crate::common::model::Uuid;

#[derive(Message, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RequestMessage {
    CreateRoom(CreateRoomParams),
    JoinRoom(JoinRoomParams),
}

#[derive(Debug, Deserialize)]
pub struct CreateRoomParams {
    pub passphrase: Option<String>, // TODO: Use SecStr
    pub card_set: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomParams {
    pub room_uuid: Uuid,
    pub passphrase: Option<String>, // TODO: Use SecStr
}
