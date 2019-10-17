use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use actix::prelude::*;
use log::error;
use rand::prelude::*;

use crate::client::channel::ClientChannel;
use crate::client::store::{ClientStore, SharedClientStore};
use crate::client::{Client, ClientId};
use crate::common::error::{Error, ErrorKind, Result as CommonResult};
use crate::common::message::request::{CreateRoomParams, RequestMessage};
use crate::common::model::Uuid;
use crate::poker::model::RoomORM;
use crate::poker::room::NewRoomParams;
use crate::poker::Room;
use crate::user::info::SharedUserInfo;
use crate::user::model::UserORM;

use message::{ConnectMessage, CreateRoomMessage};

pub mod message;

pub struct Server<U, R, S, T>
where
    U: UserORM + 'static,
    R: RoomORM + Clone + 'static,
    S: ClientStore<T> + 'static,
    T: ClientChannel + 'static,
{
    user_model: U,
    room_model: R,
    client_store: SharedClientStore<S, T>,
    rooms: HashMap<Uuid, Addr<Room<R, S, T>>>,
    rng: ThreadRng,
    client_store_channel_type: PhantomData<T>,
}

impl<U, R, S, T> Actor for Server<U, R, S, T>
where
    U: UserORM,
    R: RoomORM + Clone,
    S: ClientStore<T>,
    T: ClientChannel,
{
    type Context = Context<Self>;
}

impl<U, R, S, T> Handler<ConnectMessage<T>> for Server<U, R, S, T>
where
    U: UserORM,
    R: RoomORM + Clone,
    S: ClientStore<T>,
    T: ClientChannel,
{
    type Result = usize;

    fn handle(&mut self, msg: ConnectMessage<T>, _: &mut Context<Self>) -> Self::Result {
        let client_id = self.generate_client_id();
        let client = Client {
            user_info: msg.user_info,
            channel: msg.channel,
        };
        self.client_store.get_writable().insert(client_id, client);

        client_id
    }
}

impl<U, R, S, T> Handler<CreateRoomMessage<R, S, T>> for Server<U, R, S, T>
where
    U: UserORM,
    R: RoomORM + Clone,
    S: ClientStore<T>,
    T: ClientChannel,
{
    type Result = CommonResult<Addr<Room<R, S, T>>>;

    fn handle(&mut self, msg: CreateRoomMessage<R, S, T>, ctx: &mut Context<Self>) -> Self::Result {
        let channel = match self.client_store.get_readable().get(&msg.client_id) {
            Some(channel) => channel,
            None => {
                error!("Received request from deleted session {}", msg.client_id);
                return Err(Error::from(ErrorKind::MissingClientError));
            }
        };

        self.create_room(msg.room_params, msg.client_id)
    }
}

impl<U, R, S, T> Server<U, R, S, T>
where
    U: UserORM,
    R: RoomORM + Clone,
    S: ClientStore<T>,
    T: ClientChannel,
{
    pub fn new(user_model: U, room_model: R, client_store: S) -> Server<U, R, S, T> {
        Server {
            user_model,
            room_model,
            client_store: SharedClientStore::new(client_store),
            rooms: HashMap::new(),
            rng: rand::thread_rng(),
            client_store_channel_type: PhantomData,
        }
    }

    fn generate_client_id(&mut self) -> ClientId {
        loop {
            let client_id = self.rng.gen();

            if self.is_client_id_unique(client_id) {
                break client_id;
            }
        }
    }

    fn is_client_id_unique(&self, client_id: ClientId) -> bool {
        !self.client_store.get_readable().contains_key(&client_id)
    }

    // fn find_addr_by_client_id(&self, client_id: ClientId) -> Option<Addr<Session<U, R, S, T>>> {
    //     self.client_store
    //         .get(&client_id)
    //         .map(|client| client.addr.clone())
    // }
}

impl<U, R, S, T> Server<U, R, S, T>
where
    U: UserORM,
    R: RoomORM + Clone,
    S: ClientStore<T>,
    T: ClientChannel,
{
    /// Create room actor and return created room Uuid.
    fn create_room(
        &mut self,
        params: CreateRoomParams,
        owner: ClientId,
    ) -> CommonResult<Addr<Room<R, S, T>>> {
        let params = NewRoomParams {
            private: params.private,
            passphrase: params.passphrase,
            card_set: params.card_set,
            owner,
        };
        let mut room = Room::new(params, self.room_model.clone(), self.client_store.clone());

        let room_uuid = room.create()?;
        let room_addr = room.start();

        self.rooms.insert(room_uuid, room_addr.clone());

        Ok(room_addr)
    }
}
