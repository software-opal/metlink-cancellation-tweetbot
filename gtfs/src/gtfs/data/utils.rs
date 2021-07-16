use serde::{Deserialize, Deserializer};
use time::Date;

pub fn deserialize_num_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    match s {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid numeric boolean value: {}",
            s
        ))),
    }
}

const DATE_FORMAT: &str = "%Y%m%d";

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Date::parse(&s, DATE_FORMAT).map_err(serde::de::Error::custom)
}
