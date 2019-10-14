use crate::user::auth;

use crate::common::error::Result as CommonResult;

use super::{AuthenticateResult, Provider};

pub struct GuestProviderImpl {}

impl Provider for GuestProviderImpl {
    fn provider_id(&self) -> &str {
        "Guest"
    }
    fn authenticate(&self, _: auth::Params) -> CommonResult<AuthenticateResult> {
        Ok(AuthenticateResult::GuestUser)
    }
}

impl GuestProviderImpl {
    pub fn new() -> Self {
        GuestProviderImpl {}
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::user::auth;

    #[test]
    fn authenticate_should_always_return_guest_user() {
        let provider = GuestProviderImpl::new();

        let params = auth::Params::Guest;

        assert_eq!(
            provider.authenticate(params).unwrap(),
            AuthenticateResult::GuestUser
        );
    }
}
