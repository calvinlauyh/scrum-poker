use std::marker::PhantomData;

use actix::prelude::*;
use actix_web_actors::ws;
use log::{debug, info, warn};
use serde_json;

use crate::client::channel::DefaultClientChannel;
use crate::client::store::DefaultClientStore;
use crate::client::{ClientId, DEFAULT_CLIENT_ID};
use crate::common::message::request::CreateRoomParams;
use crate::common::message::{RequestMessage, ResponseMessage};
use crate::poker::model::RoomModel;
use crate::server::message::{ConnectMessage, CreateRoomMessage, SessionRequestMessage};
use crate::user::info::{SharedUserInfo, UserInfo};
use crate::user::model::UserORM;
use crate::AppRoom;
use crate::AppServer;

type AppCreateRoomMessage =
    CreateRoomMessage<RoomModel, DefaultClientStore<DefaultClientChannel>, DefaultClientChannel>;

pub struct Session<U>
where
    U: UserORM + 'static,
{
    user_model: U,
    client_id: ClientId,
    server_addr: Addr<AppServer>,
    room_addr: Option<Addr<AppRoom>>,
    user_info: SharedUserInfo,
}

impl<U> Actor for Session<U>
where
    U: UserORM,
{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("New websocket connection established");

        let addr = ctx.address();
        self.server_addr
            .send(ConnectMessage {
                user_info: self.user_info.clone(),
                socket: DefaultClientChannel::new(addr.recipient()),
            })
            .into_actor(self)
            .then(|client_id_result, actor, ctx| {
                match client_id_result {
                    Ok(client_id) => actor.client_id = client_id,
                    _ => ctx.stop(),
                };
                fut::ok(())
            })
            .wait(ctx);
    }
}

impl<U> Handler<ResponseMessage> for Session<U>
where
    U: UserORM,
{
    type Result = ();

    fn handle(&mut self, msg: ResponseMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).expect("Error when serializing message"));
    }
}

impl<U> StreamHandler<ws::Message, ws::ProtocolError> for Session<U>
where
    U: UserORM,
{
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        debug!(
            "Received {:?} from websocket session: {}",
            msg, self.client_id
        );

        match msg {
            ws::Message::Text(msg_text) => {
                match serde_json::from_str::<RequestMessage>(&msg_text) {
                    Ok(req) => self.handle_request_message(req, ctx),
                    Err(_) => {
                        warn!(
                            "Unrecognized text {} from websocket session {}",
                            &msg_text, self.client_id
                        );
                    }
                };
            }
            ws::Message::Binary(_) => warn!(
                "Unexpected Binary from websocket session {}",
                self.client_id
            ),
            ws::Message::Close(_) => {
                info!("Closing websocket session {}", self.client_id);
                ctx.stop();
            }
            ws::Message::Nop => (),
            _ => (),
        }

        // TODO: Removed debug message
        ctx.address()
            .do_send(ResponseMessage::RoomCreated(String::from("Haha")));
    }
}

impl<U> Session<U>
where
    U: UserORM,
{
    pub fn new(server_addr: Addr<AppServer>, user_model: U, user_info: UserInfo) -> Self {
        Self {
            user_model,
            client_id: DEFAULT_CLIENT_ID,
            server_addr,
            room_addr: None,
            user_info: SharedUserInfo::new(user_info),
        }
    }

    fn handle_request_message(&self, req: RequestMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match req {
            RequestMessage::CreateRoom(room_params) => self.handle_create_room(room_params, ctx),
            // TODO:
            _ => unreachable!(),
        }
    }

    fn handle_create_room(
        &self,
        room_params: CreateRoomParams,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        info!(
            "Receiver create room request from websocket session {}",
            self.client_id
        );

        self.server_addr
            .send(AppCreateRoomMessage {
                client_id: self.client_id,
                room_params,
                room_orm_type: PhantomData,
                client_store_type: PhantomData,
                client_channel_type: PhantomData,
            })
            .into_actor(self)
            .then(|handler_result, actor, ctx| {
                match handler_result {
                    Ok(room_addr_result) => match room_addr_result {
                        Ok(room_addr) => actor.room_addr = Some(room_addr),
                        _ => ctx.stop(),
                    },
                    _ => ctx.stop(),
                };
                fut::ok(())
            })
            .wait(ctx);
    }
}
