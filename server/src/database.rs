use sqlx::{Pool, Sqlite, SqlitePool};
use std::{fs::File, path::Path};
use tokio::sync::OnceCell;

pub static DB_POOL: OnceCell<Pool<Sqlite>> = OnceCell::const_new();

pub async fn initialize() -> Result<(), String> {
    println!("Initializing database.");
    match set_connection().await {
        Ok(_) => {
            setup_tables().await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn set_connection() -> Result<(), String> {
    // Creates the database file if it doesnt exist
    let database_path = "database.sqlite";
    if Path::new(database_path).exists() == false {
        File::create(database_path).map_err(|e| String::from(e.to_string()))?;
    }
    // Creates a new pool
    let pool = SqlitePool::connect("database.sqlite").await;
    match pool {
        Ok(pool) => {
            let set_pool_op = DB_POOL.set(pool);
            match set_pool_op {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn setup_tables() -> Result<(), String> {
    let connection = DB_POOL.get();
    match connection {
        Some(connection) => {
            let init_statement = sqlx::query(
                "BEGIN;
        CREATE TABLE IF NOT EXISTS balances (
            id INTEGER PRIMARY KEY,
            asset TEXT NOT NULL,
            free TEXT NOT NULL,
            locked TEXT NOT NULL,
            snapshot_id INTEGER NOT NULL,
            FOREIGN KEY(snapshot_id) REFERENCES snapshots(id)
        );
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY,
            total_asset_of_btc TEXT NOT NULL,
            update_time INTEGER NOT NULL,
            wallet_type TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS account_history (
            id INTEGER PRIMARY KEY,
            code INTEGER NOT NULL,
            msg TEXT NOT NULL,
            last_queried DATETIME NOT NULL
        );
        CREATE TABLE IF NOT EXISTS klines (
            symbol TEXT NOT NULL,
            open_time INTEGER NOT NULL, 
            open REAL NOT NULL, 
            high REAL NOT NULL,
            low REAL NOT NULL, 
            close REAL NOT NULL, 
            volume REAL NOT NULL, 
            close_time INTEGER NOT NULL, 
            quote_asset_volume REAL NOT NULL, 
            number_of_trades INTEGER NOT NULL,
            taker_buy_base_asset_volume REAL NOT NULL, 
            taker_buy_quote_asset_volume REAL NOT NULL,
            PRIMARY KEY (open_time, symbol)
        );
        COMMIT;",
            )
            .execute(connection)
            .await;

            match init_statement {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }
        None => Err(String::from("DB pool not ready for operation.")),
    }
}
