use super::utils::deserialize_num_bool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Trip {
    pub route_id: String,
    pub service_id: String,
    pub trip_id: String,
    pub trip_headsign: String,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub direction_id: bool,
    pub block_id: String,
    pub shape_id: String,
    // wheelchair_accessible: TripItemPermitted,
    // bikes_allowed: TripItemPermitted,
    // etm_id: String
}
