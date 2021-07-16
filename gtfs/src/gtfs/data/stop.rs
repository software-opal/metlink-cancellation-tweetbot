use super::time::Time;
use super::utils::{deserialize_date, deserialize_num_bool};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum StopLocationType {
    /// A location where passengers board or disembark from a transit vehicle. Is called a platform when defined within a parent_station.
    StopOrPlatform,
    /// A physical structure or area that contains one or more platform.
    Station,
    /// A location where passengers can enter or exit a station from the street. If an entrance/exit belongs to multiple stations, it can be linked by pathways to both, but the data provider must pick one of them as parent.
    EntranceExit,
    /// A location within a station, not matching any other location_type, which can be used to link together pathways define in pathways.txt.
    GenericNode,
    /// A specific location on a platform, where passengers can board and/or alight vehicles
    BoardingArea,
}

pub fn deserialize_stop_location_type<'de, D>(deserializer: D) -> Result<StopLocationType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    match s {
        0 => Ok(StopLocationType::StopOrPlatform),
        1 => Ok(StopLocationType::Station),
        2 => Ok(StopLocationType::EntranceExit),
        3 => Ok(StopLocationType::GenericNode),
        4 => Ok(StopLocationType::BoardingArea),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid stop location type: {}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stop {
    stop_id: String,
    stop_code: String,
    stop_name: String,
    stop_desc: String,
    stop_lat: f64,
    stop_lon: f64,
    zone_id: String,
    stop_url: String,
    #[serde(deserialize_with = "deserialize_stop_location_type")]
    location_type: StopLocationType,
    parent_station: String,
    stop_timezone: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum PickupDropoffType {
    Regular,
    NotAvaliable,
    PhoneAgency,
    CoordinateDriver,
}

pub fn deserialize_pickup_dropoff_type<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<u8> = Deserialize::deserialize(deserializer)?;
    match s {
        0 => Ok(PickupDropoffType::Regular),
        1 => Ok(PickupDropoffType::NotAvaliable),
        2 => Ok(PickupDropoffType::PhoneAgency),
        3 => Ok(PickupDropoffType::CoordinateDriver),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid pickup/dropoff type: {}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopTime {
    trip_id: String,
    arrival_time: Time,
    departure_time: Time,
    stop_id: String,
    stop_sequence: u16,
    #[serde(deserialize_with = "deserialize_pickup_dropoff_type")]
    pickup_type: PickupDropoffType,
    #[serde(deserialize_with = "deserialize_pickup_dropoff_type")]
    drop_off_type: PickupDropoffType,
    shape_dist_traveled: f64,
    stop_headsign: String,
    #[serde(deserialize_with = "deserialize_num_bool")]
    timepoint: bool,
}
