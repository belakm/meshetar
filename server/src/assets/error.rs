use thiserror::Error;

/// All errors generated in the barter::portfolio module.
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Binance stream error: {0}")]
    BinanceStreamError(String),
}
