use super::time::{deserialize_time_struct, Time};
use super::utils::deserialize_num_bool;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Trip {
    route_id: String,
    service_id: String,
    trip_id: String,
    trip_headsign: String,
    #[serde(deserialize_with = "deserialize_num_bool")]
    direction_id: bool,
    block_id: String,
    shape_id: String,
    // wheelchair_accessible: TripItemPermitted,
    // bikes_allowed: TripItemPermitted,
    // etm_id: String
}
