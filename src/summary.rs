use crate::Cancellations;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::{Duration, NaiveTime};
use std::cmp::max;
use std::cmp::min;
use std::convert::TryInto;
use std::iter::FromIterator;

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct CancellationSummary {
    stats: CancellationStats, // Map<String, >
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct CancellationStats {
    count: usize,
    earliest: Option<NaiveTime>,
    latest: Option<NaiveTime>,

    notice: CancellationNoticeStats,
    before_notice: CancellationNoticeStats,
    after_notice: CancellationNoticeStats,
}
impl<'a> FromIterator<&'a Cancellations> for CancellationStats {
    fn from_iter<T>(iter: T) -> Self
    where
        T: std::iter::IntoIterator<Item = &'a Cancellations>,
    {
        let mut count = 0;
        let mut earliest = None;
        let mut latest = None;

        let mut notice_durations = vec![];
        for can in iter {
            let can_bus_dt = can.time();
            let can_bus_time = can_bus_dt.time();
            let can_tweet_dt: DateTime<FixedOffset> =
                can.tweet_time().with_timezone(&FixedOffset::east(0));
            count += 1;
            earliest = earliest
                .map(|v| min(v, can_bus_time))
                .or_else(|| Some(can_bus_time));
            latest = latest
                .map(|v| max(v, can_bus_time))
                .or_else(|| Some(can_bus_time));
            // Tweeted 20 minutes before scheduled bus service should return +20 minutes
            notice_durations.push(*can_bus_dt - can_tweet_dt);
        }

        CancellationStats {
            count,
            earliest,
            latest,
            notice: notice_durations.iter().collect(),
            before_notice: notice_durations
                .iter()
                .filter(|duration| duration.num_seconds() > 0)
                .collect(),
            after_notice: notice_durations
                .iter()
                .filter(|duration| duration.num_seconds() <= 0)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct CancellationNoticeStats {
    count: usize,
    earliest: Option<Duration>,
    latest: Option<Duration>,
    average: Duration,
}

impl<'a> FromIterator<&'a Duration> for CancellationNoticeStats {
    fn from_iter<T>(iter: T) -> Self
    where
        T: std::iter::IntoIterator<Item = &'a Duration>,
    {
        let mut earliest = None;
        let mut latest = None;
        let mut total = Duration::zero();
        let mut count = 0;

        for item_ in iter {
            let item = *item_;
            earliest = earliest.map(|v| max(v, item)).or(Some(item));
            latest = latest.map(|v| min(v, item)).or(Some(item));
            total = total + item;
            count = count + 1;
        }

        CancellationNoticeStats {
            count,
            earliest,
            latest,
            average: if count == 0 {
                Duration::zero()
            } else {
                total / count.try_into().unwrap()
            },
        }
    }
}

pub fn summarize(cancellations: &[Cancellations]) -> CancellationSummary {
    return CancellationSummary {
        stats: cancellations.iter().collect(),
    };
}
