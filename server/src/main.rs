use actix::prelude::*;
use actix_web::{web, App, HttpServer};

use scrum_poker::client::channel::DefaultClientChannel;
use scrum_poker::client::store::DefaultClientStore;
use scrum_poker::common::error::{ContextExt, ErrorKind, Result as CommonResult};
use scrum_poker::poker::model::RoomModel;
use scrum_poker::server::Server;
use scrum_poker::user::model::UserModel;
use scrum_poker::user::route::login_user;
// TODO: Deteled after test
use scrum_poker::user::route::exchange as UserExchangeRoute;
use scrum_poker::websocket::route::websocket_route;
use scrum_poker::{make_auth_provider_service, make_database_connection_pool};

fn main() -> CommonResult<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let database_url = dotenv::var("DATABASE_URL").context(|| {
        (
            ErrorKind::MissingDatabaseUrl,
            "Missing DATABASE_URL in environment",
        )
    })?;
    let server_host = dotenv::var("SERVER_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let server_port = dotenv::var("SERVER_PORT").unwrap_or_else(|_| String::from("80"));

    let auth_provider_service = make_auth_provider_service();
    let auth_provider_service_data = web::Data::new(auth_provider_service);
    let pool = make_database_connection_pool(&database_url)?;
    let user_model = UserModel::new(pool.clone());
    let room_model = RoomModel::new(pool.clone());
    let client_store = DefaultClientStore::<DefaultClientChannel>::default();

    let sys = System::new("ScrumPoker");

    let server = Server::new(user_model.clone(), room_model.clone(), client_store).start();

    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .data(user_model.clone())
            .data(room_model.clone())
            .register_data(auth_provider_service_data.clone())
            .service(web::resource("/ws/").to(websocket_route))
            .service(web::resource("/user/login").to_async(login_user::<UserModel>))
            .service(web::resource("/user/exchange").to_async(UserExchangeRoute))
    })
    .bind(format!("{}:{}", server_host, server_port))
    .context(|| {
        (
            ErrorKind::ServerListenError,
            "Error binding HTTP server host and port",
        )
    })?
    .start();

    sys.run()
        .context(|| (ErrorKind::ActixRuntimeError, "Error starting Actix runtime"))
}
