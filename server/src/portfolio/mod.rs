pub mod account;
pub mod balance;

use crate::database::error::DatabaseError;

use self::{account::Account, balance::Balance};

pub struct Portfolio<Database>
where
    Database: BalanceHandler + AccountHandler,
{
    database: Database,
}

pub trait BalanceHandler {
    fn set_balance(&mut self, balance: Balance) -> Result<(), DatabaseError>;
    fn get_balance(&mut self) -> Result<Balance, DatabaseError>;
}

pub trait AccountHandler {
    fn set_account(&mut self, account: Account) -> Result<(), DatabaseError>;
    fn get_account(&mut self) -> Result<Account, DatabaseError>;
}
