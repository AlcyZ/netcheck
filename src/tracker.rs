use crate::model::{Connectivity, InternetCheckResult};

pub struct DowntimeTracker<'a> {
    first_offline: Option<&'a InternetCheckResult>,
}

impl<'a> DowntimeTracker<'a> {
    pub fn new() -> Self {
        Self {
            first_offline: None,
        }
    }

    pub fn track<T, F>(&mut self, result: &'a InternetCheckResult, cb: F) -> Option<T>
    where
        F: Fn(&'a InternetCheckResult, &'a InternetCheckResult) -> Option<T>,
    {
        match (self.first_offline, result.connectivity()) {
            (None, Connectivity::Offline) => {
                self.first_offline = Some(result);
                None
            }
            (Some(first), Connectivity::Online) => {
                self.first_offline = None;

                cb(first, result)
            }
            _ => None,
        }
    }
}
