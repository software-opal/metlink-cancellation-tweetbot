use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::gtfs::data::stop::StopLocationType;

#[derive(Debug, Deserialize, Serialize)]
pub struct Stop {
    id: String,
    code: String,
    name: String,
    desc: String,
    lat: f64,
    lon: f64,
    zone_id: String,
    url: String,
    location_type: StopLocationType,
    parent_station: String,
    timezone: String,
}

impl Stop {}
impl From<&crate::gtfs::data::stop::Stop> for Stop {
    fn from(stop: &crate::gtfs::data::stop::Stop) -> Self {
        Self {
            id: stop.stop_id.clone(),
            code: stop.stop_code.clone(),
            name: stop.stop_name.clone(),
            desc: stop.stop_desc.clone(),
            lat: stop.stop_lat.clone(),
            lon: stop.stop_lon.clone(),
            zone_id: stop.zone_id.clone(),
            url: stop.stop_url.clone(),
            location_type: stop.location_type,
            parent_station: stop.parent_station.clone(),
            timezone: stop.stop_timezone.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopDb {
    stops: BTreeMap<String, Stop>,
}

impl StopDb {
    pub fn get_stop<'a>(&'a self, id: &String) -> Option<&'a Stop> {
        self.stops.get(id)
    }
}

impl From<&crate::gtfs::data::GtfsData> for StopDb {
    fn from(parsed: &crate::gtfs::data::GtfsData) -> Self {
        Self {
            stops: parsed.stop
                .iter()
                .map(|stop| Stop::from(stop))
                .map(|stop| (stop.id.clone(), stop))
                .collect(),
        }
    }
}
