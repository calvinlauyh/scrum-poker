use actix_web::{error::BlockingError, web, Error as ActixWebError, HttpRequest, HttpResponse};
use futures::Future;
use serde::{Deserialize, Serialize};

use crate::common::error::{Error, ErrorKind};

use super::auth;
use super::auth::provider::AuthenticateResult;
use super::auth::provider_service::ProviderService;
use super::model::{NewUserRecordParams, UserORM};

use oauth2::basic::BasicClient;
// Alternatively, this can be oauth2::curl::http_client or a custom.
use oauth2::reqwest::http_client;
use oauth2::TokenResponse;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use url::Url;

pub fn exchange(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = ActixWebError> {
    let google_client_id =
        ClientId::new(String::from(dotenv::var("GOOGLE_OAUTH_CLIENT_ID").unwrap()));
    let google_client_secret =
        ClientSecret::new(dotenv::var("GOOGLE_OAUTH_CLIENT_SECRET").unwrap());
    let auth_url = AuthUrl::new(
        Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .expect("Invalid authorization endpoint URL"),
    );
    let token_url = TokenUrl::new(
        Url::parse("https://www.googleapis.com/oauth2/v4/token")
            .expect("Invalid token endpoint URL"),
    );

    let code = AuthorizationCode::new(String::from(dotenv::var("GOOGLE_OAUTH_SECRET").unwrap()));

    web::block(move || {
        // Set up the config for the Google OAuth2 process.
        let client = BasicClient::new(
            google_client_id,
            Some(google_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_url(RedirectUrl::new(
            Url::parse("http://localhost:4200/auth/callback").expect("Invalid redirect URL"),
        ));

        client.exchange_code(code).request(http_client)
    })
    .from_err()
    .and_then(|token| {
        println!("Token: {:?}", token.access_token().secret().to_string());
        HttpResponse::Ok().content_type("text/html").body("Hello!")
    })
}

pub fn login_user<U>(
    req: web::Json<LoginUserReq>,
    user: web::Data<U>,
    auth_provider_service: web::Data<ProviderService>,
) -> impl Future<Item = HttpResponse, Error = ActixWebError>
where
    U: UserORM + Send + Sync + 'static,
{
    web::block(move || {
        let provider = match auth_provider_service.get(&req.provider_id) {
            None => {
                return Err(Error::from(ErrorKind::UnsupportedProviderError));
            }
            Some(provider) => provider,
        };

        let maybe_email = provider
            .authenticate(req.params.clone())
            .and_then(|result| {
                let maybe_email = match result {
                    AuthenticateResult::AuthenticatedUser(email) => Some(email),
                    AuthenticateResult::GuestUser => None,
                };
                Ok(maybe_email)
            })?;

        let maybe_user_record = match maybe_email.clone() {
            Some(email) => user.find_by_email(&email)?,
            None => None,
        };

        match maybe_user_record {
            Some(user_record) => Ok(user_record),
            None => {
                let new_user_params = NewUserRecordParams {
                    email: maybe_email,
                    name: None,
                };
                user.create(new_user_params)
            }
        }
    })
    .then(|user_record_result| match user_record_result {
        Ok(user_record) => Ok(HttpResponse::Ok().json(user_record)),
        Err(err) => {
            println!("{}", err);
            match err {
                BlockingError::Canceled => Ok(HttpResponse::InternalServerError().into()),
                BlockingError::Error(login_err) => match login_err.kind() {
                    ErrorKind::UnauthorizedError => Ok(HttpResponse::Unauthorized().into()),
                    _ => Ok(HttpResponse::InternalServerError().into()),
                },
            }
        }
    })
    // .map_err(|err: Error| match err.kind() {
    //     ErrorKind::UnauthorizedError => Ok(HttpResponse::Unauthorized().into()),
    //     _ => Ok(HttpResponse::InternalServerError().into()),
    // })

    // web::block(move || {
    //     let provider = match auth_provider_service.get(&req.provider_id) {
    //         None => {
    //             return Err(ActixWebError::from(
    //                 HttpResponse::BadRequest().body("Unsupported provider"),
    //             ));
    //         }
    //         Some(provider) => provider,
    //     };

    //     let maybe_email = match provider.authenticate(req.params) {
    //         Err(err) => match err.kind() {
    //             ErrorKind::UnauthorizedError => {
    //                 return Err(ActixWebError::from(HttpResponse::Unauthorized().into()))
    //             }
    //             _ => {
    //                 return Err(ActixWebError::from(
    //                     HttpResponse::InternalServerError().into(),
    //                 ))
    //             }
    //         },
    //         Ok(authenticate_result) => match authenticate_result {
    //             AuthenticateResult::AuthenticatedUser(email) => Some(email),
    //             AuthenticateResult::GuestUser => None,
    //         },
    //     };

    //     Ok(maybe_email)
    // })
    // .from_err()
    // .and_then(|maybe_email| {
    //     let maybe_user_record = match maybe_email {
    //         Some(email) => match user.find_by_email(&email) {
    //             Err(err) => {
    //                 return Err(ActixWebError::from(
    //                     HttpResponse::InternalServerError().into(),
    //                 ))
    //             }
    //             Ok(user_record) => user_record,
    //         },
    //         None => None,
    //     };

    //     Ok((maybe_user_record, maybe_email))
    // })
    // .and_then(|(maybe_user_record, maybe_email)| {
    //     let user_record_result = match maybe_user_record {
    //         Some(user_record) => Ok(user_record),
    //         None => {
    //             let new_user_params = NewUserRecordParams {
    //                 uuid: Uuid::new_v4().to_string(),
    //                 email: maybe_email,
    //                 name: None,
    //             };
    //             user.create(new_user_params)
    //         }
    //     };

    //     match user_record_result {
    //         Ok(user_record) => Ok(HttpResponse::Ok().json(user_record)),
    //         Err(err) => {
    //             println!("{}", err.to_string());
    //             Ok(HttpResponse::InternalServerError().into())
    //         }
    //     }
    // })
}

#[derive(Deserialize, Serialize)]
pub struct LoginUserReq {
    provider_id: String,
    params: auth::Params,
}

#[cfg(test)]
mod test {
    use super::*;

    use std::time::SystemTime;

    use actix_web::test::TestRequest;
    use actix_web::{http, test, App};
    use uuid::Uuid;

    use crate::error::{Error, ErrorKind};
    use crate::user::auth::provider::{MockProvider, Provider};
    use crate::user::model::{MockUserORM, UserRecord};

    mod login_user_req {}

    mod login_user {
        use super::*;

        fn should_return_bad_request_when_provider_does_not_exist() {}

        mod user_record_does_not_exist {
            use super::*;

            #[test]
            fn should_create_user_without_email_in_database_when_login_as_quest() {
                let req = make_guest_login_request();

                let mut user = MockUserORM::new();
                let now = SystemTime::now();
                user.expect_find_by_email().return_const(Ok(None));
                user.expect_create()
                    .withf(|user| user.email.is_none())
                    .once()
                    .returning(move |params| {
                        Ok(UserRecord {
                            uuid: Uuid::new_v4().to_string(),
                            email: None,
                            name: params.name,
                            created_at: now,
                            last_updated_at: now,
                        })
                    });

                let guest_provider_service = make_successful_guest_provider_service();
                let provider_service_data =
                    make_provider_service_web_data(vec![guest_provider_service]);

                let resp =
                    test::block_on(login_user(req, web::Data::new(user), provider_service_data))
                        .unwrap();
                assert_eq!(resp.status(), http::StatusCode::OK);
            }

            #[test]
            fn should_create_user_in_database_when_login_with_oauth() {
                let req = make_oauth_login_request();

                let mut user = MockUserORM::new();
                let now = SystemTime::now();
                user.expect_find_by_email().return_const(Ok(None));
                user.expect_create().once().returning(move |params| {
                    Ok(UserRecord {
                        uuid: Uuid::new_v4().to_string(),
                        email: params.email,
                        name: params.name,
                        created_at: now,
                        last_updated_at: now,
                    })
                });

                let oauth_provider_service =
                    make_successful_oauth_provider_service(String::from("calvinlauco@gmail.com"));
                let provider_service_data =
                    make_provider_service_web_data(vec![oauth_provider_service]);

                let resp =
                    test::block_on(login_user(req, web::Data::new(user), provider_service_data))
                        .unwrap();
                assert_eq!(resp.status(), http::StatusCode::OK);
            }

            #[test]
            fn should_return_created_user_in_response() {
                let mut user = MockUserORM::new();

                let now = SystemTime::now();
                let user_record = UserRecord {
                    uuid: Uuid::new_v4().to_string(),
                    email: Some(String::from("calvinlauco@gmail.com")),
                    name: Some(String::from("Calvin Lau")),
                    created_at: now,
                    last_updated_at: now,
                };
                user.expect_find_by_email().return_const(Ok(None));
                user.expect_create().return_const(Ok(user_record.clone()));

                let provider_service_data = make_successful_provider_service_web_data();

                let mut app = test::init_service(
                    App::new()
                        .data(user)
                        .register_data(provider_service_data)
                        .route("/", web::post().to_async(login_user::<MockUserORM>)),
                );
                let req = TestRequest::post()
                    .uri("/")
                    .set_json(&make_oauth_login_user_req())
                    .to_request();

                let user_record_resp: UserRecord = test::read_response_json(&mut app, req);

                assert_eq!(user_record_resp, user_record.clone());
            }
        }

        mod when_user_record_already_exists {
            use super::*;
            #[test]
            fn should_not_create_user() {
                let req = make_oauth_login_request();

                let email = String::from("calvinlauco@gmail.com");

                let mut user = MockUserORM::new();
                let now = SystemTime::now();
                let user_record = UserRecord {
                    uuid: Uuid::new_v4().to_string(),
                    email: Some(email.clone()),
                    name: None,
                    created_at: now,
                    last_updated_at: now,
                };
                user.expect_find_by_email()
                    .return_const(Ok(Some(user_record)));
                user.expect_create()
                    .never()
                    .return_const(Err(Error::from(ErrorKind::InsertionError)));

                let oauth_provider_service = make_successful_oauth_provider_service(email);
                let provider_service_data =
                    make_provider_service_web_data(vec![oauth_provider_service]);

                let resp =
                    test::block_on(login_user(req, web::Data::new(user), provider_service_data))
                        .unwrap();

                assert_eq!(resp.status(), http::StatusCode::OK);
            }

            #[test]
            fn should_return_user_record_in_database() {
                let mut user = MockUserORM::new();
                let email = String::from("calvinlauco@gmail.com");
                let now = SystemTime::now();
                let user_record = UserRecord {
                    uuid: Uuid::new_v4().to_string(),
                    email: Some(email.clone()),
                    name: Some(String::from("Calvin Lau")),
                    created_at: now,
                    last_updated_at: now,
                };
                user.expect_find_by_email()
                    .return_const(Ok(Some(user_record.clone())));
                user.expect_create()
                    .never()
                    .return_const(Err(Error::from(ErrorKind::InsertionError)));

                let oauth_provider_service = make_successful_oauth_provider_service(email);
                let provider_service_data =
                    make_provider_service_web_data(vec![oauth_provider_service]);

                let mut app = test::init_service(
                    App::new()
                        .data(user)
                        .register_data(provider_service_data)
                        .route("/", web::post().to_async(login_user::<MockUserORM>)),
                );
                let req = TestRequest::post()
                    .uri("/")
                    .set_json(&make_oauth_login_user_req())
                    .to_request();

                let user_record_resp: UserRecord = test::read_response_json(&mut app, req);

                assert_eq!(user_record_resp, user_record.clone());
            }
        }
    }

    fn make_guest_login_request() -> web::Json<LoginUserReq> {
        web::Json(LoginUserReq {
            provider_id: String::from("Guest"),
            params: auth::Params::Guest,
        })
    }

    fn make_oauth_login_request() -> web::Json<LoginUserReq> {
        web::Json(make_oauth_login_user_req())
    }

    fn make_oauth_login_user_req() -> LoginUserReq {
        LoginUserReq {
            provider_id: String::from("OAuth"),
            params: auth::Params::OAuth(auth::OAuthParams {
                auth_code: String::from("access-code"),
            }),
        }
    }

    fn make_successful_oauth_provider_service(email: String) -> MockProvider {
        let mut oauth_provider = make_oauth_provider_service();
        oauth_provider
            .expect_authenticate()
            .return_const(Ok(AuthenticateResult::AuthenticatedUser(email)));

        oauth_provider
    }

    fn make_oauth_provider_service() -> MockProvider {
        let mut oauth_provider = MockProvider::new();
        oauth_provider
            .expect_provider_id()
            .return_const(String::from("OAuth"));

        oauth_provider
    }

    fn make_successful_guest_provider_service() -> MockProvider {
        let mut oauth_provider = make_guest_provider_service();
        oauth_provider
            .expect_authenticate()
            .return_const(Ok(AuthenticateResult::GuestUser));

        oauth_provider
    }

    fn make_guest_provider_service() -> MockProvider {
        let mut oauth_provider = MockProvider::new();
        oauth_provider
            .expect_provider_id()
            .return_const(String::from("Guest"));

        oauth_provider
    }

    fn make_provider_service_web_data(services: Vec<MockProvider>) -> web::Data<ProviderService> {
        let mut boxed_services: Vec<Box<dyn Provider + Sync + Send>> = Vec::new();
        for service in services {
            boxed_services.push(Box::new(service));
        }

        web::Data::new(ProviderService::new(boxed_services))
    }

    fn make_successful_provider_service_web_data() -> web::Data<ProviderService> {
        let mut oauth_provider =
            make_successful_oauth_provider_service(String::from("calvinlauco@gmail.com"));
        let mut guest_provider = make_successful_guest_provider_service();

        make_provider_service_web_data(vec![oauth_provider, guest_provider])
    }
}
