use super::{error::AssetError, Asset, MarketEvent, MarketEventDetail};
use crate::database::Database;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver},
    Mutex,
};

pub async fn new_ticker(
    asset: Asset,
    database: Arc<Mutex<Database>>,
) -> Result<UnboundedReceiver<MarketEvent>, AssetError> {
    let (tx, rx) = mpsc::unbounded_channel();
    let candles = database
        .lock()
        .await
        .fetch_all_candles(asset.clone())
        .await?;
    tokio::spawn(async move {
        let mut candles = candles.iter().skip(candles.len() - 1490);
        while let Some(candle) = candles.next() {
            let _ = tx.send(MarketEvent {
                time: candle.close_time,
                asset: asset.clone(),
                detail: MarketEventDetail::Candle(candle.to_owned()),
            });
        }
    });
    Ok(rx)
}
