use actix::prelude::Message;
use serde::Serialize;

#[derive(Message, Serialize, Clone)]
pub enum ResponseMessage {
    RoomCreated(String),
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
