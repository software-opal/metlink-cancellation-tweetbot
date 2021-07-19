use time::OffsetDateTime;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertRoot {
    pub header: ServiceAlertHeader,
    pub entity: Vec<ServiceAlertEntity>,
}

impl ServiceAlertRoot {
    pub fn entities<'a>(&'a self) -> impl Iterator<Item = &'a ServiceAlert> {
        self.entity.iter().map(|e| &e.alert)
    }
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

const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

pub fn deserialize_offset_timestamp<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    OffsetDateTime::parse(&s, TIMESTAMP_FORMAT).map_err(|e| {
        serde::de::Error::custom(format!(
            "Unable to parse timestamp {:?}: {:?}",
            s,
            e.to_string()
        ))
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertHeader {
    pub gtfs_realtime_version: String,
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    #[serde(deserialize_with = "deserialize_service_alert_incrementability")]
    pub incrementality: ServiceAlertIncrementability,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlertEntity {
    pub alert: ServiceAlert,
    pub id: String,
    #[serde(deserialize_with = "deserialize_offset_timestamp")]
    pub timestamp: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceAlertSeverity {
    UnknownSeverity,
    Info,
    Warning,
    Severe,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceAlert {
    pub active_period: Vec<AlertTimeRange>,
    pub effect: ServiceAlertEffect,
    pub cause: ServiceAlertCause,
    pub description_text: Translatation,
    pub header_text: Translatation,
    pub informed_entity: Vec<AlertInformedEntity>,
    pub severity_level: ServiceAlertSeverity,
}
impl ServiceAlert {
    pub fn description(&self) -> &str {
        self.description_text.get(Some("en"))
    }
    pub fn header(&self) -> &str {
        self.header_text.get(Some("en"))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertTimeRange {
    #[serde(with = "time::serde::timestamp")]
    pub start: OffsetDateTime,
    #[serde(with = "time::serde::timestamp")]
    pub end: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Translatation {
    pub translation: Vec<TranslatedText>,
}

impl Translatation {
    pub fn get_by_language(&self, lang: &str) -> Option<&str> {
        self.translation.iter().find_map(|t| -> Option<&str> {
            if t.language == lang {
                Some(&t.text)
            } else {
                None
            }
        })
    }
    pub fn get_by_language_or_first(&self, lang: &str) -> Option<&str> {
        self.translation
            .iter()
            .find_map(|t| -> Option<&str> {
                if t.language == lang {
                    Some(&t.text)
                } else {
                    None
                }
            })
            .or_else(|| self.translation.get(0).map(|t| -> &str { &t.text }))
    }
    pub fn get(&self, lang: Option<&str>) -> &str {
        if let Some(lang) = lang {
            self.get_by_language_or_first(lang).unwrap_or("")
        } else {
            ""
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TranslatedText {
    pub language: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AlertInformedEntity {
    Route { route_id: String, route_type: i32 },
    Stop { stop_id: String },
    Trip { trip: TripEntity },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TripEntity {
    pub trip_id: String,
}
