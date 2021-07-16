use time::Date;
use serde::{Deserialize, Serialize};
use super::utils::{deserialize_date, deserialize_num_bool};

#[derive(Debug, Deserialize, Serialize)]
pub struct Calendar {
    service_id: String,
    #[serde(deserialize_with = "deserialize_num_bool")]
    monday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    tuesday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    wednesday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    thursday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    friday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    saturday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    sunday: bool,
    #[serde(deserialize_with = "deserialize_date")]
    start_date: Date,
    #[serde(deserialize_with = "deserialize_date")]
    end_date: Date

}