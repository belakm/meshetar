use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone, Utc};

const DATETIME_FORMAT_SHAPE: &str = "%Y-%m-%d %H:%M:%S";
const DATETIME_FORMAT_SHAPE_SHORT: &str = "%H:%M:%S";

pub fn current_timestamp() -> String {
    Local::now().format(&DATETIME_FORMAT_SHAPE).to_string()
}

pub fn timestamp_to_string(millis: i64) -> String {
    match Utc.timestamp_millis_opt(millis) {
        LocalResult::Single(dt) => dt.format(&DATETIME_FORMAT_SHAPE).to_string(),
        _ => String::from("Incorrect timestamp millis"),
    }
}

pub fn timestamp_to_dt(timestamp: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap(),
        Utc,
    )
}

pub fn dt_to_readable(dt: DateTime<Utc>) -> String {
    dt.format(&DATETIME_FORMAT_SHAPE).to_string()
}

pub fn dt_to_readable_short(dt: DateTime<Utc>) -> String {
    dt.format(&DATETIME_FORMAT_SHAPE_SHORT).to_string()
}
