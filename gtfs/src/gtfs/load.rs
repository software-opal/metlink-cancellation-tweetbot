
const METLINK_GTFS_URL: &str = "http://www.metlink.org.nz/assets/Google_Transit/google-transit.zip";
const GTFS_ZIP_FILE: &str = "metlink-gtfs.zip";
const GTFS_ETAG_FILE: &str = format!("{}.txt", GTFS_ZIP_FILE);



fn load_gtfs(cache_dir: &Path, client: reqwest::Client) {
    // cache_dir.
    let mut req_builder = client.get(METLINK_GTFS_URL);
    // req_builder = req_builder.header("If-None-Match", )


}
