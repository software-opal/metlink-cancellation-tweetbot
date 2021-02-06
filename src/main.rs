use crate::tweet_cache::TweetCache;
use egg_mode::tweet::user_timeline;
use egg_mode::tweet::{Timeline, Tweet};
use std::fs::File;
use std::io::BufReader;

use egg_mode::Token;
use serde::Deserialize;

mod parser;
mod tweet_cache;

#[derive(Deserialize)]
struct TwitterTokenInfo {
    api_key: String,
    api_secret: String,
    access_token: String,
    access_secret: String,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn load_twitter_creds() -> Result<Token> {
    let file = File::open("./twitter-creds.json")?;
    let reader = BufReader::new(file);
    // Read the JSON contents of the file as an instance of `User`.
    let creds: TwitterTokenInfo = serde_json::from_reader(reader)?;

    let con_token = egg_mode::KeyPair::new(creds.api_key, creds.api_secret);
    let access_token = egg_mode::KeyPair::new(creds.access_token, creds.access_secret);
    Ok(egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    })
}

async fn load_tweets_from_timeline(
    cache: &mut TweetCache,
    timeline_: Timeline,
    back_to: Option<u64>,
) -> Result<()> {
    let mut timeline = timeline_;
    loop {
        println!("Fetching tweets {:?} to {:?}", timeline.min_id, back_to);
        let (inner_timeline, feed_) = timeline.older(back_to).await?;
        let feed: &Vec<Tweet> = &*feed_;
        if feed.is_empty() {
            return Ok(());
        }
        cache.add_all_tweets(feed);
        cache.write()?;
        timeline = inner_timeline;
    }
}

async fn load_tweets(token: &Token, cache: &mut TweetCache) -> Result<()> {
    let timeline = user_timeline("metlinkwgtn", false, false, token).with_page_size(200);

    load_tweets_from_timeline(
        cache,
        timeline,
        cache.tweets.iter().map(|tweet| tweet.id).max(),
    )
    .await?;

    cache.write()?;
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let creds = load_twitter_creds()?;
    let mut cache = TweetCache::read()?;

    println!("Has recent data: {}", cache.has_recent_data());

    if !cache.has_recent_data() {
        load_tweets(&creds, &mut cache).await?;
    }

    let cancellations: Vec<_> = cache
        .tweets
        .iter()
        .flat_map(|tweet| {
            println!("{:?}", &tweet);
            parser::parse_tweet(tweet)
        })
        .collect();
    serde_json::to_writer_pretty(File::create("twitter-cancellations.json")?, &cancellations);

    Ok(())
}
