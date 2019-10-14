use std::time::SystemTime;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::error::{ErrorKind, Result as CommonResult, ResultExt};
use crate::common::model::{ConnectionPool, Uuid as UuidType};
use crate::schema::users;

#[cfg(test)]
use mockall::automock;

#[derive(Clone, Debug, Insertable, Queryable, Serialize, Deserialize, PartialEq)]
#[table_name = "users"]
pub struct UserRecord {
    pub uuid: UuidType,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: SystemTime,
    pub last_updated_at: SystemTime,
}

#[derive(PartialEq)]
pub struct NewUserRecordParams {
    pub email: Option<String>,
    pub name: Option<String>,
}

#[cfg_attr(test, automock)]
pub trait UserORM: Send + Sync {
    /// Create and return user record in database based on given params.
    fn create(&self, user: NewUserRecordParams) -> CommonResult<UserRecord>;
    /// Find user record by email.
    /// # Errors
    /// Throws error when database error occurs.
    fn find_by_email(&self, target_email: &str) -> CommonResult<Option<UserRecord>>;
}

#[derive(Clone)]
pub struct UserModel {
    pool: ConnectionPool,
}

impl UserORM for UserModel {
    fn create(&self, user: NewUserRecordParams) -> CommonResult<UserRecord> {
        let pool = self.pool.get().context(|| {
            (
                ErrorKind::ConnectionPoolError,
                "Error when getting DB connection from pool",
            )
        })?;
        let conn = &pool;

        let now = SystemTime::now();
        let new_user = UserRecord {
            uuid: Uuid::new_v4().to_string(),
            email: user.email,
            name: user.name,
            created_at: now,
            last_updated_at: now,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .context(|| {
                (
                    ErrorKind::InsertionError,
                    "Error when inserting user into DB",
                )
            })?;

        Ok(new_user)
    }

    fn find_by_email(&self, target_email: &str) -> CommonResult<Option<UserRecord>> {
        use crate::schema::users::dsl::{email, users};

        let pool = self.pool.get().context(|| {
            (
                ErrorKind::ConnectionPoolError,
                "Error when getting DB connection from pool",
            )
        })?;
        let conn = &pool;

        users
            .filter(email.eq(target_email))
            .first::<UserRecord>(conn)
            .optional()
            .context(|| {
                (
                    ErrorKind::QueryError,
                    "Error when finding user record with email in DB",
                )
            })
    }
}

impl UserModel {
    pub fn new(pool: ConnectionPool) -> UserModel {
        UserModel { pool }
    }
}
