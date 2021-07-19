use std::fs::File;
use std::io;
use std::path::Path;

use log::info;
use serde::de::DeserializeOwned;
use zip::ZipArchive;

use super::data::GtfsData;
use crate::error::{Error, Result};

fn load_csv<'a, T, R>(archive: &mut ZipArchive<R>, filename: &str) -> Result<Vec<T>>
where
    T: DeserializeOwned,
    R: io::Read + io::Seek,
{
    info!("Loading {}", filename);
    let r = archive
        .by_name(filename)
        .map_err(|err| Error::InvalidGtfsFile(filename.to_string(), err))?;

    let mut output: Vec<T> = Vec::with_capacity(16);
    for record in csv::Reader::from_reader(r).deserialize() {
        output.push(record?);
    }
    info!("Loaded {} entries from {}", output.len(), filename);
    Ok(output)
}

pub fn load_gtfs_zip(zip_file: &Path) -> Result<GtfsData> {
    let mut archive = ZipArchive::new(File::open(zip_file)?)?;

    log::info!(
        "Zip file contains these files: {:?}",
        archive.file_names().collect::<Vec<_>>()
    );

    Ok(GtfsData {
        agency: load_csv(&mut archive, "agency.txt")?,
        calendar: load_csv(&mut archive, "calendar.txt")?,
        calendar_date: load_csv(&mut archive, "calendar_dates.txt")?,
        feed_info: load_csv(&mut archive, "feed_info.txt")?,
        route: load_csv(&mut archive, "routes.txt")?,
        stop: load_csv(&mut archive, "stops.txt")?,
        stop_time: load_csv(&mut archive, "stop_times.txt")?,
        trip: load_csv(&mut archive, "trips.txt")?,
    })
    // "stop_times.txt" => {}
    // "transfers.txt" => {}
    // "trips.txt" => {}

    // "shapes.txt" => {}
    // name => info!("Extra file found: {}", name),
}
