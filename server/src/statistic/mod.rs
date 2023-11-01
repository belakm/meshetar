pub mod dispersion;
pub mod error;
pub mod metric;
pub mod summary_drawdown;
pub mod summary_pnl;
pub mod welford_online;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::portfolio::position::Position;

use self::{
    metric::ratio::{CalmarRatio, SharpeRatio, SortinoRatio},
    summary_drawdown::DrawdownSummary,
    summary_pnl::PnLReturnSummary,
};

#[derive(Copy, Clone)]
pub struct Statistic {}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct TradingSummary {
    pub pnl_returns: PnLReturnSummary,
    pub drawdown: DrawdownSummary,
    pub tear_sheet: TearSheet,
}

impl TradingSummary {
    fn init(config: Config) -> Self {
        Self {
            pnl_returns: PnLReturnSummary::new(),
            drawdown: DrawdownSummary::new(config.starting_equity),
            tear_sheet: TearSheet::new(config.risk_free_return),
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Config {
    pub starting_equity: f64,
    pub trading_days_per_year: usize,
    pub risk_free_return: f64,
}

pub fn calculate_trading_duration(start_time: &DateTime<Utc>, position: &Position) -> Duration {
    match position.meta.exit_balance {
        None => {
            // Since Position is not exited, estimate duration w/ last_update_time
            position.meta.update_time.signed_duration_since(*start_time)
        }
        Some(exit_balance) => exit_balance.time.signed_duration_since(*start_time),
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct TearSheet {
    pub sharpe_ratio: SharpeRatio,
    pub sortino_ratio: SortinoRatio,
    pub calmar_ratio: CalmarRatio,
}

impl TearSheet {
    pub fn new(risk_free_return: f64) -> Self {
        Self {
            sharpe_ratio: SharpeRatio::init(risk_free_return),
            sortino_ratio: SortinoRatio::init(risk_free_return),
            calmar_ratio: CalmarRatio::init(risk_free_return),
        }
    }

    pub fn update(&mut self, pnl_returns: &PnLReturnSummary, drawdown: &DrawdownSummary) {
        self.sharpe_ratio.update(pnl_returns);
        self.sortino_ratio.update(pnl_returns);
        self.calmar_ratio
            .update(pnl_returns, drawdown.max_drawdown.drawdown.drawdown);
    }
}
