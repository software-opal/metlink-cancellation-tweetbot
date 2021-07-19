use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

use crate::error::Result;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn reqwest_client() -> Result<Client> {
    Ok(Client::builder().user_agent(APP_USER_AGENT).build()?)
}

pub fn reqwest_client_with_api_key(key: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert("X-Api-Key", HeaderValue::from_str(key).unwrap());

    Ok(Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()?)
}
