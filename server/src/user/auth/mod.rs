use serde::{Deserialize, Serialize};

pub mod oauth;
pub mod provider;
pub mod provider_service;

pub use provider_service::ProviderService;

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Params {
    OAuth(OAuthParams),
    Guest,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct OAuthParams {
    pub auth_code: String,
}
