use std::collections::HashMap;

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
