use crate::store_models::{Interval, Pair, Status};

pub async fn get_status() -> Result<String, Box<dyn std::error::Error>> {
    let resp: String = reqwest::get("http://localhost:8000/status")
        .await?
        .text()
        .await?;
    Ok(resp)
}

pub async fn stop_operation() -> Result<Status, Box<dyn std::error::Error>> {
    Ok(Status::Idle)
}

pub async fn start_operation() -> Result<Status, Box<dyn std::error::Error>> {
    Ok(Status::Idle)
}

pub async fn change_pair(pair: Pair) -> Result<Pair, Box<dyn std::error::Error>> {
    Ok(Pair::BTCUSDT)
}

pub async fn change_interval(interval: Interval) -> Result<Interval, Box<dyn std::error::Error>> {
    Ok(Interval::Minutes1)
}
