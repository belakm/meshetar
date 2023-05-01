#[derive(serde::Deserialize)]
pub struct Config {
    pub binance_api_key: String,
    pub binance_api_secret: String,
}

pub fn read_config() -> Config {
    let config_file = std::fs::read_to_string("config.toml").unwrap();
    let config: Config = toml::from_str(&config_file).expect("Could not parse config file");
    config
}
