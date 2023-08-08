use crate::utils::default_false;
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

#[derive(Debug, Clone, Deserialize)]
pub struct Chart {
    pub path: String,
    pub page: i64,
    pub total_pages: i64,
    #[serde(default = "default_false")]
    pub is_loading: bool,
}
impl Chart {
    pub fn next_disabled(&self) -> bool {
        self.page <= 1 || self.is_loading
    }
    pub fn prev_disabled(&self) -> bool {
        self.page >= self.total_pages || self.is_loading
    }
    pub fn set_is_loading(&self, is_loading: bool) -> Chart {
        Chart {
            path: self.path.clone(),
            page: self.page.clone(),
            total_pages: self.total_pages.clone(),
            is_loading,
        }
    }
}
impl Default for Chart {
    fn default() -> Self {
        Self {
            page: 1,
            total_pages: 1,
            is_loading: true,
            path: String::new(),
        }
    }
}
