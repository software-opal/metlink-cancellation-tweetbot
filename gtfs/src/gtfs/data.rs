use self::{agency::Agency, calendar::Calendar};

pub mod agency;
pub mod utils;
pub mod calendar;

#[derive(Debug)]
pub struct GtfsData {
    pub agency: Vec< Agency>,
    pub calendar: Vec<Calendar>,
}
