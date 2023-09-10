use std::sync::Arc;

use tokio::sync::{mpsc::Receiver, Mutex};

use crate::{
    database::Database, portfolio::Portfolio, trading::meshetar::Pair,
    utils::binance_client::BinanceClient,
};

use super::error::CoreError;

#[derive(PartialEq, Debug)]
pub enum Command {
    Run(Pair),
    CreateModel(Pair),
    Backtest(Pair),
    Terminate(Pair),
    TerminateAll,
    Plot(Pair),
}

pub struct Core {
    database: Database,
    portfolio: Arc<Mutex<Portfolio>>,
    binance_client: BinanceClient,
    command_rx: Receiver<Command>,
}

impl Core {
    pub fn builder() -> CoreBuilder {
        CoreBuilder::new()
    }
}

struct CoreBuilder {
    database: Option<Database>,
    portfolio: Option<Arc<Mutex<Portfolio>>>,
    binance_client: Option<BinanceClient>,
    command_rx: Option<Receiver<Command>>,
}

impl CoreBuilder {
    pub fn new() -> Self {
        CoreBuilder {
            database: None,
            portfolio: None,
            binance_client: None,
            command_rx: None,
        }
    }
    pub fn database(self, database: Database) -> Self {
        CoreBuilder {
            database: Some(database),
            ..self
        }
    }
    pub fn portfolio(self, portfolio: Arc<Mutex<Portfolio>>) -> Self {
        CoreBuilder {
            portfolio: Some(portfolio),
            ..self
        }
    }
    pub fn binance_client(self, binance_client: BinanceClient) -> Self {
        CoreBuilder {
            binance_client: Some(binance_client),
            ..self
        }
    }
    pub fn command_rx(self, command_rx: Receiver<Command>) -> Self {
        CoreBuilder {
            command_rx: Some(command_rx),
            ..self
        }
    }
    pub fn build(self) -> Result<Core, CoreError> {
        let core = Core {
            database: self
                .database
                .ok_or(CoreError::BuilderIncomplete("database"))?,
            portfolio: self
                .portfolio
                .ok_or(CoreError::BuilderIncomplete("portfolio"))?,
            binance_client: self
                .binance_client
                .ok_or(CoreError::BuilderIncomplete("binance client"))?,
            command_rx: self
                .command_rx
                .ok_or(CoreError::BuilderIncomplete("command reciever"))?,
        };
        Ok(core)
    }
}
