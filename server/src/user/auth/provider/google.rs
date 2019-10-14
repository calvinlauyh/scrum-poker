#[cfg(test)]
use mockito;
use reqwest;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::common::error::{Error, ErrorKind, Result as CommonResult, ResultExt};
use crate::user::auth;
use crate::user::auth::oauth::{OAuthClientImpl, OAuthClientImplConfig};

use super::oauth::{AccessTokenService, EmailService, OAuthProviderImpl, OAuthProviderImplConfig};
use super::{AuthenticateResult, Provider};

pub struct GoogleProviderImpl {
    oauth_provider: GoogleOAuthProvider,
}
type GoogleOAuthProvider =
    OAuthProviderImpl<OAuthClientImpl, GoogleEmailServiceImpl, GoogleAccessTokenServiceImpl>;

impl Provider for GoogleProviderImpl {
    fn provider_id(&self) -> &str {
        "Google"
    }
    fn authenticate(&self, params: auth::Params) -> CommonResult<AuthenticateResult> {
        self.oauth_provider.authenticate(params)
    }
}

impl GoogleProviderImpl {
    pub fn build(config: OAuthProviderImplConfig) -> CommonResult<Self> {
        let config = OAuthClientImplConfig {
            client_id: config.client_id,
            client_secret: config.client_secret,
            auth_url: String::from("https://accounts.google.com/o/oauth2/v2/auth"),
            token_url: String::from("https://www.googleapis.com/oauth2/v4/token"),
            redirect_uri: config.redirect_uri,
        };
        let client = OAuthClientImpl::build(config)?;
        let email_service = GoogleEmailServiceImpl::new();
        let access_token_service = GoogleAccessTokenServiceImpl::new();

        let oauth_provider = OAuthProviderImpl::new(client, email_service, access_token_service);

        Ok(Self { oauth_provider })
    }
}

pub struct GoogleEmailServiceImpl {
    client: reqwest::Client,
    url: String,
}

impl EmailService for GoogleEmailServiceImpl {
    fn get_email(&self, access_token: &str) -> CommonResult<String> {
        let mut res = self
            .client
            .post(self.url.as_str())
            .bearer_auth(access_token)
            .send()
            .context(|| {
                (
                    ErrorKind::RemoteServerError,
                    "Error when retrieving email from Google",
                )
            })?;

        let res_data = match res.status() {
            StatusCode::OK => res.json::<GetEmailResponse>(),
            StatusCode::UNAUTHORIZED => return Err(Error::from(ErrorKind::UnauthorizedError)),
            status_code => {
                return Err(Error::new(
                    ErrorKind::RemoteServerError,
                    status_code.as_str(),
                ))
            }
        };

        let res_data = res_data.context(|| {
            (
                ErrorKind::DeserializationError,
                "Error when parsing email response from Google",
            )
        })?;
        if !res_data.email_verified {
            return Err(Error::new(
                ErrorKind::UnauthorizedError,
                "Email not verified",
            ));
        }
        Ok(res_data.email)
    }
}

#[derive(Deserialize)]
struct GetEmailResponse {
    email: String,
    email_verified: bool,
}

impl GoogleEmailServiceImpl {
    pub fn new() -> Self {
        let client = reqwest::Client::new();

        #[cfg(not(test))]
        let url = String::from("https://openidconnect.googleapis.com/v1/userinfo");
        #[cfg(test)]
        let url = mockito::server_url();

        GoogleEmailServiceImpl { client, url }
    }
}

pub struct GoogleAccessTokenServiceImpl {
    client: reqwest::Client,
    url: String,
}

impl AccessTokenService for GoogleAccessTokenServiceImpl {
    fn revoke_access_token(&self, access_token: &str) -> CommonResult<()> {
        let revoke_result = self
            .client
            .get(&self.url)
            .query(&[("token", access_token)])
            .send()
            .context(|| {
                (
                    ErrorKind::RemoteServerError,
                    "Error when revoking access token from Google",
                )
            })?;

        let status = revoke_result.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::BadRequest, status.to_string()))
        }
    }
}

impl GoogleAccessTokenServiceImpl {
    pub fn new() -> Self {
        let client = reqwest::Client::new();

        #[cfg(not(test))]
        let url = String::from("https://accounts.google.com/o/oauth2/revoke");
        #[cfg(test)]
        let url = mockito::server_url();

        GoogleAccessTokenServiceImpl { client, url }
    }
}

#[cfg(test)]
mod google_email_service_impl_test {
    use super::*;
    use mockito;

    #[test]
    fn should_return_unauthorized_error_when_credential_is_invalid() {
        let service = GoogleEmailServiceImpl::new();

        let _mock = mockito::mock("POST", "/")
            .with_status(401)
            .with_body(
                "{\"error\": \"invalid_request\",\"error_description\": \"Invalid Credentials\"}",
            )
            .create();

        let get_email_result = service.get_email("access-token");
        assert!(get_email_result.is_err());
        assert_eq!(
            get_email_result.unwrap_err().kind(),
            ErrorKind::UnauthorizedError
        );
    }

    #[test]
    fn should_return_remote_server_error_when_non_unauthorized_error() {
        let service = GoogleEmailServiceImpl::new();

        let access_token = "access-token";
        let _mock = mockito::mock("POST", "/").with_status(500).create();

        let get_email_result = service.get_email(access_token);
        assert!(get_email_result.is_err());
        assert_eq!(
            get_email_result.unwrap_err(),
            Error::new(ErrorKind::RemoteServerError, "500")
        );
    }

    #[test]
    fn should_return_deserialization_error_when_response_body_cannot_be_deserialized() {
        let service = GoogleEmailServiceImpl::new();

        let _mock = mockito::mock("POST", "/")
            .with_status(200)
            .with_body("Non-json body")
            .create();

        let get_email_result = service.get_email("access-token");
        assert!(get_email_result.is_err());
        assert_eq!(
            get_email_result.unwrap_err().kind(),
            ErrorKind::DeserializationError
        );
    }

    #[test]
    fn should_call_google_user_info_api_with_access_token() {
        let service = GoogleEmailServiceImpl::new();

        let access_token = "access-token";
        let email = "calvinlauco@gmail.com";
        let bearer_token = format!("Bearer {}", access_token);
        let body = format!("{{\"sub\": \"104398762816795277085\",\"picture\":\"photo.jpg\",\"email\":\"{}\",\"email_verified\":true}}", email);
        let _mock = mockito::mock("POST", "/")
            .match_header("Authorization", bearer_token.as_str())
            .with_status(200)
            .with_body(body.as_str())
            .create();

        assert!(service.get_email(access_token).is_ok());
    }

    #[test]
    fn should_return_user_email() {
        let service = GoogleEmailServiceImpl::new();

        let access_token = "access-token";
        let email = "calvinlauco@gmail.com";
        let bearer_token = format!("Bearer {}", access_token);
        let body = format!("{{\"sub\": \"104398762816795277085\",\"picture\":\"photo.jpg\",\"email\":\"{}\",\"email_verified\":true}}", email);
        let _mock = mockito::mock("POST", "/")
            .match_header("Authorization", bearer_token.as_str())
            .with_status(200)
            .with_body(body.as_str())
            .create();

        assert_eq!(service.get_email(access_token).unwrap(), email);
    }
}

#[cfg(test)]
mod google_access_token_service_impl_test {
    use super::*;
    use mockito;
    use mockito::Matcher;

    #[test]
    fn should_return_bad_request_when_revoke_request_has_error() {
        let service = GoogleAccessTokenServiceImpl::new();
        let _mock = mockito::mock("GET", "/")
            .with_status(400)
            .with_body("{\"error\": \"invalid_token\"}")
            .create();

        let result = service.revoke_access_token("access-token");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::BadRequest);
    }

    #[test]
    fn should_call_google_revoke_access_token_api_with_provided_access_token() {
        let access_token = "access-token";
        let service = GoogleAccessTokenServiceImpl::new();
        let mock = mockito::mock("GET", "/")
            .match_query(Matcher::UrlEncoded("token".into(), access_token.into()))
            .with_status(200)
            .with_body("{\"error\": \"invalid_token\"}")
            .expect(1)
            .create();

        service
            .revoke_access_token(access_token)
            .expect("revoke_access_token should not return error");

        mock.assert();
    }

    #[test]
    fn should_return_ok_when_revoke_request_succeed() {
        let service = GoogleAccessTokenServiceImpl::new();
        let _mock = mockito::mock("GET", "/").with_status(200).create();

        let result = service.revoke_access_token("access-token");

        assert!(result.is_ok());
    }
}
