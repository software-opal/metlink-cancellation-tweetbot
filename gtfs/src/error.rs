use super::gtfs::error::Error as GtfsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to load GTFS file: {0:?})")]
    GtfsLoadError(#[from] GtfsError),
}
