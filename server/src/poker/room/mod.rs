use std::collections::HashSet;
use std::marker::PhantomData;

use actix::prelude::*;
use log::{error, warn};

use crate::client::channel::ClientChannel;
use crate::client::store::{ClientStore, SharedClientStore};
use crate::client::ClientId;
use crate::common::error::{ContextExt, Error, ErrorKind, Result as CommonResult};
use crate::common::message::request::JoinRoomParams;
use crate::common::message::response::CreatedRoom;
use crate::common::message::{RequestMessage, ResponseMessage};
use crate::common::model::Uuid;
use crate::poker::game::Game;
use crate::poker::model::{Card, NewRoomRecordParams, RoomORM};

use message::ClientRequestMessage;

pub mod message;

pub struct Room<R, S, T>
where
    R: RoomORM,
    S: ClientStore<T>,
    T: ClientChannel,
{
    room_id: Option<Uuid>,
    passphrase: Option<String>,
    card_set: Vec<Card>,
    owner_client_id: ClientId,
    players: HashSet<ClientId>,
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

impl<R, S, T> Handler<ClientRequestMessage> for Room<R, S, T>
where
    R: RoomORM + 'static,
    S: ClientStore<T> + 'static,
    T: ClientChannel + 'static,
{
    type Result = CommonResult<()>;

    fn handle(&mut self, msg: ClientRequestMessage, ctx: &mut Context<Self>) -> Self::Result {
        // TODO: Check for client existence
        let channel = match self.client_store.get_readable().get(&msg.client_id) {
            Some(channel) => channel,
            None => {
                error!(
                    "Received request from deleted websocket client {}",
                    msg.client_id
                );
                return Err(Error::from(ErrorKind::MissingClientError));
            }
        };

        self.handle_request_message(msg)?;

        Ok(())
    }
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
        let mut room = Room {
            room_id: None,
            passphrase: params.passphrase,
            card_set: params.card_set,
            owner_client_id: params.owner_client_id,
            players: HashSet::new(),
            current_game: None,

            room_model,
            client_store,
            client_store_channel_type: PhantomData,
        };

        room.players.insert(params.owner_client_id);

        room
    }

    pub fn is_private(&self) -> bool {
        self.passphrase.is_some()
    }

    pub fn handle_request_message(&self, req: ClientRequestMessage) -> CommonResult<()> {
        unreachable!()
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
            passphrase: self.passphrase.clone(),
            owner_uuid,
            card_set: self.card_set.clone(),
        };
        let room_record = self.room_model.create(params)?;

        let created_room = CreatedRoom {
            private: self.is_private(),
            uuid: room_record.uuid.clone(),
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

    // TODO: Use SecStr for passphrase
    fn join(&mut self, joiner_client_id: ClientId, passphrase: Option<String>) -> CommonResult<()> {
        if self.players.contains(&joiner_client_id) {
            return Err(Error::from(ErrorKind::AlreadyJoinedError));
        }

        if self.is_private() {
            if self.passphrase != passphrase {
                return Err(Error::from(ErrorKind::UnauthenticatedError));
            }
        }

        self.players.insert(joiner_client_id);

        let client_store = self.client_store.get_readable();
        for client_id in self.players.iter() {
            if let Some(client) = client_store.get(client_id) {
                client
                    .channel
                    .do_send(ResponseMessage::UserJoined(String::from("Test")))
                    .unwrap_or_else(|err| {
                        warn!(
                            "Error when sending UserJoined response to room player websocket client {}: {}",
                            client_id, err
                        )
                    });
            } else {
                error!(
                    "Trying to notify user joined to deleted websocket client {}",
                    client_id
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NewRoomParams {
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

    type MockRoom = Room<MockRoomORM, MockDefaultClientStore, MockClientChannel>;
    type MockSharedClientStore = SharedClientStore<MockDefaultClientStore, MockClientChannel>;
    type MockDefaultClientStore = DefaultClientStore<MockClientChannel>;
    type MockClient = Client<MockClientChannel>;

    const DEFAULT_OWNER_CLIENT_ID: usize = 1;

    #[test]
    fn new_should_instantiate_room_struct_with_given_params() {
        let (passphrase, card_set, owner_client_id, params) = default_new_room_params();

        let room_model = MockRoomORM::new();
        let client_store = DefaultClientStore::<MockClientChannel>::default();
        let shared_client_store = SharedClientStore::new(client_store);

        let room = Room::new(params, room_model, shared_client_store);

        let mut expected_players = HashSet::new();
        expected_players.insert(owner_client_id);
        assert_eq!(room.passphrase, passphrase);
        assert_eq!(room.card_set, card_set);
        assert_eq!(room.owner_client_id, owner_client_id);
        assert_eq!(room.players, expected_players);
    }

    mod is_private {
        use super::*;

        #[test]
        fn should_return_true_when_room_is_private() {
            let passphrase = Some(String::from("Passphrase"));
            let room = make_room(passphrase);

            assert!(room.is_private());
        }

        #[test]
        fn should_return_false_when_room_is_public() {
            let passphrase = None;
            let room = make_room(passphrase);

            assert!(!room.is_private());
        }
    }

    mod create {
        use super::*;

        #[test]
        fn should_return_error_when_database_persistence_has_error() {
            let (_passphrase, _card_set, owner_client_id, params) = default_new_room_params();

            let (shared_user_info, _owner_uuid, _owner_name) = default_shared_user_info();
            let shared_client_store = make_shared_client_store(vec![(
                owner_client_id,
                shared_user_info,
                default_mock_client_channel(),
            )]);
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
            let (_passphrase, _card_set, owner_client_id, params) = default_new_room_params();

            let (shared_user_info, owner_uuid, _owner_name) = default_shared_user_info();
            let shared_client_store = make_shared_client_store(vec![(
                owner_client_id,
                shared_user_info,
                default_mock_client_channel(),
            )]);
            let (room_model, _created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert!(room.create().is_ok());
        }

        #[test]
        fn should_return_created_room_uuid() {
            let (_passphrase, _card_set, owner_client_id, params) = default_new_room_params();

            let (shared_user_info, owner_uuid, _owner_name) = default_shared_user_info();
            let shared_client_store = make_shared_client_store(vec![(
                owner_client_id,
                shared_user_info,
                default_mock_client_channel(),
            )]);
            let (room_model, created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert_eq!(room.create().unwrap(), created_room_uuid);
        }

        #[test]
        fn should_send_room_created_response_to_room_owner() {
            let (_passphrase, _card_set, owner_client_id, params) = default_new_room_params();

            let mut owner_client_channel = MockClientChannel::new();
            owner_client_channel
                .expect_do_send()
                .withf(|res| match res {
                    ResponseMessage::RoomCreated(_) => true,
                    _ => false,
                })
                .once()
                .return_const(Ok(()));
            let (shared_user_info, owner_uuid, _owner_name) = default_shared_user_info();
            let shared_client_store = make_shared_client_store(vec![(
                owner_client_id,
                shared_user_info,
                owner_client_channel,
            )]);
            let (room_model, _created_room_uuid) =
                make_succeeded_create_room_model(params.clone(), owner_uuid);

            let mut room = Room::new(params, room_model, shared_client_store);

            assert!(room.create().is_ok());
        }
    }

    mod join {
        use super::*;

        #[test]
        fn should_return_already_joined_error_when_already_joined() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let join_result = room.join(DEFAULT_OWNER_CLIENT_ID, passphrase);
            assert!(join_result.is_err());
            assert_eq!(
                join_result.unwrap_err().kind(),
                ErrorKind::AlreadyJoinedError
            );
        }

        #[test]
        fn should_return_ok_when_joined_successfully() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;
            let (joiner_user_info, _joiner_uuid, _joiner_name) = default_shared_user_info();
            let joiner_client_channel = default_mock_client_channel();
            {
                let mut client_store = room.client_store.get_writable();
                client_store.insert(
                    joiner_client_id,
                    make_mock_client(joiner_user_info, joiner_client_channel),
                );
            }

            assert!(room.join(joiner_client_id, passphrase).is_ok());
        }

        #[test]
        fn should_insert_client_id_into_players() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

            assert!(room.join(joiner_client_id, passphrase).is_ok());

            assert_eq!(room.players.len(), 2);
            assert!(room.players.contains(&DEFAULT_OWNER_CLIENT_ID));
            assert!(room.players.contains(&joiner_client_id));
        }

        mod given_room_is_public {
            use super::*;

            #[test]
            fn should_return_ok_when_passphrase_is_provided() {
                let passphrase = None;
                let mut room = make_room(passphrase.clone());

                let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

                let passphrase = Some(String::from("Passphrase"));
                assert!(room.join(joiner_client_id, passphrase).is_ok());
            }
        }

        mod given_room_is_private {
            use super::*;

            #[test]
            fn should_return_unauthenticated_error_when_passphrase_is_incorrect() {
                let passphrase = Some(String::from("Passphrase"));
                let mut room = make_room(passphrase.clone());

                let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

                let passphrase = Some(String::from("Incorrect"));
                let join_result = room.join(joiner_client_id, passphrase);
                assert!(join_result.is_err());
                assert_eq!(
                    join_result.unwrap_err().kind(),
                    ErrorKind::UnauthenticatedError
                );
            }

            #[test]
            fn should_return_unauthenticated_error_when_passphrase_is_not_provided() {
                let passphrase = Some(String::from("Passphrase"));
                let mut room = make_room(passphrase.clone());

                let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

                let passphrase = None;
                let join_result = room.join(joiner_client_id, passphrase);
                assert!(join_result.is_err());
                assert_eq!(
                    join_result.unwrap_err().kind(),
                    ErrorKind::UnauthenticatedError
                );
            }
        }

        fn should_return_error_when_data_persistence_has_error() {}
        fn should_persist_new_player_to_database() {}

        #[test]
        fn should_return_ok_when_client_does_not_exist_in_client_store() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

            assert!(room.join(joiner_client_id, passphrase).is_ok());
        }

        #[test]
        fn should_send_user_joined_message_to_all_players() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;
            let (joiner_user_info, _joiner_uuid, _joiner_name) = default_shared_user_info();
            let joiner_client_channel = default_mock_client_channel();

            {
                let mut client_store = room.client_store.get_writable();
                client_store.insert(
                    joiner_client_id,
                    make_mock_client(joiner_user_info, joiner_client_channel),
                );
            }
            {
                let mut client_store = room.client_store.get_writable();
                let owner_client_channel = &mut client_store
                    .get_mut(&DEFAULT_OWNER_CLIENT_ID)
                    .unwrap()
                    .channel;
                owner_client_channel.checkpoint();

                owner_client_channel
                    .expect_do_send()
                    .withf(|res| match res {
                        ResponseMessage::UserJoined(_) => true,
                        _ => false,
                    })
                    .once()
                    .return_const(Ok(()));
            }
            {
                let mut client_store = room.client_store.get_writable();
                let joiner_client_channel =
                    &mut client_store.get_mut(&joiner_client_id).unwrap().channel;
                joiner_client_channel.checkpoint();

                joiner_client_channel
                    .expect_do_send()
                    .withf(|res| match res {
                        ResponseMessage::UserJoined(_) => true,
                        _ => false,
                    })
                    .once()
                    .return_const(Ok(()));
            }

            assert!(room.join(joiner_client_id, passphrase).is_ok());
        }

        #[test]
        fn should_return_ok_when_passphrase_is_provided() {
            let passphrase = None;
            let mut room = make_room(passphrase.clone());

            let joiner_client_id = DEFAULT_OWNER_CLIENT_ID + 1;

            let passphrase = Some(String::from("Passphrase"));
            assert!(room.join(joiner_client_id, passphrase).is_ok());
        }
    }

    fn make_room(passphrase: Option<String>) -> MockRoom {
        let owner_client_id = DEFAULT_OWNER_CLIENT_ID;
        let new_room_params = make_new_room_params(passphrase, default_card_set(), owner_client_id);
        let (shared_user_info, owner_uuid, _owner_name) = default_shared_user_info();
        let owner_client_channel = default_mock_client_channel();
        let shared_client_store = make_shared_client_store(vec![(
            owner_client_id,
            shared_user_info,
            owner_client_channel,
        )]);
        let (room_model, _created_room_uuid) =
            make_succeeded_create_room_model(new_room_params.clone(), owner_uuid);

        let mut room = Room::new(new_room_params, room_model, shared_client_store);
        room.create().expect("Room should be created");

        room
    }

    fn default_new_room_params() -> (Option<String>, Vec<Card>, ClientId, NewRoomParams) {
        let passphrase = Some(String::from("passphrase"));
        let card_set = default_card_set();
        let owner_client_id = DEFAULT_OWNER_CLIENT_ID;

        let params = NewRoomParams {
            passphrase: passphrase.clone(),
            card_set: card_set.clone(),
            owner_client_id,
        };

        (passphrase, card_set, owner_client_id, params)
    }

    fn default_card_set() -> Vec<Card> {
        vec![String::from("1"), String::from("3"), String::from("5")]
    }

    fn make_new_room_params(
        passphrase: Option<String>,
        card_set: Vec<Card>,
        owner_client_id: ClientId,
    ) -> NewRoomParams {
        NewRoomParams {
            passphrase: passphrase,
            card_set: card_set,
            owner_client_id,
        }
    }

    fn make_shared_client_store(
        clients: Vec<(ClientId, SharedUserInfo, MockClientChannel)>,
    ) -> MockSharedClientStore {
        let shared_client_store = default_shared_client_store();
        {
            let mut client_store = shared_client_store.get_writable();
            for client in clients {
                client_store.insert(client.0, make_mock_client(client.1, client.2));
            }
        }

        shared_client_store
    }

    fn default_shared_client_store(
    ) -> SharedClientStore<DefaultClientStore<MockClientChannel>, MockClientChannel> {
        SharedClientStore::new(DefaultClientStore::<MockClientChannel>::default())
    }

    fn make_mock_client(
        user_info: SharedUserInfo,
        client_channel: MockClientChannel,
    ) -> MockClient {
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

    fn default_mock_client_channel() -> MockClientChannel {
        let mut client_channel = MockClientChannel::new();
        client_channel.expect_do_send().return_const(Ok(()));
        client_channel.expect_try_send().return_const(Ok(()));

        client_channel
    }

    fn make_succeeded_create_room_model(
        new_room_params: NewRoomParams,
        owner_uuid: UuidType,
    ) -> (
        MockRoomORM,
        UuidType, // Room Uuid
    ) {
        let mut room_model = MockRoomORM::new();
        let new_room_uuid = Uuid::new_v4().to_string();
        let created_room_uuid = new_room_uuid.clone();
        room_model
            .expect_create()
            .withf(move |params| {
                return params.passphrase == new_room_params.passphrase
                    && params.card_set == new_room_params.card_set
                    && params.owner_uuid == owner_uuid;
            })
            .once()
            .returning(move |params| {
                let now = SystemTime::now();
                let room_record = RoomRecord {
                    uuid: new_room_uuid.clone(),
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
