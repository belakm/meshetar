use crate::{
    database::DB_POOL,
    load_config::{self, Config},
};
use binance_spot_connector_rust::{http::Credentials, hyper::BinanceHttpClient, wallet};
use chrono::Duration;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BalanceSnapshotItem {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub balances: Vec<BalanceSnapshotItem>,
    #[serde(rename = "totalAssetOfBtc")]
    pub total_asset_of_btc: String,
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
    pub snapshot_vos: Vec<Snapshot>,
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
        .map_err(|e| format!("Error with parsing spot wallet response, {:?}", e))
        .await?;

    let account_history: AccountHistory = serde_json::from_str(&response).unwrap();
    insert_account_history(&account_history).await;

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
    for snapshot in &account_history.snapshot_vos {
        let mut snapshot_statement = conn.prepare("INSERT INTO snapshots (total_asset_of_btc, update_time, wallet_type) VALUES (?1, ?2, ?3) RETURNING *")?;
        let snapshot_id = snapshot_statement.query_row(
            (
                &snapshot.data.total_asset_of_btc,
                &snapshot.update_time,
                &snapshot.wallet_type,
            ),
            |row| row.get::<usize, i64>(0),
        )?;

        let row = sqlx::query_as("INSERT INTO snapshots (total_asset_of_btc, update_time, wallet_type) VALUES (?1, ?2, ?3) RETURNING id")
            .bind(&snapshot.data.total_asset_of_btc)
            .bind(&snapshot.update_time)
            .bind(&snapshot.wallet_type)
            .fetch_one(connection)
            .map_err(|e| format!("Error fetching last kline. {:?}", e))
            .await?;
        let snapshot_id: i64 = row.0;

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
    let mut account_history = sqlx::query("SELECT code, msg FROM account_history LIMIT 1")
        .fetch_one(connection)
        .map_err(|e| format!("Error fetching account_history. {:?}", e))
        .await?;

    let mut account_history = AccountHistory {
        code: account_history.0,
        msg: account_history.1,
        snapshot_vos: Vec::new(),
    };

    let snapshot_rows = sqlx::query(r#"
       SELECT snapshots.id as snapshots.total_asset_of_btc, snapshots.update_time, snapshots.wallet_type, balances.asset, balances.free, balances.locked 
        FROM snapshots        
        INNER JOIN balances ON snapshots.id = balances.snapshot_id 
	    WHERE free IS NOT 0
    "#).fetch_all(connection).map_err(|e| format!("Error retrieving snapshots from database. {}", e)).await?;

    let mut snapshot_vos: Vec<Snapshot> = Vec::new();

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
        snapshot_vos.push(snapshot)
    }

    for snapshot in snapshot_vos {
        account_history.snapshot_vos.push(snapshot);
    }
    Ok(account_history)
}
