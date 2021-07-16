use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::gtfs::data::route::RouteType;

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
    id: String,
    agency_id: String,
    short_name: String,
    long_name: String,
    desc: String,
    route_type: RouteType,
    url: String,
    color: String,
    text_color: String,
}

impl Route {}
impl From<&crate::gtfs::data::route::Route> for Route {
    fn from(route: &crate::gtfs::data::route::Route) -> Self {
        Self {
            id: route.route_id.clone(),
            agency_id: route.agency_id.clone(),
            short_name: route.route_short_name.clone(),
            long_name: route.route_long_name.clone(),
            desc: route.route_desc.clone(),
            route_type: route.route_type,
            url: route.route_url.clone(),
            color: route.route_color.clone(),
            text_color: route.route_text_color.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RouteDb {
    routes: BTreeMap<String, Route>,
}

impl RouteDb {
    pub fn get_route<'a>(&'a self, id: &String) -> Option<&'a Route> {
        self.routes.get(id)
    }
}

impl From<&Vec<crate::gtfs::data::route::Route>> for RouteDb {
    fn from(parsed: &Vec<crate::gtfs::data::route::Route>) -> Self {
        Self {
            routes: parsed
                .iter()
                .map(|route| Route::from(route))
                .map(|route| (route.id.clone(), route))
                .collect(),
        }
    }
}
