use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::warn;

use crate::trading::meshetar::Pair;

#[derive(PartialEq, Debug)]
pub enum Command {
    Run(Pair),
    CreateModel(Pair),
    Backtest(Pair),
    Terminate(Pair),
    TerminateAll,
    Plot(Pair),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Event {
    Asset(Asset),
    Balance(Balance),
}

// Messages to downstream consumers.
pub trait MessageTransmitter<Message> {
    /// Attempts to send a message to an external message subscriber.
    fn send(&mut self, message: Message);

    /// Attempts to send many messages to an external message subscriber.
    fn send_many(&mut self, messages: Vec<Message>);
}

// Sending to an external sink.
#[derive(Debug, Clone)]
pub struct EventTx {
    // Flag to communicate if the external receiver has been dropped.
    receiver_dropped: bool,
    // Channel transmitter to send events to an external sink.
    event_tx: mpsc::UnboundedSender<Event>,
}

impl MessageTransmitter<Event> for EventTx {
    fn send(&mut self, message: Event) {
        if self.receiver_dropped {
            return;
        }

        if self.event_tx.send(message).is_err() {
            warn!(
                action = "setting receiver_dropped = true",
                why = "event receiver dropped",
                "cannot send Events"
            );
            self.receiver_dropped = true;
        }
    }

    fn send_many(&mut self, messages: Vec<Event>) {
        if self.receiver_dropped {
            return;
        }

        messages.into_iter().for_each(|message| {
            let _ = self.event_tx.send(message);
        })
    }
}

impl EventTx {
    pub fn new(event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            receiver_dropped: false,
            event_tx,
        }
    }
}
