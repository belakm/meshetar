pub mod error;

use crate::{
    assets::{fetch_candles, Asset},
    database::Database,
    portfolio::Portfolio,
    trading::Trader,
    utils::binance_client::{self, BinanceClient},
};
use chrono::Duration;
use error::CoreError;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(PartialEq, Debug)]
pub enum Command {
    CreateModel(Asset),
    ExitPosition(Asset),
    ExitAllPositions,
    Terminate(String),
}

pub struct Core {
    id: Uuid,
    database: Arc<Mutex<Database>>,
    portfolio: Arc<Mutex<Portfolio>>,
    binance_client: Arc<BinanceClient>,
    command_reciever: Receiver<Command>,
    command_transmitters: HashMap<Asset, mpsc::Sender<Command>>,
    traders: Vec<Trader>,
}

impl Core {
    pub fn builder() -> CoreBuilder {
        CoreBuilder::new()
    }
}

impl Core {
    pub async fn run(&mut self) {
        info!("Core {} is starting up.", &self.id);
        let mut fetching_stopped = self.fetch_history().await;
        loop {
            tokio::select! {
                _ = fetching_stopped.recv() => {
                    break;
                }
            }
        }
        let mut trading_stopped = self.run_traders().await;
        loop {
            tokio::select! {
                _ = trading_stopped.recv() => {
                    break;
                },
                command = self.command_reciever.recv() => {
                    if let Some(command) = command {
                        match command {
                            Command::CreateModel(_pair) => {

                            },
                            Command::ExitPosition(asset) => {
                                self.exit_position(asset).await;
                            }
                            Command::ExitAllPositions => {
                                self.exit_all_positions().await;
                            }
                            Command::Terminate(message) => {
                                self.terminate_traders(message).await;
                                break;
                            },
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
    async fn fetch_history(&mut self) -> mpsc::Receiver<bool> {
        let assets: Vec<Asset> = self
            .traders
            .iter()
            .map(|trader| trader.asset.clone())
            .collect();
        let binance_client = self.binance_client.clone();
        let handles = assets.into_iter().map(move |asset| {
            (
                asset.clone(),
                fetch_candles(Duration::days(1), asset.clone(), binance_client.clone()),
            )
        });
        let (notify_transmitter, notify_receiver) = mpsc::channel(1);
        let mut database = self.database.clone();
        tokio::spawn(async move {
            for handle in handles {
                match handle.1.await {
                    Ok(candles) => {
                        database.lock().await.add_candles(handle.0, candles).await;
                    }
                    Err(err) => {
                        error!(
                            error = &*format!("{:?}", err),
                            "Trader thread has panicked during execution",
                        )
                    }
                }
            }
            let _ = notify_transmitter.send(true).await;
        });
        notify_receiver
    }
    async fn run_traders(&mut self) -> mpsc::Receiver<bool> {
        let traders = std::mem::take(&mut self.traders);
        let mut thread_handles = Vec::with_capacity(traders.len());
        for mut trader in traders.into_iter() {
            let handle = tokio::spawn(async move { trader.run().await });
            thread_handles.push(handle);
        }
        let (notify_transmitter, notify_receiver) = mpsc::channel(1);
        tokio::spawn(async move {
            for handle in thread_handles {
                if let Err(err) = handle.await {
                    error!(
                        error = &*format!("{:?}", err),
                        "Trader thread has panicked during execution",
                    )
                }
            }
            let _ = notify_transmitter.send(true).await;
        });
        notify_receiver
    }
    async fn terminate_traders(&self, message: String) {
        self.exit_all_positions().await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        for (market, command_transmitter) in self.command_transmitters.iter() {
            if command_transmitter
                .send(Command::Terminate(message.clone()))
                .await
                .is_err()
            {
                error!(why = "dropped receiver", asset = &*format!("{:?}", market),);
            }
        }
    }
    async fn exit_all_positions(&self) {
        for (asset, command_transmitter) in self.command_transmitters.iter() {
            if command_transmitter
                .send(Command::ExitPosition(asset.clone()))
                .await
                .is_err()
            {
                error!(
                    asset = &*format!("{:?}", asset),
                    why = "dropped receiver",
                    "failed to send Command::Terminate to Trader command_rx"
                );
            }
        }
    }
    async fn exit_position(&self, asset: Asset) {
        if let Some((market_ref, command_tx)) = self.command_transmitters.get_key_value(&asset) {
            if command_tx.send(Command::ExitPosition(asset)).await.is_err() {
                error!(
                    market = &*format!("{:?}", market_ref),
                    why = "dropped receiver",
                    "failed to send Command::Terminate to Trader command_rx"
                );
            }
        } else {
            warn!(
                market = &*format!("{:?}", asset),
                why = "Engine has no trader_command_tx associated with provided Market",
                "failed to exit Position"
            );
        }
    }
}

pub struct CoreBuilder {
    id: Option<Uuid>,
    portfolio: Option<Arc<Mutex<Portfolio>>>,
    database: Option<Arc<Mutex<Database>>>,
    binance_client: Option<BinanceClient>,
    command_reciever: Option<Receiver<Command>>,
    command_transmitters: Option<HashMap<Asset, mpsc::Sender<Command>>>,
    traders: Option<Vec<Trader>>,
}

impl CoreBuilder {
    pub fn new() -> Self {
        CoreBuilder {
            id: None,
            database: None,
            portfolio: None,
            binance_client: None,
            command_reciever: None,
            command_transmitters: None,
            traders: None,
        }
    }
    pub fn id(self, id: Uuid) -> Self {
        CoreBuilder {
            id: Some(id),
            ..self
        }
    }
    pub fn portfolio(self, portfolio: Arc<Mutex<Portfolio>>) -> Self {
        CoreBuilder {
            portfolio: Some(portfolio),
            ..self
        }
    }
    pub fn binance_client(self, binance_client: BinanceClient) -> Self {
        CoreBuilder {
            binance_client: Some(binance_client),
            ..self
        }
    }
    pub fn command_reciever(self, command_reciever: Receiver<Command>) -> Self {
        CoreBuilder {
            command_reciever: Some(command_reciever),
            ..self
        }
    }
    pub fn command_transmitters(self, value: HashMap<Asset, mpsc::Sender<Command>>) -> Self {
        CoreBuilder {
            command_transmitters: Some(value),
            ..self
        }
    }
    pub fn database(self, value: Arc<Mutex<Database>>) -> Self {
        CoreBuilder {
            database: Some(value),
            ..self
        }
    }
    pub fn traders(self, value: Vec<Trader>) -> Self {
        CoreBuilder {
            traders: Some(value),
            ..self
        }
    }
    pub fn build(self) -> Result<Core, CoreError> {
        let binance_client = self
            .binance_client
            .ok_or(CoreError::BuilderIncomplete("binance client"))?;
        let binance_client = Arc::new(binance_client);
        let core = Core {
            id: self.id.ok_or(CoreError::BuilderIncomplete("core_id"))?,
            database: self
                .database
                .ok_or(CoreError::BuilderIncomplete("database"))?,
            portfolio: self
                .portfolio
                .ok_or(CoreError::BuilderIncomplete("portfolio"))?,
            binance_client,
            command_reciever: self
                .command_reciever
                .ok_or(CoreError::BuilderIncomplete("command reciever"))?,
            command_transmitters: self
                .command_transmitters
                .ok_or(CoreError::BuilderIncomplete("trader command transmitters"))?,
            traders: self
                .traders
                .ok_or(CoreError::BuilderIncomplete("traders"))?,
        };
        Ok(core)
    }
}
