use crate::store_models::{Interval, Meshetar, Pair, Status};

pub async fn get_status() -> Result<Meshetar, String> {
    let resp = reqwest::get("http://localhost:8000/status").await;
    match resp {
        Ok(resp) => match resp.text().await {
            Ok(meshetar) => {
                let meshetar = serde_json::from_str(&meshetar);
                match meshetar {
                    Ok(meshetar) => Ok(meshetar),
                    Err(e) => Err(String::from(format!("Error parsing response {:?}", e))),
                }
            }
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn stop_operation() -> Result<Status, Box<dyn std::error::Error>> {
    Ok(Status::Idle)
}

pub async fn start_operation() -> Result<Status, Box<dyn std::error::Error>> {
    Ok(Status::Idle)
}

pub async fn change_pair(pair: Pair) -> Result<Pair, Box<dyn std::error::Error>> {
    Ok(Pair::ETHBTC)
}

pub async fn change_interval(interval: Interval) -> Result<Interval, Box<dyn std::error::Error>> {
    Ok(Interval::Minutes1)
}
