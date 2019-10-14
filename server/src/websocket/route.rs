use actix::prelude::*;
use actix_web::{web, Error as AWError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::user::info::UserInfo;
use crate::user::model::UserModel;
use crate::AppServer;

use super::Session;

pub fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    user_model: web::Data<UserModel>,
    server: web::Data<Addr<AppServer>>,
) -> std::result::Result<HttpResponse, AWError> {
    // TODO: Authenticate user
    let user_uuid = String::from("0123-4567-8901-2345");
    let user_name = String::from("Calvin Lau");
    let user_info = UserInfo {
        uuid: user_uuid,
        name: user_name,
    };
    ws::start(
        Session::new(
            server.get_ref().clone(),
            user_model.get_ref().clone(),
            user_info,
        ),
        &req,
        stream,
    )
}
