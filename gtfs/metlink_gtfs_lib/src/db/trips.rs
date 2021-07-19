use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::datatypes::time::Time;

#[derive(Debug, Deserialize, Serialize)]
pub struct TripStop {
    stop_id: String,
    arrival_time: Time,
    departure_time: Time,
    stop_sequence: u16,
    // pickup_type: PickupDropoffType,
    // drop_off_type: PickupDropoffType,
    shape_dist_traveled: Option<f64>,
    stop_headsign: String,
    timepoint: bool,
}
impl From<&crate::gtfs::data::stop::StopTime> for TripStop {
    fn from(stop: &crate::gtfs::data::stop::StopTime) -> Self {
        Self {
            stop_id: stop.stop_id.clone(),
            arrival_time: stop.arrival_time.clone(),
            departure_time: stop.departure_time.clone(),
            stop_sequence: stop.stop_sequence.clone(),
            // pickup_type: stop.pickup_type,
            // drop_off_type: stop.drop_off_type,
            shape_dist_traveled: stop.shape_dist_traveled.clone(),
            stop_headsign: stop.stop_headsign.clone(),
            timepoint: stop.timepoint.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trip {
  pub   id: String,
  pub   route_id: String,
  pub   service_id: String,
  pub   headsign: String,
  pub   direction: bool,
  pub   block_id: String,
  pub   shape_id: String,
  pub   stops: Vec<TripStop>,
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
            stops: Vec::with_capacity(0),
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

impl From<&crate::gtfs::data::GtfsData> for TripDb {
    fn from(parsed: &crate::gtfs::data::GtfsData) -> Self {
        let mut trip_stops: BTreeMap<String, Vec<(u16, TripStop)>> = BTreeMap::new();
        parsed.stop_time.iter().for_each(|s| {
            if let Some(v) = trip_stops.get_mut(&s.trip_id) {
                v.push((s.stop_sequence, s.into()))
            } else {
                trip_stops.insert(s.trip_id.clone(), vec![(s.stop_sequence, s.into())]);
            }
        });

        Self {
            trips: parsed
                .trip
                .iter()
                .map(|trip| {
                    let mut trip = Trip::from(trip);
                    trip.stops = trip_stops
                        .remove(&trip.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(_, s)| s)
                        .collect();
                    trip
                })
                .map(|trip| (trip.id.clone(), trip))
                .collect(),
        }
    }
}
