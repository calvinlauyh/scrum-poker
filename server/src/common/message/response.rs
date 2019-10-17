use actix::prelude::Message;
use serde::Serialize;

use crate::common::model::Uuid;
use crate::poker::model::Card;

#[derive(Message, Serialize, Clone)]
pub enum ResponseMessage {
    RoomCreated(CreatedRoom),
    RoomClosed(String),
    UserJoined(String),
    Unauthorized(String),
    UserLeft(String),
    TopicUpdated(String),
    ConfigUpdated(String),
    GameStarted(String),
    CardPlayed(String),
    CardPlayFailed(String),
    GameEnded(String),
}

#[derive(Serialize, Clone)]
pub struct CreatedRoom {
    pub uuid: Uuid,
    pub private: bool,
    pub card_set: Vec<Card>,
}
