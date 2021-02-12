use chrono::{DateTime, FixedOffset, Offset, Timelike, Utc};
use chrono_tz::Pacific::Auckland;

pub fn convert_time_to_instant(
    tweeted_at: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> Result<DateTime<FixedOffset>, String> {
    let bus_time = tweeted_at
        .with_timezone(&Auckland)
        .with_hour(hour)
        .ok_or_else(|| format!("Cannot set hour to parsed hour {}", hour))?
        .with_minute(minute)
        .ok_or_else(|| format!("Cannot set minute to parsed minute {}", minute))?
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    return Ok(bus_time.with_timezone(&bus_time.offset().fix()));
}

#[cfg(test)]
mod test_time {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {

    // 1pm 2021-02-06
            static ref SAMPLE_TIME_1PM: DateTime<Utc> = DateTime::parse_from_rfc3339("2021-02-06T00:12:34.567Z")
        .unwrap()
        .with_timezone(&Utc);
    }

    #[test]
    fn test_time() {
        assert_eq!(
            Ok(DateTime::parse_from_rfc3339("2021-02-06T13:15:00+13:00").unwrap()),
            convert_time_to_instant(*SAMPLE_TIME_1PM, 13, 15)
        )
    }
}
