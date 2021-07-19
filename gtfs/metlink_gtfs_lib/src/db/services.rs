use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use time::{Date, Weekday};

#[derive(Debug, Deserialize, Serialize)]
pub struct Service {
    id: String,
    dates: BTreeSet<Date>,
}

impl Service {}
impl From<&crate::gtfs::data::calendar::Calendar> for Service {
    fn from(calendar: &crate::gtfs::data::calendar::Calendar) -> Self {
        let mut dates = BTreeSet::new();
        let mut curr = calendar.start_date;
        while curr < calendar.end_date {
            let matches = match curr.weekday() {
                Weekday::Monday => calendar.monday,
                Weekday::Tuesday => calendar.tuesday,
                Weekday::Wednesday => calendar.wednesday,
                Weekday::Thursday => calendar.thursday,
                Weekday::Friday => calendar.friday,
                Weekday::Saturday => calendar.saturday,
                Weekday::Sunday => calendar.sunday,
            };
            if matches {
                dates.insert(curr);
            }
            curr = curr.next_day();
        }

        Self {
            id: calendar.service_id.clone(),
            dates,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceDb {
    services: BTreeMap<String, Service>,
}

impl ServiceDb {
    pub fn get_service<'a>(&'a self, id: &String) -> Option<&'a Service> {
        self.services.get(id)
    }
}

impl From<&crate::gtfs::data::GtfsData> for ServiceDb {
    fn from(parsed: &crate::gtfs::data::GtfsData) -> Self {
        let mut cal_add_dates: BTreeMap<String, BTreeSet<_>> = BTreeMap::new();
        let mut cal_rem_dates: BTreeMap<String, BTreeSet<_>> = BTreeMap::new();
        parsed
            .calendar_date
            .iter()
            .for_each(|s| match s.exception_type {
                crate::gtfs::data::calendar::CalendarDateExceptionType::ServiceAdded => {
                    if let Some(set) = cal_add_dates.get_mut(&s.service_id) {
                        set.insert(s.date.clone());
                    } else {
                        let mut set = BTreeSet::new();
                        set.insert(s.date.clone());
                        cal_add_dates.insert(s.service_id.clone(), set);
                    }
                }
                crate::gtfs::data::calendar::CalendarDateExceptionType::ServiceRemoved => {
                    if let Some(set) = cal_rem_dates.get_mut(&s.service_id) {
                        set.insert(s.date.clone());
                    } else {
                        let mut set = BTreeSet::new();
                        set.insert(s.date.clone());
                        cal_rem_dates.insert(s.service_id.clone(), set);
                    }
                }
            });

        Self {
            services: parsed
                .calendar
                .iter()
                .map(|service| {
                    let mut s = Service::from(service);
                    if let Some(remove) = cal_rem_dates.get(&s.id) {
                        s.dates.retain(|v| remove.contains(v));
                    }
                    if let Some(add) = cal_add_dates.get_mut(&s.id) {
                        s.dates.append(add);
                    }
                    s
                })
                .map(|service| (service.id.clone(), service))
                .collect(),
        }
    }
}
