
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Agency {
    agency_id: String,
    agency_name: String,
    agency_url: String,
    agency_timezone: String,
    agency_lang: String,
    agency_phone: String,
    agency_fare_url: String,
}
