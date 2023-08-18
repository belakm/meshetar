use binance_spot_connector_rust::market::klines::KlineInterval;
use rocket::serde::{json::Json, Serialize};
use strum::{Display, EnumString};

#[derive(Copy, Clone, Debug, Serialize, PartialEq)]
pub enum MeshetarStatus {
    Idle,
    Stopping,
    FetchingHistory,
    CreatingNewModel,
    Running,
}

#[derive(Copy, Clone, Debug, Serialize, Display, EnumString)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

#[derive(Copy, Clone, Debug, Serialize, Display, EnumString)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

impl Interval {
    pub fn to_kline_interval(&self) -> KlineInterval {
        match self {
            Interval::Minutes1 => KlineInterval::Minutes1,
            Interval::Minutes3 => KlineInterval::Minutes3,
        }
    }
}

// Core struct
//
#[derive(Serialize, Copy, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Meshetar {
    pub pair: Pair,
    pub interval: Interval,
    pub status: MeshetarStatus,
}

impl Meshetar {
    pub fn new() -> Self {
        Meshetar {
            interval: Interval::Minutes1,
            pair: Pair::BTCUSDT,
            status: MeshetarStatus::Idle,
        }
    }
    pub fn change_pair(&mut self, pair: Pair) -> Result<&mut Self, String> {
        if self.status == MeshetarStatus::Idle {
            Err(String::from("Cant change pair while working."))
        } else {
            self.pair = pair;
            Ok(self)
        }
    }
    pub fn change_interval(&mut self, interval: Interval) -> Result<&mut Self, String> {
        if self.status == MeshetarStatus::Idle {
            Err(String::from("Cant change interval while working."))
        } else {
            self.interval = interval;
            Ok(self)
        }
    }
    pub fn summerize_json(self) -> Json<Meshetar> {
        Json(self)
    }
}
