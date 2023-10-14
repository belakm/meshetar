use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::assets::{Asset, MarketEvent};

use self::error::StrategyError;

pub mod error;
pub mod prediction_model;
pub mod routes;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Signal {
    pub time: DateTime<Utc>,
    pub asset: Asset,
    pub signals: HashMap<Decision, SignalStrength>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum Decision {
    Long,
    CloseLong,
    Short,
    CloseShort,
}

impl Default for Decision {
    fn default() -> Self {
        Self::Long
    }
}

impl Decision {
    pub fn is_long(&self) -> bool {
        matches!(self, Decision::Long)
    }
    pub fn is_short(&self) -> bool {
        matches!(self, Decision::Short)
    }
    pub fn is_entry(&self) -> bool {
        matches!(self, Decision::Short | Decision::Long)
    }
    pub fn is_exit(&self) -> bool {
        matches!(self, Decision::CloseLong | Decision::CloseShort)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct SignalStrength(pub f64);

pub struct Strategy {}
impl Strategy {
    pub fn new() -> Self {
        Strategy {}
    }
    pub async fn generate_signal(
        &mut self,
        _market_event: &MarketEvent,
    ) -> Result<Option<Signal>, StrategyError> {
        // Run model
        Err(StrategyError::NoSignalProduced)
    }
}
