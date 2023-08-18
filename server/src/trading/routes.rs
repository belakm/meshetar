use super::{
    meshetar::{Interval, Meshetar, MeshetarStatus, Pair},
    portfolio::{self, BalanceSheetWithBalances},
};
use crate::{assets::book, TaskControl};
use rocket::{
    form::Form,
    http::Status,
    response::status::{Accepted, Custom},
    serde::json::Json,
    State,
};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[get("/balance_sheet")]
pub async fn balance_sheet() -> Result<Accepted<Json<BalanceSheetWithBalances>>, Custom<String>> {
    match portfolio::get_balance_sheet().await {
        Ok(balance_sheet) => Ok(Accepted(Some(Json(balance_sheet.clone())))),
        Err(e) => Err(Custom(Status::NotFound, format!("{:?}", e))),
    }
}

#[post("/run")]
pub async fn run(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
) -> Accepted<Json<Meshetar>> {
    // Set state to run
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

#[post("/stop")]
pub async fn stop_all_operations(
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

#[get("/status")]
pub async fn meshetar_status(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
    let meshetar_clone = Arc::clone(&meshetar.inner());
    let meshetar = meshetar_clone.lock().await;
    Accepted(Some(meshetar.summerize_json()))
}

#[derive(FromForm, Deserialize)]
pub struct IntervalPutPayload<'r> {
    interval: &'r str,
}
#[put("/interval", data = "<data>")]
pub async fn interval_put(
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
pub struct PairPutPayload<'r> {
    pair: &'r str,
}
#[put("/pair", data = "<data>")]
pub async fn pair_put(
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
