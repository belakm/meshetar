use crate::portfolio::{AccountHandler, BalanceHandler};

use self::error::DatabaseError;

pub mod error;

pub struct Database {}

impl BalanceHandler for Database {
    fn set_balance(&mut self, balance: Balance) -> Result<(), DatabaseError> {}
    fn get_balance(&mut self) -> Result<Balance, DatabaseError> {}
}
impl AccountHandler for Database {
    fn set_account(&mut self, account: Account) -> Result<(), DatabaseError> {}
    fn get_account(&mut self) -> Result<Account, DatabaseError> {}
}
