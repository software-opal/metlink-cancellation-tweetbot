use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use reqwest::header::{self, ETAG};
use time::OffsetDateTime;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::spawn_blocking;

use super::data::GtfsData;
use super::error::Result;
use super::read::load_gtfs_zip;

const METLINK_GTFS_URL: &str = "https://static.opendata.metlink.org.nz/v1/gtfs/full.zip";
const GTFS_ZIP_FILE: &str = "metlink-gtfs.zip";
const GTFS_ETAG_FILE: &str = "metlink-gtfs.zip.txt";

fn get_gtfs_zip_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join(GTFS_ZIP_FILE)
}
fn get_gtfs_etag_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join(GTFS_ETAG_FILE)
}

async fn load_gtfs_etag(cache_dir: &Path) -> Result<Option<Vec<u8>>> {
    let etag_file = get_gtfs_etag_file(cache_dir);
    match File::open(etag_file).await {
        Ok(mut f) => {
            let mut buf = Vec::with_capacity(100);
            f.read_to_end(&mut buf).await?;
            Ok(Some(buf))
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(e)?,
        },
    }
}

async fn load_gtfs_mod_date(cache_dir: &Path) -> Result<Option<String>> {
    let zip_file = get_gtfs_zip_file(cache_dir);
    match File::open(zip_file).await {
        Ok(f) => {
            let metadata = f.metadata().await?;
            match metadata.modified() {
                Ok(mod_time) => match mod_time.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => {
                        let datetime =
                            OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64);
                        Ok(Some(datetime.format("%a, %d %b %Y %H:%M:%S GMT")))
                    }
                    Err(_) => Ok(None),
                },
                Err(_) => Ok(None),
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(e)?,
        },
    }
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
    if let Some(etag) = load_gtfs_etag(cache_dir).await? {
        req_builder = req_builder.header(header::IF_NONE_MATCH, etag)
    }
    if let Some(mod_date) = load_gtfs_mod_date(cache_dir).await? {
        req_builder = req_builder.header("If-Modified-Since", mod_date)
    }
    let mut response = req_builder.send().await?.error_for_status()?;

    match response.status().as_u16() {
        200 => {
            let mut writer = File::create(get_gtfs_zip_file(cache_dir)).await?;
            while let Some(mut item) = response.chunk().await? {
                writer.write(&mut item).await?;
            }
            writer.flush().await?;
            if let Some(value) = response.headers().get(ETAG) {
                let mut writer = File::create(get_gtfs_etag_file(cache_dir)).await?;
                writer.write_all(value.as_bytes());
                writer.flush().await?;
            }
            load_gtfs_from_(cache_dir).await
        }
        304 => load_gtfs_from_(cache_dir).await,
        _ => load_gtfs_from_(cache_dir).await,
    }
}
