mod assets;
mod core;
mod database;
mod events;
mod plotting;
mod portfolio;
mod statistic;
mod strategy;
mod trading;
mod utils;

use crate::core::{error::CoreError, Command, Core};
use assets::{error::AssetError, Asset, MarketFeed};
use database::{error::DatabaseError, Database};
use env_logger::Builder;
use events::{core_events_listener, EventTx};
use log::LevelFilter;
use portfolio::{allocator::Allocator, error::PortfolioError, risk::RiskEvaluator, Portfolio};
use rocket::{
    catch,
    fairing::{Fairing, Info, Kind},
    futures::TryFutureExt,
    http::Header,
    http::Status,
    Error as RocketError, Request, Response,
};
use statistic::{StatisticConfig, TradingSummary};
use std::{collections::HashMap, sync::Arc};
use strategy::Strategy;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};
use trading::{error::TraderError, execution::Execution, Trader};
use utils::binance_client::{BinanceClient, BinanceClientError};
use uuid::Uuid;

const IS_LIVE: bool = false;
const BACKTEST_LAST_N_CANDLES: usize = 1490;
const FETCH_N_DAYS_HISTORY: i64 = 2;

pub struct CORS;

#[derive(Error, Debug)]
enum MainError {
    #[error("Portfolio error: {0}")]
    Portfolio(#[from] PortfolioError),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
    #[error("Trader error: {0}")]
    Trader(#[from] TraderError),
    #[error("Binance client error: {0}")]
    BinanceClient(#[from] BinanceClientError),
    #[error("Assets: {0}")]
    Asset(#[from] AssetError),
    #[error("Rocket server error: {0}")]
    Rocket(#[from] RocketError),
}

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }
    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// Rocket server
// macro needs to in the root crate
// ...
#[macro_use]
extern crate rocket;

#[catch(500)]
fn internal_error() -> &'static str {
    "Error 500; something is not clicking right."
}

#[catch(404)]
fn not_found() -> &'static str {
    "Error 404; nothing here fren."
}

#[catch(default)]
fn default(status: Status, req: &Request) -> String {
    format!("{} ({})", status, req.uri())
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
/// https://stackoverflow.com/a/72702246
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

pub struct TaskControl {
    sender: watch::Sender<bool>,
    receiver: watch::Receiver<bool>,
}

fn main() {
    match run() {
        Ok(_) => info!("Leaving Meshetar. See you soon! :)"),
        Err(e) => error!("Whoops, error: {}", e),
    }
}

#[rocket::main]
async fn run() -> Result<(), MainError> {
    // Point PYTHONHOME to the virtual environment directory
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Info); // a default for other libs
    builder.filter(Some("sqlx"), LevelFilter::Warn);
    builder.init();
    let core_id = Uuid::new_v4();
    let (event_transmitter, event_receiver) = mpsc::unbounded_channel();
    let event_transmitter = EventTx::new(event_transmitter);
    let database: Arc<Mutex<Database>> =
        Arc::new(Mutex::new(Database::new().map_err(MainError::from).await?));
    let portfolio: Arc<Mutex<Portfolio>> = Arc::new(Mutex::new(
        Portfolio::builder()
            .database(database.clone())
            .core_id(core_id.clone())
            .allocation_manager(Allocator {
                default_order_value: 100.0,
            })
            .risk_manager(RiskEvaluator {})
            .starting_cash(1000.0)
            .assets(vec![Asset::BTCUSDT])
            .statistic_config(StatisticConfig {
                starting_equity: 1000.0,
                trading_days_per_year: 365,
                risk_free_return: 0.0,
            })
            .build()
            .await?,
    ));

    let mut traders = Vec::new();
    let (command_transmitter, command_receiver) = mpsc::channel::<Command>(20);
    let (trader_command_transmitter, trader_command_receiver) = mpsc::channel::<Command>(20);
    let command_transmitters = HashMap::from([(Asset::BTCUSDT, trader_command_transmitter)]);
    traders.push(
        Trader::builder()
            .core_id(core_id)
            .asset(Asset::BTCUSDT)
            .is_live(IS_LIVE)
            .command_reciever(trader_command_receiver)
            .event_transmitter(event_transmitter)
            .portfolio(Arc::clone(&portfolio))
            .market_feed(MarketFeed::new(
                Asset::BTCUSDT,
                IS_LIVE,
                database.clone(),
                BACKTEST_LAST_N_CANDLES,
            ))
            .strategy(Strategy::new(Asset::BTCUSDT))
            .execution(Execution::new())
            .build()?,
    );

    let core = Core::builder()
        .id(core_id)
        .binance_client(BinanceClient::new().map_err(MainError::from).await?)
        .portfolio(portfolio)
        .command_reciever(command_receiver)
        .command_transmitters(command_transmitters)
        .traders(traders)
        .database(database.clone())
        .statistics_summary(TradingSummary::init(StatisticConfig {
            starting_equity: 1000.0,
            trading_days_per_year: 365,
            risk_free_return: 0.0,
        }))
        .n_days_history_fetch(FETCH_N_DAYS_HISTORY)
        .build()?;

    let listener_task = tokio::spawn(core_events_listener(event_receiver, database, IS_LIVE));
    // let _ = tokio::time::timeout(Duration::from_secs(20), core.run());
    let core_task = tokio::spawn(async move { core.run().await });
    let (core_result, listener_result) = tokio::join!(core_task, listener_task);
    if let Err(core_error) = core_result {
        error!("{}", core_error);
    }
    if let Err(listener_error) = listener_result {
        error!("{}", listener_error);
    }

    //let _3 = tokio::spawn(async { loop {} }).await;

    // rocket::build()
    //     .attach(CORS)
    //     .mount(
    //         "/",
    //         routes![
    //             all_options, // needed for Rocket to serve to browsers
    //                          // create_new_model,
    //                          // balance_sheet,
    //                          // plot_chart,
    //                          // meshetar_status,
    //                          // interval_put,
    //                          // stop_all_operations,
    //                          // fetch_history,
    //                          // clear_history,
    //                          // last_kline_time,
    //                          // run,
    //                          // plot_chart,
    //         ],
    //     )
    //     .mount("/", FileServer::new("static", Options::None).rank(1))
    //     .register("/", catchers![internal_error, not_found, default])
    //     .launch()
    //     .map_err(MainError::from)
    //     .await?;

    Ok(())
}
