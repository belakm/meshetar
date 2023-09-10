use crate::utils::load_config::{read_config, Config};
use binance_spot_connector_rust::{http::Credentials, hyper::BinanceHttpClient};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use thiserror::Error;
use tokio::sync::OnceCell;

use super::load_config::ConfigError;

pub const BINANCE_WSS_BASE_URL: &str = "wss://stream.binance.com:9443/ws";

pub static BINANCE_CLIENT: OnceCell<BinanceHttpClient<HttpsConnector<HttpConnector>>> =
    OnceCell::const_new();

pub struct BinanceClient {
    client: BinanceHttpClient<HttpsConnector<HttpConnector>>,
}

#[derive(Error, Debug)]
pub enum BinanceClientError {
    #[error("Init failed {0}")]
    ConfigOnInit(#[from] ConfigError),
}

impl BinanceClient {
    pub async fn new() -> Result<BinanceClient, BinanceClientError> {
        let config: Config = read_config().map_err(|e| BinanceClientError::ConfigOnInit(e))?;
        let credentials = Credentials::from_hmac(
            config.binance_api_key.to_owned(),
            config.binance_api_secret.to_owned(),
        );
        let client =
            BinanceHttpClient::with_url("https://testnet.binance.vision").credentials(credentials);
        Ok(BinanceClient { client })
    }
}
