use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tokio::task::spawn_blocking;

use crate::{error::Result, utils::file_mod_time};

pub mod routes;
pub mod stops;
pub mod trips;

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub routes: self::routes::RouteDb,
    pub stops: self::stops::StopDb,
    pub trips: self::trips::TripDb,
}

impl From<&crate::gtfs::data::GtfsData> for Database {
    fn from(parsed: &crate::gtfs::data::GtfsData) -> Self {
        Self {
            routes: parsed.into(),
            stops: parsed.into(),
            trips: parsed.into(),
        }
    }
}

pub fn db_cache_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join("gtfs_db.json")
}

pub async fn load_db_from_cache(cache_dir: &Path, allow_old: bool) -> Result<Option<Database>> {
    let db_file = db_cache_file(cache_dir);
    if let Some(mod_time) = file_mod_time(&db_file).await? {
        let age = OffsetDateTime::now_utc() - mod_time;
        if allow_old || age < Duration::days(1) {
            Ok(Some(
                spawn_blocking(|| -> Result<Database> {
                    Ok(serde_json::from_reader(std::fs::File::open(db_file)?)?)
                })
                .await
                .unwrap()?,
            ))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub async fn save_db_to_cache<'a>(cache_dir: &Path, db: Database) -> Result<Database> {
    let db_file = db_cache_file(cache_dir);

    Ok(spawn_blocking(move || -> Result<_> {
        serde_json::to_writer_pretty(std::fs::File::create(db_file)?, &db)?;
        Ok(db)
    })
    .await
    .unwrap()?)
}
