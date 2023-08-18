use crate::{
    model::prediction_model::TradeSignal,
    utils::{
        database::DB_POOL,
        formatting::{dt_to_readable_short, timestamp_to_dt},
    },
};
use chrono::{DateTime, Duration, Utc};
use futures::TryFutureExt;
use plotters::{prelude::*, style::full_palette::PINK};
use std::str::FromStr;

static PLOT_PATH: &str = "static/plot.svg";
static PLOT_PATH_CONSUMER: &str = "plot.svg";

pub async fn plot_chart(
    data: Vec<(DateTime<Utc>, (f32, f32, f32, f32))>,
    signals: Vec<(DateTime<Utc>, TradeSignal)>,
) -> Result<String, String> {
    let font = ("sans-serif", 20).into_font();
    let text_style = TextStyle::from(font).color(&WHITE);
    let axis_style = ShapeStyle {
        color: WHITE.mix(1f64),
        filled: true,
        stroke_width: 2,
    };
    let thin_guide_style = ShapeStyle {
        color: WHITE.mix(0.05),
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
        stroke_width: 1,
    };

    let hold_indicator = ShapeStyle {
        color: WHITE.mix(0.5),
        filled: false,
        stroke_width: 1,
    };
    let buy_indicator = ShapeStyle {
        color: GREEN.mix(0.8f64),
        filled: true,
        stroke_width: 1,
    };
    let sell_indicator = ShapeStyle {
        color: RED.mix(0.8f64),
        filled: false,
        stroke_width: 1,
    };
    let last_value = ShapeStyle {
        color: PINK.mix(0.5f64),
        filled: true,
        stroke_width: 1,
    };

    let mut global_min = f32::MAX;
    let mut global_max = f32::MIN;

    for (_, ohlc) in data.iter() {
        global_min = global_min.min(ohlc.0).min(ohlc.1).min(ohlc.2).min(ohlc.3);
        global_max = global_max.max(ohlc.0).max(ohlc.1).max(ohlc.2).max(ohlc.3);
    }

    global_min = global_min * 0.95;
    global_max = global_max * 1.05;

    let root_area = SVGBackend::new(PLOT_PATH, (1024, 480)).into_drawing_area();
    root_area.fill(&RGBColor(20, 30, 38)).unwrap();
    let root_area = root_area.margin(10, 10, 10, 10);

    let (from_date, to_date) = (
        *&data[0].0 - Duration::minutes(1),
        *&data[*&data.len() - 1].0 + Duration::minutes(1),
    );

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Signals", text_style.clone())
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
        .y_desc("Price")
        .x_labels(10)
        .x_label_formatter(&|x| format!("{}", dt_to_readable_short(*x)))
        .y_labels(10)
        .y_label_formatter(&|y| format!("{:.4}", y))
        .draw()
        .unwrap();

    // Candlesticks
    chart
        .draw_series(data.iter().map(|(x, (o, h, l, c))| {
            CandleStick::new(*x, *o, *h, *l, *c, gain_style, lose_style, 3)
        }))
        .unwrap();

    // Trade signals
    for (x, signal) in signals.iter() {
        let style = match signal {
            TradeSignal::Buy => buy_indicator,
            TradeSignal::Sell => sell_indicator,
            TradeSignal::Hold => hold_indicator,
        };
        chart
            .draw_series(std::iter::once(Circle::new(
                (*x, global_max),
                4,
                style.clone(),
            )))
            .unwrap();
    }

    // Last price
    if let Some(last_kline) = data.last() {
        let (x, (_o, _h, _l, c)) = *last_kline;
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [
                    (x + Duration::seconds(30), c),
                    (x + Duration::minutes(10), c + 100f32),
                ],
                last_value,
            )))
            .unwrap();
    }

    root_area.present().unwrap();
    Ok(PLOT_PATH_CONSUMER.to_string())
}

#[derive(sqlx::FromRow)]
struct SimpleKline {
    open_time: i64,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
}

#[derive(sqlx::FromRow, Debug)]
struct SimpleSignal {
    time: i64,
    signal: String,
}

pub struct ChartPlotData {
    pub page: i64,
    pub total_pages: i64,
    pub klines: Vec<(DateTime<Utc>, (f32, f32, f32, f32))>,
    pub signals: Vec<(DateTime<Utc>, TradeSignal)>,
}
pub async fn generate_plot_data(
    pair: String,
    interval: String,
    page: i64,
) -> Result<ChartPlotData, String> {
    let klines: Vec<SimpleKline>;
    let signals: Vec<SimpleSignal>;
    let page_to_go: i64;
    let total_pages: i64;
    let points_per_page: i64 = 180;
    {
        let connection = DB_POOL.get().unwrap();
        let pages_row: (i64,) =
            sqlx::query_as("SELECT CAST((COUNT(*) + ?1 - 1) / ?1 AS INTEGER) AS pages FROM klines WHERE symbol = ?2")
                .bind(&points_per_page)
                .bind(&pair)
                .fetch_one(connection)
                .map_err(|e| format!("Error getting total number of pages for klines, {:?}", e))
                .await?;
        total_pages = pages_row.0;
        page_to_go = page.clamp(1, total_pages);
        klines = sqlx::query_as::<_, SimpleKline>(
            "SELECT open_time, open, high, low, close 
            FROM klines 
            WHERE interval = ?1 AND symbol = ?2 
            ORDER BY open_time DESC 
            LIMIT ?3
            OFFSET ?4",
        )
        .bind(interval.clone())
        .bind(pair.clone())
        .bind(points_per_page)
        .bind(points_per_page * (page_to_go - 1))
        .fetch_all(connection)
        .await
        .map_err(|e| format!("Error fetching last kline. {:?}", e))?;

        let max_time: i64 = klines.first().map(|i| i.open_time).unwrap_or(0);
        let min_time: i64 = klines.last().map(|i| i.open_time).unwrap_or(0);

        signals = sqlx::query_as::<_, SimpleSignal>(
            "SELECT signal, time 
            FROM signals 
            WHERE interval = ?1 AND symbol = ?2 AND time >= ?3 AND time <= ?4
            ORDER BY time DESC",
        )
        .bind(interval)
        .bind(pair)
        .bind(min_time)
        .bind(max_time)
        .fetch_all(connection)
        .await
        .map_err(|e| format!("Error fetching last kline. {:?}", e))?;
    }

    let mut rows: Vec<(DateTime<Utc>, (f32, f32, f32, f32))> = klines
        .into_iter()
        .map(|kline| {
            (
                timestamp_to_dt(kline.open_time / 1000),
                (kline.open, kline.high, kline.low, kline.close),
            )
        })
        .collect();
    rows.reverse();

    let mut signal_rows: Vec<(DateTime<Utc>, TradeSignal)> = signals
        .into_iter()
        .map(|signal| {
            (
                timestamp_to_dt(signal.time / 1000),
                TradeSignal::from_str(&signal.signal).unwrap(),
            )
        })
        .collect();
    signal_rows.reverse();

    Ok(ChartPlotData {
        klines: rows,
        signals: signal_rows,
        page: page_to_go,
        total_pages,
    })
}
