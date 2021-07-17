use std::{io::ErrorKind, path::Path, time::SystemTime};

use time::OffsetDateTime;
use tokio::fs::File;


pub async fn file_mod_time(file: &Path) -> crate::error::Result<Option<OffsetDateTime>> {
    match File::open(file).await {
        Ok(file) => {
            let metadata = file.metadata().await?;
            match metadata.modified() {
                Ok(mod_time) => match mod_time.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => {
                        let datetime =
                            OffsetDateTime::from_unix_timestamp(duration.as_secs() as i64);
                        Ok(Some(datetime))
                    }
                    Err(_) => Ok(None),
                },
                Err(_) => Ok(None),
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(e)?,
        },
    }
}
