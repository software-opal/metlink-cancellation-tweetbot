use std::{path::{Path, PathBuf}, time::Duration};

use reqwest::Client;
use time::{ OffsetDateTime};
use serde::{Deserialize, Deserializer, Serialize};

use super::utils::CachedRealtimeApi;

pub struct TripUpdateRealtimeApi {
    cache_dir: PathBuf,
    client: Client,
}

impl TripUpdateRealtimeApi {
    pub fn new(cache_dir: &Path, client: Client) -> Self {
        Self {
            cache_dir: cache_dir.to_owned(),
            client
        }
    }
}

impl CachedRealtimeApi for TripUpdateRealtimeApi {
    fn root_cache_dir(&self) -> &std::path::Path {
        &self.cache_dir
    }

    fn name(&self) -> &'static str {
        "tripupdates"
    }

    fn download(&self) -> reqwest::RequestBuilder {
        self.client
            .get("https://api.opendata.metlink.org.nz/v1/gtfs-rt/tripupdates")
    }

    fn min_fetch_frequency(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}