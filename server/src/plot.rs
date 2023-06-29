use chrono::{DateTime, Duration, Utc};
use plotters::prelude::*;

use crate::formatting::dt_to_readable_short;

static PLOT_PATH: &str = "static/plot.svg";

pub async fn plot_chart(
    data: Vec<(DateTime<Utc>, (f32, f32, f32, f32))>,
) -> Result<String, String> {
    let font = ("sans-serif", 20).into_font();
    let text_style = TextStyle::from(font).color(&WHITE);
    let axis_style = ShapeStyle {
        color: WHITE.mix(1f64),
        filled: true,
        stroke_width: 2,
    };
    let thin_guide_style = ShapeStyle {
        color: WHITE.mix(0.1),
        filled: true,
        stroke_width: 1,
    };
    let guide_style = ShapeStyle {
        color: WHITE.mix(0.2),
        filled: true,
        stroke_width: 1,
    };
    let gain_style = ShapeStyle {
        color: GREEN.mix(1f64),
        filled: true,
        stroke_width: 1,
    };
    let lose_style = ShapeStyle {
        color: RED.mix(1f64),
        filled: false,
        stroke_width: 2,
    };

    let mut global_min = f32::MAX;
    let mut global_max = f32::MIN;

    for (_, ohlc) in data.iter() {
        global_min = global_min.min(ohlc.0).min(ohlc.1).min(ohlc.2).min(ohlc.3);
        global_max = global_max.max(ohlc.0).max(ohlc.1).max(ohlc.2).max(ohlc.3);
    }

    global_min = global_min;
    global_max = global_max;

    let root_area = SVGBackend::new(PLOT_PATH, (1024, 640)).into_drawing_area();
    root_area.fill(&BLACK).unwrap();
    let root_area = root_area.margin(10, 10, 10, 10);

    let (from_date, to_date) = (
        *&data[0].0 - Duration::minutes(1),
        *&data[*&data.len() - 1].0 + Duration::minutes(1),
    );

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Signals", ("sans-serif", 16).into_font())
        .margin(12)
        .x_label_area_size(50)
        .y_label_area_size(120)
        .build_cartesian_2d(from_date..to_date, global_min..global_max)
        .unwrap();

    chart
        .configure_mesh()
        .label_style(text_style.clone())
        .light_line_style(thin_guide_style)
        .y_max_light_lines(5)
        .x_max_light_lines(5)
        .bold_line_style(guide_style)
        .axis_style(axis_style)
        .x_desc("Time")
        .y_desc("Value")
        .x_labels(10)
        .x_label_formatter(&|x| format!("{}", dt_to_readable_short(*x)))
        .y_labels(10)
        .y_label_formatter(&|y| format!("{:.4}", y))
        .draw()
        .unwrap();

    chart
        .draw_series(data.iter().map(|(x, (o, h, l, c))| {
            log::info!("OHLC: {:?}", (o, h, l, c));
            CandleStick::new(*x, *o, *h, *l, *c, gain_style, lose_style, 8)
        }))
        .unwrap();

    root_area.present().unwrap();
    Ok(PLOT_PATH.to_string())
}
