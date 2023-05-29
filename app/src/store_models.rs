use std::collections::HashMap;
use strum::Display;

#[derive(Clone)]
pub struct PortfolioAsset {
    symbol: String,
    btc_value: i64,
}

#[derive(Clone)]
pub struct Portfolio {
    assets: HashMap<String, PortfolioAsset>,
    btc_value: i64,
    usd_value: i64,
}

#[derive(Clone)]
pub struct Kline {
    // TODO
}

#[derive(Clone)]
pub struct Asset {
    symbol: String,
    history: Vec<Kline>,
}

#[derive(Clone)]
pub enum ServerStatus {
    Running,
    Unreachable,
}

#[derive(Copy, Clone, Debug)]
pub enum Status {
    Idle,
    Running,
    FetchingHistory,
}

impl Default for Status {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Copy, Clone, Debug, Display)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

impl Default for Pair {
    fn default() -> Self {
        Self::BTCUSDT
    }
}

impl Pair {
    fn from(input: &str) -> Option<Self> {
        match input {
            "BTCUSDT" => Some(Pair::BTCUSDT),
            "ETHBTC" => Some(Pair::ETHBTC),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

impl Default for Interval {
    fn default() -> Self {
        Self::Minutes1
    }
}

impl Interval {
    fn from(input: &str) -> Option<Self> {
        match input {
            "Minutes1" => Some(Interval::Minutes1),
            "Minutes3" => Some(Interval::Minutes3),
            _ => None,
        }
    }
}
