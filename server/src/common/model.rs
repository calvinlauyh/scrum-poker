use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

pub type Uuid = String;

pub type ConnectionPool = r2d2::Pool<ConnectionManager<PgConnection>>;
