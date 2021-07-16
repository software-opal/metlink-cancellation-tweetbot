use super::utils::deserialize_num_bool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StopPattern {
    stop_pattern_id: String,
    stop_id: String,
    stop_sequence: u16,
    #[serde(deserialize_with = "deserialize_num_bool")]
    timepoint: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopPatternTrip {
    stop_pattern_id: String,
    trip_id: String,
    trip_sequence: u16,
}
