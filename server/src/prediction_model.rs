use std::sync::Arc;

use strum::{Display, EnumString};
use tokio::sync::Mutex;

use crate::{rlang_runner, TaskControl};

#[derive(Debug, Copy, Clone, Display, EnumString)]
pub enum TradeSignal {
    Hold,
    Buy,
    Sell,
}

pub async fn run_model(task_control: Arc<Mutex<TaskControl>>) -> Result<TradeSignal, String> {
    match rlang_runner::run_script("models/default_run.R", task_control).await {
        Ok(signal) => match signal.as_str() {
            "buy" => Ok(TradeSignal::Buy),
            "sell" => Ok(TradeSignal::Sell),
            "hold" => Ok(TradeSignal::Hold),
            _ => Err(format!(
                "Model runner encountered unexpected signal: {:?}",
                &signal
            )),
        },
        Err(e) => Err(e),
    }
}

pub async fn create_model(task_control: Arc<Mutex<TaskControl>>) -> Result<(), String> {
    match rlang_runner::run_script("models/default_create.R", task_control).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}", e)),
    }
}
