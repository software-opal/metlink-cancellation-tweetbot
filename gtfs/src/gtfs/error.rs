use std::io;

use thiserror::Error;
// use tokio::task::JoinError;
use csv;
use zip::{self, result::ZipError};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to load file: {0:?})")]
    Io(#[from] io::Error),
    #[error("Unable to parse CSV: {0:?})")]
    Csv(#[from] csv::Error),
    #[error("Unable to load ZIP: {0:?})")]
    Zip(#[from] ZipError),

    #[error("Invalid GTFS Content. Missing {0:?}: {1:?}")]
    InvalidGtfsFile(String, ZipError),
    #[error("Requests error: {0:?})")]
    Reuest(#[from] reqwest::Error),
    // Async(#[from] JoinError)
}

pub type Result<T> = std::result::Result<T, Error>;
