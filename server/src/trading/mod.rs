pub mod error;

use serde::Serialize;
use std::{collections::VecDeque, sync::Arc};
use strum::{Display, EnumString};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::{
    core::Command,
    events::{Event, EventTx},
    portfolio::Portfolio,
};

use self::error::TraderError;

#[derive(Copy, Clone, Debug, Serialize, Display, EnumString, PartialEq)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}

pub mod meshetar;
pub mod routes;

pub struct Trader {
    engine_id: Uuid,
    pair: Pair,
    command_reciever: mpsc::Receiver<Command>,
    event_transmitter: EventTx,
    event_queue: VecDeque<Event>,
    portfolio: Arc<Mutex<Portfolio>>,
}

impl Trader {
    pub fn builder() -> TraderBuilder {
        TraderBuilder::new()
    }
    pub async fn run(&mut self) -> Result<(), TraderError> {
        loop {
            // Check for new remote Commands before continuing to generate another MarketEvent
            while let Some(command) = self.receive_remote_command() {
                match command {
                    Command::Terminate(_) => break,
                    Command::ExitPosition(market) => {
                        self.event_q
                            .push_back(Event::SignalForceExit(SignalForceExit::from(market)));
                    }
                    _ => continue,
                }
            }

            // If the Feed<MarketEvent> yields, populate event_q with the next MarketEvent
            match self.data.next() {
                Feed::Next(market) => {
                    self.event_tx.send(Event::Market(market.clone()));
                    self.event_q.push_back(Event::Market(market));
                }
                Feed::Unhealthy => {
                    warn!(
                        engine_id = %self.engine_id,
                        market = ?self.market,
                        action = "continuing while waiting for healthy Feed",
                        "MarketFeed unhealthy"
                    );
                    continue 'trading;
                }
                Feed::Finished => break 'trading,
            }

            // Handle Events in the event_q
            // '--> While loop will break when event_q is empty and requires another MarketEvent
            while let Some(event) = self.event_q.pop_front() {
                match event {
                    Event::Market(market) => {
                        if let Some(signal) = self.strategy.generate_signal(&market) {
                            self.event_tx.send(Event::Signal(signal.clone()));
                            self.event_q.push_back(Event::Signal(signal));
                        }

                        if let Some(position_update) = self
                            .portfolio
                            .lock()
                            .update_from_market(&market)
                            .expect("failed to update Portfolio from market")
                        {
                            self.event_tx.send(Event::PositionUpdate(position_update));
                        }
                    }

                    Event::Signal(signal) => {
                        if let Some(order) = self
                            .portfolio
                            .lock()
                            .generate_order(&signal)
                            .expect("failed to generate order")
                        {
                            self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_q.push_back(Event::OrderNew(order));
                        }
                    }

                    Event::SignalForceExit(signal_force_exit) => {
                        if let Some(order) = self
                            .portfolio
                            .lock()
                            .generate_exit_order(signal_force_exit)
                            .expect("failed to generate forced exit order")
                        {
                            self.event_tx.send(Event::OrderNew(order.clone()));
                            self.event_q.push_back(Event::OrderNew(order));
                        }
                    }

                    Event::OrderNew(order) => {
                        let fill = self
                            .execution
                            .generate_fill(&order)
                            .expect("failed to generate Fill");

                        self.event_tx.send(Event::Fill(fill.clone()));
                        self.event_q.push_back(Event::Fill(fill));
                    }

                    Event::Fill(fill) => {
                        let fill_side_effect_events = self
                            .portfolio
                            .lock()
                            .update_from_fill(&fill)
                            .expect("failed to update Portfolio from fill");

                        self.event_tx.send_many(fill_side_effect_events);
                    }
                    _ => {}
                }
            }

            debug!(
                engine_id = &*self.engine_id.to_string(),
                market = &*format!("{:?}", self.market),
                "Trader trading loop stopped"
            );
        }
    }
}

pub struct TraderBuilder {
    engine_id: Option<Uuid>,
    pair: Option<Pair>,
    command_reciever: Option<mpsc::Receiver<Command>>,
    event_transmitter: Option<EventTx>,
    event_queue: Option<VecDeque<Event>>,
    portfolio: Option<Arc<Mutex<Portfolio>>>,
}
impl TraderBuilder {
    pub fn new() -> TraderBuilder {
        TraderBuilder {
            engine_id: None,
            command_reciever: None,
            pair: None,
            event_transmitter: None,
            portfolio: None,
            event_queue: None,
        }
    }
    pub fn build() {}
}
