use std::vec;

use crate::database::DATABASE_CONNECTION;
use crate::portfolio::get_account_history_with_snapshots;
use crate::server_error::{ServerError, MAP_TO_404, MAP_TO_500};
use plotters::prelude::*;
use rocket::fs::NamedFile;
use rocket::get;

static PATH: &str = "static/account_balance.svg";

pub async fn plot_account_balance() -> Result<(), ServerError> {
    let account_balance_history =
        get_account_history_with_snapshots(&DATABASE_CONNECTION.clone().lock().unwrap());

    match account_balance_history {
        Ok(balance_history) => {
            let balance_history = balance_history
                .snapshot_vos
                .iter()
                .enumerate()
                .map(|(index, snap)| (index as f32, snap.data.total_asset_of_btc.parse().unwrap()));

            let (min, max) = balance_history
                .clone()
                .into_iter()
                .fold((20f32, 0f32), |acc, (_, snap)| {
                    (acc.0.min(snap), acc.1.max(snap))
                });
            let (min, max) = (min * 0.9, max * 1.1);

            let drawing_area = SVGBackend::new(&PATH, (1024, 768)).into_drawing_area();

            // Chart metadata
            let mut chart = ChartBuilder::on(&drawing_area)
                .caption("Balance over time", ("monospace", 50).into_font())
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(0f32..balance_history.len() as f32, min..max)
                .unwrap();

            chart
                .configure_mesh()
                .bold_line_style(&WHITE.mix(0.2))
                .light_line_style(&WHITE.mix(0.01))
                .axis_style(&WHITE)
                .x_label_style(&WHITE)
                .y_label_style(&WHITE)
                .draw()
                .unwrap();

            chart
                .draw_series(LineSeries::new(balance_history, &GREEN))
                .map_err(|e| MAP_TO_500(&e.to_string()))?
                .label("Balance over time")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&WHITE)
                .draw()
                .map_err(|e| MAP_TO_500(&e.to_string()))
        }
        Err(error) => Err(MAP_TO_500(&error.to_string())),
    }
}
