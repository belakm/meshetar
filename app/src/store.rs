use sycamore::reactive::RcSignal;

use crate::store_models::{BalanceSheetWithBalances, Chart, Status};

#[derive(Debug, Default, Clone)]
pub struct Store {
    pub message: RcSignal<String>,
    pub pair: RcSignal<String>,
    pub mode: RcSignal<String>,
    pub interval: RcSignal<String>,
    pub fetch_history_from: RcSignal<String>,
    pub server_state: RcSignal<Status>,
    pub last_kline_time: RcSignal<String>,
    pub balance_sheet: RcSignal<BalanceSheetWithBalances>,
    pub chart: RcSignal<Chart>,
}
