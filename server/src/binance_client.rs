pub static BINANCE_CLIENT: OnceCell<BinanceHttpClient<HttpsConnector<HttpConnector>>> =
    OnceCell::const_new();
use crate::load_config::{read_config, Config};
use binance_spot_connector_rust::{http::Credentials, hyper::BinanceHttpClient};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio::sync::OnceCell;

pub async fn initialize() -> Result<(), String> {
    log::info!("Initializing client.");
    let config: Config = read_config();
    let credentials = Credentials::from_hmac(
        config.binance_api_key.to_owned(),
        config.binance_api_secret.to_owned(),
    );
    BINANCE_CLIENT.set(BinanceHttpClient::default().credentials(credentials));
    Ok(())
}
