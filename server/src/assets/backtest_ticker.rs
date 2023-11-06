use super::{error::AssetError, Asset, MarketEvent, MarketEventDetail};
use crate::{database::Database, strategy::Strategy};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver},
    Mutex,
};

pub async fn new_ticker(
    asset: Asset,
    database: Arc<Mutex<Database>>,
    last_n_candles: usize,
) -> Result<UnboundedReceiver<MarketEvent>, AssetError> {
    let (tx, rx) = mpsc::unbounded_channel();
    let candles = database
        .lock()
        .await
        .fetch_all_candles(asset.clone())
        .await?;
    let skip_n_candles = candles.len() - last_n_candles;
    tokio::spawn(async move {
        let candles_copy = candles.clone();
        let open_time = candles_copy.first().unwrap().open_time;
        match Strategy::generate_backtest_signals(open_time, candles_copy, asset.clone()).await {
            Ok(Some(signals)) => {
                let mut stream_candles = candles.iter().skip(skip_n_candles).enumerate();
                while let Some((index, candle)) = stream_candles.next() {
                    let _ = tx.send(MarketEvent {
                        time: candle.close_time,
                        asset: asset.clone(),
                        detail: MarketEventDetail::BacktestCandle((
                            candle.to_owned(),
                            signals.get(index).unwrap().to_owned(),
                        )),
                    });
                }
            }
            Ok(None) => (),
            Err(e) => error!("Err on backtest: {:?}", e),
        };
    });
    Ok(rx)
}
