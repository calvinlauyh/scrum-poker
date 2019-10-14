use std::time::SystemTime;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::error::{ErrorKind, Result as CommonResult, ResultExt};
use crate::common::model::{ConnectionPool, Uuid as UuidType};
use crate::schema::rooms;
use crate::user::model::UserRecord;

pub type Card = String;

#[derive(Insertable, Queryable, Associations, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[belongs_to(UserRecord, foreign_key = "owner_uuid")]
#[table_name = "rooms"]
pub struct RoomRecord {
    pub uuid: UuidType,
    pub private: bool,
    pub passphrase: Option<String>,
    pub card_set: Vec<Card>,
    pub owner_uuid: UuidType,
    pub created_at: SystemTime,
    pub last_updated_at: SystemTime,
}

#[derive(PartialEq)]
pub struct NewRoomRecordParams {
    pub private: bool,
    pub passphrase: Option<String>,
    pub owner_uuid: UuidType,
    pub card_set: Vec<Card>,
}

pub trait RoomORM: Send + Sync + Clone {
    /// Create and return room record in database based on given params.
    fn create(&self, room: NewRoomRecordParams) -> CommonResult<RoomRecord>;
}

#[derive(Clone)]
pub struct RoomModel {
    pool: ConnectionPool,
}

impl RoomORM for RoomModel {
    fn create(&self, room: NewRoomRecordParams) -> CommonResult<RoomRecord> {
        let pool = self.pool.get().context(|| {
            (
                ErrorKind::ConnectionPoolError,
                "Error when getting DB connection from pool",
            )
        })?;
        let conn = &pool;

        let now = SystemTime::now();
        let new_room = RoomRecord {
            uuid: Uuid::new_v4().to_string(),
            private: room.private,
            passphrase: room.passphrase,
            card_set: room.card_set,
            owner_uuid: room.owner_uuid,
            created_at: now,
            last_updated_at: now,
        };
        diesel::insert_into(rooms::table)
            .values(&new_room)
            .execute(conn)
            .context(|| {
                (
                    ErrorKind::InsertionError,
                    "Error when inserting room record into DB",
                )
            })?;

        Ok(new_room)
    }
}

impl RoomModel {
    pub fn new(pool: ConnectionPool) -> RoomModel {
        RoomModel { pool }
    }
}
