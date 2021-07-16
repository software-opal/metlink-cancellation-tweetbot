use std::path::{Path, PathBuf};

use reqwest::{Client, Request};
use tokio::fs::create_dir_all;

pub mod error;
pub mod gtfs;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let cache_dir = PathBuf::from("./.cache");
    create_dir_all(&cache_dir).await.unwrap();
    let c = gtfs::load::load_gtfs(&cache_dir, &client).await.unwrap();
    println!(
        "Hello, world: {:#?}",
        c.feed_info
    );
}
