use std::{path::{Path, PathBuf}, time::Duration};

use reqwest::Client;
use time::{ OffsetDateTime};
use serde::{Deserialize, Deserializer, Serialize};

use super::utils::CachedRealtimeApi;

pub struct VehiclePositionsRealtimeApi {
    cache_dir: PathBuf,
    client: Client,
}

impl VehiclePositionsRealtimeApi {
    pub fn new(cache_dir: &Path, client: Client) -> Self {
        Self {
            cache_dir: cache_dir.to_owned(),
            client
        }
    }
}

impl CachedRealtimeApi for VehiclePositionsRealtimeApi {
    fn root_cache_dir(&self) -> &std::path::Path {
        &self.cache_dir
    }

    fn name(&self) -> &'static str {
        "vehiclepositions"
    }

    fn download(&self) -> reqwest::RequestBuilder {
        self.client
            .get("https://api.opendata.metlink.org.nz/v1/gtfs-rt/vehiclepositions")
    }

    fn min_fetch_frequency(&self) -> Duration {
        Duration::from_secs(5 * 60)
    }
}