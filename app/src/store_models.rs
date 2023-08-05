use chrono::NaiveDateTime;
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

#[derive(Deserialize, Display, EnumString)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

#[derive(Deserialize, Display, EnumString)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

#[derive(Deserialize)]
pub struct Meshetar {
    pub pair: Pair,
    pub interval: Interval,
    pub status: Status,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Balance {
    pub id: i64,
    pub asset: String,
    pub free: f64,
    pub locked: f64,
    pub btc_valuation: f64,
    pub balance_sheet_id: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BalanceSheet {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub btc_valuation: f64,
    pub busd_valuation: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BalanceSheetWithBalances {
    pub sheet: BalanceSheet,
    pub balances: Vec<Balance>,
}

impl Default for BalanceSheetWithBalances {
    fn default() -> Self {
        Self {
            sheet: BalanceSheet {
                id: 0,
                timestamp: NaiveDateTime::from_timestamp_millis(0).unwrap(),
                btc_valuation: 0f64,
                busd_valuation: 0f64,
            },
            balances: Vec::new(),
        }
    }
}
