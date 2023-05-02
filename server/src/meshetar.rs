use std::marker::PhantomData;
use tokio::runtime::Runtime;

use crate::rlang_runner::r_script;

// Valid states
//
pub struct Idle;
pub struct Initialization;
pub struct Running;
pub struct CriticalError;

pub enum Pair {
    USDTBTC,
    //USDTETH,
}

pub trait Summary {
    fn summerize(&self) -> String {
        String::from("SUS! Nothing here ...")
    }
}

// actual struct
pub struct Meshetar<State> {
    pub message_log: Vec<String>,
    pub selected_pair: Pair,
    state: PhantomData<State>,
}

impl<T> Summary for Meshetar<T> {
    fn summerize(&self) -> String {
        match self.message_log.last() {
            Some(message) => message.clone().to_string(),
            None => String::from("No message."),
        }
    }
}

// implement Meshetar for all states
impl Meshetar<Idle> {}
impl Meshetar<CriticalError> {}
impl Meshetar<Initialization> {
    pub async fn start() -> Result<Meshetar<Idle>, Meshetar<CriticalError>> {
        match r_script("renv_install.R", None) {
            Ok(_) => match crate::database::initialize().await {
                Ok(_) => Ok(Meshetar::<Idle> {
                    selected_pair: Pair::USDTBTC,
                    message_log: Vec::new(),
                    state: PhantomData,
                }),
                Err(e) => Err(Meshetar::<CriticalError> {
                    selected_pair: Pair::USDTBTC,
                    message_log: {
                        let mut ml = Vec::new();
                        ml.push(e);
                        ml
                    },
                    state: PhantomData,
                }),
            },
            Err(e) => Err(Meshetar::<CriticalError> {
                selected_pair: Pair::USDTBTC,
                message_log: {
                    let mut ml = Vec::new();
                    ml.push(e);
                    ml
                },
                state: PhantomData,
            }),
        }
    }
}

impl Meshetar<Running> {}
