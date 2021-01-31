use egg_mode::tweet::user_timeline;
use crate::tweet_cache::TweetCache;
use std::fs::File;
use std::io::BufReader;

use egg_mode::Token;
use serde::Deserialize;

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

async fn load_tweets(token: &Token, cache: &mut TweetCache) -> Result<()> {
    let timeline = user_timeline("metlinkwgtn", false, false, token).with_page_size(200);

    if cache.tweets.is_empty() {
        let (timeline, feed) = timeline.start().await.unwrap();
        for tweet in &*feed {
            println!("{:#?}", tweet);
            cache.add_tweet(tweet);
        }
        cache.write()?;
    } 

    




    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let creds = load_twitter_creds()?;
    let mut cache = TweetCache::read()?;
 load_tweets(&creds, &mut cache).await?;
    Ok(())
}
