use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Clone, Serialize)]
pub struct BalanceAsset {
    pub id: i64,
    pub asset: String,
    pub free: f64,
    pub locked: f64,
    pub balance_sheet_id: i64,
    #[serde(default = "f64_default")]
    pub btc_valuation: f64,
}

#[derive(Serialize, Clone)]
pub struct Balance {
    pub timestamp: NaiveDateTime,
    pub btc_valuation: f64,
    pub busd_valuation: f64,
    pub balances: Vec<BalanceAsset>,
}

#[derive(FromRow, Clone, Serialize)]
pub struct BalanceSheet {
    pub id: i64,
    pub timestamp: NaiveDateTime,
    pub btc_valuation: f64,
    pub busd_valuation: f64,
}
