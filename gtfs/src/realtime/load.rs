use std::path::{Path, PathBuf};

use tokio::{fs::File, io::AsyncWriteExt, task::spawn_blocking};

use super::service_alerts::ServiceAlertRoot;
use crate::error::Result;

const SERVICE_ALERT_URL: &str = "https://api.opendata.metlink.org.nz/v1/gtfs-rt/servicealerts";

fn get_servicealert_cache_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join("servicealerts").join(format!(
        "{}.json",
        time::OffsetDateTime::now_utc().format(time::Format::Rfc3339)
    ))
}

pub async fn load_gtfs(cache_dir: &Path, client: &reqwest::Client) -> Result<ServiceAlertRoot> {
    let req_builder = client.get(SERVICE_ALERT_URL);
    let mut response = req_builder.send().await?.error_for_status()?;
    let cache_file = get_servicealert_cache_file(cache_dir);

    let mut writer = File::create(&cache_file).await?;
    while let Some(mut item) = response.chunk().await? {
        writer.write(&mut item).await?;
    }

    spawn_blocking(move || -> Result<ServiceAlertRoot> {
        Ok(serde_json::from_reader(std::fs::File::open(cache_file)?)?)
    })
    .await
    .unwrap()
}
