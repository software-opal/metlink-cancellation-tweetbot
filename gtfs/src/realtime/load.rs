use std::path::{Path, PathBuf};

use log::info;
use time::{NumericalDuration, OffsetDateTime};
use tokio::{
    fs::{create_dir_all, read_dir, File},
    io::AsyncWriteExt,
    task::spawn_blocking,
};

use super::service_alerts::ServiceAlertRoot;
use crate::error::Result;

const SERVICE_ALERT_URL: &str = "https://api.opendata.metlink.org.nz/v1/gtfs-rt/servicealerts";

fn get_servicealert_cache_folder(cache_dir: &Path) -> PathBuf {
    cache_dir.join("servicealerts")
}

const FILENAME_FORMAT: &str = "%Y-%m-%d %H.%M.%S%z.json";

fn get_servicealert_cache_file(cache_dir: &Path, now: OffsetDateTime) -> PathBuf {
    get_servicealert_cache_folder(cache_dir).join(now.format(FILENAME_FORMAT))
}

async fn get_servicealert_latest_file(
    cache_dir: &Path,
) -> Result<Option<(OffsetDateTime, PathBuf)>> {
    let sa_cache_dir = get_servicealert_cache_folder(cache_dir);
    create_dir_all(&get_servicealert_cache_folder(cache_dir)).await?;

    let mut read_dirs = read_dir(sa_cache_dir).await?;
    let mut newest = None;
    while let Some(e) = read_dirs.next_entry().await? {
        match e.file_name().into_string() {
            Ok(name) => match time::OffsetDateTime::parse(name, FILENAME_FORMAT) {
                Ok(date) => {
                    if let Some((existing_date, _)) = newest {
                        if date > existing_date {
                            newest = Some((date, e.path()))
                        }
                    } else {
                        newest = Some((date, e.path()))
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(newest)
}

pub async fn load_service_alerts(
    cache_dir: &Path,
    client: &reqwest::Client,
) -> Result<ServiceAlertRoot> {
    let now = OffsetDateTime::now_utc();
    let cache_file = if let Some((date, path)) = get_servicealert_latest_file(cache_dir).await? {
        if now - date < 5.minutes() {
            Some(path)
        } else {
            None
        }
    } else {
        None
    };

    let cache_file = if let Some(file) = cache_file {
        file
    } else {
        let file = get_servicealert_cache_file(cache_dir, now);

        let req_builder = client.get(SERVICE_ALERT_URL);
        let mut response = req_builder.send().await?.error_for_status()?;

        let mut writer = File::create(&file).await?;
        while let Some(mut item) = response.chunk().await? {
            writer.write(&mut item).await?;
        }
        file
    };

    let x = spawn_blocking(move || -> Result<ServiceAlertRoot> {
        Ok(serde_json::from_reader(std::fs::File::open(cache_file)?)?)
    })
    .await
    .unwrap()?;
    info!("{:#?}", x.entity.get(0));
    Ok(x)
}
