use chrono::{Local, LocalResult, TimeZone, Utc};

const DATETIME_FORMAT_SHAPE: &str = "%Y-%m-%d %H:%M:%S";

pub fn current_timestamp() -> String {
    Local::now().format(&DATETIME_FORMAT_SHAPE).to_string()
}

pub fn timestamp_to_string(millis: i64) -> String {
    match Utc.timestamp_millis_opt(millis) {
        LocalResult::Single(dt) => dt.format(&DATETIME_FORMAT_SHAPE).to_string(),
        _ => String::from("Incorrect timestamp millis"),
    }
}
