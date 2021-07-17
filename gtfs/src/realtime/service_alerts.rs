use time::OffsetDateTime;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertRoot {
    header: ServiceAlertHeader,
    entity: Vec<ServiceAlertEntity>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ServiceAlertIncrementability {
    FullDataset,
    Differential,
}

pub fn deserialize_service_alert_incrementability<'de, D>(
    deserializer: D,
) -> Result<ServiceAlertIncrementability, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    match s {
        0 => Ok(ServiceAlertIncrementability::FullDataset),
        1 => Ok(ServiceAlertIncrementability::Differential),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid pickup/dropoff type: {:?}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertHeader {
    gtfs_realtime_version: String,
    timestamp: u64,
    #[serde(deserialize_with = "deserialize_service_alert_incrementability")]
    incrementality: ServiceAlertIncrementability,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertEntity {
    alert: ServiceAlert,
    id: String,
    timestamp: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceAlertEffect {
    NoService,
    ReducedService,
    SignificantDelays,
    Detour,
    AdditionalService,
    ModifiedService,
    OtherEffect,
    UnknownEffect,
    StopMoved,
    NoEffect,
    AccessibilityIssue,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceAlertCause {
    UnknownCause,
    OtherCause,
    TechnicalProblem,
    Strike,
    Demonstration,
    Accident,
    Holiday,
    Weather,
    Maintenance,
    Construction,
    PoliceActivity,
    MedicalEmergency,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceAlertSeverity {
    UnknownSeverity,
    Info,
    Warning,
    Severe,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlert {
    active_period: Vec<AlertTimeRange>,
    effect: ServiceAlertEffect,
    cause: ServiceAlertCause,
    description_text: Vec<TranslatedText>,
    header_text: Vec<TranslatedText>,
    informed_entity: AlertInformedEntity,
    severity_level: ServiceAlertSeverity,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertTimeRange {
    start: u64,
    end: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TranslatedText {
    language: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertInformedEntity {
    agency_id: String,
    route_id: String,
    route_type: i32,
    // trip: TripDescriptor,
    stop_id: String,
}
