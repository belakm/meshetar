use std::str::FromStr;

use chrono::{DateTime, Duration, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};

pub fn console_log(str_to_log: &str) {
    web_sys::console::log_1(&str_to_log.to_string().into())
}

pub fn get_timestamp() -> i64 {
    Utc::now().timestamp_millis()
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

pub fn to_fiat_format(value: f64) -> String {
    let integer = value as u64;
    let fractional = (value.fract() * 100.0).round() as u64;

    let mut formatted_integer = String::new();
    let integer_str = format!("{}", integer);

    for (i, c) in integer_str.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            formatted_integer.insert(0, '.');
        }
        formatted_integer.insert(0, c);
    }

    format!("{},{:02}", formatted_integer, fractional)
}

pub fn default_false() -> bool {
    false
}

pub fn date_string_to_integer(date: &String) -> i64 {
    console_log(date);
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(date) => Utc
            .from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
            .timestamp(),
        Err(_) => get_timestamp() / 1000,
    }
}

pub fn get_default_fetch_date() -> String {
    let date = Utc::now() - Duration::days(2);
    date.format("%Y-%m-%d").to_string()
}
