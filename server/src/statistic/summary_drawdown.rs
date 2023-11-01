use crate::portfolio::position::Position;

use super::metric::{
    drawdown::{AvgDrawdown, Drawdown, MaxDrawdown},
    EquityPoint,
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct DrawdownSummary {
    pub current_drawdown: Drawdown,
    pub avg_drawdown: AvgDrawdown,
    pub max_drawdown: MaxDrawdown,
}

impl DrawdownSummary {
    pub fn update(&mut self, position: &Position) {
        // Only update DrawdownSummary with closed Positions
        let equity_point = match position.meta.exit_balance {
            None => return,
            Some(exit_balance) => EquityPoint::from(exit_balance),
        };

        // Updates
        if let Some(ended_drawdown) = self.current_drawdown.update(equity_point) {
            self.avg_drawdown.update(&ended_drawdown);
            self.max_drawdown.update(&ended_drawdown);
        }
    }
    pub fn new(starting_equity: f64) -> Self {
        Self {
            current_drawdown: Drawdown::init(starting_equity),
            avg_drawdown: AvgDrawdown::init(),
            max_drawdown: MaxDrawdown::init(),
        }
    }
}
