use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::common::model::Uuid;

#[derive(Clone)]
pub struct SharedUserInfo {
    user_info: Arc<RwLock<UserInfo>>,
}

impl SharedUserInfo {
    pub fn new(user_info: UserInfo) -> Self {
        SharedUserInfo {
            user_info: Arc::new(RwLock::new(user_info)),
        }
    }

    pub fn get_readable(&self) -> RwLockReadGuard<UserInfo> {
        self.user_info
            .read()
            .expect("Poison error when acquiring read lock of user_info")
    }

    pub fn get_writable(&self) -> RwLockWriteGuard<UserInfo> {
        self.user_info
            .write()
            .expect("Poison error when acquiring write lock of user_info")
    }
}

pub struct UserInfo {
    pub uuid: Uuid,
    pub name: String,
}
