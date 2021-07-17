use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum RouteType {
    /// Tram, Streetcar, Light rail. Any light rail or street level system within a metropolitan area.
    Tram,
    /// Subway, Metro. Any underground rail system within a metropolitan area.
    Subway,
    /// Rail. Used for intercity or long-distance travel.
    Rail,
    /// Bus. Used for short- and long-distance bus routes.
    Bus,
    /// Ferry. Used for short- and long-distance boat service.
    Ferry,
    /// Cable tram. Used for street-level rail cars where the cable runs beneath the vehicle, e.g., cable car in San Francisco.
    CableTram,
    /// Aerial lift, suspended cable car (e.g., gondola lift, aerial tramway). Cable transport where cabins, cars, gondolas or open chairs are suspended by means of one or more cables.
    AerialLift,
    /// Funicular. Any rail system designed for steep inclines.
    Funicular,
    /// Trolleybus. Electric buses that draw power from overhead wires using poles.
    Trolleybus,
    /// Monorail. Railway in which the track consists of a single rail or a beam.
    Monorail,
}

impl RouteType {
    pub fn pretty_name(&self) -> &'static str {
        match self {
            RouteType::Tram => "Tram",
            RouteType::Subway => "Subway",
            RouteType::Rail => "Rail",
            RouteType::Bus => "Bus",
            RouteType::Ferry => "Ferry",
            RouteType::CableTram => "Cable Tram",
            RouteType::AerialLift => "Aerial Lift",
            RouteType::Funicular => "Funicular",
            RouteType::Trolleybus => "Trolleybus",
            RouteType::Monorail => "Monorail",
        }
    }
}

pub fn deserialize_route_type<'de, D>(deserializer: D) -> Result<RouteType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u8 = Deserialize::deserialize(deserializer)?;
    match s {
        0 => Ok(RouteType::Tram),
        1 => Ok(RouteType::Subway),
        2 => Ok(RouteType::Rail),
        3 => Ok(RouteType::Bus),
        4 => Ok(RouteType::Ferry),
        5 => Ok(RouteType::CableTram),
        6 => Ok(RouteType::AerialLift),
        7 => Ok(RouteType::Funicular),
        11 => Ok(RouteType::Trolleybus),
        12 => Ok(RouteType::Monorail),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid route type: {:?}",
            s
        ))),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
    pub route_id: String,
    pub agency_id: String,
    pub route_short_name: String,
    pub route_long_name: String,
    pub route_desc: String,
    pub route_type: RouteType,
    pub route_url: String,
    pub route_color: String,
    pub route_text_color: String,
}
