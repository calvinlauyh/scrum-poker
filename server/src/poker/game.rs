use std::collections::HashMap;

use crate::client::ClientId;
use crate::poker::model::Card;

pub struct Game {
    pub title: String,
    pub description: Option<String>,
    pub players_hands: HashMap<ClientId, Card>,
}
