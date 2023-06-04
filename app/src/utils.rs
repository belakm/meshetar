use chrono::{LocalResult, TimeZone, Utc};

pub fn console_log(str_to_log: &str) {
    web_sys::console::log_1(&str_to_log.to_string().into())
}

pub fn readable_date(epoch: &str) -> String {
    match epoch.parse::<i64>() {
        Ok(epoch) => {
            let utc = Utc.timestamp_millis_opt(epoch);
            match utc {
                LocalResult::Single(dt) => dt.to_rfc2822(),
                _ => String::from("Invalid date."),
            }
        }
        Err(_) => String::from("Invalid date (parse error)"),
    }
}
