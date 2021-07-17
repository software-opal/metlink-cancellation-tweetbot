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
    RailwayService,
    /// Ex: TGV (FR), ICE (DE), Eurostar (GB)
    HighSpeedRailService,
    /// Ex: InterCity/EuroCity
    LongDistanceTrains,
    /// Ex: InterRegio (DE), Cross County Rail (GB)
    InterRegionalRailService,
    CarTransportRailService,
    /// Ex: GNER Sleeper (GB)
    SleeperRailService,
    /// Ex: TER (FR), Regionalzug (DE)
    RegionalRailService,
    /// Ex: Romney, Hythe & Dymchurch (GB)
    TouristRailwayService,
    /// Ex: Gatwick Shuttle (GB), Sky Line (DE)
    RailShuttleWithinComplex,
    /// Ex: S-Bahn (DE), RER (FR), S-tog (Kopenhagen)
    SuburbanRailway,
    ReplacementRailService,
    SpecialRailService,
    LorryTransportRailService,
    AllRailServices,
    CrossCountryRailService,
    VehicleTransportRailService,
    // Ex: Rochers de Naye (CH), Dolderbahn (CH)
    RackandPinionRailway ,
    AdditionalRailService,
    CoachService,
    /// Ex: EuroLine, Touring
    InternationalCoachService,
    /// Ex: National Express (GB)
    NationalCoachService,
    /// Ex: Roissy Bus (FR), Reading-Heathrow (GB)
    ShuttleCoachService,
    RegionalCoachService,
    SpecialCoachService,
    SightseeingCoachService,
    TouristCoachService,
    CommuterCoachService,
    AllCoachServices,
    UrbanRailwayService,
    /// Ex: Métro de Paris
    MetroService,
    /// Ex: London Underground, U-Bahn
    UndergroundService,
    // UrbanRailwayService,
    AllUrbanRailwayServices,
    // Monorail,
    BusService,
    /// Ex: Eastbourne-Maidstone (GB)
    RegionalBusService,
    /// Ex: X19 Wokingham-Heathrow (GB)
    ExpressBusService,
    /// Ex: 38 London: Clapton Pond-Victoria (GB)
    StoppingBusService,
    LocalBusService,
    /// Ex: N prefixed buses in London (GB)
    NightBusService,
    /// Ex: Maidstone P4 (GB)
    PostBusService,
    SpecialNeedsBus,
    MobilityBusService,
    MobilityBusForRegisteredDisabled,
    SightseeingBus,
    /// Ex: 747 Heathrow-Gatwick Airport Service (GB)
    ShuttleBus,
    SchoolBus,
    SchoolandPublicServiceBus,
    RailReplacementBusService,
    DemandandResponseBusService,
    AllBusServices,
    TrolleybusService,
    TramService,
    CityTramService,
    /// Ex: Munich (DE), Brussels (BE), Croydon (GB)
    LocalTramService,
    RegionalTramService,
    /// Ex: Blackpool Seafront (GB)
    SightseeingTramService,
    ShuttleTramService,
    AllTramServices,
    WaterTransportService,
    AirService,
    FerryService,
    /// Ex: Telefèric de Montjuïc (ES), Saleve (CH), Roosevelt Island Tramway (US)
    AerialLiftService,
    /// Ex: Rigiblick (Zürich, CH)
    FunicularService,
    TaxiService,
    /// Ex: Marshrutka (RU), dolmuş (TR)
    CommunalTaxiService,
    WaterTaxiService,
    RailTaxiService,
    BikeTaxiService,
    LicensedTaxiService,
    PrivateHireServiceVehicle,
    AllTaxiServices,
    MiscellaneousService,
    HorseDrawnCarriage,
}

impl RouteType {}

pub fn deserialize_route_type<'de, D>(deserializer: D) -> Result<RouteType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: u16 = Deserialize::deserialize(deserializer)?;
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
        100 => Ok(RouteType::RailwayService),
        101 => Ok(RouteType::HighSpeedRailService),
        102 => Ok(RouteType::LongDistanceTrains),
        103 => Ok(RouteType::InterRegionalRailService),
        104 => Ok(RouteType::CarTransportRailService),
        105 => Ok(RouteType::SleeperRailService),
        106 => Ok(RouteType::RegionalRailService),
        107 => Ok(RouteType::TouristRailwayService),
        108 => Ok(RouteType::RailShuttleWithinComplex),
        109 => Ok(RouteType::SuburbanRailway),
        110 => Ok(RouteType::ReplacementRailService),
        111 => Ok(RouteType::SpecialRailService),
        112 => Ok(RouteType::LorryTransportRailService),
        113 => Ok(RouteType::AllRailServices),
        114 => Ok(RouteType::CrossCountryRailService),
        115 => Ok(RouteType::VehicleTransportRailService),
        116 => Ok(RouteType::RackandPinionRailway ),
        117 => Ok(RouteType::AdditionalRailService),
        200 => Ok(RouteType::CoachService),
        201 => Ok(RouteType::InternationalCoachService),
        202 => Ok(RouteType::NationalCoachService),
        203 => Ok(RouteType::ShuttleCoachService),
        204 => Ok(RouteType::RegionalCoachService),
        205 => Ok(RouteType::SpecialCoachService),
        206 => Ok(RouteType::SightseeingCoachService),
        207 => Ok(RouteType::TouristCoachService),
        208 => Ok(RouteType::CommuterCoachService),
        209 => Ok(RouteType::AllCoachServices),
        400 => Ok(RouteType::UrbanRailwayService),
        401 => Ok(RouteType::MetroService),
        402 => Ok(RouteType::UndergroundService),
        403 => Ok(RouteType::UrbanRailwayService),
        404 => Ok(RouteType::AllUrbanRailwayServices),
        405 => Ok(RouteType::Monorail),
        700 => Ok(RouteType::BusService),
        701 => Ok(RouteType::RegionalBusService),
        702 => Ok(RouteType::ExpressBusService),
        703 => Ok(RouteType::StoppingBusService),
        704 => Ok(RouteType::LocalBusService),
        705 => Ok(RouteType::NightBusService),
        706 => Ok(RouteType::PostBusService),
        707 => Ok(RouteType::SpecialNeedsBus),
        708 => Ok(RouteType::MobilityBusService),
        709 => Ok(RouteType::MobilityBusForRegisteredDisabled),
        710 => Ok(RouteType::SightseeingBus),
        711 => Ok(RouteType::ShuttleBus),
        712 => Ok(RouteType::SchoolBus),
        713 => Ok(RouteType::SchoolandPublicServiceBus),
        714 => Ok(RouteType::RailReplacementBusService),
        715 => Ok(RouteType::DemandandResponseBusService),
        716 => Ok(RouteType::AllBusServices),
        800 => Ok(RouteType::TrolleybusService),
        900 => Ok(RouteType::TramService),
        901 => Ok(RouteType::CityTramService),
        902 => Ok(RouteType::LocalTramService),
        903 => Ok(RouteType::RegionalTramService),
        904 => Ok(RouteType::SightseeingTramService),
        905 => Ok(RouteType::ShuttleTramService),
        906 => Ok(RouteType::AllTramServices),
        1000 => Ok(RouteType::WaterTransportService),
        1100 => Ok(RouteType::AirService),
        1200 => Ok(RouteType::FerryService),
        1300 => Ok(RouteType::AerialLiftService),
        1400 => Ok(RouteType::FunicularService),
        1500 => Ok(RouteType::TaxiService),
        1501 => Ok(RouteType::CommunalTaxiService),
        1502 => Ok(RouteType::WaterTaxiService),
        1503 => Ok(RouteType::RailTaxiService),
        1504 => Ok(RouteType::BikeTaxiService),
        1505 => Ok(RouteType::LicensedTaxiService),
        1506 => Ok(RouteType::PrivateHireServiceVehicle),
        1507 => Ok(RouteType::AllTaxiServices),
        1700 => Ok(RouteType::MiscellaneousService),
        1702 => Ok(RouteType::HorseDrawnCarriage),
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
    #[serde(deserialize_with = "deserialize_route_type")]
    pub route_type: RouteType,
    pub route_url: String,
    pub route_color: String,
    pub route_text_color: String,
}
