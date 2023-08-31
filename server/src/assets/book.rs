use crate::{
    binance_client::{BINANCE_CLIENT, BINANCE_WSS_BASE_URL},
    database::DB_POOL,
    model::prediction_model::{self, TradeSignal},
    trading::meshetar::Meshetar,
    utils::formatting::timestamp_to_string,
    TaskControl,
};
use binance_spot_connector_rust::{
    market::{self, klines::KlineInterval},
    market_stream::kline::KlineStream,
    tokio_tungstenite::BinanceWebSocketClient,
};
use futures::StreamExt; // needed for websocket to binance
use rocket::futures::TryFutureExt;
use serde::Deserialize;
use sqlx::Row;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[allow(unused)]
#[derive(Deserialize)]
pub struct Kline {
    pub symbol: String,
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_asset_volume: String,
    pub trades: i64,
    pub taker_buy_base_asset_volume: String,
    pub taker_buy_quote_asset_volume: String,
    // pub ignore: String,
    pub interval: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct WebsocketKline {
    t: i64,    // Kline start time
    T: i64,    // Kline close time
    s: String, // Symbol
    i: String, // Interval
    f: i64,    // First trade ID
    L: i64,    // Last trade ID
    o: String, // Open price
    c: String, // Close price
    h: String, // High price
    l: String, // Low price
    v: String, // Base asset volume
    n: i64,    // Number of trades
    x: bool,   // Is this kline closed?
    q: String, // Quote asset volume
    V: String, // Taker buy base asset volume
    Q: String, // Taker buy quote asset volume
    B: String, // Ignore
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct WebsocketResponse {
    e: String, // Event type
    E: i64,    // Event time
    s: String, // Symbol
    k: WebsocketKline,
}

pub async fn run(
    task_control: Arc<Mutex<TaskControl>>,
    meshetar: Arc<Mutex<Meshetar>>,
) -> Result<(), String> {
    // Get params ready
    let meshetar = meshetar.lock().await;
    let pair = meshetar.pair.to_string();
    let interval = meshetar.interval.to_kline_interval();
    let interval_string = meshetar.interval.to_kline_interval().to_string();
    drop(meshetar);

    let (mut conn, _) = BinanceWebSocketClient::connect_async(BINANCE_WSS_BASE_URL)
        .await
        .expect("Failed to connect");

    conn.subscribe(vec![&KlineStream::new(&pair, interval).into()])
        .await;

    let mut receiver = task_control.lock().await.receiver.clone();

    loop {
        tokio::select! {
            _ = receiver.changed() => {
                if *receiver.borrow() == false {
                    break;
                }
            },
            Some(message) = conn.as_mut().next() => {
                match message {
                    Ok(message) => {
                        let data = message.into_data();
                        let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
                        let response: Result<WebsocketResponse, serde_json::Error> =
                            serde_json::from_str(&string_data);
                        match response {
                            Ok(response) => {
                                let response = response.k;
                                let symbol = response.s;
                                let time = response.t.clone(); // take_open time for time
                                                               // identifier as to align with
                                                               // klines
                                let kline = Kline {
                                    symbol: symbol.clone(),
                                    interval: interval_string.clone(),
                                    open_time: response.t,
                                    open: response.o,
                                    high: response.h,
                                    low: response.l,
                                    close: response.c,
                                    volume: response.v,
                                    close_time: response.T,
                                    quote_asset_volume: response.q,
                                    trades: response.n,
                                    taker_buy_base_asset_volume: response.V,
                                    taker_buy_quote_asset_volume: response.Q,
                                };
                                let mut vec_kline: Vec<Kline> = Vec::new();
                                vec_kline.push(kline);
                                match insert_klines_to_database(vec_kline).await {
                                    Ok(_) => {
                                        let task_control2 = Arc::clone(&task_control);
                                        match prediction_model::run_model(task_control2).await {
                                            Ok(signal) => {
                                                match insert_signal_to_database(signal, symbol.clone(), interval_string.clone(), time).await {
                                                    Ok(_) => log::info!("New signal inserted."),
                                                    Err(e) => log::warn!("{}", e)
                                                };
                                                log::info!("Kline analyzed: {:?}", signal)
                                            },
                                            Err(e) => {
                                                log::warn!("{:?}", e)
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        log::warn!("{}", e);
                                    }
                                }
                            }
                            Err(e) => {
                               log::warn!("Error on socket {}", e)
                            }
                        }
                    },
                    Err(e) => log::warn!("Empty socket {}", e)
                }
            },
            else => {
                log::info!("Weird socket.")
            }
        }
    }
    // Disconnect
    conn.close().await.expect("Failed to disconnect");
    Ok(())
}

async fn insert_klines_to_database(klines: Vec<Kline>) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    let mut tx = connection
        .begin()
        .map_err(|e| format!("Error on creating transaction on klines: {:?}", e))
        .await?;
    for kline in klines {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO klines (symbol, interval, open_time, open, high, low, close, volume, close_time, quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, taker_buy_quote_asset_volume)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(kline.symbol)
        .bind(kline.interval)
        .bind(kline.open_time)
        .bind(kline.open)
        .bind(kline.high)
        .bind(kline.low)
        .bind(kline.close)
        .bind(kline.volume)
        .bind(kline.close_time)
        .bind(kline.quote_asset_volume)
        .bind(kline.trades)
        .bind(kline.taker_buy_base_asset_volume)
        .bind(kline.taker_buy_quote_asset_volume)
        .execute(tx.as_mut()).map_err(|e| format!("Error inserting a kline into Database. {:?}", e)).await?;
    }
    tx.commit()
        .map_err(|e| format!("Error committing new klines: {:?}", e))
        .await?;
    Ok(())
}

async fn insert_signal_to_database(
    signal: TradeSignal,
    symbol: String,
    interval: String,
    time: i64,
) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO signals (symbol, interval, time, signal)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(symbol)
    .bind(interval)
    .bind(time)
    .bind(signal.to_string())
    .execute(connection)
    .map_err(|e| format!("Error inserting a kline into Database. {:?}", e))
    .await?;
    Ok(())
}

pub async fn latest_kline_date() -> Result<i64, String> {
    let connection = DB_POOL.get().unwrap();
    let row = sqlx::query("SELECT close_time FROM klines ORDER BY close_time DESC LIMIT 1")
        .fetch_one(connection)
        .map_err(|e| format!("Error fetching last kline. {:?}", e))
        .await?;
    let close_time: i64 = row.get("close_time");
    Ok(close_time)
}

pub async fn clear_history(symbol: String, interval: KlineInterval) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    sqlx::query(
        r#"
                DELETE FROM klines
                WHERE interval = ?1 AND symbol = ?2;
                "#,
    )
    .bind(interval.to_string())
    .bind(symbol)
    .execute(connection)
    .map_err(|e| format!("Error deleting klines, {:?}", e))
    .await?;
    Ok(())
}

fn parse_binance_klines(klines: &String, symbol: &String, interval: &KlineInterval) -> Vec<Kline> {
    let data: Vec<(
        i64,
        String,
        String,
        String,
        String,
        String,
        i64,
        String,
        i64,
        String,
        String,
        String,
    )> = serde_json::from_str(klines).unwrap();
    let mut new_klines: Vec<Kline> = Vec::new();
    for inner_array in data {
        let kline = Kline {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
            open_time: inner_array.0,
            open: inner_array.1,
            high: inner_array.2,
            low: inner_array.3,
            close: inner_array.4,
            volume: inner_array.5,
            close_time: inner_array.6,
            quote_asset_volume: inner_array.7,
            trades: inner_array.8,
            taker_buy_base_asset_volume: inner_array.9,
            taker_buy_quote_asset_volume: inner_array.10,
        };
        new_klines.push(kline);
    }
    new_klines
}

pub async fn fetch_history(
    task_control: Arc<Mutex<TaskControl>>,
    meshetar: Arc<Mutex<Meshetar>>,
    start_time: i64,
) -> Result<(), String> {
    let mut receiver = task_control.lock().await.receiver.clone();
    let meshetar = meshetar.lock().await;
    let symbol = meshetar.pair.to_string();
    let interval = meshetar.interval.to_kline_interval();
    drop(meshetar);
    let mut start_time: i64 = start_time.clone() * 1000;
    let client = BINANCE_CLIENT.get().unwrap();
    log::info!("Fetching {} history.", symbol);
    loop {
        tokio::select! {
            _ = receiver.changed() => {
                if *receiver.borrow() == false {
                    break;
                }
            },
            _ = sleep(Duration::from_millis(10)) => {
                log::info!("Loading candles from: {:?}", timestamp_to_string(start_time));
                let request = market::klines(&symbol, interval)
                    .start_time(start_time as u64)
                    .limit(1000);
                let klines;
                {
                    let data = client
                        .send(request)
                        .map_err(|e| format!("Error sending binance request. {:?}", e))
                        .await?;
                    klines = data
                        .into_body_str()
                        .map_err(|e| format!("Failed parsing binance data. {:?}", e))
                        .await?;
                };
                let new_klines = parse_binance_klines(&klines, &symbol, &interval);
                let last_kline = &new_klines.last();
                match last_kline {
                    Some(last_kline) => {
                        start_time = last_kline.close_time;
                        // insert new klines
                        if !new_klines.is_empty() {
                            match insert_klines_to_database(new_klines).await {
                                Ok(_) => (),
                                Err(e) => {
                                    println!("{:?}", e)
                                }
                            };
                            log::info!(
                                "New klines inserted up to {}.",
                                timestamp_to_string(start_time.clone())
                            );
                        }
                    }
                    None => break,
                };
            }
        }
    }
    Ok(())
}
