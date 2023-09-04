use serde::Deserialize;
use yata::prelude::OHLCV;

use super::book::Kline;

#[derive(Deserialize)]
pub struct Indicators {
    pub symbol: String,
    pub interval: String,
    pub open_time: i64,
    pub adi: f64,
    pub cci: f64,
    pub dema: f64,
    pub dma: f64,
    pub ema: f64,
    pub hma: f64,
    pub rma: f64,
    pub sma: f64,
    pub smm: f64,
    pub swma: f64,
    pub tema: f64,
    pub tma: f64,
    pub tr: f64,
    pub trima: f64,
    pub tsi: f64,
    pub vwma: f64,
    pub vidya: f64,
    pub wma: f64,
    pub wsma: f64,
}

fn klines_to_tuples(klines: Vec<Kline>) -> Vec<(f64, f64, f64, f64, f64)> {
    /*klines
        .into_iter()
        .map(|kline| (kline.open, kline.high, kline.low, kline.close, kline.volume))
        .collect()
    */
    todo!()
}

pub fn add_ta(klines: Vec<Kline>) -> Result<Indicators, String> {
    let mut indicators: Vec<Indicators> = Vec::new();
    if let Some(last_value) = klines.last() {
        //let last_value = last_value.close;
        //let mut adi = ADI::new(20, last_value);
        todo!("TA todo")
    } else {
        Err(String::from("Add_ta missing any value"))
    }
}
