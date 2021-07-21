use std::path::PathBuf;

use color_eyre::eyre::Result;
use tokio::{
    fs::{create_dir_all, File},
    io::AsyncReadExt,
};

use metlink_gtfs_lib::{
    client::reqwest_client_with_api_key,
    realtime::{
        self,
        service_alerts::ServiceAlertRealtimeApi,
        trip_updates::TripUpdateRealtimeApi,
        utils::{download_latest_if_needed, CachedRealtimeApi},
        vehicle_positions::VehiclePositionsRealtimeApi,
    },
};

pub async fn cron_realtime_api<T: CachedRealtimeApi>(api: T) {
    let mut interval = tokio::time::interval(api.min_fetch_frequency());
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        interval.tick().await;
        match download_latest_if_needed(&api).await {
            Ok(_) => log::info!("Updated {}.", api.name()),
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

    let service_alert_api =
        ServiceAlertRealtimeApi::new(&cache_dir, reqwest_client_with_api_key(&api_key)?);
    let trip_update_api =
        TripUpdateRealtimeApi::new(&cache_dir, reqwest_client_with_api_key(&api_key)?);
    let vehicle_position_api =
        VehiclePositionsRealtimeApi::new(&cache_dir, reqwest_client_with_api_key(&api_key)?);

    tokio::try_join!(
        tokio::spawn(cron_realtime_api(service_alert_api)),
        tokio::spawn(cron_realtime_api(trip_update_api)),
        tokio::spawn(cron_realtime_api(vehicle_position_api))
    )?;

    Ok(())
}
