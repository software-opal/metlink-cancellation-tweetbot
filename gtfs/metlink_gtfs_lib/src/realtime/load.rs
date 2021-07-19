use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Duration,
};

use log::info;
use time::OffsetDateTime;
use tokio::{
    fs::{create_dir_all, read_dir, File},
    io::AsyncWriteExt,
    task::spawn_blocking,
};

use super::service_alerts::{ServiceAlertEntity, ServiceAlertRoot};
use crate::{error::Result, utils::IF_MODIFIED_SINCE_DATE_FORMAT};

pub const MIN_FETCH_FREQUENCY: Duration = Duration::from_secs(5 * 60);

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

pub async fn load_service_alert_from_file(file: PathBuf) -> Result<ServiceAlertRoot> {
    spawn_blocking(move || -> Result<ServiceAlertRoot> {
        Ok(serde_json::from_reader(std::fs::File::open(file)?)?)
    })
    .await
    .unwrap()
}

pub async fn load_all_service_alerts(
    cache_dir: &Path,
    client: &reqwest::Client,
) -> Result<Vec<ServiceAlertEntity>> {
    get_service_alerts_if_outdated(cache_dir, client).await?;

    let mut read_dirs = read_dir(get_servicealert_cache_folder(cache_dir)).await?;
    let mut entities = vec![];
    while let Some(e) = read_dirs.next_entry().await? {
        match load_service_alert_from_file(e.path()).await {
            Ok(root) => entities.extend(root.entity),
            _ => {}
        }
    }

    // Earliest to latest
    entities.sort_unstable_by_key(|e| e.timestamp);
    entities.reverse();

    let mut entity_ids = BTreeSet::new();

    Ok(entities
        .into_iter()
        .filter(|e| {
            if entity_ids.contains(&e.id) {
                false
            } else {
                entity_ids.insert(e.id.clone());
                true
            }
        })
        .collect())
}

pub async fn get_service_alerts_if_outdated(
    cache_dir: &Path,
    client: &reqwest::Client,
) -> Result<PathBuf> {
    let now = OffsetDateTime::now_utc();
    let cache_file_and_date = get_servicealert_latest_file(cache_dir).await?;

    let (should_request, last_mod) = if let Some((date, path)) = &cache_file_and_date {
        if now - *date < MIN_FETCH_FREQUENCY {
            (false, Some(date))
        } else {
            (true, Some(date))
        }
    } else {
        (true, None)
    };

    if should_request {
        let file = get_servicealert_cache_file(cache_dir, now);

        let mut req_builder = client.get(SERVICE_ALERT_URL);
        if let Some(date) = last_mod {
            req_builder = req_builder.header(
                "If-Modified-Since",
                date.format(IF_MODIFIED_SINCE_DATE_FORMAT),
            )
        }

        log::info!("Requesting latest service alerts. Last Mod: {:?}", last_mod.map(|date| date.to_string()));
        let mut response = req_builder.send().await?.error_for_status()?;

        log::info!("Response {}. headers: {:?}", response.status(), response.headers());
        if response.status() == 200 {
            log::info!("Retrieved latest service alerts.");
            let mut writer = File::create(&file).await?;
            while let Some(mut item) = response.chunk().await? {
                writer.write(&mut item).await?;
            }
            return Ok(file);
        } else if cache_file_and_date.is_none() {
            panic!(
                "Cached file is None, but response status was {}",
                response.status()
            );
        }
    }
    log::info!("Using latest service alerts from cache.");
    let (_, existing_file) = cache_file_and_date.unwrap();
    Ok(existing_file)
}

pub async fn load_service_alerts(
    cache_dir: &Path,
    client: &reqwest::Client,
) -> Result<ServiceAlertRoot> {
    let cache_file = get_service_alerts_if_outdated(cache_dir, client).await?;
    let x = load_service_alert_from_file(cache_file).await?;
    info!("{:#?}", x.entity.get(0));
    Ok(x)
}
