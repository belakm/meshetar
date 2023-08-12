// Main modules
//
mod asset_ticker;
mod binance_client;
mod book;
mod database;
mod formatting;
mod load_config;
mod meshetar;
mod plot;
mod portfolio;
mod prediction_model;
mod rlang_runner;
mod serde_utils;

use env_logger::Builder;
use log::LevelFilter;
use meshetar::Interval;
use meshetar::Meshetar;
use meshetar::MeshetarStatus;
use meshetar::Pair;
use plot::ChartPlotData;
use portfolio::BalanceSheetWithBalances;
use rocket::catch;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Header;
use rocket::http::Status;
use rocket::response::status::Accepted;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{Request, Response};
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::sync::Mutex;

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
            match portfolio::fetch_balances().await {
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

#[post("/run")]
async fn run(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
) -> Accepted<Json<Meshetar>> {
    // Set state to running
    let meshetar_clone = Arc::clone(&meshetar.inner());
    meshetar_clone.lock().await.status = MeshetarStatus::Running;
    drop(meshetar_clone);
    // Set task control to running
    &task_control.lock().await.sender.send(true);
    let reciever = Arc::clone(&task_control.inner());

    // used as input to function
    let meshetar_clone2 = Arc::clone(&meshetar.inner());
    // used for setting the state back to Idle later
    let meshetar_clone3 = Arc::clone(&meshetar.inner());
    // used for extracting the early response status
    let meshetar_clone4 = Arc::clone(&meshetar.inner());
    // Start running
    tokio::spawn(async move {
        match book::run(reciever, meshetar_clone2).await {
            Ok(_) => log::warn!("Running ended successfully"),
            Err(e) => log::error!("Running failed with error {}", e),
        };
        let mut meshetar_clone = meshetar_clone3.lock().await;
        meshetar_clone.status = MeshetarStatus::Idle;
    });

    let summary = meshetar_clone4.lock().await.summerize_json();
    Accepted(Some(summary))
}

#[post("/create_new_model")]
async fn create_new_model(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
) -> Accepted<Json<Meshetar>> {
    // Set state to running
    let meshetar_clone = Arc::clone(&meshetar.inner());
    meshetar_clone.lock().await.status = MeshetarStatus::CreatingNewModel;
    drop(meshetar_clone);
    // Set task control to running
    &task_control.lock().await.sender.send(true);
    let reciever = Arc::clone(&task_control.inner());

    let meshetar_clone2 = Arc::clone(&meshetar.inner());
    let meshetar_clone3 = Arc::clone(&meshetar.inner());
    // Start running
    tokio::spawn(async move {
        match prediction_model::create_model(reciever).await {
            Ok(_) => log::warn!("Created model successfully"),
            Err(e) => log::error!("Creating model failed with error {}", e),
        };
        let mut meshetar_clone = meshetar_clone2.lock().await;
        meshetar_clone.status = MeshetarStatus::Idle;
    });

    let summary = meshetar_clone3.lock().await.summerize_json();
    Accepted(Some(summary))
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
        let fetch_start = (chrono::Utc::now() - chrono::Duration::days(2)).timestamp();
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

#[get("/balance_sheet")]
async fn balance_sheet() -> Result<Accepted<Json<BalanceSheetWithBalances>>, Custom<String>> {
    match portfolio::get_balance_sheet().await {
        Ok(balance_sheet) => Ok(Accepted(Some(Json(balance_sheet.clone())))),
        Err(e) => Err(Custom(Status::NotFound, format!("{:?}", e))),
    }
}

#[get("/status")]
async fn meshetar_status(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
    let meshetar_clone = Arc::clone(&meshetar.inner());
    let meshetar = meshetar_clone.lock().await;
    Accepted(Some(meshetar.summerize_json()))
}

#[derive(FromForm, Deserialize)]
struct PlotChartPayload<'r> {
    page: &'r str,
}
#[derive(Serialize)]
pub struct ChartPlotWithPagination {
    path: String,
    model_path: String,
    page: i64,
    total_pages: i64,
}
#[post("/plot_chart", data = "<data>")]
async fn plot_chart(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    data: Form<PlotChartPayload<'_>>,
) -> Result<Json<ChartPlotWithPagination>, ()> {
    let page = match data.page.parse::<i64>() {
        Ok(page) => page,
        Err(_) => 0,
    };
    let meshetar = meshetar.lock().await;
    let pair = meshetar.pair.to_string();
    let interval = meshetar.interval.to_kline_interval().to_string();
    drop(meshetar);
    match plot::generate_plot_data(pair, interval, page).await {
        Ok(chart_plot_data) => {
            match plot::plot_chart(chart_plot_data.klines, chart_plot_data.signals).await {
                Ok(path) => Ok(Json(ChartPlotWithPagination {
                    path,
                    model_path: "historical_trading_signals_model.svg".to_string(),
                    page: chart_plot_data.page,
                    total_pages: chart_plot_data.total_pages,
                })),
                Err(e) => Err(log::warn!("Error plotting chart. {e}")),
            }
        }
        Err(e) => Err(log::warn!("Error plotting chart. {e}")),
    }
}

#[get("/last_kline_time")]
async fn last_kline_time() -> Accepted<String> {
    match book::latest_kline_date().await {
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
