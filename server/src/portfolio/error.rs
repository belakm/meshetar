use thiserror::Error;

/// All errors generated in the barter::portfolio module.
#[derive(Error, Debug)]
pub enum PortfolioError {
    #[error("Failed to build struct due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),
}
