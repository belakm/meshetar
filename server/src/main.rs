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

use meshetar::Meshetar;
use rocket::State;
use std::sync::Arc;
// Dependencies
//
use rocket::catch;
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::status::Accepted;
use tokio::sync::Mutex;

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

#[rocket::main]
async fn main() -> Result<(), String> {
    log::info!("Igniting rocket.");
    binance_client::initialize().await?;
    database::initialize().await?;
    let meshetar = Arc::new(Mutex::new(Meshetar::new()));
    match rocket::build()
        .manage(meshetar)
        .mount(
            "/",
            routes![
                history_fetch,
                meshetar_status /*, interval_put, pair_put*/
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
        let pair = m.pair.to_str();
        match book::fetch_history(pair).await {
            Ok(_) => log::info!("History fetching success."),
            Err(e) => log::info!("History fetching err: {:?}", e),
        };
        m.status = meshetar::Status::Idle;
    });
    Accepted(Some(meshetar.lock().await.summerize()))
}

#[get("/status")]
async fn meshetar_status(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<String> {
    let meshetar = meshetar.lock().await;
    Accepted(Some(meshetar.summerize()))
}

/*
#[put("/interval")]
fn interval_put(meshetar: &State<Mutex<Meshetar>>) -> status::Accepted<String> {
    let mut meshetar = meshetar.lock().unwrap();
    meshetar.interval = Interval::Minutes3;
    status::Accepted(Some(meshetar.summerize()))
}

#[put("/pair")]
fn pair_put(meshetar: &State<Mutex<Meshetar>>) -> status::Accepted<String> {
    let mut meshetar = meshetar.lock().unwrap();
    meshetar.interval = Interval::Minutes3;
    status::Accepted(Some(meshetar.summerize()))
}*/
