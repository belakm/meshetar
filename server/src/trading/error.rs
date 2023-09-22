use thiserror::Error;

/// All errors generated in the barter::portfolio module.
#[derive(Error, Debug)]
pub enum TraderError {
    #[error("Failed to build core due to missing attributes: {0}")]
    BuilderIncomplete(&'static str),
}
