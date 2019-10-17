use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use actix::prelude::*;
use log::warn;

use crate::client::channel::ClientChannel;
use crate::client::store::{ClientStore, SharedClientStore};
use crate::client::ClientId;
use crate::common::error::{ErrorKind, Result as CommonResult, ResultExt};
use crate::common::message::response::CreatedRoom;
use crate::common::message::ResponseMessage;
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
    owner_client_id: ClientId,
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
            owner_client_id: params.owner_client_id,
            players: vec![params.owner_client_id],
            current_game: None,

            room_model,
            client_store,
            client_store_channel_type: PhantomData,
        }
    }

    /// Create the room and turn it into functional state. Return the created
    /// room Uuid.
    pub fn create(&mut self) -> CommonResult<Uuid> {
        let client_store = self.client_store.get_readable();
        let owner_client = client_store.get(&self.owner_client_id).context(|| {
            (
                ErrorKind::MissingClientError,
                "Missing owner client when creating room",
            )
        })?;
        let owner_uuid = owner_client.user_info.get_readable().uuid.clone();

        let params = NewRoomRecordParams {
            private: self.private,
            passphrase: self.passphrase.clone(),
            owner_uuid,
            card_set: self.card_set.clone(),
        };
        let room_record = self.room_model.create(params)?;

        let created_room = CreatedRoom {
            uuid: room_record.uuid.clone(),
            private: room_record.private,
            card_set: room_record.card_set.clone(),
        };
        owner_client
            .channel
            .do_send(ResponseMessage::RoomCreated(created_room))
            .unwrap_or_else(|err| {
                warn!(
                    "Error when sending RoomCreated response to room owner websocket client {}: {}",
                    self.owner_client_id, err
                )
            });

        Ok(room_record.uuid)
    }
}

#[derive(Clone)]
pub struct NewRoomParams {
    pub private: bool,
    pub passphrase: Option<String>,
    pub card_set: Vec<Card>,
    pub owner_client_id: ClientId,
}

#[cfg(test)]
mod test {
    use super::*;

    use std::time::SystemTime;

    use uuid::Uuid;

    use crate::client::channel::MockClientChannel;
    use crate::client::store::DefaultClientStore;
    use crate::client::Client;
    use crate::common::error::Error;
    use crate::common::model::Uuid as UuidType;
    use crate::poker::model::{MockRoomORM, RoomRecord};
    use crate::user::info::{SharedUserInfo, UserInfo};

    #[test]
    fn new_should_instantiate_room_struct_with_given_params() {
        let (private, passphrase, card_set, owner_client_id, params) = default_new_room_params();

        let room_model = MockRoomORM::new();
        let client_store = DefaultClientStore::<MockClientChannel>::default();
        let shared_client_store = SharedClientStore::new(client_store);

        let room = Room::new(params, room_model, shared_client_store);

        assert_eq!(room.private, private);
        assert_eq!(room.passphrase, passphrase);
        assert_eq!(room.card_set, card_set);
        assert_eq!(room.owner_client_id, owner_client_id);
        assert_eq!(room.players, vec![owner_client_id]);
    }

    mod create {
        use super::*;

        #[test]
        fn should_return_error_when_database_persistence_has_error() {
            let (_private, _passphrase, _card_set, owner_client_id, params) =
                default_new_room_params();

            let (shared_client_store, _owner_uuid, _owner_name) =
                make_shared_client_store(owner_client_id, default_mock_client_channel());
            let mut room_model = MockRoomORM::new();
            room_model
                .expect_create()
                .once()
                .return_const(Err(Error::from(ErrorKind::ConnectionPoolError)));

            let mut room = Room::new(params, room_model, shared_client_store);

            let err = room.create().unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ConnectionPoolError);
        }

        #[test]
        fn should_persist_the_room_into_database() {
            let (_private, _passphrase, _card_set, owner_client_id, params) =
                default_new_room_params();

            let (shared_client_store, owner_uuid, _owner_name) =
                make_shared_client_store(owner_client_id, default_mock_client_channel());
            let (room_model, _created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert!(room.create().is_ok());
        }

        #[test]
        fn should_return_created_room_uuid() {
            let (_private, _passphrase, _card_set, owner_client_id, params) =
                default_new_room_params();

            let (shared_client_store, owner_uuid, _owner_name) =
                make_shared_client_store(owner_client_id, default_mock_client_channel());
            let (room_model, created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert_eq!(room.create().unwrap(), created_room_uuid);
        }

        #[test]
        fn should_send_room_created_response_to_room_owner() {
            let (_private, _passphrase, _card_set, owner_client_id, params) =
                default_new_room_params();

            let mut owner_client_channel = MockClientChannel::new();
            owner_client_channel
                .expect_do_send()
                .once()
                .withf(|res| match res {
                    ResponseMessage::RoomCreated(_) => true,
                    _ => false,
                })
                .return_const(Ok(()));
            let (shared_client_store, owner_uuid, _owner_name) =
                make_shared_client_store(owner_client_id, owner_client_channel);
            let (room_model, _created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert!(room.create().is_ok());
        }
    }

    fn default_new_room_params() -> (bool, Option<String>, Vec<String>, ClientId, NewRoomParams) {
        let private = false;
        let passphrase = Some(String::from("passphrase"));
        let card_set = vec![String::from("1"), String::from("3"), String::from("5")];
        let owner_client_id = 1;

        let params = NewRoomParams {
            private,
            passphrase: passphrase.clone(),
            card_set: card_set.clone(),
            owner_client_id,
        };

        (private, passphrase, card_set, owner_client_id, params)
    }

    fn make_shared_client_store(
        owner_client_id: ClientId,
        mock_client_channel: MockClientChannel,
    ) -> (
        SharedClientStore<DefaultClientStore<MockClientChannel>, MockClientChannel>,
        UuidType, // User Uuid
        String,   // User name
    ) {
        let mut client_store = DefaultClientStore::<MockClientChannel>::default();
        let (shared_user_info, user_uuid, user_name) = default_shared_user_info();
        client_store.insert(
            owner_client_id,
            make_mock_client(shared_user_info, mock_client_channel),
        );

        (SharedClientStore::new(client_store), user_uuid, user_name)
    }

    fn make_mock_client(
        user_info: SharedUserInfo,
        client_channel: MockClientChannel,
    ) -> Client<MockClientChannel> {
        Client {
            user_info,
            channel: client_channel,
        }
    }

    fn default_shared_user_info() -> (SharedUserInfo, UuidType, String) {
        let uuid = Uuid::new_v4().to_string();
        let name = String::from("Calvin Lau");

        let shared_user_info = make_shared_user_info(uuid.clone(), name.clone());

        (shared_user_info, uuid, name)
    }

    fn make_shared_user_info(uuid: UuidType, name: String) -> SharedUserInfo {
        SharedUserInfo::new(UserInfo { uuid, name })
    }

    fn default_mock_client_channel() -> (MockClientChannel) {
        let mut client_channel = MockClientChannel::new();
        client_channel.expect_do_send().return_const(Ok(()));
        client_channel.expect_try_send().return_const(Ok(()));

        client_channel
    }

    fn make_succeeded_create_room_model(
        new_room_params: NewRoomParams,
        owner_uuid: UuidType,
    ) -> (MockRoomORM, UuidType) {
        let mut room_model = MockRoomORM::new();
        let new_room_uuid = Uuid::new_v4().to_string();
        let created_room_uuid = new_room_uuid.clone();
        room_model
            .expect_create()
            .withf(move |params| {
                params.private == new_room_params.private
                    && params.passphrase == new_room_params.passphrase
                    && params.card_set == new_room_params.card_set
                    && params.owner_uuid == owner_uuid
            })
            .once()
            .returning(move |params| {
                let now = SystemTime::now();
                let room_record = RoomRecord {
                    uuid: new_room_uuid.clone(),
                    private: params.private,
                    passphrase: params.passphrase,
                    card_set: params.card_set,
                    owner_uuid: params.owner_uuid,
                    created_at: now,
                    last_updated_at: now,
                };
                Ok(room_record)
            });

        (room_model, created_room_uuid)
    }
}
