// Valid states
//
use rocket::serde::{json::Json, Serialize};
use strum::Display;

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Status {
    Idle,
    FetchingHistory,
}

#[derive(Copy, Clone, Debug, Serialize, Display)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

// Core struct
//
#[derive(Serialize, Copy, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Meshetar {
    pub pair: Pair,
    pub interval: Interval,
    pub status: Status,
}

impl Meshetar {
    pub fn new() -> Self {
        Meshetar {
            interval: Interval::Minutes1,
            pair: Pair::BTCUSDT,
            status: Status::Idle,
        }
    }
    pub fn summerize(&self) -> String {
        format!("Pair: {:?} --- Interval: {:?}", self.pair, self.interval)
    }
    pub fn go_to_idle(mut self) -> Self {
        self.status = Status::Idle;
        self
    }
    pub fn summerize_json(self) -> Json<Meshetar> {
        Json(self)
    }
}
