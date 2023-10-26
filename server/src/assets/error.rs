use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Binance stream error: {0}")]
    BinanceStreamError(String),
    #[error("Binance client error: {0}")]
    BinanceClientError(String),
    #[error("Failed to serialize/deserialize JSON due to: {0}")]
    JsonSerDe(#[from] serde_json::Error),
}
