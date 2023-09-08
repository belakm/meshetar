use crate::database::Database;

use self::error::PortfolioError;

pub mod account;
pub mod balance;
pub mod error;

pub struct Portfolio {
    database: Database,
}

impl Portfolio {
    pub fn builder() -> PortfolioBuilder {
        PortfolioBuilder::new()
    }
}

pub struct PortfolioBuilder {
    database: Option<Database>,
}

impl PortfolioBuilder {
    pub fn new() -> Self {
        PortfolioBuilder { database: None }
    }
    pub fn database(self, database: Database) -> Self {
        Self {
            database: Some(database),
            ..self
        }
    }
    pub fn build(self) -> Result<Portfolio, PortfolioError> {
        let mut portfolio = Portfolio {
            database: self
                .database
                .ok_or(PortfolioError::BuilderIncomplete("database"))?,
        };
        Ok(portfolio)
    }
}
