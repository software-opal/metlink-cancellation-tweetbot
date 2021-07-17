use self::{
    agency::Agency,
    calendar::{Calendar, CalendarDate},
    feed_data::FeedInfo,
    route::Route,
    stop::{Stop, StopTime},
    stop_pattern::{StopPattern, StopPatternTrip},
    trip::Trip,
};
use serde::{Deserialize, Serialize};

pub mod agency;
pub mod calendar;
pub mod feed_data;
pub mod route;
pub mod stop;
pub mod stop_pattern;
pub mod time;
pub mod trip;
pub mod utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct GtfsData {
    pub agency: Vec<Agency>,
    pub calendar: Vec<Calendar>,
    pub calendar_date: Vec<CalendarDate>,
    pub feed_info: Vec<FeedInfo>,
    pub route: Vec<Route>,
    pub stop_pattern: Vec<StopPattern>,
    pub stop_pattern_trip: Vec<StopPatternTrip>,
    pub stop: Vec<Stop>,
    pub stop_time: Vec<StopTime>,
    pub trip: Vec<Trip>,
}
