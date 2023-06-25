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
use env_logger::Builder;
use log::LevelFilter;
use meshetar::Interval;
use meshetar::Meshetar;
use meshetar::MeshetarStatus;
use meshetar::Pair;
use rocket::form::Form;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
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

    log::info!("Igniting rocket.");
    binance_client::initialize().await?;
    database::initialize().await?;
    let meshetar = Arc::new(Mutex::new(Meshetar::new()));
    let (sender, receiver) = watch::channel(false);
    let task_control = Arc::new(Mutex::new(TaskControl { sender, receiver }));

    match rocket::build()
        .attach(CORS)
        .manage(meshetar)
        .manage(task_control)
        .mount(
            "/",
            routes![
                fetch_history,
                clear_history,
                stop_all_operations,
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

#[post("/fetch_history")]
async fn fetch_history(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
) -> Accepted<Json<Meshetar>> {
    // Change status
    let meshetar_clone = Arc::clone(&meshetar.inner());
    meshetar_clone.lock().await.status = MeshetarStatus::FetchingHistory;
    let meshetar_clone2 = Arc::clone(&meshetar.inner());
    let meshetar_clone3 = Arc::clone(&meshetar.inner());
    let meshetar_clone4 = Arc::clone(&meshetar.inner());

    // Set status of task control to "working"
    &task_control.lock().await.sender.send(true);
    let reciever = Arc::clone(&task_control.inner());

    tokio::spawn(async move {
        let fetch_start = (chrono::Utc::now() - chrono::Duration::days(100)).timestamp();
        match book::fetch_history(reciever, meshetar_clone2, fetch_start).await {
            Ok(_) => log::info!("History fetching success."),
            Err(e) => log::info!("History fetching err: {:?}", e),
        };
        let mut meshetar_clone = meshetar_clone3.lock().await;
        meshetar_clone.status = MeshetarStatus::Idle;
    });
    let summary = meshetar_clone4.lock().await.summerize_json();
    Accepted(Some(summary))
}

#[post("/stop")]
async fn stop_all_operations(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
) -> Accepted<Json<Meshetar>> {
    if let Err(e) = task_control.lock().await.sender.send(false) {
        log::warn!("Failed to stop task. {}", e);
    }
    let mut meshetar = meshetar.lock().await;
    meshetar.status = MeshetarStatus::Stopping;
    Accepted(Some(meshetar.summerize_json()))
}

#[post("/clear_history")]
async fn clear_history(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
    tokio::join!(async {
        let m = meshetar.lock().await;
        let pair = m.pair.to_string();
        let interval = m.interval.to_kline_interval();
        match book::clear_history(pair, interval).await {
            Ok(_) => log::info!("History cleaning success."),
            Err(e) => log::info!("History cleaning err: {:?}", e),
        };
    });
    Accepted(Some(meshetar.lock().await.summerize_json()))
}

#[get("/status")]
async fn meshetar_status(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
    let meshetar_clone = Arc::clone(&meshetar.inner());
    let meshetar = meshetar_clone.lock().await;
    Accepted(Some(meshetar.summerize_json()))
}

#[get("/last_kline_time")]
async fn last_kline_time() -> Accepted<String> {
    match latest_kline_date().await {
        Ok(last_kline_time) => Accepted(Some(last_kline_time.to_string())),
        Err(_) => Accepted(Some(String::from("0"))),
    }
}

#[derive(FromForm, Deserialize)]
struct IntervalPutPayload<'r> {
    interval: &'r str,
}
#[put("/interval", data = "<data>")]
async fn interval_put(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    data: Form<IntervalPutPayload<'_>>,
) -> Result<Accepted<String>, Custom<String>> {
    let mut meshetar = meshetar.lock().await;
    if let Ok(value) = Interval::from_str(data.interval) {
        match meshetar.change_interval(value) {
            Ok(_) => Ok(Accepted(Some(value.to_string()))),
            Err(e) => Err(Custom(Status::ServiceUnavailable, e)),
        }
    } else {
        Err(Custom(
            Status::BadRequest,
            String::from("Couldnt parse interval."),
        ))
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
) -> Result<Accepted<String>, Custom<String>> {
    let mut meshetar = meshetar.lock().await;
    if let Ok(value) = Pair::from_str(data.pair) {
        match meshetar.change_pair(value) {
            Ok(_) => Ok(Accepted(Some(value.to_string()))),
            Err(e) => Err(Custom(Status::ServiceUnavailable, e)),
        }
    } else {
        Err(Custom(
            Status::BadRequest,
            String::from("Couldnt parse pair."),
        ))
    }
}
