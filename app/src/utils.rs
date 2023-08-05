use chrono::{LocalResult, TimeZone, Utc};

pub fn console_log(str_to_log: &str) {
    web_sys::console::log_1(&str_to_log.to_string().into())
}

pub fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
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
