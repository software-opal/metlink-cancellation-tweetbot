use super::utils::deserialize_date;
use serde::{Deserialize, Deserializer, Serialize};
use time::Date;

#[derive(Debug, Deserialize, Serialize)]

pub struct FeedInfo {
    feed_publisher_name: String,
    feed_publisher_url: String,
    feed_lang: String,
    #[serde(deserialize_with = "deserialize_date")]
    feed_start_date: Date,
    #[serde(deserialize_with = "deserialize_date")]
    feed_end_date: Date,
    feed_version: String,
}
