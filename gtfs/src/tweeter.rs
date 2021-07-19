use log::warn;

use metlink_gtfs_lib::{
    db::Database,
    realtime::service_alerts::{
        AlertInformedEntity, ServiceAlert, ServiceAlertCause, ServiceAlertEffect,
        ServiceAlertEntity,
    },
};

pub fn tweet_service_alert(entity: &ServiceAlertEntity, db: &Database) -> Option<String> {
    let alert = &entity.alert;
    if alert.cause == ServiceAlertCause::Strike && alert.effect == ServiceAlertEffect::NoService {
        return Some(format!("{}", alert.header()));
    }

    let effect = alert_effect(&alert.effect)?;
    let cause = alert_cause(&alert.cause)?;
    let description = alert.description();
    let header = alert.header();
    let routes: Vec<_> = alert
        .informed_entity
        .iter()
        .filter_map(|e| match e {
            AlertInformedEntity::Route { route_id, .. } => Some(route_id),
            _ => None,
        })
        .filter_map(|id| db.routes.get_route(id))
        .map(|route| format!("{} / {}", route.short_name, route.desc))
        .collect();
    let stops: Vec<_> = alert
        .informed_entity
        .iter()
        .filter_map(|e| match e {
            AlertInformedEntity::Stop { stop_id, .. } => Some(stop_id),
            _ => None,
        })
        .filter_map(|id| db.stops.get_stop(id))
        .map(|stop| format!("{} / {}", stop.code, stop.name))
        .collect();
    let trips: Vec<_> = alert
        .informed_entity
        .iter()
        .filter_map(|e| match e {
            AlertInformedEntity::Trip { trip, .. } => Some(&trip.trip_id),
            _ => None,
        })
        .filter_map(|id| db.trips.get_trip(id))
        .map(|trip| (trip, db.routes.get_route(&trip.route_id).unwrap()))
        .map(|(trip, route)| format!("{} / {}", trip.id, route.short_name))
        .collect();

    // if !routes.len() == 1 && description == "" {
    warn!("{:#?}", alert);
    // }

    let text = if description != "" {
        description
    } else {
        header
    };
    return None;
    // let x =  [Route { route_id: "10", route_type: Some(3) }]
    Some(format!(
        "{cause} has caused {effect}: {text}\n{metadata:#?}",
        cause = cause,
        effect = effect,
        // header = header,
        // description = description,
        text = text,
        metadata = (routes, stops, trips)
    ))
}

pub fn alert_effect(effect: &ServiceAlertEffect) -> Option<&'static str> {
    match effect {
        ServiceAlertEffect::NoService => Some("Cancelled"),
        // ServiceAlertEffect::ReducedService => Some("Reduced Service"),
        // ServiceAlertEffect::SignificantDelays => Some("Delays"),
        // ServiceAlertEffect::Detour => Some("Detour"),
        // ServiceAlertEffect::AdditionalService => Some("Additional Service"),
        // ServiceAlertEffect::ModifiedService => Some("Modified Service"),
        // ServiceAlertEffect::OtherEffect => Some("Other Effect"),
        // ServiceAlertEffect::UnknownEffect => Some("Unknow Effect"),
        // ServiceAlertEffect::StopMoved => Some("Stop Moved"),
        // ServiceAlertEffect::NoEffect => Some("No Effect"),
        // ServiceAlertEffect::AccessibilityIssue => Some("Accessibility Issue"),
        _ => None,
    }
}
pub fn alert_cause(cause: &ServiceAlertCause) -> Option<&'static str> {
    match cause {
        // ServiceAlertCause::UnknownCause => Some("Unknow Cause"),
        // ServiceAlertCause::OtherCause => Some("Other Cause"),
        ServiceAlertCause::TechnicalProblem => Some("Techincal Issues"),
        ServiceAlertCause::Strike => Some("Strike"),
        // ServiceAlertCause::Demonstration => Some("Demonstration"),
        // ServiceAlertCause::Accident => Some("Accident"),
        // ServiceAlertCause::Holiday => Some("Holiday"),
        // ServiceAlertCause::Weather => Some("Weather"),
        // ServiceAlertCause::Maintenance => Some("Maintainance"),
        // ServiceAlertCause::Construction => Some("Construction"),
        // ServiceAlertCause::PoliceActivity => Some("Police Activity"),
        // ServiceAlertCause::MedicalEmergency => Some("Medical Emergency"),
        _ => None,
    }
}
