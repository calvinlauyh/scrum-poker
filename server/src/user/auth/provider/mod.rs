#[cfg(test)]
use mockall::automock;

use crate::common::error::Result as CommonResult;
use crate::user::auth;

pub mod google;
pub mod guest;
pub mod oauth;

pub use google::GoogleProviderImpl;
pub use guest::GuestProviderImpl;
pub use oauth::OAuthProviderImpl;

#[cfg_attr(test, automock)]
pub trait Provider {
    /// Get provider unique id
    fn provider_id(&self) -> &str;
    /// Authenticate with the given state and return authenticated user with
    /// email or guest user.
    fn authenticate(&self, params: auth::Params) -> CommonResult<AuthenticateResult>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticateResult {
    AuthenticatedUser(String),
    GuestUser,
}
