use crate::{
    database::DB_POOL,
    load_config::{self, Config},
};
use binance_spot_connector_rust::{http::Credentials, hyper::BinanceHttpClient, wallet};
use chrono::Duration;
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

#[derive(FromRow)]
struct IdRow {
    id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceSnapshotItem {
    pub asset: String,
    #[serde(deserialize_with = "f64_from_string")]
    pub free: f64,
    #[serde(deserialize_with = "f64_from_string")]
    pub locked: f64,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub balances: Vec<BalanceSnapshotItem>,
    #[serde(rename = "totalAssetOfBtc", deserialize_with = "f64_from_string")]
    pub total_asset_of_btc: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub data: BalanceSnapshot,
    #[serde(rename = "updateTime")]
    pub update_time: i64,
    #[serde(rename = "type")]
    pub wallet_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccountHistory {
    pub code: i32,
    pub msg: String,
    #[serde(rename = "snapshotVos")]
    pub snapshots: Vec<Snapshot>,
}

pub async fn fetch_account_balance_history() -> Result<(), String> {
    let config: Config = load_config::read_config();
    let credentials = Credentials::from_hmac(
        config.binance_api_key.to_owned(),
        config.binance_api_secret.to_owned(),
    );
    let client = BinanceHttpClient::default().credentials(credentials);
    let period = chrono::Utc::now() - Duration::days(10);

    // Get account information
    let response = client
        .send(wallet::account_snapshot("SPOT").start_time(period.timestamp_millis() as u64))
        .map_err(|e| format!("Error fetching spot wallet {:?}", e))
        .await?
        .into_body_str()
        .map_err(|e| format!("Error parsing spot wallet response, {:?}", e))
        .await?;

    match serde_json::from_str::<AccountHistory>(&response) {
        Ok(account_history) => {
            insert_account_history(&account_history).await?;
        }
        Err(e) => log::warn!("Error parsing account history: {:?}", e),
    }
    Ok(())
}

async fn insert_account_history(account_history: &AccountHistory) -> Result<(), String> {
    let connection = DB_POOL.get().unwrap();

    sqlx::query("DELETE FROM account_history; DELETE FROM snapshots; DELETE FROM balances;")
        .execute(connection)
        .map_err(|e| format!("Error deleting old account history. {:?}", e))
        .await?;

    sqlx::query("INSERT INTO account_history (code, msg, last_queried) VALUES (?1, ?2, ?3)")
        .bind(&account_history.code)
        .bind(&account_history.msg)
        .bind(chrono::Utc::now().timestamp_millis())
        .execute(connection)
        .map_err(|e| format!("Error inserting new account history. {:?}", e))
        .await?;

    // Insert snapshot data
    for snapshot in &account_history.snapshots {
        let snapshot_id: i64 = sqlx::query_as::<_, IdRow>(
            r#"
                    INSERT INTO snapshots (total_asset_of_btc, update_time, wallet_type) 
                    VALUES (?1, ?2, ?3) 
                    RETURNING id
                "#,
        )
        .bind(&snapshot.data.total_asset_of_btc)
        .bind(&snapshot.update_time)
        .bind(&snapshot.wallet_type)
        .fetch_one(connection)
        .map_err(|e| format!("Error fetching last kline. {:?}", e))
        .await?
        .id;

        // Insert balances data
        let balances = &snapshot.data.balances;
        for balance in balances {
            sqlx::query(
                "INSERT INTO balances (asset, free, locked, snapshot_id) VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(&balance.asset)
            .bind(&balance.free)
            .bind(&balance.locked)
            .bind(&snapshot_id)
            .execute(connection)
            .map_err(|e| format!("Error inserting balance snapshot. {:?}", e))
            .await?;
        }
    }

    // Commit transaction
    Ok(())
}

struct QuerySnapshot {
    total_asset_of_btc: f64,
    update_time: i64,
    wallet_type: String,
}

struct QueryAccountHistory {
    code: String,
    msg: String,
}

pub async fn get_account_history_with_snapshots() -> Result<AccountHistory, String> {
    let connection = DB_POOL.get().unwrap();
    let account_history: (i32, String) =
        sqlx::query_as("SELECT code, msg FROM account_history LIMIT 1")
            .fetch_one(connection)
            .map_err(|e| format!("Error fetching account_history. {:?}", e))
            .await?;

    let mut account_history = AccountHistory {
        code: account_history.0,
        msg: account_history.1,
        snapshots: Vec::new(),
    };

    let snapshot_rows: Vec<(f64, i64, String, String, f64, f64)> = sqlx::query_as(r#"
       SELECT snapshots.id as snapshots.total_asset_of_btc, snapshots.update_time, snapshots.wallet_type, balances.asset, balances.free, balances.locked 
        FROM snapshots        
        INNER JOIN balances ON snapshots.id = balances.snapshot_id 
	    WHERE free IS NOT 0
    "#)
        .fetch_all(connection)
        .map_err(|e| format!("Error retrieving snapshots from database. {}", e))
        .await?;

    let mut snapshots: Vec<Snapshot> = Vec::new();

    for row in snapshot_rows {
        let mut snapshot = Snapshot {
            data: BalanceSnapshot {
                balances: Vec::new(),
                total_asset_of_btc: row.0,
            },
            update_time: row.1,
            wallet_type: row.2,
        };
        snapshot.data.balances.push(BalanceSnapshotItem {
            asset: row.3,
            free: row.4,
            locked: row.5,
        });
        snapshots.push(snapshot)
    }

    for snapshot in snapshots {
        account_history.snapshots.push(snapshot);
    }
    Ok(account_history)
}
