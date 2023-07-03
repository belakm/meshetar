use reqwest::Response;

use crate::store_models::{Interval, Meshetar, Pair};

async fn parse_status(payload: Response) -> Result<Meshetar, String> {
    match payload.text().await {
        Ok(meshetar) => {
            let meshetar = serde_json::from_str(&meshetar);
            match meshetar {
                Ok(meshetar) => Ok(meshetar),
                Err(e) => Err(String::from(format!("Error parsing response {:?}", e))),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
async fn parse_response_string(payload: Response) -> Result<String, String> {
    match payload.text().await {
        Ok(payload) => Ok(payload),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn get_status() -> Result<Meshetar, String> {
    let resp = reqwest::get("http://localhost:8000/status").await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn fetch_last_kline_time() -> Result<String, String> {
    let resp = reqwest::get("http://localhost:8000/last_kline_time").await;
    match resp {
        Ok(resp) => match resp.text().await {
            Ok(last_kline_time) => Ok(last_kline_time),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn stop() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client.post("http://localhost:8000/stop").send().await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn clear_history() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/clear_history")
        .send()
        .await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn fetch_history() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/fetch_history")
        .send()
        .await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn run() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client.post("http://localhost:8000/run").send().await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn plot_chart() -> Result<String, String> {
    let resp = reqwest::get("http://localhost:8000/plot_chart").await;
    match resp {
        Ok(resp) => {
            let resp = parse_response_string(resp).await?;
            println!("{}", resp);
            Ok(resp)
        }
        Err(e) => Err(format!("{}", e)),
    }
}

pub async fn create_model() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/create_model")
        .send()
        .await;
    match resp {
        Ok(resp) => {
            let meshetar = parse_status(resp).await?;
            Ok(meshetar)
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn change_pair(pair: Pair) -> Result<Pair, String> {
    let params = [("pair", pair.to_string())];
    let client = reqwest::Client::new();
    let resp = client
        .put("http://localhost:8000/pair")
        .form(&params)
        .send()
        .await;
    match resp {
        Ok(resp) => match resp.text().await {
            Ok(pair) => {
                let pair = pair.parse::<Pair>();
                match pair {
                    Ok(pair) => Ok(pair),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn change_interval(interval: Interval) -> Result<Interval, String> {
    let params = [("interval", interval.to_string())];
    let client = reqwest::Client::new();
    let resp = client
        .put("http://localhost:8000/interval")
        .form(&params)
        .send()
        .await;
    match resp {
        Ok(resp) => match resp.text().await {
            Ok(interval) => {
                let interval = interval.parse::<Interval>();
                match interval {
                    Ok(interval) => Ok(interval),
                    Err(e) => Err(e.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}
