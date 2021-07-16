use std::io;

use thiserror::Error;
// use tokio::task::JoinError;
use csv;
use zip;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to load file: {0:?})")]
    Io(#[from] io::Error),
    #[error("Unable to parse CSV: {0:?})")]
    Csv(#[from] csv::Error),
    #[error("Unable to load ZIP: {0:?})")]
    Zip(#[from] zip::result::ZipError),

    #[error("Invalid GTFS Content File: {0:?})")]
    InvalidGtfsFile(String),
    #[error("Requests error: {0:?})")]
    Reuest(#[from] reqwest::Error),
    // Async(#[from] JoinError)
}

pub type Result<T> = std::result::Result<T, Error>;
