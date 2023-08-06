use sycamore::reactive::RcSignal;

use crate::store_models::{BalanceSheetWithBalances, ChartPagination, Status};

#[derive(Debug, Default, Clone)]
pub struct Store {
    pub message: RcSignal<String>,
    pub pair: RcSignal<String>,
    pub mode: RcSignal<String>,
    pub interval: RcSignal<String>,
    pub server_state: RcSignal<Status>,
    pub last_kline_time: RcSignal<String>,
    pub balance_sheet: RcSignal<BalanceSheetWithBalances>,
    pub chart_pagination: RcSignal<ChartPagination>,
}

impl Store {
    /*fn start_operation(&self) {
        self.todos.modify().push(create_rc_signal(Todo {
            title,
            completed: false,
            id: Uuid::new_v4(),
        }))
    }*/
}
