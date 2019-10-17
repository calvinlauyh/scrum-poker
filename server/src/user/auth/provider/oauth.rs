use log::warn;

use crate::common::error::{Error, ErrorKind, Result as CommonResult};
use crate::user::auth::oauth::OAuthClient;
use crate::user::auth::Params;

use super::{AuthenticateResult, Provider};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait EmailService {
    fn get_email(&self, access_token: &str) -> CommonResult<String>;
}

#[cfg_attr(test, automock)]
pub trait AccessTokenService {
    fn revoke_access_token(&self, access_token: &str) -> CommonResult<()>;
}

pub struct OAuthProviderImpl<C, E, A>
where
    C: OAuthClient,
    E: EmailService,
    A: AccessTokenService,
{
    client: C,
    email_service: E,
    access_token_service: A,
}

impl<C, E, A> Provider for OAuthProviderImpl<C, E, A>
where
    C: OAuthClient,
    E: EmailService,
    A: AccessTokenService,
{
    fn provider_id(&self) -> &str {
        unreachable!()
    }
    fn authenticate(&self, params: Params) -> CommonResult<AuthenticateResult> {
        let auth_code = match params {
            Params::OAuth(oauth_code) => oauth_code.auth_code,
            _ => return Err(Error::new(ErrorKind::InvalidParams, "Missing auth_code")),
        };
        let access_token = self
            .client
            .exchange_access_token_from_code(auth_code.as_str())?;

        let get_email_result = self.email_service.get_email(access_token.as_str());

        self.access_token_service
            .revoke_access_token(access_token.as_str())
            .unwrap_or_else(|err| {
                warn!("Error when revoking access_token: {}", err);
            });

        let email = get_email_result?;

        Ok(AuthenticateResult::AuthenticatedUser(email))
    }
}

impl<C, E, A> OAuthProviderImpl<C, E, A>
where
    C: OAuthClient,
    E: EmailService,
    A: AccessTokenService,
{
    pub fn new(client: C, email_service: E, access_token_service: A) -> Self {
        OAuthProviderImpl {
            client,
            email_service,
            access_token_service,
        }
    }
}

pub struct OAuthProviderImplConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[cfg(test)]
mod authenticate_test {
    use super::*;

    use crate::user::auth;
    use crate::user::auth::oauth::MockOAuthClient;

    use mockall::predicate::*;

    #[test]
    fn should_return_invalid_params_error_when_auth_code_is_missing() {
        let mut oauth_client = MockOAuthClient::new();
        let mut email_service = MockEmailService::new();
        let mut access_token_service = MockAccessTokenService::new();
        oauth_client
            .expect_exchange_access_token_from_code()
            .return_const(Err(Error::from(ErrorKind::UnauthorizedError)));
        email_service
            .expect_get_email()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));
        access_token_service
            .expect_revoke_access_token()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));

        let oauth_provider =
            OAuthProviderImpl::new(oauth_client, email_service, access_token_service);

        let auth_params = Params::Guest;

        let auth_result = oauth_provider.authenticate(auth_params);
        assert!(auth_result.is_err());
        assert_eq!(auth_result.unwrap_err().kind(), ErrorKind::InvalidParams);
    }

    #[test]
    fn should_return_unauthroized_error_when_oauth_failed() {
        let mut oauth_client = MockOAuthClient::new();
        let mut email_service = MockEmailService::new();
        let mut access_token_service = MockAccessTokenService::new();
        oauth_client
            .expect_exchange_access_token_from_code()
            .return_const(Err(Error::from(ErrorKind::UnauthorizedError)));
        email_service
            .expect_get_email()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));
        access_token_service
            .expect_revoke_access_token()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));

        let oauth_provider =
            OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
        let auth_params = make_oauth_params("auth-code");

        let auth_result = oauth_provider.authenticate(auth_params);

        assert!(auth_result.is_err());
        assert_eq!(
            auth_result.unwrap_err().kind(),
            ErrorKind::UnauthorizedError
        );
    }

    #[test]
    fn should_exchange_access_token_with_provided_auth_code() {
        let mut oauth_client = MockOAuthClient::new();
        let mut email_service = MockEmailService::new();
        let mut access_token_service = MockAccessTokenService::new();

        let auth_code = String::from("auth-code");
        let expected_auth_code = auth_code.clone();
        oauth_client
            .expect_exchange_access_token_from_code()
            .withf(move |auth_code| auth_code == expected_auth_code)
            .once()
            .return_const(Err(Error::from(ErrorKind::UnauthorizedError)));
        email_service
            .expect_get_email()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));
        access_token_service
            .expect_revoke_access_token()
            .return_const(Err(Error::from(ErrorKind::UnknownError)));

        let oauth_provider =
            OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
        let auth_params = make_oauth_params(auth_code.as_ref());

        oauth_provider
            .authenticate(auth_params)
            .expect_err("authenticate should return errors when get email fails");
    }

    mod when_exchange_access_token_succeeded {
        use super::*;

        mod when_get_email_fails {
            use super::*;

            #[test]
            fn should_return_get_email_error() {
                let oauth_client = make_authorized_oauth_client();
                let mut email_service = MockEmailService::new();
                let mut access_token_service = MockAccessTokenService::new();

                email_service
                    .expect_get_email()
                    .return_const(Err(Error::from(ErrorKind::RemoteServerError)));
                access_token_service
                    .expect_revoke_access_token()
                    .return_const(Err(Error::from(ErrorKind::UnknownError)));

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                let auth_result = oauth_provider.authenticate(auth_params);

                assert!(auth_result.is_err());
                assert_eq!(
                    auth_result.unwrap_err().kind(),
                    ErrorKind::RemoteServerError
                );
            }

            #[test]
            fn should_revoke_access_token() {
                let expected_access_token = "access-token";
                let oauth_client =
                    make_authorized_oauth_client_with_access_token(expected_access_token);
                let mut email_service = MockEmailService::new();
                let mut access_token_service = MockAccessTokenService::new();

                email_service
                    .expect_get_email()
                    .return_const(Err(Error::from(ErrorKind::RemoteServerError)));
                access_token_service
                    .expect_revoke_access_token()
                    .withf(move |access_token| access_token == expected_access_token)
                    .once()
                    .return_const(Ok(()));

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                oauth_provider
                    .authenticate(auth_params)
                    .expect_err("authenticate should return errors when get email fails");
            }
        }

        mod when_get_email_works {
            use super::*;

            #[test]
            fn should_call_get_email_with_exchanged_access_token() {
                let exchanged_access_token = "access-token";
                let oauth_client =
                    make_authorized_oauth_client_with_access_token(exchanged_access_token);
                let mut email_service = MockEmailService::new();
                let access_token_service = make_authorized_access_token_service();

                email_service
                    .expect_get_email()
                    .withf(move |access_token| access_token == exchanged_access_token)
                    .once()
                    .return_const(Ok(String::from("calvinlauco@gmail.com")));

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                oauth_provider
                    .authenticate(auth_params)
                    .expect("authenticate should not return error");
            }

            #[test]
            fn should_ignore_revoke_access_token_error() {
                let oauth_client = make_authorized_oauth_client();
                let email_service = make_authroized_email_service();
                let mut access_token_service = MockAccessTokenService::new();

                access_token_service
                    .expect_revoke_access_token()
                    .once()
                    .return_const(Err(Error::from(ErrorKind::RemoteServerError)));

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                let auth_result = oauth_provider.authenticate(auth_params);

                assert!(auth_result.is_ok());
            }

            #[test]
            fn should_call_revoke_access_token_with_exchanged_access_token() {
                let exchanged_access_token = "access-token";
                let oauth_client =
                    make_authorized_oauth_client_with_access_token(exchanged_access_token);
                let mut email_service = MockEmailService::new();
                let mut access_token_service = MockAccessTokenService::new();

                email_service
                    .expect_get_email()
                    .return_const(Ok(String::from("calvinlauco@gmail.com")));
                access_token_service
                    .expect_revoke_access_token()
                    .withf(move |access_token| access_token == exchanged_access_token)
                    .once()
                    .return_const(Err(Error::from(ErrorKind::RemoteServerError)));

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                let auth_result = oauth_provider.authenticate(auth_params);

                assert!(auth_result.is_ok());
            }

            #[test]
            fn should_return_authenticated_user_with_email() {
                let expected_email = String::from("calvinlauco@gmail.com");
                let oauth_client = make_authorized_oauth_client();
                let email_service =
                    make_authroized_email_service_with_email(expected_email.as_str());
                let access_token_service = make_authorized_access_token_service();

                let oauth_provider =
                    OAuthProviderImpl::new(oauth_client, email_service, access_token_service);
                let auth_params = make_oauth_params("auth-code");

                let auth_result = oauth_provider.authenticate(auth_params);

                assert!(auth_result.is_ok());
                assert_eq!(
                    auth_result.unwrap(),
                    AuthenticateResult::AuthenticatedUser(expected_email)
                );
            }
        }

        fn make_authorized_oauth_client() -> MockOAuthClient {
            let access_token = String::from("access-token");
            make_authorized_oauth_client_with_access_token(access_token.as_str())
        }

        fn make_authorized_oauth_client_with_access_token(access_token: &str) -> MockOAuthClient {
            let mut oauth_client = MockOAuthClient::new();

            oauth_client
                .expect_exchange_access_token_from_code()
                .return_const(Ok(String::from(access_token)));

            oauth_client
        }

        fn make_authroized_email_service() -> MockEmailService {
            let email = String::from("calvinlauco@gmail.com");
            make_authroized_email_service_with_email(email.as_str())
        }

        fn make_authroized_email_service_with_email(email: &str) -> MockEmailService {
            let mut email_service = MockEmailService::new();

            email_service
                .expect_get_email()
                .return_const(Ok(String::from(email)));

            email_service
        }

        fn make_authorized_access_token_service() -> MockAccessTokenService {
            let mut access_token_service = MockAccessTokenService::new();

            access_token_service
                .expect_revoke_access_token()
                .return_const(Ok(()));

            access_token_service
        }
    }

    fn make_oauth_params(auth_code: &str) -> auth::Params {
        auth::Params::OAuth(auth::OAuthParams {
            auth_code: auth_code.to_owned(),
        })
    }
}
