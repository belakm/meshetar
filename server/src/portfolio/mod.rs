pub mod account;
pub mod allocator;
pub mod balance;
pub mod error;
pub mod position;
pub mod risk;
pub mod routes;

use self::{
    allocator::Allocator,
    balance::Balance,
    error::PortfolioError,
    position::{determine_position_id, Position},
    risk::RiskEvaluator,
};
use crate::{
    assets::{Asset, MarketMeta, Side},
    database::Database,
    events::Event,
    statistic::{Config, Statistic},
    strategy::{Decision, Signal, SignalStrength},
    trading::{execution::FillEvent, SignalForceExit},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub asset: Asset,
    pub decision: Decision,
    pub market_meta: MarketMeta,
    pub quantity: f64,
}

pub struct Portfolio {
    database: Arc<Mutex<Database>>,
    core_id: Uuid,
    allocation_manager: Allocator,
    risk_manager: RiskEvaluator,
    starting_cash: f64,
}

impl Portfolio {
    pub fn builder() -> PortfolioBuilder {
        PortfolioBuilder::new()
    }
    pub async fn bootstrap_database(
        &self,
        statistic_config: Config,
        assets: Vec<Asset>,
    ) -> Result<(), PortfolioError> {
        self.database.lock().await.set_balance(
            self.core_id,
            Balance {
                time: Utc::now(),
                total: self.starting_cash,
                available: self.starting_cash,
            },
        )?;
        let database = self.database.lock().await;
        for asset in assets {
            database
                .set_statistics(asset, Statistic::init(statistic_config))
                .map_err(PortfolioError::RepositoryInteraction)?;
        }
        Ok(())
    }
    pub async fn generate_order(
        &mut self,
        signal: &Signal,
    ) -> Result<Option<OrderEvent>, PortfolioError> {
        info!("Generating new order.");

        // Determine the position_id & associated Option<Position> related to input SignalEvent
        let position_id = determine_position_id(self.core_id, &signal.asset);
        let position = { self.database.lock().await.get_open_position(&position_id)? };

        info!(
            "Determining liquidity neccessary for new order. {:?}",
            &position
        );
        // If signal is advising to open a new Position rather than close one, check we have cash
        if position.is_none() && self.no_cash_to_enter_new_position().await? {
            info!("No cash available to open a new position.");
            return Ok(None);
        }

        // Parse signals from Strategy to determine net signal decision & associated strength
        let position = position.as_ref();
        info!("Parsing signal decision");
        let (signal_decision, signal_strength) =
            match parse_signal_decisions(&position, &signal.signals) {
                None => return Ok(None),
                Some(net_signal) => net_signal,
            };

        // Construct mutable OrderEvent that can be modified by Allocation & Risk management
        let mut order = OrderEvent {
            time: Utc::now(),
            asset: signal.asset.clone(),
            market_meta: signal.market_meta,
            decision: *signal_decision,
            quantity: 0.0,
        };

        info!("Allocating order");
        // Manage OrderEvent size allocation
        self.allocation_manager
            .allocate_order(&mut order, position, *signal_strength);

        info!("Running safety check");
        // Manage global risk when evaluating OrderEvent - keep the same, refine or cancel
        Ok(self.risk_manager.evaluate_order(order))
    }
    async fn no_cash_to_enter_new_position(&mut self) -> Result<bool, PortfolioError> {
        info!("Start check");
        let res = self
            .database
            .lock()
            .await
            .get_balance(self.core_id)
            .map(|balance| {
                info!("balance {:?}", &balance);
                Ok(balance.available == 0.0)
            })
            .map_err(PortfolioError::RepositoryInteraction)?;
        info!("End check");
        res
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
        let mut database = self.database.lock().await;
        let mut balance = database.get_balance(self.core_id)?;
        let position_id = determine_position_id(self.core_id, &fill.asset);

        match database.remove_position(&position_id)? {
            Some(mut position) => {
                let position_exit = position.exit(balance, fill)?;
                generated_events.push(Event::PositionExit(position_exit));
                balance.available += position.enter_value_gross
                    + position.realised_profit_loss
                    + position.enter_fees_total;
                balance.total += position.realised_profit_loss;
                database.set_exited_position(self.core_id, position)?;
            }
            None => {
                let position = Position::enter(self.core_id, fill)?;
                generated_events.push(Event::PositionNew(position.clone()));
                balance.available += -position.enter_value_gross - position.enter_fees_total;
                database.set_open_position(position)?;
            }
        };
        generated_events.push(Event::Balance(balance));
        database.set_balance(self.core_id, balance)?;
        Ok(generated_events)
    }
}

fn parse_signal_decisions<'a>(
    position: &'a Option<&Position>,
    signals: &'a HashMap<Decision, SignalStrength>,
) -> Option<(&'a Decision, &'a SignalStrength)> {
    let signal_close_long = signals.get_key_value(&Decision::CloseLong);
    let signal_long = signals.get_key_value(&Decision::Long);
    let signal_close_short = signals.get_key_value(&Decision::CloseShort);
    let signal_short = signals.get_key_value(&Decision::Short);

    // If an existing Position exists, check for net close signals
    if let Some(position) = position {
        return match position.side {
            Side::Buy if signal_close_long.is_some() => signal_close_long,
            Side::Sell if signal_close_short.is_some() => signal_close_short,
            _ => None,
        };
    }

    // Else check for net open signals
    match (signal_long, signal_short) {
        (Some(signal_long), None) => Some(signal_long),
        (None, Some(signal_short)) => Some(signal_short),
        _ => None,
    }
}

pub struct PortfolioBuilder {
    database: Option<Arc<Mutex<Database>>>,
    core_id: Option<Uuid>,
    allocation_manager: Option<Allocator>,
    risk_manager: Option<RiskEvaluator>,
    starting_cash: Option<f64>,
}

impl PortfolioBuilder {
    pub fn new() -> Self {
        PortfolioBuilder {
            database: None,
            core_id: None,
            allocation_manager: None,
            risk_manager: None,
            starting_cash: None,
        }
    }
    pub fn database(self, database: Arc<Mutex<Database>>) -> Self {
        Self {
            database: Some(database),
            ..self
        }
    }
    pub fn core_id(self, value: Uuid) -> Self {
        Self {
            core_id: Some(value),
            ..self
        }
    }
    pub fn allocation_manager(self, value: Allocator) -> Self {
        Self {
            allocation_manager: Some(value),
            ..self
        }
    }
    pub fn risk_manager(self, value: RiskEvaluator) -> Self {
        Self {
            risk_manager: Some(value),
            ..self
        }
    }
    pub fn build(self) -> Result<Portfolio, PortfolioError> {
        let portfolio = Portfolio {
            core_id: self
                .core_id
                .ok_or(PortfolioError::BuilderIncomplete("core_id"))?,
            allocation_manager: self
                .allocation_manager
                .ok_or(PortfolioError::BuilderIncomplete("allocation_manager"))?,
            risk_manager: self
                .risk_manager
                .ok_or(PortfolioError::BuilderIncomplete("risk_manager"))?,
            starting_cash: self
                .starting_cash
                .ok_or(PortfolioError::BuilderIncomplete("risk_manager"))?,
            database: self
                .database
                .ok_or(PortfolioError::BuilderIncomplete("database"))?,
        };
        Ok(portfolio)
    }
}
