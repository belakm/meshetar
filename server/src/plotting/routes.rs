use super::plot;
use crate::trading::meshetar::Meshetar;
use rocket::{form::Form, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(FromForm, Deserialize)]
pub struct PlotChartPayload<'r> {
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
pub async fn plot_chart(
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
