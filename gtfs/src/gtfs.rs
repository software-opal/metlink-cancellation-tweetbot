use std::path::Path;

use serde::Serialize;

use self::data::GtfsData;

pub mod data;
pub mod load;
pub mod read;

pub async fn write_json<T>(cache_dir: &Path, name: &str, object: &T) -> crate::error::Result<()>
where
    T: Serialize,
{
    serde_json::to_writer_pretty(std::fs::File::create(cache_dir.join(name))?, object)?;
    Ok(())
}

pub async fn load_gtfs(
    cache_dir: &Path,
    client: &reqwest::Client,
) -> crate::error::Result<GtfsData> {
    let c = self::load::load_gtfs(&cache_dir, &client).await?;

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
    } = &c;

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

    let (
        agency_r,
        calendar_r,
        calendar_date_r,
        feed_info_r,
        route_r,
        stop_pattern_r,
        stop_pattern_trip_r,
        stop_r,
        stop_time_r,
        trip_r,
    ) = tokio::join!(
        write_json(cache_dir, "agency.json", &agency),
        write_json(cache_dir, "calendar.json", &calendar),
        write_json(cache_dir, "calendar_date.json", &calendar_date,),
        write_json(cache_dir, "feed_info.json", &feed_info),
        write_json(cache_dir, "route.json", &route),
        write_json(cache_dir, "stop_pattern.json", &stop_pattern,),
        write_json(cache_dir, "stop_pattern_trip.json", &stop_pattern_trip,),
        write_json(cache_dir, "stop.json", &stop),
        write_json(cache_dir, "stop_time.json", &stop_time),
        write_json(cache_dir, "trip.json", &trip),
    );

    agency_r?;
    calendar_r?;
    calendar_date_r?;
    feed_info_r?;
    route_r?;
    stop_pattern_r?;
    stop_pattern_trip_r?;
    stop_r?;
    stop_time_r?;
    trip_r?;

    Ok(c)
}
