use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use time::{Duration, OffsetDateTime};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::task::spawn_blocking;

use super::data::GtfsData;
use super::read::load_gtfs_zip;
use crate::error::Result;
use crate::utils::{IF_MODIFIED_SINCE_DATE_FORMAT, file_mod_time};

const METLINK_GTFS_URL: &str = "https://static.opendata.metlink.org.nz/v1/gtfs/full.zip";
const GTFS_ZIP_FILE: &str = "metlink-gtfs.zip";

fn get_gtfs_zip_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join(GTFS_ZIP_FILE)
}

async fn load_gtfs_from_(cache_dir: &Path) -> Result<GtfsData> {
    let zip_file = get_gtfs_zip_file(cache_dir);
    let result = spawn_blocking(move || load_gtfs_zip(&zip_file))
        .await
        .unwrap();
    result
}

pub async fn load_gtfs(cache_dir: &Path, client: &reqwest::Client) -> Result<GtfsData> {
    let mut req_builder = client.get(METLINK_GTFS_URL);
    let zip_file = get_gtfs_zip_file(cache_dir);
    if let Some(mod_date) = file_mod_time(&zip_file).await? {
        let age = OffsetDateTime::now_utc() - mod_date;
        if age < Duration::days(1) {
            return load_gtfs_from_(cache_dir).await;
        }
        req_builder = req_builder.header(
            "If-Modified-Since",
            mod_date.format(IF_MODIFIED_SINCE_DATE_FORMAT),
        )
    }
    let mut response = req_builder.send().await?.error_for_status()?;

    let mut writer = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(zip_file)
        .await?;

    match response.status().as_u16() {
        200 => {
            writer.seek(SeekFrom::Start(0)).await?;
            writer.set_len(0).await?;
            while let Some(mut item) = response.chunk().await? {
                writer.write(&mut item).await?;
            }
            writer.flush().await?;
            load_gtfs_from_(cache_dir).await
        }
        _ => {
            // Update the modification time(works on Windows).
            let len = writer.seek(SeekFrom::End(0)).await?;
            // Write an extra byte
            writer.write(&[0]).await?;
            writer.flush().await?;
            // Now remove that byte.
            writer.set_len(len).await?;
            writer.flush().await?;
            load_gtfs_from_(cache_dir).await
        }
    }
}
