use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::DeserializeOwned;
use zip::ZipArchive;

use super::data::GtfsData;
use super::error::{Error, Result};

fn load_csv<'a, T, R>(r: R) -> Result<Vec<T>>
where
    T: DeserializeOwned,
    R: Read,
{
    let mut output: Vec<T> = Vec::with_capacity(16);
    for record in csv::Reader::from_reader(r).deserialize() {
        output.push(record?);
    }
    Ok(output)
}

pub fn load_gtfs_zip(zip_file: &Path) -> Result<GtfsData> {
    let mut archive = ZipArchive::new(File::open(zip_file)?)?;

    let mut agency = None;
    let mut calendar = None;

    for idx in 0..archive.len() {
        let file = archive.by_index(idx)?;
        match file.name() {
            "agency.txt" => agency = Some(load_csv(file)?),
            "calendar.txt" => calendar = Some(load_csv(file)?),
            _ => {}
        }
    }

    match (agency, calendar) {
        (Some(agency), Some(calendar)) => Ok(GtfsData { agency, calendar }),
        (None, _) => Err(Error::InvalidGtfsFile(
            "No agency found in archive".to_string(),
        ))?,
        (_, None) => Err(Error::InvalidGtfsFile(
            "No calendar found in archive".to_string(),
        ))?,
    }
}
