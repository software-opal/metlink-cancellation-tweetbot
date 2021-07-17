use std::path::PathBuf;

use color_eyre::eyre::Result;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};

use crate::realtime::load::load_service_alerts;

pub mod db;
pub mod error;
pub mod gtfs;
pub mod realtime;
pub mod utils;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let api_key = {
        let mut buf = String::new();
        File::open("./metlink_api_key.txt")
            .await?
            .read_to_string(&mut buf)
            .await?;
        buf.trim().to_owned()
    };
    let mut headers = HeaderMap::new();
    headers.insert("X-Api-Key", HeaderValue::from_str(&api_key).unwrap());

    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()?;
    let cache_dir = PathBuf::from("./.cache");
    create_dir_all(&cache_dir).await?;

    let db = crate::gtfs::load_gtfs(&cache_dir, &client).await?;
    load_service_alerts(&cache_dir, &client).await?;

    Ok(())
}
