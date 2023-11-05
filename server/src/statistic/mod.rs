pub mod dispersion;
pub mod error;
pub mod metric;
pub mod summary_drawdown;
pub mod summary_pnl;
pub mod welford_online;

use self::{
    metric::ratio::{CalmarRatio, SharpeRatio, SortinoRatio},
    summary_drawdown::DrawdownSummary,
    summary_pnl::{PnLReturnSummary, ProfitLossSummary},
};
use crate::portfolio::position::Position;
use chrono::{DateTime, Duration, Utc};
use prettytable::{row, Cell, Row, Table};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct TradingSummary {
    pub pnl_returns: PnLReturnSummary,
    pub pnl: ProfitLossSummary,
    pub drawdown: DrawdownSummary,
    pub tear_sheet: TearSheet,
}

impl TradingSummary {
    pub fn init(config: StatisticConfig) -> Self {
        Self {
            pnl_returns: PnLReturnSummary::new(),
            pnl: ProfitLossSummary::new(),
            drawdown: DrawdownSummary::new(config.starting_equity),
            tear_sheet: TearSheet::new(config.risk_free_return),
        }
    }
    pub fn update(&mut self, position: &Position) {
        self.pnl_returns.update(position);
        self.drawdown.update(position);
        self.tear_sheet.update(&self.pnl_returns, &self.drawdown);
        self.pnl.update(position);
    }
    pub fn generate_summary(&mut self, positions: &[Position]) {
        for position in positions.iter() {
            self.update(position)
        }
    }
}

impl TableBuilder for TradingSummary {
    fn titles(&self) -> Row {
        let mut titles = Vec::<Cell>::new();

        for title in &self.pnl_returns.titles() {
            titles.push(title.clone())
        }

        for title in &self.tear_sheet.titles() {
            titles.push(title.clone())
        }

        for title in &self.drawdown.titles() {
            titles.push(title.clone())
        }

        for title in &self.pnl.titles() {
            titles.push(title.clone())
        }

        Row::new(titles)
    }

    fn row(&self) -> Row {
        let mut cells = Vec::<Cell>::new();

        for cell in &self.pnl_returns.row() {
            cells.push(cell.clone())
        }

        for cell in &self.tear_sheet.row() {
            cells.push(cell.clone())
        }

        for cell in &self.drawdown.row() {
            cells.push(cell.clone())
        }

        for cell in &self.pnl.row() {
            cells.push(cell.clone())
        }

        Row::new(cells)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct StatisticConfig {
    pub starting_equity: f64,
    pub trading_days_per_year: usize,
    pub risk_free_return: f64,
}

pub fn calculate_trading_duration(start_time: &DateTime<Utc>, position: &Position) -> Duration {
    match position.meta.exit_balance {
        None => {
            // Since Position is not exited, estimate duration w/ last_update_time
            position.meta.update_time.signed_duration_since(*start_time)
        }
        Some(exit_balance) => exit_balance.time.signed_duration_since(*start_time),
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct TearSheet {
    pub sharpe_ratio: SharpeRatio,
    pub sortino_ratio: SortinoRatio,
    pub calmar_ratio: CalmarRatio,
}

impl TearSheet {
    pub fn new(risk_free_return: f64) -> Self {
        Self {
            sharpe_ratio: SharpeRatio::init(risk_free_return),
            sortino_ratio: SortinoRatio::init(risk_free_return),
            calmar_ratio: CalmarRatio::init(risk_free_return),
        }
    }

    pub fn update(&mut self, pnl_returns: &PnLReturnSummary, drawdown: &DrawdownSummary) {
        self.sharpe_ratio.update(pnl_returns);
        self.sortino_ratio.update(pnl_returns);
        self.calmar_ratio
            .update(pnl_returns, drawdown.max_drawdown.drawdown.drawdown);
    }
}

impl TableBuilder for TearSheet {
    fn titles(&self) -> Row {
        row!["Sharpe Ratio", "Sortino Ratio", "Calmar Ratio"]
    }

    fn row(&self) -> Row {
        row![
            format!("{:.3}", self.sharpe_ratio.daily()),
            format!("{:.3}", self.sortino_ratio.daily()),
            format!("{:.3}", self.calmar_ratio.daily()),
        ]
    }
}

pub trait TableBuilder {
    fn titles(&self) -> Row;
    fn row(&self) -> Row;
    fn table(&self, id_cell: &str) -> Table {
        let mut table = Table::new();

        let mut titles = self.titles();
        titles.insert_cell(0, Cell::new(""));
        table.set_titles(titles);

        let mut row = self.row();
        row.insert_cell(0, Cell::new(id_cell));
        table.add_row(row);

        table
    }
    fn table_with<T: TableBuilder>(&self, id_cell: &str, another: (T, &str)) -> Table {
        let mut table = Table::new();

        let mut titles = self.titles();
        titles.insert_cell(0, Cell::new(""));
        table.set_titles(titles);

        let mut first_row = self.row();
        first_row.insert_cell(0, Cell::new(id_cell));
        table.add_row(first_row);

        let mut another_row = another.0.row();
        another_row.insert_cell(0, Cell::new(another.1));
        table.add_row(another_row);

        table
    }
}

pub fn combine<Iter, T>(builders: Iter) -> Table
where
    Iter: IntoIterator<Item = (String, T)>,
    T: TableBuilder,
{
    builders
        .into_iter()
        .enumerate()
        .fold(Table::new(), |mut table, (index, (id, builder))| {
            // Set Table titles using the first builder
            if index == 0 {
                let titles = builder.titles();
                //titles.insert_cell(0, Cell::new(""));
                table.set_titles(titles);
            }

            // Add rows for each builder
            let mut row = builder.row();
            row.insert_cell(0, Cell::new(&id));
            table.add_row(row);

            table
        })
}
