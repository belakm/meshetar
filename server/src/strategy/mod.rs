extern crate cpython;

use self::error::StrategyError;
use crate::assets::{Asset, MarketEvent};
use chrono::{DateTime, Utc};
use cpython::{NoArgs, PyModule, PyResult, Python};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub struct Strategy {
    asset: Asset,
}
impl Strategy {
    pub fn new(asset: Asset) -> Self {
        Strategy { asset }
    }
    pub async fn generate_signal(
        &mut self,
        _market_event: &MarketEvent,
    ) -> Result<Option<Signal>, StrategyError> {
        // Run model
        let pyscript = include_str!("../../models/run_model.py");
        let model_output = run_python_script(pyscript)?;
        let signals = generate_signals_map(&model_output);
        if signals.len() == 0 {
            return Ok(None);
        }
        let time = Utc::now();
        let signal = Signal {
            time,
            asset: self.asset.clone(),
            signals,
        };
        Ok(Some(signal))
    }
}

fn generate_signals_map(model_output: &str) -> HashMap<Decision, SignalStrength> {
    let mut signals = HashMap::with_capacity(4);
    match model_output {
        "buy" => {
            signals.insert(Decision::Short, SignalStrength(1.0));
            signals.insert(Decision::CloseLong, SignalStrength(1.0));
        }
        "sell" => {
            signals.insert(Decision::Long, SignalStrength(1.0));
            signals.insert(Decision::CloseShort, SignalStrength(1.0));
        }
        _ => (),
    };
    signals
}

fn run_python_script(script: &str) -> PyResult<String> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let main_module = PyModule::import(py, "__main__")?;
    py.run(script, Some(&main_module.dict(py)), None)?;

    let output: String = main_module.call(py, "run", NoArgs, None)?.extract(py)?;
    info!("{}", output);
    Ok(output)
}
