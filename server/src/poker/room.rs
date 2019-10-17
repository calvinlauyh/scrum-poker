use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use actix::prelude::*;

use crate::client::channel::ClientChannel;
use crate::client::store::{ClientStore, SharedClientStore};
use crate::client::ClientId;
use crate::common::error::{ErrorKind, Result as CommonResult, ResultExt};
use crate::common::model::Uuid;
use crate::poker::game::Game;
use crate::poker::model::{Card, NewRoomRecordParams, RoomORM};

pub struct Room<R, S, T>
where
    R: RoomORM,
    S: ClientStore<T>,
    T: ClientChannel,
{
    room_id: Option<Uuid>,
    private: bool,
    passphrase: Option<String>,
    card_set: Vec<Card>,
    owner: ClientId,
    players: Vec<ClientId>,
    current_game: Option<Game>,

    room_model: R,
    client_store: SharedClientStore<S, T>,
    client_store_channel_type: PhantomData<T>,
}

impl<R, S, T> Actor for Room<R, S, T>
where
    R: RoomORM + 'static,
    S: ClientStore<T> + 'static,
    T: ClientChannel + 'static,
{
    type Context = Context<Self>;
}

impl<R, S, T> Room<R, S, T>
where
    R: RoomORM,
    S: ClientStore<T>,
    T: ClientChannel,
{
    /// Instantiate a Room object, but is not ready to be functional yet.
    /// To make the room start functioning, call create() method.
    pub fn new(
        params: NewRoomParams,
        room_model: R,
        client_store: SharedClientStore<S, T>,
    ) -> Self {
        Room {
            room_id: None,
            private: params.private,
            passphrase: params.passphrase,
            card_set: params.card_set,
            owner: params.owner,
            players: vec![params.owner],
            current_game: None,

            room_model,
            client_store,
            client_store_channel_type: PhantomData,
        }
    }

    /// Create the room and turn it into functional state. Return the created
    /// room Uuid.
    pub fn create(&mut self) -> CommonResult<Uuid> {
        let owner_uuid = self
            .client_store
            .get_readable()
            .get(&self.owner)
            .context(|| {
                (
                    ErrorKind::MissingClientError,
                    "Missing owner client when creating room",
                )
            })?
            .user_info
            .get_readable()
            .uuid
            .clone();

        let params = NewRoomRecordParams {
            private: self.private,
            passphrase: self.passphrase.clone(),
            owner_uuid,
            card_set: self.card_set.clone(),
        };
        let room_record = self.room_model.create(params)?;

        Ok(room_record.uuid)
    }
}

pub struct NewRoomParams {
    pub private: bool,
    pub passphrase: Option<String>,
    pub card_set: Vec<Card>,
    pub owner: ClientId,
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::client::channel::MockClientChannel;
    use crate::client::store::DefaultClientStore;
    use crate::client::Client;
    use crate::poker::model::MockRoomORM;

    #[test]
    fn new_should_instantiate_room_struct_with_given_params() {
        let private = false;
        let passphrase = Some(String::from("passphrase"));
        let card_set = vec![
            String::from("1"),
            String::from("3"),
            String::from("5"),
        ];
        let owner = 1;

        let params = NewRoomParams {
            private,
            passphrase: passphrase.clone(),
            card_set: card_set.clone(),
            owner,
        };
        let room_orm = MockRoomORM::new();
        let client_store = DefaultClientStore::<MockClientChannel>::default();
        let shared_client_store = SharedClientStore::new(client_store);

        let room = Room::new(params, room_orm, shared_client_store.clone());

        assert_eq!(room.private, false);
        assert_eq!(room.passphrase, passphrase);
        assert_eq!(room.card_set, card_set);
        assert_eq!(room.owner, owner);
        assert_eq!(room.players, vec![owner]);
    }
}
