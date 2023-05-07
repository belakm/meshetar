use std::marker::PhantomData;

use crate::rlang_runner::r_script;

// Valid states
//
pub struct New;
pub struct Initialization;
pub struct Idle;
pub struct Running;
pub struct CriticalError;

#[derive(Copy, Clone)]
pub enum Pair {
    USDTBTC,
    //USDTETH,
}

pub trait Summary {
    fn summerize(&self) -> String {
        "SUS! Nothing here ...".into()
    }
}

// Core struct
//
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

// State implementations
//
impl Meshetar<New> {
    pub fn new() -> Meshetar<Initialization> {
        Meshetar::<Initialization> {
            message_log: Vec::new(),
            selected_pair: Pair::USDTBTC,
            state: PhantomData,
        }
    }
}
impl Meshetar<Initialization> {
    pub async fn initialize(self) -> Result<Meshetar<Idle>, Meshetar<CriticalError>> {
        // Create new log
        let mut message_log: Vec<String> = Vec::new();
        message_log.push("Starting initialization.".into());

        // Install R dependencies - we cant do much without them
        //
        match r_script("renv_install.R", None) {
            // Initialize our database
            //
            Ok(_) => Ok(Meshetar::<Idle> {
                selected_pair: Pair::USDTBTC,
                message_log: {
                    message_log.push("Database Inititiated".to_string());
                    message_log
                },
                state: PhantomData,
            }),
            Err(e) => Err(Meshetar::<CriticalError> {
                selected_pair: Pair::USDTBTC,
                message_log: {
                    message_log.push(e);
                    message_log
                },
                state: PhantomData,
            }),
        }
    }
}
impl Meshetar<Idle> {}
impl Meshetar<CriticalError> {}
impl Meshetar<Running> {}
