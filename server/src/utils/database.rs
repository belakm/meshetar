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
            free REAL NOT NULL,
            locked REAL NOT NULL,
            balance_sheet_id INTEGER,
            btc_valuation REAL NOT NULL,
            FOREIGN KEY (balance_sheet_id) REFERENCES balance_sheets (id)
        );
        CREATE TABLE IF NOT EXISTS balance_sheets (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            btc_valuation REAL NOT NULL,
            busd_valuation REAL NOT NULL
        );
        CREATE TABLE IF NOT EXISTS klines (
            symbol TEXT NOT NULL,
            interval TEXT NOT NULL,
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
            PRIMARY KEY (open_time, symbol, interval)
        );
        CREATE TABLE IF NOT EXISTS signals (
            symbol TEXT NOT NULL,
            interval TEXT NOT NULL,
            time INTEGER NOT NULL,
            signal TEXT NOT NULL,
            PRIMARY KEY (symbol, interval, time)
        );
        CREATE TABLE IF NOT EXISTS account(
            maker_commission INTEGER NOT NULL,
            taker_commission INTEGER NOT NULL,
            buyer_commission INTEGER NOT NULL,
            seller_commission INTEGER NOT NULL,
            can_trade INTEGER NOT NULL,
            can_withdraw INTEGER NOT NULL,
            can_deposit INTEGER NOT NULL,
            brokered INTEGER NOT NULL,
            require_self_rade_prevention INTEGER NOT NULL,
            prevent_sor INTEGER NOT NULL,
            update_time INTEGER NOT NULL,
            account_type TEXT NOT NULL,
            uid INTEGER NOT NULL,
            PRIMARY KEY (uid)
        );
        CREATE TABLE IF NOT EXISTS asset_ticker (
            symbol TEXT NOT NULL,
            price_change REAL NOT NULL,
            price_change_percent REAL NOT NULL,
            weighted_average_price REAL NOT NULL,
            first_price REAL NOT NULL,
            last_price REAL NOT NULL,
            last_quantity REAL NOT NULL,
            best_bid_price REAL NOT NULL,
            best_bid_quantity REAL NOT NULL,
            best_ask_price REAL NOT NULL,
            best_ask_quantity REAL NOT NULL,
            open_price REAL NOT NULL,
            high_price REAL NOT NULL,
            low_price REAL NOT NULL,
            total_traded_base_volume REAL NOT NULL,
            total_traded_quote_volume REAL NOT NULL,
            number_of_trades INTEGER NOT NULL,
            PRIMARY KEY (symbol)
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
