use std::{fs::File, path::PathBuf};

use reqwest::Client;
use tokio::fs::create_dir_all;

use crate::gtfs::data::GtfsData;

pub mod db;
pub mod error;
pub mod gtfs;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let cache_dir = PathBuf::from("./.cache");
    create_dir_all(&cache_dir).await.unwrap();
    let c = gtfs::load::load_gtfs(&cache_dir, &client).await.unwrap();

    let GtfsData {
        agency,
        calendar,
        calendar_date,
        feed_info,
        route,
        stop_pattern,
        stop_pattern_trip,
        stop,
        stop_time,
        trip,
    } = c;

    println!("agency: {:#?}", agency.get(0));
    println!("calendar: {:#?}", calendar.get(0));
    println!("calendar_date: {:#?}", calendar_date.get(0));
    println!("feed_info: {:#?}", feed_info.get(0));
    println!("route: {:#?}", route.get(0));
    println!("stop_pattern: {:#?}", stop_pattern.get(0));
    println!("stop_pattern_trip: {:#?}", stop_pattern_trip.get(0));
    println!("stop: {:#?}", stop.get(0));
    println!("stop_time: {:#?}", stop_time.get(0));
    println!("trip: {:#?}", trip.get(0));

    serde_json::to_writer_pretty(
        File::create(cache_dir.join("agency.json")).unwrap(),
        &agency,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("calendar.json")).unwrap(),
        &calendar,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("calendar_date.json")).unwrap(),
        &calendar_date,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("feed_info.json")).unwrap(),
        &feed_info,
    )
    .unwrap();
    serde_json::to_writer_pretty(File::create(cache_dir.join("route.json")).unwrap(), &route)
        .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("stop_pattern.json")).unwrap(),
        &stop_pattern,
    )
    .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("stop_pattern_trip.json")).unwrap(),
        &stop_pattern_trip,
    )
    .unwrap();
    serde_json::to_writer_pretty(File::create(cache_dir.join("stop.json")).unwrap(), &stop)
        .unwrap();
    serde_json::to_writer_pretty(
        File::create(cache_dir.join("stop_time.json")).unwrap(),
        &stop_time,
    )
    .unwrap();
    serde_json::to_writer_pretty(File::create(cache_dir.join("trip.json")).unwrap(), &trip)
        .unwrap();
}
