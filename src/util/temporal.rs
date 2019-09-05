use std::time::{Duration, SystemTime, SystemTimeError};
use chrono::{DateTime, Utc, NaiveDateTime};
use chrono::offset::TimeZone;

pub fn utc_datetime_from_timestamp(timestamp: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc)
}

pub fn systemtime_from_datetime<Tz: TimeZone>(datetime: DateTime<Tz>) -> SystemTime {
    SystemTime::from(datetime)
}

pub fn get_duration(start: SystemTime, end: SystemTime) -> Result<Duration, SystemTimeError> {
    end.duration_since(start)
}