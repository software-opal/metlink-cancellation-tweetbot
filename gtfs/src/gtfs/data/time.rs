use serde::{Deserialize, Deserializer, Serialize};
use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeError {
    #[error("{0:?}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Invalid Second")]
    InvalidSecond,
    #[error("Invalid Minute")]
    InvalidMinute,
    #[error("Invalid Format")]
    InvalidFormat,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

pub fn deserialize_time_struct<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Time::from_str(&s).map_err(|e| serde::de::Error::custom(e))
}

impl Time {
    fn new(hour: u8, minute: u8, second: u8) -> Result<Self, TimeError> {
        if minute > 60 {
            Err(TimeError::InvalidMinute)
        } else if minute > 60 {
            Err(TimeError::InvalidSecond)
        } else {
            Ok(Self {
                hour,
                minute,
                second,
            })
        }
    }
    fn to_time(&self) -> (time::Time, u8) {
        (
            time::Time::try_from_hms(self.hour % 24, self.minute, self.second).unwrap(),
            self.hour / 24,
        )
    }
    fn to_datetime(&self, date: time::Date) -> time::PrimitiveDateTime {
        let (time, days_to_add) = self.to_time();
        let mut date = date;
        for _ in 0..days_to_add {
            date = date.next_day()
        }
        return date.with_time(time);
    }
}

impl FromStr for Time {
    type Err = TimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((hour_s, min_sec_s)) = s.split_once(":") {
            if let Some((min_s, sec_s)) = min_sec_s.split_once(":") {
                return Self::new(
                    u8::from_str(hour_s)?,
                    u8::from_str(min_s)?,
                    u8::from_str(sec_s)?,
                );
            }
        }
        return Err(TimeError::InvalidFormat);
    }
}
