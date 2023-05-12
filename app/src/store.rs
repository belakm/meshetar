use std::collections::HashMap;

use crate::store_models::{Asset, Portfolio, ServerStatus};

#[derive(Clone)]
pub struct Store {
    pub portfolio: Portfolio,
    pub assets: HashMap<String, Asset>,
    pub server_status: ServerStatus,
    pub server_summary: String,
}

impl Store {
    pub fn new() -> Self {
        Self {
            portfolio: Portfolio {
                assets: HashMap::new(),
                btc_value: 0,
                usd_value: 0,
            },
            assets: HashMap::new(),
            server_status: ServerStatus::Unreachable,
            server_summary: String::from("No summary"),
        }
    }
    pub fn change_summary(mut self, summary: String) {
        self.server_summary = summary;
    }
}
