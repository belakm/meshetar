use crate::{
    binance_client::BINANCE_CLIENT, database::DB_POOL, formatting::timestamp_to_string,
    prediction_model,
};
use binance_spot_connector_rust::{
    hyper::BinanceHttpClient,
    market::{self, klines::KlineInterval},
    market_stream::kline::KlineStream,
    tungstenite::BinanceWebSocketClient,
};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use rocket::futures::TryFutureExt;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct Kline {
    symbol: String,
    open_time: i64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    close_time: i64,
    quote_asset_volume: String,
    trades: i64,
    taker_buy_base_asset_volume: String,
    taker_buy_quote_asset_volume: String,
    ignore: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct WebsocketResponse {
    e: String, // Event type
    E: i64,    // Event time
    s: String, // Symbol
    k: WebsocketKline,
}

const BINANCE_WSS_BASE_URL: &str = "wss://stream.binance.com:9443/ws";

async fn subscribe_to_price_updates() {
    // Establish connection
    let mut conn =
        BinanceWebSocketClient::connect_with_url(BINANCE_WSS_BASE_URL).expect("Failed to connect");
    // Subscribe to streams
    conn.subscribe(vec![
        &KlineStream::new("BTCUSDT", KlineInterval::Minutes1).into(),
        // &KlineStream::new("BNBBUSD", KlineInterval::Minutes3).into(),
    ]);
    // Read messages
    while let Ok(message) = conn.as_mut().read_message() {
        let data = message.into_data();
        let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
        let response: Result<WebsocketResponse, serde_json::Error> =
            serde_json::from_str(&string_data);
        match response {
            Ok(response) => {
                let response = response.k;
                let symbol = response.s;
                let kline = Kline {
                    symbol: symbol.clone(),
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
                    ignore: response.B,
                };
                let connection = DB_POOL.get().unwrap();
                match insert_kline_to_database(connection, kline).await {
                    Ok(_) => match prediction_model::run().await {
                        Ok(signal) => println!("Kline analyzed: {:?}", signal),
                        Err(e) => {
                            println!("{:?}", e)
                        }
                    },
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error on socket {:?}", e)
            }
        }
    }
    // Disconnect
    conn.close().expect("Failed to disconnect");
}

async fn insert_kline_to_database(connection: &Pool<Sqlite>, kline: Kline) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO klines (symbol, open_time, open, high, low, close, volume, close_time, quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, taker_buy_quote_asset_volume)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
    )
    .bind(kline.symbol)
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
    .execute(connection).map_err(|e| format!("Error inserting a kline into Database. {:?}", e)).await?;
    Ok(())
}

pub async fn fetch_history(symbol: String) -> Result<(), String> {
    log::info!("Fetching {} history.", symbol);
    let mut klines: Vec<Kline> = Vec::new();
    let mut start_time: i64 = 1683843793488;
    let client = BINANCE_CLIENT.get().unwrap();
    loop {
        log::info!(
            "Loading candles from: {:?}",
            timestamp_to_string(start_time)
        );
        let request = market::klines(&symbol, KlineInterval::Minutes1)
            .start_time(start_time as u64)
            .limit(1000);
        let mut res = String::new();
        {
            let data = client
                .send(request)
                .map_err(|e| format!("Error sending binance request. {:?}", e))
                .await?;
            res = data
                .into_body_str()
                .map_err(|e| format!("Failed parsing binance data. {:?}", e))
                .await?;
        };

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
        )> = serde_json::from_str(&res).unwrap();
        let mut new_klines: Vec<Kline> = Vec::new();
        for inner_array in data {
            let kline = Kline {
                symbol: symbol.to_string(),
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
                ignore: inner_array.11,
            };
            new_klines.push(kline);
        }

        let last_kline = new_klines.last(); // we know Vec has items at
                                            // this point
        match last_kline {
            Some(last_kline) => {
                start_time = last_kline.close_time;
                klines.extend(new_klines);
            }
            None => break,
        };
        sleep(Duration::from_secs(1)).await;
    }
    log::info!("No klines.");
    if klines.is_empty() {
        Err(String::from("No history klines inserted."))
    } else {
        log::info!("Starting instertion of klines.");
        let connection = DB_POOL.get().unwrap();
        for kline in klines {
            match insert_kline_to_database(connection, kline).await {
                Ok(_) => (),
                Err(e) => {
                    println!("{:?}", e)
                }
            }
        }
        Ok(())
    }
}
