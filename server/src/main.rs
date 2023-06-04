// Main modules
//
mod binance_client;
mod book;
mod database;
mod formatting;
mod load_config;
mod meshetar;
mod prediction_model;
mod rlang_runner;
// mod api;
// mod plot;

use book::latest_kline_date;
use hyper::server::accept::Accept;
use meshetar::Interval;
use meshetar::Meshetar;
use meshetar::Pair;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
// Dependencies
//
use rocket::catch;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Header;
use rocket::http::Status;
use rocket::response::status::Accepted;
use rocket::{Request, Response};

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

#[rocket::main]
async fn main() -> Result<(), String> {
    log::info!("Igniting rocket.");
    binance_client::initialize().await?;
    database::initialize().await?;
    let meshetar = Arc::new(Mutex::new(Meshetar::new()));

    match rocket::build()
        .attach(CORS)
        .manage(meshetar)
        .mount(
            "/",
            routes![
                history_fetch,
                meshetar_status,
                interval_put,
                pair_put,
                all_options,
                last_kline_time
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

#[post("/history_fetch")]
async fn history_fetch(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<String> {
    tokio::join!(async {
        let mut m = meshetar.lock().await; // Change the field in Meshetar
        m.status = meshetar::Status::FetchingHistory;
        let pair = m.pair.to_string();
        match book::fetch_history(pair).await {
            Ok(_) => log::info!("History fetching success."),
            Err(e) => log::info!("History fetching err: {:?}", e),
        };
        m.status = meshetar::Status::Idle;
    });
    Accepted(Some(meshetar.lock().await.summerize()))
}

#[get("/status")]
async fn meshetar_status(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
    let meshetar = meshetar.lock().await;
    Accepted(Some(meshetar.summerize_json()))
}

#[get("/last_kline_time")]
async fn last_kline_time() -> Accepted<String> {
    let last_kline_time = latest_kline_date().await.unwrap();
    Accepted(Some(last_kline_time.to_string()))
}

#[derive(FromForm, Deserialize)]
struct IntervalPutPayload<'r> {
    interval: &'r str,
}
#[put("/interval", data = "<data>")]
async fn interval_put(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    data: Form<IntervalPutPayload<'_>>,
) -> Accepted<String> {
    let mut meshetar = meshetar.lock().await;
    match meshetar.status {
        meshetar::Status::Idle => match data.interval {
            "Minutes3" => {
                meshetar.interval = Interval::Minutes3;
                Accepted(Some(meshetar.interval.to_string()))
            }
            "Minutes1" => {
                meshetar.interval = Interval::Minutes1;
                Accepted(Some(meshetar.interval.to_string()))
            }
            _ => Accepted(Some(meshetar.interval.to_string())),
        },
        _ => Accepted(Some("Currently working on something else.".to_string())),
    }
}

#[derive(FromForm, Deserialize)]
struct PairPutPayload<'r> {
    pair: &'r str,
}
#[put("/pair", data = "<data>")]
async fn pair_put(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    data: Form<PairPutPayload<'_>>,
) -> Accepted<String> {
    let mut meshetar = meshetar.lock().await;
    match meshetar.status {
        meshetar::Status::Idle => match data.pair {
            "BTCUSDT" => {
                meshetar.pair = Pair::BTCUSDT;
                Accepted(Some(data.pair.to_string()))
            }
            "ETHBTC" => {
                meshetar.pair = Pair::ETHBTC;
                Accepted(Some(data.pair.to_string()))
            }
            _ => Accepted(Some(meshetar.pair.to_string())),
        },
        _ => Accepted(Some("Currently working on something else.".to_string())),
    }
}
