use std::marker::PhantomData;

use actix::prelude::*;

use crate::client::channel::ClientChannel;
use crate::client::ClientId;
use crate::common::error::Result as CommonResult;
use crate::common::message::request::CreateRoomParams;
use crate::common::message::RequestMessage;
use crate::poker::Room;
use crate::user::info::SharedUserInfo;

use crate::client::store::ClientStore;
use crate::poker::model::RoomORM;

#[derive(Message)]
#[rtype(usize)]
pub struct ConnectMessage<T>
where
    T: ClientChannel,
{
    pub user_info: SharedUserInfo,
    pub channel: T,
}

#[derive(Message)]
#[rtype(result = "CommonResult<Addr<Room<R, S, T>>>")]
pub struct CreateRoomMessage<R, S, T>
where
    R: RoomORM + 'static,
    S: ClientStore<T> + 'static,
    T: ClientChannel + 'static,
{
    pub client_id: ClientId,
    pub room_params: CreateRoomParams,
    pub room_orm_type: PhantomData<R>,
    pub client_store_type: PhantomData<S>,
    pub client_channel_type: PhantomData<T>,
}

#[derive(Message)]
pub struct SessionRequestMessage {
    pub client_id: ClientId,
    pub req: RequestMessage,
}
