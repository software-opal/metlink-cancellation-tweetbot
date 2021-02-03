use crate::tweet_cache::TweetContent;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexSet};
use std::unimplemented;

// Turns out am/pm is called the "period". Thanks StackExchange
// https://english.stackexchange.com/questions/35315/what-is-the-proper-name-for-am-and-pm#35317
const TIME_RE: &str = r"(?P<hour>[0-9]{1,2})[.:](?P<minute>[0-9]{2})(?P<period>am|pm)";
const BUS_NUM_RE: &str = r"Bus (?P<bus_num>[0-9ex]+)";
const BUS_DEST_RE: &str = r"(?P<origin>.*?) to (?P<destination>.*?) (?:is|has been) cancelled";
const TRAIN_LINE_NAMES: [&str; 5] = ["WRL", "KPL", "HVL", "JVL", "MEL"];
const TRAIN_SERVICE: &str = r"(?P<origin>[A-Z]{3,4}) to (?P<destination>[A-Z]{3,4})";

lazy_static! {
    static ref TRAIN_LINE_NAME: String = TRAIN_LINE_NAMES.join("|");

    static ref BUS_FULL_CANCELLED_RE: Regex =
        Regex::new(&format!("{}: {} {}", BUS_NUM_RE, TIME_RE, BUS_DEST_RE)).unwrap();
    static ref TRAIN_LINE_NAMES_RE: RegexSet = RegexSet::new(&TRAIN_LINE_NAMES).unwrap();
    static ref TRAIN_REPLACEMENT_BUS_DELAY_RE: Regex = Regex::new(&format!(
        "(?P<route>{}): .* expect delays to (?P<direction>northbound|southbound|north and southbound) bus replacement",
        *TRAIN_LINE_NAME
    ))    .unwrap();
    static ref TRAIN_REPLACED_BY_BUS_RE: Regex = Regex::new(&format!("(?P<route>{}): (?P<inner>.* {} {} .*) replaced by bus", *TRAIN_LINE_NAME, TIME_RE, TRAIN_SERVICE)).unwrap();
    static ref TRAIN_SERVICE_TIME_RE: Regex = Regex::new(&format!("{} {}", TIME_RE, TRAIN_SERVICE)).unwrap();
    static ref TRAIN_DELAYED_RE: Regex = Regex::new(&format!("{}: {} svc from {}", TIME_RE, TRAIN_SERVICE)).unwrap();
}

#[derive(Clone, PartialEq, Debug)]
pub enum Cancellations {
    BusCancelled {
        route: String,
        origin: String,
        destination: String,
        raw_time: String,
        tweet_time: DateTime<Utc>, // time: DateTime
    },
    TrainReplacementBusDelay {
        route: String,
        direction: String,
        tweet_time: DateTime<Utc>, // time: DateTime
    },
    TrainReplacementBus {
        route: String,
        origin: String,
        destination: String,
        raw_time: String,
        tweet_time: DateTime<Utc>, // time: DateTime
    },
}

fn time_from_capture(capture: &Captures) -> String {
    format!(
        "{}:{} {}",
        capture.name("hour").unwrap().as_str(),
        capture.name("minute").unwrap().as_str(),
        capture.name("period").unwrap().as_str()
    )
}

fn parse_bus_tweet(tweet: &TweetContent) -> Vec<Cancellations> {
    BUS_FULL_CANCELLED_RE
        .captures(&tweet.text)
        .map(|capture| {
            vec![Cancellations::BusCancelled {
                route: capture.name("bus_num").unwrap().as_str().to_string(),
                origin: capture.name("origin").unwrap().as_str().to_string(),
                destination: capture.name("destination").unwrap().as_str().to_string(),
                raw_time: time_from_capture(&capture),
                tweet_time: tweet.created_at,
            }]
        })
        .unwrap_or_default()
}

fn parse_train_tweet(tweet: &TweetContent) -> Vec<Cancellations> {
    println!("{}", TRAIN_REPLACEMENT_BUS_DELAY_RE.as_str());
    None.or_else(|| {
        TRAIN_REPLACEMENT_BUS_DELAY_RE
            .captures(&tweet.text)
            .map(|capture| {
                let direction = capture.name("direction").unwrap().as_str().to_string();
                if direction == "north and southbound" {
                    vec![
                        Cancellations::TrainReplacementBusDelay {
                            route: capture.name("route").unwrap().as_str().to_string(),
                            direction: "northbound".to_string(),
                            tweet_time: tweet.created_at,
                        },
                        Cancellations::TrainReplacementBusDelay {
                            route: capture.name("route").unwrap().as_str().to_string(),
                            direction: "southbound".to_string(),
                            tweet_time: tweet.created_at,
                        },
                    ]
                } else {
                    vec![Cancellations::TrainReplacementBusDelay {
                        route: capture.name("route").unwrap().as_str().to_string(),
                        direction,
                        tweet_time: tweet.created_at,
                    }]
                }
            })
    })
    .or_else(|| {
        TRAIN_REPLACED_BY_BUS_RE
            .captures(&tweet.text)
            .map(|capture| {
                let route = capture.name("route").unwrap().as_str();
                let inner = capture.name("inner").unwrap().as_str().to_string();
                TRAIN_SERVICE_TIME_RE
                    .captures_iter(&inner)
                    .map(|capture| Cancellations::TrainReplacementBus {
                        route: route.to_string(),
                        origin: capture.name("origin").unwrap().as_str().to_string(),
                        destination: capture.name("destination").unwrap().as_str().to_string(),
                        raw_time: time_from_capture(&capture),
                        tweet_time: tweet.created_at,
                    })
                    .collect()
            })
    })
    .unwrap_or_default()
}

pub fn parse_tweet(tweet: &TweetContent) -> Vec<Cancellations> {
    if tweet.text.starts_with("Bus") {
        parse_bus_tweet(tweet)
    } else if TRAIN_LINE_NAMES_RE.is_match_at(&tweet.text, 0) {
        parse_train_tweet(tweet)
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
        fn test_train_replacement_delayed() {
            parse_tweet_str(
                &"MEL: MEL: Please expect delays to northbound bus replacement services due to a traffic accident at Horokiwi",
                vec![Cancellations::TrainReplacementBusDelay {
                    route: "MEL".to_string(),
                    direction: "northbound".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }]
            );
        }

        #[test]
        fn test_train_replacement_delayed_alt() {
            parse_tweet_str(
                &"MEL: MEL: Due to traffic congestion, please expect delays to north and southbound bus replacement services" ,
                vec![Cancellations::TrainReplacementBusDelay {
                    route: "MEL".to_string(),
                    direction: "northbound".to_string(),
                    tweet_time: *SAMPLE_TIME,
                },Cancellations::TrainReplacementBusDelay {
                    route: "MEL".to_string(),
                    direction: "southbound".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }]
            );
        }

        #[test]
        fn test_train_replacement() {
            parse_tweet_str(
                &"WRL: WRL: Due to an operational issue the 3.38pm MAST to WELL and the 6.18pm WELL to MAST services are replaced by bus.",
                vec![Cancellations::TrainReplacementBus {
                    route: "WRL".to_string(),
                    origin: "MAST".to_string(),
                    destination: "WELL".to_string(),
                    raw_time: "3:38 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                },Cancellations::TrainReplacementBus {
                    route: "WRL".to_string(),
                    origin: "WELL".to_string(),
                    destination: "MAST".to_string(),
                    raw_time: "6:18 pm".to_string(),
                    tweet_time: *SAMPLE_TIME,
                }]
            );
        }
    }
}
