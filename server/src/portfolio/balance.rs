use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Clone, Serialize)]
pub struct BalanceAsset {
    id: i64,
    asset: String,
    free: f64,
    locked: f64,
    balance_sheet_id: i64,
    #[serde(default = "f64_default")]
    btc_valuation: f64,
}

#[derive(Serialize, Clone)]
pub struct Balance {
    id: i64,
    timestamp: NaiveDateTime,
    btc_valuation: f64,
    busd_valuation: f64,
    balances: Vec<BalanceAsset>,
}
