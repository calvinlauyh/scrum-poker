use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::TokenResponse;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use url::Url;

use crate::common::error::{ContextExt, ErrorKind, Result as CommonResult};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait OAuthClient {
    fn exchange_access_token_from_code(&self, code: &str) -> CommonResult<String>;
    fn revoke_access_token(&self, access_token: &str) -> CommonResult<()>;
}

#[derive(Debug)]
pub struct OAuthClientImpl {
    client: BasicClient,
}

impl OAuthClient for OAuthClientImpl {
    fn exchange_access_token_from_code(&self, code: &str) -> CommonResult<String> {
        let code = AuthorizationCode::new(code.to_owned());

        // TODO: Use SecUtf8
        self.client
            .exchange_code(code)
            .request(http_client)
            .context(|| {
                (
                    ErrorKind::UnauthorizedError,
                    "Error when exchanging access token from code",
                )
            })
            .map(|token_response| token_response.access_token().secret().to_string())
    }

    fn revoke_access_token(&self, access_token: &str) -> CommonResult<()> {
        Ok(())
    }
}

impl OAuthClientImpl {
    pub fn build(config: OAuthClientImplConfig) -> CommonResult<Self> {
        let client_id = ClientId::new(config.client_id);
        let client_secret = ClientSecret::new(config.client_secret);
        let auth_url = AuthUrl::new(Url::parse(&config.auth_url).context(|| {
            (
                ErrorKind::InvalidOAuthConfig,
                "Invalid authorization endpoint URL",
            )
        })?);
        let token_url = TokenUrl::new(
            Url::parse(&config.token_url)
                .context(|| (ErrorKind::InvalidOAuthConfig, "Invalid token endpoint URL"))?,
        );
        let redirect_uri = RedirectUrl::new(
            Url::parse(&config.redirect_uri)
                .context(|| (ErrorKind::InvalidOAuthConfig, "Invalid redirect URL"))?,
        );

        let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
            .set_redirect_url(redirect_uri);
        Ok(OAuthClientImpl { client })
    }
}

#[derive(Clone, Debug)]
pub struct OAuthClientImplConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
}

#[cfg(test)]
mod test {
    use super::*;

    use mockito;
    use mockito::{mock, Matcher};
    use url::form_urlencoded;

    mod build {
        use super::*;

        #[test]
        fn should_return_config_error_when_config_is_invalid() {
            let valid_config = OAuthClientImplConfig {
                client_id: String::from("client"),
                client_secret: String::from("secret"),
                auth_url: String::from("https://accounts.google.com/o/oauth2/v2/auth"),
                token_url: String::from("https://www.googleapis.com/oauth2/v4/token"),
                redirect_uri: String::from("http://localhost/auth/callback"),
            };

            assert!(OAuthClientImpl::build(valid_config.clone()).is_ok());

            let mut invalid_auth_url_config = valid_config.clone();
            invalid_auth_url_config.auth_url = String::from("invalid-url");
            assert_eq!(
                OAuthClientImpl::build(invalid_auth_url_config)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidOAuthConfig,
                "Invalid auth url should throw error"
            );

            let mut invalid_token_url_config = valid_config.clone();
            invalid_token_url_config.token_url = String::from("invalid-url");
            assert_eq!(
                OAuthClientImpl::build(invalid_token_url_config)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidOAuthConfig,
                "Invalid token url should throw error"
            );

            let mut invalid_redirect_uri_config = valid_config.clone();
            invalid_redirect_uri_config.redirect_uri = String::from("invalid-url");
            assert_eq!(
                OAuthClientImpl::build(invalid_redirect_uri_config)
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidOAuthConfig,
                "Invalid redirect url should throw error"
            );
        }
    }

    mod exchange_access_token_from_code {
        use super::*;

        #[test]
        fn should_call_token_url_with_exhcnage_token_params() {
            let client = make_client();

            println!(
                "{}",
                form_urlencoded::Serializer::new(String::new())
                    .append_pair("redirect_uri", "http%3A%2F%2Flocalhost%2Fauth%2Fcallback")
                    .finish()
            );

            let mocking = mock("POST", "/")
                // client id and secret as Basic Auth
                .match_header("Authorization", "Basic Y2xpZW50OnNlY3JldA==")
                .match_body(Matcher::AllOf(vec![
                    Matcher::Regex("code=auth-code".to_string()),
                    Matcher::Regex(form_urlencoded::Serializer::new(String::new()).append_pair("redirect_uri", "http://localhost/auth/callback").finish()),
                    Matcher::Regex("grant_type=authorization_code".to_string()),
                ]))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"access_token\":\"1/fFAGRNJru1FTz70BzhT3Zg\",\"expires_in\":3600,\"token_type\":\"Bearer\"}")
                .create();

            let code = "auth-code";
            client.exchange_access_token_from_code(code).unwrap();

            mocking.assert();
        }

        #[test]
        fn should_return_unauthorized_error_when_oauth_service_returns_error() {
            let client = make_client();

            let mocking = mock("POST", "/")
                .with_status(400)
                .with_header("content-type", "application/json")
                .with_body(
                    "{
\"error\": \"invalid_request\",
\"error_description\": \"client_secret is missing.\"
}",
                )
                .create();

            let code = "code";
            assert_eq!(
                client
                    .exchange_access_token_from_code(code)
                    .unwrap_err()
                    .kind(),
                ErrorKind::UnauthorizedError,
            );

            mocking.assert();
        }

        #[test]
        fn should_return_exchanged_token_on_authorized() {
            let client = make_client();

            let mocking = mock("POST", "/")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body("{\"access_token\":\"1/fFAGRNJru1FTz70BzhT3Zg\",\"expires_in\":3600,\"token_type\":\"Bearer\"}")
                .create();

            let code = "code";
            assert_eq!(
                client.exchange_access_token_from_code(code).unwrap(),
                "1/fFAGRNJru1FTz70BzhT3Zg"
            );
            mocking.assert();
        }

        fn make_client() -> OAuthClientImpl {
            let config = OAuthClientImplConfig {
                client_id: String::from("client"),
                client_secret: String::from("secret"),
                auth_url: String::from("https://accounts.google.com/o/oauth2/v2/auth"),
                token_url: String::from(&mockito::server_url()),
                redirect_uri: String::from("http://localhost/auth/callback"),
            };

            OAuthClientImpl::build(config).unwrap()
        }
    }
}
