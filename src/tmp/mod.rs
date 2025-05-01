use std::time::Duration;

use moka::future::Cache;

pub struct CacheService {
    pub ical_cache: Cache<String, Vec<CalendarEvent>>,
}

impl Default for CacheService {
    fn default() -> Self {
        Self {
            ical_cache: Cache::builder().time_to_live(Duration::from_secs(60 * 60)).build(),
        }
    }
}
