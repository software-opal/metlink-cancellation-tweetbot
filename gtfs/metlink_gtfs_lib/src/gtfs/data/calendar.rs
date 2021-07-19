use super::utils::{deserialize_date, deserialize_num_bool};
use serde::{Deserialize, Deserializer, Serialize};
use time::Date;

#[derive(Debug, Deserialize, Serialize)]
pub struct Calendar {
    pub service_id: String,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub monday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub tuesday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub wednesday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub thursday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub friday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub saturday: bool,
    #[serde(deserialize_with = "deserialize_num_bool")]
    pub sunday: bool,
    #[serde(deserialize_with = "deserialize_date")]
    pub start_date: Date,
    #[serde(deserialize_with = "deserialize_date")]
    pub end_date: Date,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CalendarDateExceptionType {
    ServiceAdded,
    ServiceRemoved,
}

pub fn deserialize_exception_type<'de, D>(
    deserializer: D,
) -> Result<CalendarDateExceptionType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    match s {
        1 => Ok(CalendarDateExceptionType::ServiceAdded),
        2 => Ok(CalendarDateExceptionType::ServiceRemoved),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid numeric exception type: {}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CalendarDate {
    pub service_id: String,
    #[serde(deserialize_with = "deserialize_date")]
    pub date: Date,
    #[serde(deserialize_with = "deserialize_exception_type")]
    pub exception_type: CalendarDateExceptionType,
}
