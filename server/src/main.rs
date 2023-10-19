mod assets;
mod core;
mod database;
mod events;
mod plotting;
mod portfolio;
mod strategy;
mod trading;
mod utils;

use assets::{error::AssetError, Asset, Candle, MarketEvent, MarketEventDetail, MarketFeed};
use core::{error::CoreError, Command, Core};
use database::{error::DatabaseError, Database};
use env_logger::Builder;
use events::{core_events_listener, Event, EventTx};
use log::LevelFilter;
use portfolio::{error::PortfolioError, Portfolio};
use rocket::{
    catch,
    fairing::{Fairing, Info, Kind},
    fs::FileServer,
    fs::Options,
    futures::TryFutureExt,
    http::Header,
    http::Status,
    Error as RocketError, Request, Response,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use strategy::Strategy;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info};
use trading::{error::TraderError, execution::Execution, Trader};
use utils::binance_client::{self, BinanceClient, BinanceClientError};
use uuid::Uuid;

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
    #[error("Asset Feed error: {0}")]
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
    // Sets logging for sqlx to warn and above, info logs are too verbose
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Info); // a default for other libs
    builder.filter(Some("sqlx"), LevelFilter::Warn);
    builder.init();

    let core_id = Uuid::new_v4();

    let (event_transmitter, event_receiver) = mpsc::unbounded_channel();
    let event_transmitter = EventTx::new(event_transmitter);
    let portfolio: Arc<Mutex<Portfolio>> = Arc::new(Mutex::new(
        Portfolio::builder()
            .database(Database::new().map_err(MainError::from).await?)
            .core_id(core_id.clone())
            .build()
            .map_err(MainError::from)?,
    ));

    let mut traders = Vec::new();
    let (command_transmitter, command_receiver) = mpsc::channel::<Command>(20);
    let (trader_command_transmitter, trader_command_receiver) = mpsc::channel::<Command>(20);
    let command_transmitters = HashMap::from([(Asset::BTCUSDT, trader_command_transmitter)]);

    traders.push(
        Trader::builder()
            .core_id(core_id)
            .asset(Asset::BTCUSDT)
            .command_reciever(trader_command_receiver)
            .event_transmitter(event_transmitter)
            .portfolio(Arc::clone(&portfolio))
            .market_feed(MarketFeed {
                market_receiver: MarketFeed::new(Asset::BTCUSDT).await?.market_receiver,
            })
            .strategy(Strategy::new())
            .execution(Execution::new())
            .build()?,
    );

    let mut core = Core::builder()
        .id(core_id)
        .binance_client(BinanceClient::new().map_err(MainError::from).await?)
        .portfolio(portfolio)
        .command_reciever(command_receiver)
        .command_transmitters(command_transmitters)
        .traders(traders)
        .build()?;

    tokio::spawn(core_events_listener(event_receiver));
    let _ = tokio::time::timeout(Duration::from_secs(120), core.run()).await;

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

/// TESTING
///
///
async fn stream_market_event_trades() -> mpsc::UnboundedReceiver<MarketEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut candles = load_json_market_event_candles().into_iter();
        while let Some(event) = candles.next() {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            tx.send(event);
        }
    });
    rx
}

fn load_json_market_event_candles() -> Vec<MarketEvent> {
    let candles = r#"[
  {
    "start_time": "2022-04-05 20:00:00.000000000 UTC",
    "close_time": "2022-04-05 21:00:00.000000000 UTC",
    "open": 1000.0,
    "high": 1100.0,
    "low": 900.0,
    "close": 1050.0,
    "volume": 1000000000.0,
    "trade_count": 100
  },
  {
    "start_time": "2022-04-05 21:00:00.000000000 UTC",
    "close_time": "2022-04-05 22:00:00.000000000 UTC",
    "open": 1050.0,
    "high": 1100.0,
    "low": 800.0,
    "close": 1060.0,
    "volume": 1000000000.0,
    "trade_count": 50
  },
  {
    "start_time": "2022-04-05 22:00:00.000000000 UTC",
    "close_time": "2022-04-05 23:00:00.000000000 UTC",
    "open": 1060.0,
    "high": 1200.0,
    "low": 800.0,
    "close": 1200.0,
    "volume": 1000000000.0,
    "trade_count": 200
  },
  {
    "start_time": "2022-04-05 23:00:00.000000000 UTC",
    "close_time": "2022-04-06 00:00:00.000000000 UTC",
    "open": 1200.0,
    "high": 1200.0,
    "low": 1100.0,
    "close": 1300.0,
    "volume": 1000000000.0,
    "trade_count": 500
  }
]"#;

    let candles =
        serde_json::from_str::<Vec<Candle>>(&candles).expect("failed to parse candles String");

    candles
        .into_iter()
        .map(|candle| MarketEvent {
            time: candle.close_time,
            asset: Asset::BTCUSDT,
            detail: MarketEventDetail::Candle(candle),
        })
        .collect()
}

// Listen to Events that occur in the Engine. These can be used for updating event-sourcing,
// updating dashboard, etc etc.
async fn listen_to_engine_events(mut event_rx: mpsc::UnboundedReceiver<Event>) {
    while let Some(event) = event_rx.recv().await {
        debug!("EVENT: {:?}\n", &event);
        match event {
            Event::Market(_) => {
                // Market Event occurred in Engine
            }
            Event::Signal(signal) => {
                // Signal Event occurred in Engine
                println!("{signal:?}");
            }
            Event::SignalForceExit(_) => {
                // SignalForceExit Event occurred in Engine
            }
            Event::Order(new_order) => {
                // OrderNew Event occurred in Engine
                println!("{new_order:?}");
            }
            Event::Fill(fill_event) => {
                // Fill Event occurred in Engine
                println!("{fill_event:?}");
            }
            Event::PositionNew(new_position) => {
                // PositionNew Event occurred in Engine
                println!("{new_position:?}");
            }
            Event::PositionUpdate(updated_position) => {
                // PositionUpdate Event occurred in Engine
                println!("{updated_position:?}");
            }
            Event::PositionExit(exited_position) => {
                // PositionExit Event occurred in Engine
                println!("{exited_position:?}");
            }
            Event::Balance(balance_update) => {
                // Balance update Event occurred in Engine
                println!("{balance_update:?}");
            }
        }
    }
}
