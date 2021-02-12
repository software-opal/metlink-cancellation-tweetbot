use chrono::DateTime;
use chrono::Utc;
use egg_mode::tweet::Tweet;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::io::BufWriter;
use std::{collections::BTreeMap, fs::File};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize, Serialize, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct TweetContent {
    pub id: u64,
    pub created_at: DateTime<Utc>,
    pub text: String,
}
impl From<&Tweet> for TweetContent {
    fn from(t: &Tweet) -> Self {
        Self {
            id: t.id,
            created_at: t.created_at,
            text: t.text.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TweetCache {
    pub tweets: Vec<TweetContent>,
}

impl TweetCache {
    pub fn add_tweet(&mut self, tweet: &Tweet) {
        if !self.tweets.iter().any(|c| c.id == tweet.id) {
            self.tweets.push(TweetContent::from(tweet));
            self.tweets.sort();
        }
    }
    pub fn add_all_tweets(&mut self, tweets: &[Tweet]) {
        tweets.iter().for_each(|t| self.add_tweet(t));
    }
    pub fn write(&self) -> Result<()> {
        let file = File::create("./twitter-cache.json")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
    pub fn read() -> Result<Self> {
        match File::open("./twitter-cache.json") {
            Ok(file) => {
                let reader = BufReader::new(file);
                // Read the JSON contents of the file as an instance of `User`.
                let cache: TweetCache = serde_json::from_reader(reader)?;

                let tweets = cache
                    .tweets
                    .into_iter()
                    .map(|c| (c.id, c))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .map(|(_, c)| c)
                    .collect();

                Ok(TweetCache { tweets })
            }
            Err(_) => {
                let cache = TweetCache {
                    tweets: Vec::with_capacity(50),
                };
                cache.write()?;
                Ok(cache)
            }
        }
    }
    pub fn has_recent_data(&self) -> bool {
        let now = Utc::now();
        self.tweets
            .iter()
            .map(|tweet| now - tweet.created_at)
            .min()
            .map(|diff| diff.num_hours() < 2)
            .unwrap_or(false)
    }
}
