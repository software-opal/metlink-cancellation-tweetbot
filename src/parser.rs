use crate::{time::convert_time_to_instant, tweet_cache::TweetContent};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::Sub};

// Turns out am/pm is called the " delay_minutes: ()period". Thanks StackExchange
// https://english.stackexchange.com/questions/35315/what-is-the-proper-name-for-am-and-pm#35317
const TIME_RE: &str = r"(?P<hour>[0-9]{1,2})(?:[.:](?P<minute>[0-9]{2}))?(?P<period>am|pm|m|)";
const BUS_NUM_RE: &str = r"Bu[sa] ?(?P<bus_num>[0-9ex]+)";
const BUS_DEST_RE: &str = r"(?:from )?(?P<origin>.*?) (?:to|tp|-) (?P<destination>.*?)";
const TRAIN_LINE_NAMES: [&str; 5] = ["WRL", "KPL", "HVL", "JVL", "MEL"];

lazy_static! {
    static ref TRAIN_LINE_NAME: String = TRAIN_LINE_NAMES.join("|");
    static ref BUS_FULL_CANCELLED_RE: Regex = Regex::new(&format!(
        "{0}(?:: +|:| ){1}[:]? {2} +(?:(?:is|has been|was) |)cancelled",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_REINSTATED_RE: Regex = Regex::new(&format!(
        "{0}: *{1} {2} +(?:is REINSTATED|is reinstated|has been REINSTATED|has been reinstated and will now run|t?hat was(?: previously)? cancelled will now run|will now run)",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_DELAYED_RE: Regex = Regex::new(&format!(
        "{0}: {1} {2} +(?:is|has been|will be) delayed(?: by)? (?:[0-9]+-)?(?P<delay_mins>[0-9]+) min",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    )).unwrap();
    static ref BUS_DELAYED_LATE_RE: Regex = Regex::new(&format!(
        "{0}: {1} {2} +(?:will) run (?:[0-9]+-)?(?P<delay_mins>[0-9]+) minutes? late\\.",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    )).unwrap();
    static ref BUS_DELAYED_UNDETERMINATE_RE: Regex = Regex::new(&format!(
        "{0}: {1} {2} (has been|is) delayed(?: due to vehicle breakdown| due to road block|)\\.",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_PART_CANCELLED_RE: Regex = Regex::new(&format!(
        "{0}: {1} {2} +(?:is|has been) part[- ]cancelled from (?P<cancelled_from>.*?)\\.",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_PART_CANCELLED_BETWEEN_RE: Regex = Regex::new(&format!(
            "{0}: {1} {2}(?: is| has been| will be|\\. Is|) part[- ]cancelled (?:between|from) (?P<cancelled_from>.*?) (?:and|to|&amp;|&) (?P<cancelled_to>.*?) *\\.",
            BUS_NUM_RE, TIME_RE, BUS_DEST_RE
        ))
    .unwrap();
    static ref BUS_PART_CANCELLED_BETWEEN_NO_ORIGIN_RE: Regex = Regex::new(&format!(
            "{0}: {1} {2} (?:is|has been) part[- ]cancelled between (?P<cancelled_from>.*?) (?:and|to|&amp;|&) (?P<cancelled_to>.*?) *\\.",
            BUS_NUM_RE, TIME_RE, r"(?:to|-) (?P<destination>.*?)"
        ))
    .unwrap();
    static ref TRAIN_LINE_NAMES_RE: Regex = Regex::new(&TRAIN_LINE_NAME).unwrap();
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub enum Cancellations {
    BusCancelled {
        route: String,
        origin: String,
        destination: String,
        raw_time: String,
        tweet_time: DateTime<Utc>,
        time: DateTime<FixedOffset>,
    },
    BusPartCancelled {
        route: String,
        origin: String,
        destination: String,
        cancelled_from: String,
        cancelled_to: String,
        raw_time: String,
        tweet_time: DateTime<Utc>,
        time: DateTime<FixedOffset>,
    },
    BusReinstated {
        route: String,
        origin: String,
        destination: String,
        raw_time: String,
        tweet_time: DateTime<Utc>,
        time: DateTime<FixedOffset>,
    },
    BusDelayed {
        route: String,
        origin: String,
        destination: String,
        delay_minutes: String,
        raw_time: String,
        tweet_time: DateTime<Utc>,
        time: DateTime<FixedOffset>,
    },
}

fn do_time_from(
    time: DateTime<Utc>,
    hour: u32,
    minute: u32,
    period: &str,
) -> Result<(String, DateTime<FixedOffset>), String> {
    if period == "am" {
        Ok((
            format!("{}:{:02} am", hour, minute),
            convert_time_to_instant(time, hour, minute)?,
        ))
    } else if period == "pm" {
        Ok((
            format!("{}:{:02} pm", hour, minute),
            convert_time_to_instant(time, (hour % 12) + 12, minute)?,
        ))
    } else {
        let fixed_time: DateTime<FixedOffset> = time.into();
        let (am_str, am_time) = do_time_from(time, hour, minute, &"am")?;
        let (pm_str, pm_time) = do_time_from(time, hour, minute, &"pm")?;

        if time < am_time {
            // Before the AM time, so must be AM
            Ok((am_str, am_time))
        } else if time > pm_time {
            // After PM time, so must be PM
            Ok((pm_str, pm_time))
        } else if fixed_time.sub(am_time) < Duration::hours(2) {
            // Within 2 hours of AM time; I'm hoping that Metlink won't cancel a bus more than 2
            //  hours after it's scheduled to start. And tweet about it without AM/PM at the end.
            Ok((am_str, am_time))
        } else if fixed_time.sub(pm_time) < Duration::hours(2) {
            Ok((pm_str, pm_time))
        } else {
            Err(format!(
                "Parsing {}:{} {:?} failed at {}. AM: {:?}(diff {:?}); PM: {:?}(diff {:?})",
                hour,
                minute,
                period,
                time,
                (am_str, am_time),
                time.sub(DateTime::<Utc>::from(am_time)),
                (pm_str, pm_time),
                time.sub(DateTime::<Utc>::from(pm_time))
            ))
        }
    }
}

fn time_from_capture(
    tweet: &TweetContent,
    capture: &Captures,
) -> Result<(String, DateTime<FixedOffset>), String> {
    let hour = capture.name("hour").unwrap().as_str().parse().unwrap();
    let minute = capture
        .name("minute")
        .map(|m| m.as_str())
        .unwrap_or("00")
        .parse()
        .unwrap();
    let period = capture
        .name("period")
        .unwrap()
        .as_str()
        .to_ascii_lowercase();
    return do_time_from(tweet.created_at, hour, minute, &period);
}

fn parse_bus_tweet(tweet: &TweetContent) -> Result<Vec<Cancellations>, String> {
    None.or_else(|| {
        BUS_REINSTATED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture)?;
            Ok(vec![Cancellations::BusReinstated {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
                time,
            }])
        })
    })
    .or_else(|| {
        BUS_DELAYED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture)?;
            Ok(vec![Cancellations::BusDelayed {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                delay_minutes: capture.name("delay_mins").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
                time,
            }])
        })
    })
    .or_else(|| {
        BUS_DELAYED_LATE_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture)?;
            Ok(vec![Cancellations::BusDelayed {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                delay_minutes: capture.name("delay_mins").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
                time,
            }])
        })
    })
    .or_else(|| {
        BUS_DELAYED_UNDETERMINATE_RE
            .captures(&tweet.text)
            .map(|capture| {
                let (raw_time, time) = time_from_capture(tweet, &capture)?;
                Ok(vec![Cancellations::BusDelayed {
                    route: capture.name("bus_num").unwrap().as_str().to_string(),
                    origin: capture.name("origin").unwrap().as_str().to_string(),
                    destination: capture.name("destination").unwrap().as_str().to_string(),
                    delay_minutes: "".to_string(),
                    raw_time,
                    tweet_time: tweet.created_at,
                    time,
                }])
            })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_BETWEEN_RE
            .captures(&tweet.text)
            .map(|capture| {
                let (raw_time, time) = time_from_capture(tweet, &capture)?;
                Ok(vec![Cancellations::BusPartCancelled {
                    route: capture.name("bus_num").unwrap().as_str().to_string(),
                    origin: capture.name("origin").unwrap().as_str().to_string(),
                    destination: capture.name("destination").unwrap().as_str().to_string(),
                    cancelled_from: capture.name("cancelled_from").unwrap().as_str().to_string(),
                    cancelled_to: capture.name("cancelled_to").unwrap().as_str().to_string(),
                    raw_time,
                    tweet_time: tweet.created_at,
                    time,
                }])
            })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_BETWEEN_NO_ORIGIN_RE
            .captures(&tweet.text)
            .map(|capture| {
                let cancelled_from = capture.name("cancelled_from").unwrap().as_str().to_string();
                let (raw_time, time) = time_from_capture(tweet, &capture)?;
                Ok(vec![Cancellations::BusPartCancelled {
                    route: capture.name("bus_num").unwrap().as_str().to_string(),
                    origin: cancelled_from.clone(),
                    destination: capture.name("destination").unwrap().as_str().to_string(),
                    cancelled_from,
                    cancelled_to: capture.name("cancelled_to").unwrap().as_str().to_string(),
                    raw_time,
                    tweet_time: tweet.created_at,
                    time,
                }])
            })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_RE.captures(&tweet.text).map(|capture| {
            let destination = capture.name("destination").unwrap().as_str().to_string();
            let (raw_time, time) = time_from_capture(tweet, &capture)?;
            Ok(vec![Cancellations::BusPartCancelled {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: destination.clone(),
                cancelled_from: capture.name("cancelled_from").unwrap().as_str().to_string(),
                cancelled_to: destination,
                raw_time,
                tweet_time: tweet.created_at,
                time,
            }])
        })
    })
    .or_else(|| {
        BUS_FULL_CANCELLED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture)?;
            Ok(vec![Cancellations::BusCancelled {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
                time,
            }])
        })
    })
    .unwrap_or_else(|| {
        if tweet.text.contains("https://t.co/") || tweet.text.contains("buses cannot pass") {
            Ok(vec![])
        } else {
            Err("Unable to detect bus impact".to_string())
        }
    })
}

lazy_static! {
    static ref IGNORED_TWEET_IDS: HashSet<u64> = {
        let mut h = HashSet::new();
        h.extend(&[
            1313942434309525504,
            1314353142742482944,
            1314733655579922435,
            1314737174181367812,
            1314737926467579907,
            1318041702188216321,
            1318391755377496065,
            1319510878396370946,
            1322661891441745921,
            1325978495604830208,
            1330659892303052801,
            1331688367751237632,
            1332174572502716417,
            1333449475386294272,
            1334628241722626050,
            1335667339166167046,
            1335745856847286275,
            1337504190365462528,
            1339338275341791233,
            1340450608818601984,
            1346992964510248961,
            1348859010426892290,
            1349203333723033603,
            1346657252422270976,
            1350225276978909184,
            1351244750372921345,
            1346375145888206849,
            1353179739381395456,
            1315364812663083010,
            1339766601579548672,
            1354564608745381890,
            1354966519957000195,
            1354966526365892612,
            1356690879805681664,
            1356836623581806593,
            1357189987502944257,
            1357200754935758849,
            1358956553324228610,
            1360050266813329410,
            1360053285831364608,
            1360870398640807937,
        ]);
        h
    };
}

pub fn parse_tweet(tweet: &TweetContent) -> Result<Vec<Cancellations>, String> {
    if IGNORED_TWEET_IDS.contains(&tweet.id) {
        Ok(vec![])
    } else if tweet.text.contains("https://t.co/") {
        Ok(vec![])
    } else if TRAIN_LINE_NAMES_RE.is_match_at(&tweet.text, 0) || tweet.text.starts_with("Trains") {
        // Don't care about trains
        Ok(vec![])
    } else if tweet.text.starts_with("Ferry WHF:") {
        Ok(vec![])
    } else if tweet.text.starts_with("Bus") || tweet.text.starts_with("School") {
        Ok(parse_bus_tweet(tweet)?)
    } else {
        Err("Not able to detect tweet type.".to_string())
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;

    lazy_static! {
        static ref SAMPLE_TIME: DateTime<Utc> =
            DateTime::parse_from_rfc3339("2021-01-24T21:00:06Z")
                .unwrap()
                .with_timezone(&Utc);
    }

    pub fn parse_tweet_str(text: &dyn ToString, expected: Vec<Cancellations>) {
        parse_tweet_time_str(*SAMPLE_TIME, text, expected)
    }

    pub fn parse_tweet_time_str(
        created_at: DateTime<Utc>,
        text: &dyn ToString,
        expected: Vec<Cancellations>,
    ) {
        let text = text.to_string();
        let tweet = TweetContent {
            id: 1353447509805342721,
            created_at,
            text: text.clone(),
        };
        assert_eq!((&text, parse_tweet(&tweet)), (&text, Ok(expected)));
    }

    #[test]
    fn test_time_re() {
        assert!(Regex::new(TIME_RE).is_ok());
        assert!(Regex::new(TIME_RE).unwrap().is_match("5.30pm"));
        assert!(Regex::new(TIME_RE).unwrap().is_match("5:30pm"));
        assert!(Regex::new(TIME_RE).unwrap().is_match("12.30am"));
        assert!(Regex::new(TIME_RE).unwrap().is_match("12:30am"));
    }

    mod parse_tweet {

        use super::*;

        #[test]
        fn test_cancelled_bus() {
            parse_tweet_str(
                &"Bus 3: Bus 3: 10:30am Wellington Station to Lyall Bay is cancelled. Please check RTI for next available bus.",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Lyall Bay".to_string(),
                    raw_time: "10:30 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 10, 30).unwrap()
                }]
            );
        }
        #[test]
        fn test_cancelled_bus_alt() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 27: Bus 27: 5:23pm Kingston - Wellington Stn has been cancelled. Please check RTI for next available service.",
                vec![Cancellations::BusCancelled {
                    route: "27".to_string(),
                    origin: "Kingston".to_string(),
                    destination: "Wellington Stn".to_string(),
                    raw_time: "5:23 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 17, 23).unwrap()
                }]
            );
        }
        #[test]
        fn test_cancelled_bus_alt2() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 3: Bus 3: 6pm Tirangi Road to Wellington Station has been cancelled.",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Tirangi Road".to_string(),
                    destination: "Wellington Station".to_string(),
                    raw_time: "6:00 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 18, 00).unwrap(),
                }],
            );
        }
        #[test]
        fn test_cancelled_bus_alt3() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 83: Bus 83: 8:35pm Courtenay Place tp Eastbourne is cancelled. Check RTI for next service.",
                vec![Cancellations::BusCancelled {
                    route: "83".to_string(),
                    origin: "Courtenay Place".to_string(),
                    destination: "Eastbourne".to_string(),
                    raw_time: "8:35 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 20, 35).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt4() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 14: Bus 14: 1:30pm Wilton to Kilbirnie was cancelled. Please check RTI for next available bus.",
                vec![Cancellations::BusCancelled {
                    route: "14".to_string(),
                    origin: "Wilton".to_string(),
                    destination: "Kilbirnie".to_string(),
                    raw_time: "1:30 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 13, 30).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt5() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 3: Bus 3: 3:40pm Tirangi Road to Wellington Station cancelled. Please check RTI for next available bus.",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Tirangi Road".to_string(),
                    destination: "Wellington Station".to_string(),
                    raw_time: "3:40 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 15, 40).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt6() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 12: Bus 12: 12:48pm Strathmore to Kilbirnie  is cancelled. Please check RTI for next service.",
                vec![Cancellations::BusCancelled {
                    route: "12".to_string(),
                    origin: "Strathmore".to_string(),
                    destination: "Kilbirnie".to_string(),
                    raw_time: "12:48 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 12, 48).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt7() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());
            let tweet_time = DateTime::parse_from_rfc3339("2021-02-04T03:58:06Z")
                .unwrap()
                .into();
            parse_tweet_time_str(
                tweet_time,
                &"Bus 3: Bus 3: 5:10 Lyall Bay - Wellington Stn has been cancelled. Please check RTI for next available service.",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Lyall Bay".to_string(),
                    destination: "Wellington Stn".to_string(),
                    raw_time: "5:10 pm".to_string(),
                    tweet_time,
                    time: convert_time_to_instant(tweet_time, 17, 10).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt8() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 17: Bua 17: 8:20pm Wellington Station to Kowhai Park is cancelled. Please check RTI for next service.",
                vec![Cancellations::BusCancelled {
                    route: "17".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Kowhai Park".to_string(),
                    raw_time: "8:20 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 20, 20).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt9() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 22: Bus 22 8pm: Wellington Station to Mairangi is cancelled. Please check RTI for next service.",
                vec![Cancellations::BusCancelled {
                    route: "22".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Mairangi".to_string(),
                    raw_time: "8:00 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 20, 00).unwrap()
                }],
            );
        }
        #[test]
        fn test_cancelled_bus_alt10() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 3: Bus 3 3:50pm Wellington Station to Tirangi Road is cancelled. Check RTI to find next available bus",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Tirangi Road".to_string(),
                    raw_time: "3:50 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 15, 50).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt11() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 2: Bus 2 3:57pm Miramar to Karori is cancelled. Check RTI to find next available bus",
                vec![Cancellations::BusCancelled {
                    route: "2".to_string(),
                    origin: "Miramar".to_string(),
                    destination: "Karori".to_string(),
                    raw_time: "3:57 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 15, 57).unwrap()
                }],
            );
        }
        #[test]
        fn test_cancelled_bus_alt12() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 3: Bus 3 3:50pm Wellington Station to Tirangi Road is cancelled. Check RTI to find next available bus",
                vec![Cancellations:: BusCancelled {
                    route: "3".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Tirangi Road".to_string(),
                    raw_time: "3:50 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 15, 50).unwrap()
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt13() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 1: 6:36pm Grenada Village to Island Bay is cancelled. Check RTI to find next available bus",
                vec![Cancellations:: BusCancelled {
                    route: "1".to_string(),
                    origin: "Grenada Village".to_string(),
                    destination: "Island Bay".to_string(),
                    raw_time: "6:36 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 18, 36).unwrap()
                }],
            );
        }

        #[test]
        fn test_reinstated_bus() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 27: Bus 27: 5:23pm Kingston - Wellington Stn has been REINSTATED.",
                vec![Cancellations::BusReinstated {
                    route: "27".to_string(),
                    origin: "Kingston".to_string(),
                    destination: "Wellington Stn".to_string(),
                    raw_time: "5:23 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 17, 23).unwrap(),
                }],
            );
        }

        #[test]
        fn test_reinstated_bus_alt() {
            parse_tweet_str(
                &"Bus 14: Bus 14: 7:43am Kilbirnie to Wilton has been reinstated and will now run.",
                vec![Cancellations::BusReinstated {
                    route: "14".to_string(),
                    origin: "Kilbirnie".to_string(),
                    destination: "Wilton".to_string(),
                    raw_time: "7:43 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 7, 43).unwrap(),
                }],
            );
        }
        #[test]
        fn test_reinstated_bus_alt2() {
            parse_tweet_str(
                &"Bus 1: Bus 1: 8:13am Churton Park to Island Bay that was cancelled will now run.",
                vec![Cancellations::BusReinstated {
                    route: "1".to_string(),
                    origin: "Churton Park".to_string(),
                    destination: "Island Bay".to_string(),
                    raw_time: "8:13 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 8, 13).unwrap(),
                }],
            );
        }
        #[test]
        fn test_reinstated_bus_alt3() {
            println!("{}", BUS_REINSTATED_RE.as_str());

            parse_tweet_str(
                &"Bus 160: Bus 160: 9:00am from Lower Hutt to Wainuiomata is REINSTATED.",
                vec![Cancellations::BusReinstated {
                    route: "160".to_string(),
                    origin: "Lower Hutt".to_string(),
                    destination: "Wainuiomata".to_string(),
                    raw_time: "9:00 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 9, 00).unwrap(),
                }],
            );
        }

        #[test]
        fn test_reinstated_bus_alt4() {
            println!("{}", BUS_REINSTATED_RE.as_str());

            parse_tweet_str(
                &"Bus 160: Bus 160: 8:23am from Wainuiomata to Lower Hutt is reinstated.",
                vec![Cancellations::BusReinstated {
                    route: "160".to_string(),
                    origin: "Wainuiomata".to_string(),
                    destination: "Lower Hutt".to_string(),
                    raw_time: "8:23 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 8, 23).unwrap(),
                }],
            );
        }

        #[test]
        fn test_delayed_bus() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 29: Bus 29: 7:00pm Brooklyn to Wellington Station is delayed by 20 minutes. Please check RTI for next available bus.",
                vec![Cancellations::BusDelayed {
                    route: "29".to_string(),
                    origin: "Brooklyn".to_string(),
                    destination: "Wellington Station".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "7:00 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 19, 00).unwrap()
                }],
            );
        }
        #[test]
        fn test_delayed_bus_alt() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 17: Bus 17: 5:03pm Wellington Station - Kowhai Park has been delayed 20 minutes due to mechanical issues. Please check RTI for updates.",
                vec![Cancellations::BusDelayed {
                    route: "17".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Kowhai Park".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "5:03 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 17, 03).unwrap()
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt2() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 220: Bus 220: 7:10am Titahi Bay to Ascot Park will be delayed 20 minutes. Sorry for the inconvenience!",
                vec![Cancellations::BusDelayed {
                    route: "220".to_string(),
                    origin: "Titahi Bay".to_string(),
                    destination: "Ascot Park".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "7:10 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 7, 10).unwrap()
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt3() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 130: Bus 130: 11:00am from Naenae to Petone is delayed 15-20 minutes due to mechanical issues. Check RTI for updates.",
                vec![Cancellations::BusDelayed {
                    route: "130".to_string(),
                    origin: "Naenae".to_string(),
                    destination: "Petone".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "11:00 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 11, 00).unwrap()
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt4() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 130: Bus 130: 10:15am from Petone to Naenae is delayed due to vehicle breakdown. Check RTI for updates.",
                vec![Cancellations::BusDelayed {
                    route: "130".to_string(),
                    origin: "Petone".to_string(),
                    destination: "Naenae".to_string(),
                    delay_minutes: "".to_string(),
                    raw_time: "10:15 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 10, 15).unwrap()
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt5() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 25: Bus 25: 7:05am Khandallah to Highbury will run 15 minutes late.",
                vec![Cancellations::BusDelayed {
                    route: "25".to_string(),
                    origin: "Khandallah".to_string(),
                    destination: "Highbury".to_string(),
                    delay_minutes: "15".to_string(),
                    raw_time: "7:05 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 7, 05).unwrap(),
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt6() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &"Bus 220: 1.14pm Ascot Park to Titahi Bay is delayed. Please check RTI for updates on this service.",
                vec![Cancellations::BusDelayed {
                    route: "220".to_string(),
                    origin: "Ascot Park".to_string(),
                    destination: "Titahi Bay".to_string(),
                    delay_minutes: "".to_string(),
                    raw_time: "1:14 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 13, 14).unwrap(),
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt7() {
            println!("{}", BUS_DELAYED_UNDETERMINATE_RE.as_str());

            parse_tweet_str(
                &"Bus 24: Bus 24: 9:10am Miramar - Johnsonville has been delayed due to road block. Please check RTI for updates.",
                vec![Cancellations::BusDelayed {
                    route: "24".to_string(),
                    origin: "Miramar".to_string(),
                    destination: "Johnsonville".to_string(),
                    delay_minutes: "".to_string(),
                    raw_time: "9:10 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 9, 10).unwrap(),
                }],
            );
        }

        #[test]
        fn test_part_cancelled_bus() {
            println!("{}", BUS_PART_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 2: Bus 2: 6:15pm Karori - Miramar has been part cancelled from Rongotai. Please check RTI for next available service.",
                vec![Cancellations::BusPartCancelled {
                    route: "2".to_string(),
                    origin: "Karori".to_string(),
                    destination: "Miramar".to_string(),
                    cancelled_from: "Rongotai".to_string(),
                    cancelled_to: "Miramar".to_string(),
                    raw_time: "6:15 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 18, 15).unwrap()
                }],
            );
        }

        #[test]
        fn test_part_cancelled_bus_alt() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                &"Bus 2: Bus 2: 2:50pm Karori - Miramar is part cancelled between Kilbrinie and Miramar. Please check RTI for next service.",
                vec![Cancellations::BusPartCancelled {
                    route: "2".to_string(),
                    origin: "Karori".to_string(),
                    destination: "Miramar".to_string(),
                    cancelled_from: "Kilbrinie".to_string(),
                    cancelled_to: "Miramar".to_string(),
                    raw_time: "2:50 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 14, 50).unwrap()
                }],
            );
        }
        #[test]
        fn test_part_cancelled_bus_alt2() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                &"Bus 29: Bus 29: 6.20pm from Wellington Stn to Island Bay is part-cancelled between Wgtn Stn and Courtenay Pl. Check RTI to find next service.",
                vec![Cancellations::BusPartCancelled {
                    route: "29".to_string(),
                    origin: "Wellington Stn".to_string(),
                    destination: "Island Bay".to_string(),
                    cancelled_from: "Wgtn Stn".to_string(),
                    cancelled_to: "Courtenay Pl".to_string(),
                    raw_time: "6:20 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 18, 20).unwrap()
                }],
            );
        }
        #[test]
        fn test_part_cancelled_bus_alt3() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                &"Bus 220: Bus 220: 9:11am Titahi Bay to Ascot Park will be part cancelled between Titahi Bay and Porirua Station. Please check RTI",
                vec![Cancellations::BusPartCancelled {
                    route: "220".to_string(),
                    origin: "Titahi Bay".to_string(),
                    destination: "Ascot Park".to_string(),
                    cancelled_from: "Titahi Bay".to_string(),
                    cancelled_to: "Porirua Station".to_string(),
                    raw_time: "9:11 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 9, 11).unwrap()
                }],
            );
        }
        #[test]
        fn test_part_cancelled_bus_alt4() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                &"Bus 110: Bus 110: 5pm Emerald Hill to Petone. Is part-cancelled from Emerald hill to Upper Hutt.",
                vec![Cancellations::BusPartCancelled {
                    route: "110".to_string(),
                    origin: "Emerald Hill".to_string(),
                    destination: "Petone".to_string(),
                    cancelled_from: "Emerald hill".to_string(),
                    cancelled_to: "Upper Hutt".to_string(),
                    raw_time: "5:00 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 17, 00).unwrap()
                }],
            );
        }
        #[test]
        fn test_part_cancelled_bus_alt5() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                & "Bus 2: Bus 2: 6:46pm Seatoun Park to Kilbirnie part-cancelled from Seatount to Rongotai Rd.",
                vec![Cancellations::BusPartCancelled {
                    route: "2".to_string(),
                    origin: "Seatoun Park".to_string(),
                    destination: "Kilbirnie".to_string(),
                    cancelled_from: "Seatount".to_string(),
                    cancelled_to: "Rongotai Rd".to_string(),
                    raw_time: "6:46 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                    time: convert_time_to_instant(*SAMPLE_TIME, 18, 46).unwrap()
                }],
            );
        }
    }
}
