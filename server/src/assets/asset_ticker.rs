use crate::utils::{
    binance_client::BINANCE_WSS_BASE_URL, database::DB_POOL, serde_utils::f64_from_string,
};
use binance_spot_connector_rust::{
    market_stream::ticker::TickerStream, tokio_tungstenite::BinanceWebSocketClient,
};
use futures::{StreamExt, TryFutureExt};
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TickerAsset {
    // e: String,          // Event type
    // E: i64,             // Event time
    #[serde(rename = "s")]
    symbol: String, // Symbol
    #[serde(rename = "p", deserialize_with = "f64_from_string")]
    price_change: f64, // Price change
    #[serde(rename = "P", deserialize_with = "f64_from_string")]
    price_change_percent: f64, // Price change percent
    #[serde(rename = "w", deserialize_with = "f64_from_string")]
    weighted_average_price: f64, // Weighted average price
    #[serde(rename = "x", deserialize_with = "f64_from_string")]
    first_price: f64, // First trade(F)-1 price (first trade before the 24hr rolling window)
    #[serde(rename = "c", deserialize_with = "f64_from_string")]
    last_price: f64, // Last price
    #[serde(rename = "Q", deserialize_with = "f64_from_string")]
    last_quantity: f64, // Last quantity
    #[serde(rename = "b", deserialize_with = "f64_from_string")]
    best_bid_price: f64, // Best bid price
    #[serde(rename = "B", deserialize_with = "f64_from_string")]
    best_bid_quantity: f64, // Best bid quantity
    #[serde(rename = "a", deserialize_with = "f64_from_string")]
    best_ask_price: f64, // Best ask price
    #[serde(rename = "A", deserialize_with = "f64_from_string")]
    best_ask_quantity: f64, // Best ask quantity
    #[serde(rename = "o", deserialize_with = "f64_from_string")]
    open_price: f64, // Open price
    #[serde(rename = "h", deserialize_with = "f64_from_string")]
    high_price: f64, // High price
    #[serde(rename = "l", deserialize_with = "f64_from_string")]
    low_price: f64, // Low price
    #[serde(rename = "v", deserialize_with = "f64_from_string")]
    total_traded_base_volume: f64, // Total traded base asset volume
    #[serde(rename = "q", deserialize_with = "f64_from_string")]
    total_traded_quote_volume: f64, // Total traded quote asset volume
    // O: 0,             // Statistics open time
    // C: 86400000,      // Statistics close time
    // F: 0,             // First trade ID
    // L: 18150,         // Last trade Id
    #[serde(rename = "n")]
    number_of_trades: i64, // Total number of trades
}

pub async fn insert_assets(assets: Vec<TickerAsset>) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    let mut tx = connection.begin().map_err(|e| format!("{:?}", e)).await?;
    let query = "
        INSERT OR REPLACE INTO asset_ticker (
            symbol, 
            price_change,
            price_change_percent,
            weighted_average_price,
            first_price,
            last_price,
            last_quantity,
            best_bid_price,
            best_bid_quantity,
            best_ask_price,
            best_ask_quantity,
            open_price,
            high_price,
            low_price,
            total_traded_base_volume,
            total_traded_quote_volume,
            number_of_trades
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17);
    ";
    for asset in assets {
        sqlx::query(query)
            .bind(&asset.symbol)
            .bind(&asset.price_change)
            .bind(&asset.price_change_percent)
            .bind(&asset.weighted_average_price)
            .bind(&asset.first_price)
            .bind(&asset.last_price)
            .bind(&asset.last_quantity)
            .bind(&asset.best_bid_price)
            .bind(&asset.best_bid_quantity)
            .bind(&asset.best_ask_price)
            .bind(&asset.best_ask_quantity)
            .bind(&asset.open_price)
            .bind(&asset.high_price)
            .bind(&asset.low_price)
            .bind(&asset.total_traded_base_volume)
            .bind(&asset.total_traded_quote_volume)
            .bind(&asset.number_of_trades)
            .execute(tx.as_mut())
            .map_err(|e| format!("Error inserting a asset (ticker) into Database. {:?}", e))
            .await?;
    }

    tx.commit()
        .map_err(|e| format!("Error on commiting TX on asset ticker: {:?}", e))
        .await?;

    Ok(())
}

pub async fn subscribe() -> Result<(), String> {
    let (mut conn, _) = BinanceWebSocketClient::connect_async(BINANCE_WSS_BASE_URL)
        .await
        .expect("Failed to connect");

    conn.subscribe(vec![&TickerStream::all_symbols().into()])
        .await;

    while let Some(message) = conn.as_mut().next().await {
        match message {
            Ok(message) => {
                let data = message.into_data();
                let string_data = String::from_utf8(data).expect("Found invalid UTF-8 chars");
                let assets: Result<Vec<TickerAsset>, serde_json::Error> =
                    serde_json::from_str(&string_data);
                match assets {
                    Ok(assets) => match insert_assets(assets).await {
                        Err(e) => {
                            log::error!("Error inserting new assets, {:?}", e);
                        }
                        _ => (),
                    },
                    Err(e) => log::warn!("Error parsing PRICE SOCKET: {:?}", e),
                }
            }
            Err(e) => log::warn!("Error recieving on PRICE SOCKET: {:?}", e),
        }
    }

    Ok(())
}
