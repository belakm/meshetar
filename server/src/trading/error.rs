use thiserror::Error;

use crate::portfolio::error::PortfolioError;

/// All errors generated in the barter::portfolio module.
#[derive(Error, Debug)]
pub enum TraderError {
    #[error("Failed to build core due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),
    #[error("Failed to build fill event due to missing attributes: {0}")]
    FillBuilderIncomplete(&'static str),
    #[error("Failed to interact with Portfolio")]
    RepositoryInteraction(#[from] PortfolioError),
}
