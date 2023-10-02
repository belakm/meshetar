use thiserror::Error;

/// All errors generated in the barter::portfolio module.
#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("No signal produced")]
    NoSignalProduced,
}
