use super::utils::{deserialize_date, deserialize_num_bool};
use serde::{Deserialize, Deserializer, Serialize};
use time::Date;

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
    end_date: Date,
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
    service_id: String,
    #[serde(deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(deserialize_with = "deserialize_exception_type")]
    exception_type: CalendarDateExceptionType,
}
