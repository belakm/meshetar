use crate::{
    database::DB_POOL,
    load_config::{self, Config},
};
use binance_spot_connector_rust::{http::Credentials, hyper::BinanceHttpClient, wallet};
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::TryFutureExt;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;

fn f64_from_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiBalance {
    #[serde(rename = "asset")]
    symbol: String,
    #[serde(deserialize_with = "f64_from_string")]
    free: f64,
    #[serde(deserialize_with = "f64_from_string")]
    locked: f64,
    #[serde(deserialize_with = "f64_from_string")]
    freeze: f64,
    #[serde(deserialize_with = "f64_from_string")]
    withdrawing: f64,
    #[serde(deserialize_with = "f64_from_string")]
    ipoable: f64,
    #[serde(deserialize_with = "f64_from_string", rename = "btcValuation")]
    btc_valuation: f64,
}

#[derive(FromRow, Clone, Serialize)]
pub struct Balance {
    id: i64,
    symbol: String,
    free: f64,
    locked: f64,
    freeze: f64,
    withdrawing: f64,
    ipoable: f64,
    btc_valuation: f64,
    balance_sheet_id: i64,
}

#[derive(FromRow, Clone, Serialize)]
pub struct BalanceSheet {
    id: i64,
    timestamp: NaiveDateTime,
    total_btc_valuation: f64,
}

#[derive(Serialize, Clone)]
pub struct BalanceSheetWithBalances {
    sheet: BalanceSheet,
    balances: Vec<Balance>,
}

pub async fn fetch_balances() -> Result<(), String> {
    let config: Config = load_config::read_config();
    let credentials = Credentials::from_hmac(
        config.binance_api_key.to_owned(),
        config.binance_api_secret.to_owned(),
    );
    let client = BinanceHttpClient::default().credentials(credentials);
    let response = client
        .send(wallet::user_asset().need_btc_valuation(true))
        .map_err(|e| format!("Error fetching spot wallet {:?}", e))
        .await?
        .into_body_str()
        .map_err(|e| format!("Error parsing spot wallet response, {:?}", e))
        .await?;

    match serde_json::from_str::<Vec<ApiBalance>>(&response) {
        Ok(balances) => {
            insert_balances(balances).await?;
        }
        Err(e) => log::warn!("Error parsing balances: {:?}", e),
    }
    log::info!("Inserted new balances.");
    Ok(())
}

async fn insert_balances(api_balances: Vec<ApiBalance>) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    let timestamp: String = DateTime::to_rfc3339(&Utc::now());
    let total_btc_valuation: &f64 = &api_balances
        .iter()
        .map(|balance| balance.btc_valuation)
        .sum();
    let balance_sheet: BalanceSheet = sqlx::query_as(
        "INSERT INTO balance_sheets (timestamp, total_btc_valuation) VALUES (?1, ?2) RETURNING *",
    )
    .bind(timestamp)
    .bind(total_btc_valuation)
    .fetch_one(connection)
    .map_err(|e| format!("Error inserting new balances. {:?}", e))
    .await?;

    // Insert snapshot data
    for balance in api_balances {
        sqlx::query(
            "INSERT INTO balances (symbol, free, locked, freeze, withdrawing, ipoable, btc_valuation, balance_sheet_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&balance.symbol)
        .bind(&balance.free)
        .bind(&balance.locked)
        .bind(&balance.freeze)
        .bind(&balance.withdrawing)
        .bind(&balance.ipoable)
        .bind(&balance.btc_valuation)
        .bind(&balance_sheet.id)
        .execute(connection)
        .map_err(|e| format!("Error inserting a balance for {:?}. {:?}", &balance.symbol, e))
        .await?;
    }

    // Commit transaction
    Ok(())
}

pub async fn get_balance_sheet() -> Result<BalanceSheetWithBalances, String> {
    let connection = DB_POOL.get().unwrap();
    let balance_sheet: BalanceSheet = sqlx::query_as(
        "SELECT * FROM balance_sheets WHERE id = (SELECT MAX(id) FROM balance_sheets)",
    )
    .fetch_one(connection)
    .map_err(|e| format!("Error fetching last balance sheet. {:?}", e))
    .await?;
    let query = &format!(
        "SELECT * 
        FROM balances
        WHERE balance_sheet_id = {:?}",
        &balance_sheet.id
    );
    let balances: Vec<Balance> = sqlx::query_as(query)
        .fetch_all(connection)
        .map_err(|e| format!("Error retrieving balances from database. {:?}", e))
        .await?;

    let balance_sheet_with_balances = BalanceSheetWithBalances {
        sheet: balance_sheet,
        balances,
    };

    Ok(balance_sheet_with_balances)
}
