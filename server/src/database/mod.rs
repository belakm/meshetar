pub mod error;
pub mod sqlite;

use crate::portfolio::{
    account::Account,
    balance::{Balance, BalanceSheet},
};
use chrono::{DateTime, Utc};

use self::{error::DatabaseError, sqlite::DB_POOL};

pub struct Database {}

impl Database {
    pub async fn set_balance(&mut self, balance: Balance) -> Result<(), DatabaseError> {
        let connection = DB_POOL.get().unwrap();
        let mut tx = connection.begin().await?;
        let timestamp: String = DateTime::to_rfc3339(&Utc::now());
        let balance_sheet: BalanceSheet = sqlx::query_as(
        "INSERT INTO balance_sheets (timestamp, btc_valuation, busd_valuation) VALUES (?1, 0.0, 0.0) RETURNING *",
    )
    .bind(timestamp)
    .fetch_one(connection)
    .await.map_err(|_| DatabaseError::ReadError)?;

        // Insert snapshot data
        for balance in balance.balances {
            sqlx::query(
                "INSERT INTO balances (asset, free, locked, balance_sheet_id, btc_valuation) 
                VALUES (
                    ?1, 
                    ?2, 
                    ?3, 
                    ?4,
                        CASE WHEN ?1 = 'BTC' THEN ?2 -- get valuation from ticker otherwise default 
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
            .await?;
        }

        sqlx::query(
            "UPDATE balance_sheets
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
        .await?;

        tx.commit().await?;

        // Commit transaction
        Ok(())
    }

    pub async fn get_balance(&mut self) -> Result<Balance, DatabaseError> {
        let connection = DB_POOL.get().unwrap();
        let balance_sheet: BalanceSheet = sqlx::query_as(
            "SELECT * FROM balance_sheets WHERE id = (SELECT MAX(id) FROM balance_sheets)",
        )
        .fetch_one(connection)
        .await
        .map_err(|_| DatabaseError::ReadError)?;

        Ok()
    }

    pub async fn set_account(&mut self, account: Account) -> Result<(), DatabaseError> {
        let connection = DB_POOL.get().unwrap();
        sqlx::query(
            "INSERT OR REPLACE INTO account (
            maker_commission,
            taker_commission,
            buyer_commission,
            seller_commission,
            can_trade,
            can_withdraw,
            can_deposit,
            brokered,
            require_self_rade_prevention,
            prevent_sor,
            update_time,
            account_type,
            uid
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .bind(account.maker_commission)
        .bind(account.taker_commission)
        .bind(account.buyer_commission)
        .bind(account.seller_commission)
        .bind(account.can_trade)
        .bind(account.can_withdraw)
        .bind(account.can_deposit)
        .bind(account.brokered)
        .bind(account.require_self_rade_prevention)
        .bind(account.prevent_sor)
        .bind(account.update_time)
        .bind(account.account_type.clone())
        .bind(account.uid)
        .execute(connection)
        .await?;
        Ok(())
    }

    pub async fn get_account(&mut self, uid: i64) -> Result<Account, DatabaseError> {
        let connection = DB_POOL.get().unwrap();
        let account: Account = sqlx::query_as("SELECT * FROM account WHERE uid = $1")
            .bind(uid)
            .fetch_one(connection)
            .await
            .map_err(|_| DatabaseError::ReadError)?;
        Ok(account)
    }
}
