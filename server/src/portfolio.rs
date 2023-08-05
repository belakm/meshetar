use crate::{
    binance_client::BINANCE_CLIENT,
    database::DB_POOL,
    serde_utils::{f64_default, f64_from_string},
};
use binance_spot_connector_rust::trade;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug)]
struct ApiBalance {
    asset: String,
    #[serde(deserialize_with = "f64_from_string")]
    free: f64,
    #[serde(deserialize_with = "f64_from_string")]
    locked: f64,
    #[serde(default = "f64_default")]
    btc_valuation: f64,
}

#[derive(Deserialize, Debug)]
struct ApiAccount {
    balances: Vec<ApiBalance>,
}

#[derive(FromRow, Clone, Serialize)]
pub struct Balance {
    id: i64,
    asset: String,
    free: f64,
    locked: f64,
    balance_sheet_id: i64,
    #[serde(default = "f64_default")]
    btc_valuation: f64,
}

#[derive(FromRow, Clone, Serialize)]
pub struct BalanceSheet {
    id: i64,
    timestamp: NaiveDateTime,
    btc_valuation: f64,
    busd_valuation: f64,
}

#[derive(Serialize, Clone)]
pub struct BalanceSheetWithBalances {
    sheet: BalanceSheet,
    balances: Vec<Balance>,
}

pub async fn fetch_balances() -> Result<(), String> {
    let client = BINANCE_CLIENT.get().unwrap();
    let response = client
        .send(trade::account())
        .map_err(|e| format!("Error fetching spot wallet {:?}", e))
        .await?
        .into_body_str()
        .map_err(|e| format!("Error parsing spot wallet response, {:?}", e))
        .await?;

    match serde_json::from_str::<ApiAccount>(&response) {
        Ok(balances) => {
            insert_balances(balances.balances).await?;
        }
        Err(e) => log::warn!("Error parsing balances: {:?}", e),
    }
    log::info!("Inserted new balances.");
    Ok(())
}

async fn insert_balances(api_balances: Vec<ApiBalance>) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();
    let mut tx = connection
        .begin()
        .map_err(|e| format!("Error on creating transaction on balances: {:?}", e))
        .await?;
    let timestamp: String = DateTime::to_rfc3339(&Utc::now());
    let balance_sheet: BalanceSheet = sqlx::query_as(
        "INSERT INTO balance_sheets (timestamp, btc_valuation, busd_valuation) VALUES (?1, 0.0, 0.0) RETURNING *",
    )
    .bind(timestamp)
    .fetch_one(connection)
    .map_err(|e| format!("Error inserting new balances. {:?}", e))
    .await?;

    // Insert snapshot data
    for balance in api_balances {
        sqlx::query(
            "INSERT INTO balances (asset, free, locked, balance_sheet_id, btc_valuation) 
            VALUES (
                ?1, 
                ?2, 
                ?3, 
                ?4,
                CASE
                    WHEN ?1 = 'BTC' THEN ?2 -- Btc valuation for btc is btc free field
                    ELSE COALESCE((SELECT last_price FROM asset_ticker WHERE symbol = ?5), 0) * ?6
                END
            )",
        )
        .bind(&balance.asset)
        .bind(&balance.free)
        .bind(&balance.locked)
        .bind(&balance_sheet.id)
        .bind(&format!("{}{}", balance.asset.to_string(), "BTC"))
        .bind(&balance.free)
        .execute(tx.as_mut())
        .map_err(|e| {
            format!(
                "Error inserting a balance for {:?}. {:?}",
                &balance.asset, e
            )
        })
        .await?;
    }

    sqlx::query(
        "
        UPDATE balance_sheets
        SET btc_valuation = (
            SELECT SUM(btc_valuation)
            FROM balances
            WHERE balance_sheet_id = ?1
        ),
        busd_valuation = (
            SELECT SUM(btc_valuation) * asset_ticker.last_price
            FROM balances
            LEFT JOIN asset_ticker ON asset_ticker.symbol = 'BTCBUSD'
            WHERE balance_sheet_id = ?1
        )
        WHERE id = ?1",
    )
    .bind(&balance_sheet.id)
    .execute(tx.as_mut())
    .map_err(|e| format!("Error summing balances on balance_sheet {:?}", e))
    .await?;

    tx.commit()
        .map_err(|e| format!("Error committing new balances: {:?}", e))
        .await?;

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
