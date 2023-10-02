use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    assets::{Asset, MarketMeta},
    database::Database,
    events::Event,
    strategy::{Decision, Signal},
    trading::{
        execution::{Fees, FillEvent},
        SignalForceExit,
    },
};

use self::{
    error::PortfolioError,
    position::{determine_position_id, Position},
};

pub mod account;
pub mod balance;
pub mod error;
pub mod position;
pub mod routes;

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub asset: Asset,
    pub decision: Decision,
    pub market_meta: MarketMeta,
    pub quantity: f64,
    pub fees: Fees,
}

pub struct Portfolio {
    database: Database,
    core_id: Uuid,
}

impl Portfolio {
    pub fn builder() -> PortfolioBuilder {
        PortfolioBuilder::new()
    }
    pub async fn generate_order(
        &mut self,
        _signal: &Signal,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        Ok(None)
    }
    pub async fn generate_exit_order(
        &mut self,
        _signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        Ok(None)
    }
    pub async fn update_from_fill(
        &mut self,
        fill: &FillEvent,
    ) -> Result<Vec<Event>, PortfolioError> {
        let mut generated_events: Vec<Event> = Vec::with_capacity(2);
        let mut balance = self.database.get_balance(self.core_id)?;
        let position_id = determine_position_id(self.core_id, &fill.asset);

        match self.database.remove_position(&position_id)? {
            Some(mut position) => {
                let position_exit = position.exit(balance, fill)?;
                generated_events.push(Event::PositionExit(position_exit));
                balance.available += position.enter_value_gross
                    + position.realised_profit_loss
                    + position.enter_fees_total;
                balance.total += position.realised_profit_loss;
                self.database.set_exited_position(self.core_id, position)?;
            }
            None => {
                let position = Position::enter(self.core_id, fill)?;
                generated_events.push(Event::PositionNew(position.clone()));
                balance.available += -position.enter_value_gross - position.enter_fees_total;
                self.database.set_open_position(position)?;
            }
        };
        generated_events.push(Event::Balance(balance));
        self.database.set_balance(self.core_id, balance)?;
        Ok(generated_events)
    }
}

pub struct PortfolioBuilder {
    database: Option<Database>,
    core_id: Option<Uuid>,
}

impl PortfolioBuilder {
    pub fn new() -> Self {
        PortfolioBuilder {
            database: None,
            core_id: None,
        }
    }
    pub fn database(self, database: Database) -> Self {
        Self {
            database: Some(database),
            ..self
        }
    }
    pub fn build(self) -> Result<Portfolio, PortfolioError> {
        let mut portfolio = Portfolio {
            core_id: self
                .core_id
                .ok_or(PortfolioError::BuilderIncomplete("core_id"))?,
            database: self
                .database
                .ok_or(PortfolioError::BuilderIncomplete("database"))?,
        };
        Ok(portfolio)
    }
}
