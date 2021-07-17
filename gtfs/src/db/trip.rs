use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Trip {
    id: String,
    route_id: String,
    service_id: String,
    headsign: String,
    direction: bool,
    block_id: String,
    shape_id: String,
}

impl Trip {}
impl From<&crate::gtfs::data::trip::Trip> for Trip {
    fn from(trip: &crate::gtfs::data::trip::Trip) -> Self {
        Self {
            id: trip.trip_id.clone(),
            route_id: trip.route_id.clone(),
            service_id: trip.service_id.clone(),
            headsign: trip.trip_headsign.clone(),
            direction: trip.direction_id,
            block_id: trip.block_id.clone(),
            shape_id: trip.shape_id.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TripDb {
    trips: BTreeMap<String, Trip>,
}

impl TripDb {
    pub fn get_trip<'a>(&'a self, id: &String) -> Option<&'a Trip> {
        self.trips.get(id)
    }
}

impl From<&Vec<crate::gtfs::data::trip::Trip>> for TripDb {
    fn from(parsed: &Vec<crate::gtfs::data::trip::Trip>) -> Self {
        Self {
            trips: parsed
                .iter()
                .map(|trip| Trip::from(trip))
                .map(|trip| (trip.id.clone(), trip))
                .collect(),
        }
    }
}
