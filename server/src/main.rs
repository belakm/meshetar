mod assets;
mod core;
mod database;
mod events;
mod model;
mod plotting;
mod portfolio;
mod trading;
mod utils;

use core::{Command, Core};
use database::{error::DatabaseError, Database};
use env_logger::Builder;
use events::EventTx;
use log::LevelFilter;
use model::routes::create_new_model;
use plotting::routes::plot_chart;
use portfolio::routes::balance_sheet;
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
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, watch};
use utils::binance_client::{self, BinanceClient, BinanceClientError};

pub struct CORS;

#[derive(Error, Debug)]
enum MainError {
    #[error("Portfolio error: {0}")]
    Portfolio(#[from] PortfolioError),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Binance client error: {0}")]
    BinanceClient(#[from] BinanceClientError),
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

#[rocket::main]
async fn main() -> Result<(), MainError> {
    // Sets logging for sqlx to warn and above, info logs are too verbose
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Info); // a default for other libs
    builder.filter(Some("sqlx"), LevelFilter::Warn);
    builder.init();

    let (_command_tx, command_rx) = mpsc::channel::<Command>(20);
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let event_tx = EventTx::new(event_tx);

    let portfolio: Arc<Mutex<Portfolio>> = Arc::new(Mutex::new(
        Portfolio::builder()
            .database(Database::new().map_err(MainError::from).await?)
            .build()
            .map_err(MainError::from)?,
    ));

    let core = Arc::new(Mutex::new(
        Core::builder()
            .database(Database::new().await?)
            .binance_client(BinanceClient::new().map_err(MainError::from).await?)
            .portfolio(portfolio)
            .command_rx(command_rx)
            .build(),
    ));

    let ignition = rocket::build()
        .attach(CORS)
        .manage(core)
        .mount(
            "/",
            routes![
                all_options, // needed for Rocket to serve to browsers
                create_new_model,
                balance_sheet,
                plot_chart,
                /*meshetar_status,
                interval_put,
                stop_all_operations,
                fetch_history,
                clear_history,
                last_kline_time,
                run,
                plot_chart,*/
            ],
        )
        .mount("/", FileServer::new("static", Options::None).rank(1))
        .register("/", catchers![internal_error, not_found, default])
        .launch()
        .map_err(MainError::from)
        .await?;

    Ok(())
}
