// Valid states
//
pub struct New;
pub struct Initialization;
pub struct Idle;
pub struct Running;
pub struct CriticalError;

#[derive(Copy, Clone, Debug)]
pub enum Status {
    Idle,
    Running,
    FetchingHistory,
}

#[derive(Copy, Clone, Debug)]
pub enum Pair {
    BTCUSDT,
    ETHBTC,
}
impl Pair {
    pub fn to_str(&self) -> String {
        match self {
            Pair::BTCUSDT => "BTCUSDT".to_string(),
            Pair::ETHBTC => "ETHBTC".to_string(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Interval {
    Minutes1,
    Minutes3,
}

// Core struct
//
pub struct Meshetar {
    pub pair: Pair,
    pub interval: Interval,
    pub status: Status,
}

impl Meshetar {
    pub fn new() -> Self {
        Meshetar {
            interval: Interval::Minutes1,
            pair: Pair::BTCUSDT,
            status: Status::Idle,
        }
    }
    pub fn summerize(&self) -> String {
        format!("Pair: {:?} --- Interval: {:?}", self.pair, self.interval)
    }
    pub fn go_to_idle(mut self) -> Self {
        self.status = Status::Idle;
        self
    }
}
