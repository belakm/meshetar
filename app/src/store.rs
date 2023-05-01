use std::collections::HashMap;

use crate::store_models::{Asset, Portfolio, ServerStatus};

pub struct Store {
    pub portfolio: Portfolio,
    pub assets: HashMap<String, Asset>,
    pub server_status: ServerStatus,
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
        }
    }
}
