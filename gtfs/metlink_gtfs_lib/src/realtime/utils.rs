use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use reqwest::RequestBuilder;
use time::OffsetDateTime;
use tokio::{
    fs::{create_dir_all, read_dir, File},
    io::AsyncWriteExt,
};

use crate::error::Result;

const FILENAME_FORMAT: &str = "%Y-%m-%d %H.%M.%S%z.json";


pub trait CachedRealtimeApi {
    fn root_cache_dir(&self) -> &Path;
    fn name(&self) -> &'static str;
    fn download(&self) -> RequestBuilder;
    fn min_fetch_frequency(&self) -> Duration;

    fn cache_folder(&self) -> PathBuf {
        self.root_cache_dir().join(self.name())
    }

    fn cache_file(&self, now: OffsetDateTime) -> PathBuf {
        self.cache_folder().join(now.format(FILENAME_FORMAT))
    }
}

pub async fn all_cache_files<T: CachedRealtimeApi>(
    api: &T,
) -> Result<Vec<(OffsetDateTime, PathBuf)>> {
    let cache_folder = api.cache_folder();
    create_dir_all(&cache_folder).await?;

    let mut read_dirs = read_dir(cache_folder).await?;

    let mut items = vec![];
    while let Some(e) = read_dirs.next_entry().await? {
        match e.file_name().into_string() {
            Ok(name) => match time::OffsetDateTime::parse(name, FILENAME_FORMAT) {
                Ok(date) => items.push((date, e.path())),
                _ => {}
            },
            _ => {}
        }
    }
    items.sort_unstable_by_key(|(date, _)| date.unix_timestamp());
    Ok(items)
}

// pub async fn load_from_file<I: 'static + DeserializeOwned + Send>(file: PathBuf) -> Result<I> {
//     spawn_blocking(move || -> Result<I> {
//         Ok(serde_json::from_reader(std::fs::File::open(file)?)?)
//     })
//     .await
//     .unwrap()
// }

// pub async fn load_all_from_files<T: CachedRealtimeApi<I>, I: 'static + DeserializeOwned + Send>(
//     api: &T,
// ) -> Result<Vec<I>> {
//     let all_files = all_cache_files(api).await?;

//     let mut entities = vec![];
//     for (_, path) in all_files {
//         match load_from_file::<I>(path).await {
//             Ok(obj) => entities.push(obj),
//             _ => {}
//         }
//     }
//     Ok(entities)
// }

pub async fn download_latest<T: CachedRealtimeApi>(api: &T) -> Result<PathBuf> {
    let now = OffsetDateTime::now_utc();
    let file = api.cache_file(now);
    let mut response = api.download().send().await?.error_for_status()?;

    let mut writer = File::create(&file).await?;
    while let Some(mut item) = response.chunk().await? {
        writer.write(&mut item).await?;
    }
    return Ok(file);
}

pub async fn download_latest_if_needed<T: CachedRealtimeApi>(api: &T) -> Result<PathBuf> {
    let all_files = all_cache_files(api).await?;
    if let Some((date, path)) = all_files.into_iter().next() {
        if OffsetDateTime::now_utc() - date < api.min_fetch_frequency() {
            return Ok(path);
        }
    }

    download_latest(api).await
}
