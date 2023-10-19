pub mod asset_ticker;
pub mod book;
pub mod error;
pub mod routes;
use self::{asset_ticker::TickerAsset, error::AssetError};
use crate::{
    database,
    utils::{
        binance_client::BINANCE_CLIENT,
        formatting::{timestamp_to_dt, timestamp_to_string},
        serde_utils::f64_from_string,
    },
};
use binance_spot_connector_rust::market::klines::KlineInterval;
use chrono::{DateTime, Duration, Utc};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::mpsc;
use tracing::info;

#[derive(PartialEq, Display, Debug, Hash, Eq, Clone, Serialize, Deserialize, PartialOrd)]
pub enum Asset {
    BTCUSDT,
    ETHBTC,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum Feed {
    Next(MarketEvent),
    Unhealthy,
    Finished,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MarketEvent {
    pub time: DateTime<Utc>,
    pub asset: Asset,
    pub detail: MarketEventDetail,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum MarketEventDetail {
    Trade(PublicTrade),
    OrderBookL1(OrderBookL1),
    Candle(Candle),
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Liquidation {
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub time: DateTime<Utc>,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Candle {
    pub close_time: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

impl From<&TickerAsset> for Candle {
    fn from(asset: &TickerAsset) -> Self {
        Candle {
            close_time: timestamp_to_dt(asset.timestamp),
            open: asset.open_price,
            high: asset.high_price,
            low: asset.low_price,
            close: asset.last_price,
            volume: asset.total_traded_base_volume,
            trade_count: asset.number_of_trades,
        }
    }
}

impl From<&BinanceKline> for Candle {
    fn from(kline: &BinanceKline) -> Self {
        Candle {
            close_time: timestamp_to_dt(kline.6),
            open: kline.1.parse().unwrap(),
            high: kline.2.parse().unwrap(),
            low: kline.3.parse().unwrap(),
            close: kline.4.parse().unwrap(),
            volume: kline.5.parse().unwrap(),
            trade_count: kline.8,
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Level {
    pub price: f64,
    pub amount: f64,
}
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderBookL1 {
    pub last_update_time: DateTime<Utc>,
    pub best_bid: Level,
    pub best_ask: Level,
}
impl OrderBookL1 {
    pub fn mid_price(&self) -> f64 {
        (self.best_bid.price + self.best_ask.price) / 2.0
    }
    pub fn volume_weighted_mid_price(&self) -> f64 {
        (self.best_bid.price * self.best_bid.amount + self.best_ask.price * self.best_ask.amount)
            / (self.best_bid.amount + self.best_ask.amount)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct PublicTrade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub enum Side {
    Buy,
    Sell,
}

pub struct MarketFeed {
    pub market_receiver: mpsc::UnboundedReceiver<MarketEvent>,
}
impl MarketFeed {
    pub fn next(&mut self) -> Feed {
        loop {
            match self.market_receiver.try_recv() {
                Ok(event) => break Feed::Next(event),
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break Feed::Finished,
            }
        }
    }
    pub async fn new(asset: Asset) -> Result<Self, AssetError> {
        let receiver = asset_ticker::new_ticker(asset).await?;
        Ok(Self {
            market_receiver: receiver,
        })
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct MarketMeta {
    pub close: f64,
    pub time: DateTime<Utc>,
}

impl Default for MarketMeta {
    fn default() -> Self {
        Self {
            close: 100.0,
            time: Utc::now(),
        }
    }
}

pub async fn fetch_candles(duration: Duration, asset: Asset) -> Result<Vec<Candle>, AssetError> {
    let mut start_time: i64 = (Utc::now() - duration).timestamp_millis();
    let client = BINANCE_CLIENT.get().unwrap();
    info!("Fetching {} history.", asset);
    let mut candles = Vec::<Candle>::new();
    loop {
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
                info!("Loading candles from: {:?}", timestamp_to_string(start_time));
                let request = binance_spot_connector_rust::market::klines(&asset.to_string(), KlineInterval::Minutes1)
                    .start_time(start_time as u64)
                    .limit(1000);
                let klines;
                {
                    let data = client
                        .send(request)
                        .map_err(|e| AssetError::BinanceClientError(format!("{:?}", e)))
                        .await?;
                    klines = data
                        .into_body_str()
                        .map_err(|e| AssetError::BinanceClientError(format!("{:?}", e)))
                        .await?;
                };

                let new_candles = parse_binance_klines(&klines).await?;
                let last_candle = &new_candles.last();
                if let Some(last_candle) = last_candle {
                    start_time = last_candle.close_time.timestamp_micros();
                    candles.extend(new_candles);// .concat(new_candles);
                } else {
                    break
                }
            }
        }
    }
    Ok(candles)
}

pub type BinanceKline = (
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
);

async fn parse_binance_klines(klines: &String) -> Result<Vec<Candle>, AssetError> {
    let data: Vec<BinanceKline> = serde_json::from_str(klines)?;
    let mut new_candles: Vec<Candle> = Vec::new();
    for candle in data {
        let new_candle = Candle::from(&candle);
        new_candles.push(Candle::from(new_candle));
    }
    Ok(new_candles)
}
