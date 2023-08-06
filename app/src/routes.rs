use crate::store_models::{BalanceSheetWithBalances, Interval, Meshetar, Pair};
use reqwest::Response;

async fn parse_status(payload: Response) -> Result<Meshetar, String> {
    match payload.text().await {
        Ok(meshetar) => match serde_json::from_str(&meshetar) {
            Ok(meshetar) => Ok(meshetar),
            Err(e) => Err(String::from(format!("Error parsing response {:?}", e))),
        },
        Err(e) => Err(e.to_string()),
    }
}
async fn parse_response_string(payload: Response) -> Result<String, String> {
    match payload.text().await {
        Ok(payload) => Ok(payload),
        Err(e) => Err(e.to_string()),
    }
}
async fn parse_balance_sheet(payload: Response) -> Result<BalanceSheetWithBalances, String> {
    match payload.text().await {
        Ok(balance_sheet) => match serde_json::from_str(&balance_sheet) {
            Ok(balance_sheet) => Ok(balance_sheet),
            Err(e) => Err(e.to_string()),
        },
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

pub async fn fetch_balance_sheet() -> Result<BalanceSheetWithBalances, String> {
    let resp = reqwest::get("http://localhost:8000/balance_sheet").await;
    match resp {
        Ok(resp) => {
            let balance_sheet = parse_balance_sheet(resp).await?;
            Ok(balance_sheet)
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

pub async fn plot_chart(page: i64) -> Result<String, String> {
    let params = [("page", page.to_string())];
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/plot_chart")
        .form(&params)
        .send()
        .await;

    match resp {
        Ok(resp) => {
            let resp = parse_response_string(resp).await?;
            println!("{}", resp);
            Ok(resp)
        }
        Err(e) => Err(format!("{}", e)),
    }
}

pub async fn create_new_model() -> Result<Meshetar, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8000/create_new_model")
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
