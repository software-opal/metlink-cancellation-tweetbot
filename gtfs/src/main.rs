pub mod tweeter;

use std::path::PathBuf;

use color_eyre::eyre::Result;
use log::info;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};
use crate::tweeter::tweet_service_alert;

use metlink_gtfs_lib::{client::{reqwest_client, reqwest_client_with_api_key}, realtime::load::{load_all_service_alerts, load_service_alerts}};

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
    let cache_dir = PathBuf::from("./.cache");
    create_dir_all(&cache_dir).await?;

    let keyless_client = reqwest_client()?;
    let api_key_client = reqwest_client_with_api_key(&api_key)?;

    let (db, alerts) = tokio::join!(
        metlink_gtfs_lib::gtfs::load_gtfs(&cache_dir, &keyless_client),
        load_all_service_alerts(&cache_dir, &api_key_client)
    );
    let db = db?;
    let alerts = alerts?;
    alerts
        .into_iter()
        .filter_map(|alert| tweet_service_alert(&alert, &db))
        .for_each(|tweet| info!("{}", tweet));

    Ok(())
}
