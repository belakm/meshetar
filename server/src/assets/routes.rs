use super::book;
use crate::{
    trading::meshetar::{Meshetar, MeshetarStatus},
    TaskControl,
};
use rocket::{form::Form, response::status::Accepted, serde::json::Json, State};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(FromForm, Deserialize)]
pub struct FetchHistoryPayload {
    from: i64,
}

#[post("/fetch_history", data = "<data>")]
pub async fn fetch_history(
    meshetar: &State<Arc<Mutex<Meshetar>>>,
    task_control: &State<Arc<Mutex<TaskControl>>>,
    data: Form<FetchHistoryPayload>,
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

    log::warn!("{}", data.from.clone());

    tokio::spawn(async move {
        match book::fetch_history(reciever, meshetar_clone2, data.from).await {
            Ok(_) => log::info!("History fetching success."),
            Err(e) => log::info!("History fetching err: {:?}", e),
        };
        let mut meshetar_clone = meshetar_clone3.lock().await;
        meshetar_clone.status = MeshetarStatus::Idle;
    });
    let summary = meshetar_clone4.lock().await.summerize_json();
    Accepted(Some(summary))
}

#[post("/clear_history")]
pub async fn clear_history(meshetar: &State<Arc<Mutex<Meshetar>>>) -> Accepted<Json<Meshetar>> {
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

#[get("/last_kline_time")]
pub async fn last_kline_time() -> Accepted<String> {
    match book::latest_kline_date().await {
        Ok(last_kline_time) => Accepted(Some(last_kline_time.to_string())),
        Err(_) => Accepted(Some(String::from("0"))),
    }
}
