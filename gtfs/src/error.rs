use thiserror::Error;
use super::gtfs::error::Error as GtfsError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to load GTFS file: {0:?})")]
    GtfsLoadError(        #[from] GtfsError)
}