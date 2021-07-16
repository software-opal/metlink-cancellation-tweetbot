use std::path::{Path, PathBuf};

use reqwest::{Client, Request};

pub mod error;
pub mod gtfs;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let cache_dir = PathBuf::from("./");
    println!(
        "Hello, world: {:#?}",
        gtfs::load::load_gtfs(&cache_dir, &client).await
    );
}
