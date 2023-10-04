pub mod asset_ticker;
pub mod book;
pub mod routes;
// pub mod technical_analysis;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::mpsc;

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
    pub trade_count: u64,
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
    pub market_reciever: mpsc::UnboundedReceiver<MarketEvent>,
}
impl MarketFeed {
    pub fn next(&mut self) -> Feed {
        loop {
            match self.market_reciever.try_recv() {
                Ok(event) => break Feed::Next(event),
                Err(mpsc::error::TryRecvError::Empty) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break Feed::Finished,
            }
        }
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
