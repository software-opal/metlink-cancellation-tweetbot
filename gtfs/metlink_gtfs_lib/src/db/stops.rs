use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::gtfs::data::stop::StopLocationType;

#[derive(Debug, Deserialize, Serialize)]
pub struct Stop {
  pub  id: String,
  pub  code: String,
  pub  name: String,
  pub  desc: String,
  pub  lat: f64,
  pub  lon: f64,
  pub  zone_id: String,
  pub  url: String,
  pub  location_type: StopLocationType,
  pub  parent_station: String,
  pub  timezone: String,
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
