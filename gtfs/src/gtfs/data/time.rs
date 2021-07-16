use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;
use serde::{Deserialize, Deserializer, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

pub fn deserialize_time_struct<'de, D>(
    deserializer: D,
) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Time::from_str(s).map_err(|e| serde::de::Error::custom(e))
}

impl Time {
    fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour, minute, second
        }
    }
}

#[derive(Error, Debug)]
pub enum TimeFromStrError {
    #[error("{0:?})")]
    ParseIntError(#[from] ParseIntError),
    #[error("Invalid Format")]
    InvalidFormat,
}

impl FromStr for Time {
    type Err = TimeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((hour_s, min_sec_s)) = s.split_once(":") {
            if let Some((min_s, sec_s)) = min_sec_s.split_once(":") {
                return Ok( Self {
                    hour: u8::from_str(hour_s)?,
                    minute: u8::from_str(min_s)?,
                    second: u8::from_str(sec_s)?
                })
            }
        }
        return Err(TimeFromStrError::InvalidFormat)
    }
}
