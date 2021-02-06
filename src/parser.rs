use crate::tweet_cache::TweetContent;
use chrono::{DateTime, Utc};
use chrono_tz::{Pacific::Auckland, Tz};
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexSet};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, unimplemented};

// Turns out am/pm is called the " delay_minutes: ()period". Thanks StackExchange
// https://english.stackexchange.com/questions/35315/what-is-the-proper-name-for-am-and-pm#35317
const TIME_RE: &str = r"(?P<hour>[0-9]{1,2})(?:[.:](?P<minute>[0-9]{2}))?(?P<period>am|pm|)";
const BUS_NUM_RE: &str = r"Bus ?(?P<bus_num>[0-9ex]+)";
const BUS_DEST_RE: &str = r"(?:from )?(?P<origin>.*?) (?:to|tp|-) (?P<destination>.*?)";
const TRAIN_LINE_NAMES: [&str; 5] = ["WRL", "KPL", "HVL", "JVL", "MEL"];

lazy_static! {
    static ref TRAIN_LINE_NAME: String = TRAIN_LINE_NAMES.join("|");
    static ref BUS_FULL_CANCELLED_RE: Regex = Regex::new(&format!(
        r"{}: +{} {} (?:(?:is|has been|was) |)cancelled",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_REINSTATED_RE: Regex = Regex::new(&format!(
        "{}: +{} {} (has been REINSTATED|has been reinstated and will now run|that was cancelled will now run)",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_DELAYED_RE: Regex = Regex::new(&format!(
        "{}: +{} {} (?:is|has been|will be) delayed(?: by)? (?P<delay_mins>[0-9]+) min",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_PART_CANCELLED_RE: Regex = Regex::new(&format!(
        "{}: +{} {} +(?:is|has been) part[- ]cancelled from (?P<cancelled_from>.*?)\\.",
        BUS_NUM_RE, TIME_RE, BUS_DEST_RE
    ))
    .unwrap();
    static ref BUS_PART_CANCELLED_BETWEEN_RE: Regex = Regex::new(&format!(
            "{}: +{} {}(?: is| has been| will be|. Is) part[- ]cancelled (?:between|from) (?P<cancelled_from>.*?) (?:and|to|&amp;|&) (?P<cancelled_to>.*?) *\\.",
            BUS_NUM_RE, TIME_RE, BUS_DEST_RE
        ))
    .unwrap();
    static ref BUS_PART_CANCELLED_BETWEEN_NO_ORIGIN_RE: Regex = Regex::new(&format!(
            "{}: +{} {} (?:is|has been) part[- ]cancelled between (?P<cancelled_from>.*?) (?:and|to|&amp;|&) (?P<cancelled_to>.*?) *\\.",
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
        // time: DateTime<Tz>,
    },
    BusPartCancelled {
        route: String,
        origin: String,
        destination: String,
        cancelled_from: String,
        cancelled_to: String,
        raw_time: String,
        tweet_time: DateTime<Utc>, // time: DateTime<Tz>,
    },
    BusReinstated {
        route: String,
        origin: String,
        destination: String,
        raw_time: String,
        tweet_time: DateTime<Utc>, // time: DateTime<Tz>,
    },
    BusDelayed {
        route: String,
        origin: String,
        destination: String,
        delay_minutes: String,
        raw_time: String,
        tweet_time: DateTime<Utc>, // time: DateTime<Tz>,
    },
}

fn time_from_capture(tweet: &TweetContent, capture: &Captures) -> (String, Option<()>) {
    (
        format!(
            "{}:{} {}",
            capture.name("hour").unwrap().as_str(),
            capture.name("minute").map(|m| m.as_str()).unwrap_or("00"),
            capture.name("period").unwrap().as_str()
        ),
        None,
    )
}

fn parse_bus_tweet(tweet: &TweetContent) -> Vec<Cancellations> {
    None.or_else(|| {
        BUS_REINSTATED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture);
            vec![Cancellations::BusReinstated {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
            }]
        })
    })
    .or_else(|| {
        BUS_DELAYED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture);
            vec![Cancellations::BusDelayed {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                delay_minutes: capture.name("delay_mins").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
            }]
        })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_BETWEEN_RE
            .captures(&tweet.text)
            .map(|capture| {
                let (raw_time, time) = time_from_capture(tweet, &capture);
                vec![Cancellations::BusPartCancelled {
                    route: capture.name("bus_num").unwrap().as_str().to_string(),
                    origin: capture.name("origin").unwrap().as_str().to_string(),
                    destination: capture.name("destination").unwrap().as_str().to_string(),
                    cancelled_from: capture.name("cancelled_from").unwrap().as_str().to_string(),
                    cancelled_to: capture.name("cancelled_to").unwrap().as_str().to_string(),
                    raw_time,
                    tweet_time: tweet.created_at,
                }]
            })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_BETWEEN_NO_ORIGIN_RE
            .captures(&tweet.text)
            .map(|capture| {
                let cancelled_from = capture.name("cancelled_from").unwrap().as_str().to_string();
                let (raw_time, time) = time_from_capture(tweet, &capture);
                vec![Cancellations::BusPartCancelled {
                    route: capture.name("bus_num").unwrap().as_str().to_string(),
                    origin: cancelled_from.clone(),
                    destination: capture.name("destination").unwrap().as_str().to_string(),
                    cancelled_from,
                    cancelled_to: capture.name("cancelled_to").unwrap().as_str().to_string(),
                    raw_time,
                    tweet_time: tweet.created_at,
                }]
            })
    })
    .or_else(|| {
        BUS_PART_CANCELLED_RE.captures(&tweet.text).map(|capture| {
            let destination = capture.name("destination").unwrap().as_str().to_string();
            let (raw_time, time) = time_from_capture(tweet, &capture);
            vec![Cancellations::BusPartCancelled {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: destination.clone(),
                cancelled_from: capture.name("cancelled_from").unwrap().as_str().to_string(),
                cancelled_to: destination,
                raw_time,
                tweet_time: tweet.created_at,
            }]
        })
    })
    .or_else(|| {
        BUS_FULL_CANCELLED_RE.captures(&tweet.text).map(|capture| {
            let (raw_time, time) = time_from_capture(tweet, &capture);
            vec![Cancellations::BusCancelled {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                raw_time,
                tweet_time: tweet.created_at,
            }]
        })
    })
    .or_else(|| {
        if tweet.text.contains("https://t.co/") || tweet.text.contains("buses cannot pass") {
            Some(vec![])
        } else {
            None
        }
    })
    .unwrap()
}

lazy_static! {
    static ref IGNORED_TWEET_IDS: HashSet<u64> = {
        let mut h = HashSet::new();
        h.extend(&[
            1356690879805681664,
            1354966519957000195,
            1354966526365892612,
            1356836623581806593,
            1357189987502944257,
            1357200754935758849,
        ]);
        h
    };
}

pub fn parse_tweet(tweet: &TweetContent) -> Vec<Cancellations> {
    if IGNORED_TWEET_IDS.contains(&tweet.id) {
        vec![]
    } else if tweet.text.starts_with("Bus") || tweet.text.starts_with("School") {
        parse_bus_tweet(tweet)
    } else if TRAIN_LINE_NAMES_RE.is_match_at(&tweet.text, 0) || tweet.text.starts_with("Trains") {
        // Don't care about trains
        vec![]
    } else if tweet.text.contains("https://t.co/") {
        vec![]
    } else {
        unimplemented!()
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
        let tweet = TweetContent {
            id: 1353447509805342721,
            created_at: *SAMPLE_TIME,
            text: text.to_string(),
        };
        assert_eq!(parse_tweet(&tweet), expected);
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
                }]
            );
        }
        #[test]
        fn test_cancelled_bus_alt() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                & "Bus 27: Bus 27: 5:23pm Kingston - Wellington Stn has been cancelled. Please check RTI for next available service.",
                vec![Cancellations::BusCancelled {
                    route: "27".to_string(),
                    origin: "Kingston".to_string(),
                    destination: "Wellington Stn".to_string(),
                    raw_time: "5:23 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
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
                }],
            );
        }

        #[test]
        fn test_cancelled_bus_alt5() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                & "Bus 3: Bus 3: 3:40pm Tirangi Road to Wellington Station cancelled. Please check RTI for next available bus.",
                vec![Cancellations::BusCancelled {
                    route: "3".to_string(),
                    origin: "Tirangi Road".to_string(),
                    destination: "Wellington Station".to_string(),
                    raw_time: "3:40 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
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
                }],
            );
        }

        #[test]
        fn test_reinstated_bus_alt() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 14: Bus 14: 7:43am Kilbirnie to Wilton has been reinstated and will now run.",
                vec![Cancellations::BusReinstated {
                    route: "14".to_string(),
                    origin: "Kilbirnie".to_string(),
                    destination: "Wilton".to_string(),
                    raw_time: "7:43 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }],
            );
        }
        #[test]
        fn test_reinstated_bus_alt2() {
            println!("{}", BUS_FULL_CANCELLED_RE.as_str());

            parse_tweet_str(
                &"Bus 1: Bus 1: 8:13am Churton Park to Island Bay that was cancelled will now run.",
                vec![Cancellations::BusReinstated {
                    route: "1".to_string(),
                    origin: "Churton Park".to_string(),
                    destination: "Island Bay".to_string(),
                    raw_time: "8:13 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
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
                }],
            );
        }
        #[test]
        fn test_delayed_bus_alt() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &
                "Bus 17: Bus 17: 5:03pm Wellington Station - Kowhai Park has been delayed 20 minutes due to mechanical issues. Please check RTI for updates.",
                vec![Cancellations::BusDelayed {
                    route: "17".to_string(),
                    origin: "Wellington Station".to_string(),
                    destination: "Kowhai Park".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "5:03 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }],
            );
        }

        #[test]
        fn test_delayed_bus_alt2() {
            println!("{}", BUS_DELAYED_RE.as_str());

            parse_tweet_str(
                &
                "Bus 220: Bus 220: 7:10am Titahi Bay to Ascot Park will be delayed 20 minutes. Sorry for the inconvenience!",
                vec![Cancellations::BusDelayed {
                    route: "220".to_string(),
                    origin: "Titahi Bay".to_string(),
                    destination: "Ascot Park".to_string(),
                    delay_minutes: "20".to_string(),
                    raw_time: "7:10 am".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }],
            );
        }

        #[test]
        fn test_part_cancelled_bus() {
            println!("{}", BUS_PART_CANCELLED_RE.as_str());

            parse_tweet_str(
                & "Bus 2: Bus 2: 6:15pm Karori - Miramar has been part cancelled from Rongotai. Please check RTI for next available service.",
                vec![Cancellations::BusPartCancelled {
                    route: "2".to_string(),
                    origin: "Karori".to_string(),
                    destination: "Miramar".to_string(),
                    cancelled_from: "Rongotai".to_string(),
                    cancelled_to: "Miramar".to_string(),
                    raw_time: "6:15 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }],
            );
        }

        #[test]
        fn test_part_cancelled_bus_alt() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                & "Bus 2: Bus 2: 2:50pm Karori - Miramar is part cancelled between Kilbrinie and Miramar. Please check RTI for next service.",
                vec![Cancellations::BusPartCancelled {
                    route: "2".to_string(),
                    origin: "Karori".to_string(),
                    destination: "Miramar".to_string(),
                    cancelled_from: "Kilbrinie".to_string(),
                    cancelled_to: "Miramar".to_string(),
                    raw_time: "2:50 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }],
            );
        }
        #[test]
        fn test_part_cancelled_bus_alt2() {
            println!("{}", BUS_PART_CANCELLED_BETWEEN_RE.as_str());

            parse_tweet_str(
                & "Bus 29: Bus 29: 6.20pm from Wellington Stn to Island Bay is part-cancelled between Wgtn Stn and Courtenay Pl. Check RTI to find next service.",
                vec![Cancellations::BusPartCancelled {
                    route: "29".to_string(),
                    origin: "Wellington Stn".to_string(),
                    destination: "Island Bay".to_string(),
                    cancelled_from: "Wgtn Stn".to_string(),
                    cancelled_to: "Courtenay Pl".to_string(),
                    raw_time: "6:20 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
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
                }],
            );
        }
    }
}
