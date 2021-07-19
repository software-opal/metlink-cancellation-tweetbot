
use std::path::PathBuf;

use color_eyre::eyre::Result;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};

use metlink_gtfs_lib::{client::reqwest_client_with_api_key, realtime::{self, load::get_service_alerts_if_outdated}};


pub async fn cron_service_alerts(api_key: String, cache_dir: PathBuf) -> Result<()> {
    let mut interval = tokio::time::interval(realtime::load::MIN_FETCH_FREQUENCY);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let client = reqwest_client_with_api_key(&api_key)?;

    loop {
        interval.tick().await;
        match get_service_alerts_if_outdated(&cache_dir, &client).await {
            Ok(_) => log::info!("Service alerts updated."),
            Err(e) => log::error!("Service alerts update failed: {:?}", e),
        }
    }
}


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

   let (service_alerts, ) = tokio::try_join!(tokio::spawn(cron_service_alerts(api_key, cache_dir)))?;
   service_alerts?;

    Ok(())
}