// macro powers up schema.rs and models, which is much more convenient than
// explicitly specifying what to import using use diesel::{macro}.
#[macro_use]
extern crate diesel;

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

use client::channel::DefaultClientChannel;
use client::store::DefaultClientStore;
use common::error::ResultExt;
use common::error::{ErrorKind, Result};
use poker::model::RoomModel;
use poker::room::Room;
use server::message::CreateRoomMessage;
use server::Server;
use user::auth::provider::oauth::OAuthProviderImplConfig;
use user::auth::provider::{GoogleProviderImpl, GuestProviderImpl, Provider};
use user::auth::ProviderService;
use user::model::UserModel;
use websocket::Session;

pub mod client;
pub mod common;
pub mod poker;
pub mod schema;
pub mod server;
pub mod user;
pub mod websocket;

pub type AppServer =
    Server<UserModel, RoomModel, DefaultClientStore<DefaultClientChannel>, DefaultClientChannel>;
pub type AppRoom = Room<RoomModel, DefaultClientStore<DefaultClientChannel>, DefaultClientChannel>;
pub type AppSession = Session<UserModel>;

pub fn make_database_connection_pool(
    database_url: &str,
) -> Result<r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .context(|| {
            (
                ErrorKind::ConnectionPoolError,
                "Error when building connection pool",
            )
        })
}

pub fn make_auth_provider_service() -> ProviderService {
    let guest_provider = GuestProviderImpl::new();
    let google_provider = make_google_auth_provider().expect("Error creating Google auth provider");

    ProviderService::new(vec![Box::new(guest_provider), Box::new(google_provider)])
}

pub fn make_google_auth_provider() -> Result<GoogleProviderImpl> {
    let client_id = dotenv::var("GOOGLE_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| panic!("Missing GOOGLE_OAUTH_CLIENT_ID in env"));
    let client_secret = dotenv::var("GOOGLE_OAUTH_CLIENT_SECRET")
        .unwrap_or_else(|_| panic!("Missing GOOGLE_OAUTH_CLIENT_SECRET in env"));
    let redirect_uri = dotenv::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| panic!("Missing OAUTH_REDIRECT_URI in env"));

    let config = OAuthProviderImplConfig {
        client_id,
        client_secret,
        redirect_uri,
    };

    GoogleProviderImpl::build(config)
}
