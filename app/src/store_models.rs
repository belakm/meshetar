use std::collections::HashMap;

pub struct PortfolioAsset {
    symbol: String,
    btc_value: i64,
}

pub struct Portfolio {
    assets: HashMap<String, PortfolioAsset>,
    btc_value: i64,
    usd_value: i64,
}

pub struct Kline {
    // TODO
}

pub struct Asset {
    symbol: String,
    history: Vec<Kline>,
}

pub enum ServerStatus {
    Running,
    Unreachable,
}
