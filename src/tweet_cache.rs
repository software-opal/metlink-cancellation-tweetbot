use crate::BufReader;
use chrono::DateTime;
use chrono::Utc;
use egg_mode::tweet::Tweet;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize, Serialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct TweetContent {
  pub  id: u64,
  pub  created_at: DateTime<Utc>,
  pub  text: String,
}
impl From<&Tweet> for TweetContent {
    fn from(t: &Tweet) -> Self {
        Self {
            id: t.id,
            created_at: t.created_at,
            text: t.text,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TweetCache {
  pub  tweets: Vec<TweetContent>,
}

impl TweetCache {
    pub fn add_tweet(&mut self, tweet: &Tweet) {
        self.tweets.push(tweet.into());
        self.tweets.sort();
    }
    pub fn write(&self) -> Result<()> {
        let file = File::create("./twitter-cache.json")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
    pub fn read() -> Result<Self> {
        let file = File::open("./twitter-cache.json")?;
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an instance of `User`.
        let cache = serde_json::from_reader(reader)?;
        Ok(cache)
    }
}
