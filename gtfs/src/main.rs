use std::{io::Read, path::PathBuf};

use gtfs::load_gtfs;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};

pub mod db;
pub mod error;
pub mod gtfs;
pub mod realtime;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() -> self::error::Result<()> {
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

    load_gtfs(&cache_dir, &client).await?;

    Ok(())
}
