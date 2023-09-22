pub mod asset_ticker;
pub mod book;
pub mod routes;
pub mod technical_analysis;

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub struct Asset {
    pair: Pair,
}
