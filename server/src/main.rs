// Main modules
mod assets;
mod model;
mod plotting;
mod trading;
mod utils;

use assets::{
    asset_ticker,
    routes::{clear_history, fetch_history, last_kline_time},
};
use env_logger::Builder;
use log::LevelFilter;
use model::routes::create_new_model;
use plotting::routes::plot_chart;
use rocket::catch;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Header;
use rocket::http::Status;
use rocket::{Request, Response};
use std::sync::Arc;
use tokio::sync::watch;
use tokio::sync::Mutex;
use trading::routes::{interval_put, meshetar_status, pair_put, run, stop_all_operations};
use trading::{meshetar::Meshetar, portfolio, routes::balance_sheet};
use utils::{binance_client, database};

pub struct CORS;

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
async fn main() -> Result<(), String> {
    // Sets logging for sqlx to warn and above, info logs are too verbose
    let mut builder = Builder::new();
    builder.filter(None, LevelFilter::Info); // a default for other libs
    builder.filter(Some("sqlx"), LevelFilter::Warn);
    builder.init();

    binance_client::initialize().await?;

    database::initialize().await?;
    let meshetar = Arc::new(Mutex::new(Meshetar::new()));
    let (sender, receiver) = watch::channel(false);
    let task_control = Arc::new(Mutex::new(TaskControl { sender, receiver }));

    // Hook to assets ticker
    tokio::spawn(async {
        match asset_ticker::subscribe().await {
            _ => log::warn!("Price fetching ended."),
        }
    });

    // Periodically get account status
    tokio::spawn(async {
        loop {
            match portfolio::fetch_account_data().await {
                Err(e) => log::warn!("Error fetching balance: {:?}", e),
                _ => (),
            }
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });

    match rocket::build()
        .attach(CORS)
        .manage(meshetar)
        .manage(task_control)
        .mount(
            "/",
            routes![
                all_options,
                meshetar_status,
                stop_all_operations,
                fetch_history,
                clear_history,
                interval_put,
                pair_put,
                last_kline_time,
                run,
                create_new_model,
                plot_chart,
                balance_sheet
            ],
        )
        .mount("/", FileServer::new("static", Options::None).rank(1))
        .register("/", catchers![internal_error, not_found, default])
        .launch()
        .map_err(|e| e.to_string())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
