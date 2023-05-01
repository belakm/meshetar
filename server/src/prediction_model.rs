use crate::rlang_runner;

#[derive(Debug)]
pub enum TradeSignal {
    Hold,
    Buy,
    Sell,
}

pub async fn run() -> Result<TradeSignal, String> {
    println!("Creating new model.");
    match create().await {
        Ok(_) => {
            println!("Running new model.");
            match rlang_runner::r_script("models/default_run.R", None) {
                Ok(signal) => match &signal[..] {
                    "buy" => Ok(TradeSignal::Buy),
                    "sell" => Ok(TradeSignal::Sell),
                    "hold" => Ok(TradeSignal::Hold),
                    _ => {
                        println!("Model runner encountered unexpected signal: {:?}", &signal);
                        Ok(TradeSignal::Hold)
                    }
                },
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

async fn create() -> Result<(), String> {
    match rlang_runner::r_script("models/default_create.R", None) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
