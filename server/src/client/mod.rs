use crate::user::info::SharedUserInfo;

use channel::ClientChannel;

pub mod channel;
pub mod store;

pub type ClientId = usize;

// TODO: Use Option instead of a special value 0 (Clean Code)
pub const DEFAULT_CLIENT_ID: ClientId = 0;

pub struct Client<T>
where
    T: ClientChannel,
{
    pub user_info: SharedUserInfo,
    pub socket: T,
}
