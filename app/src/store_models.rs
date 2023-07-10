use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Deserialize, Display, Default, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    #[default]
    Idle,
    Stopping,
    FetchingHistory,
    CreatingNewModel,
    Running,
}

#[derive(Debug, Deserialize, Display, EnumString)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

#[derive(Debug, Deserialize, Display, EnumString)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

#[derive(Debug, Deserialize)]
pub struct Meshetar {
    pub pair: Pair,
    pub interval: Interval,
    pub status: Status,
}
