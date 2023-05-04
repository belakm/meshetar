use std::marker::PhantomData;

use tokio::runtime::Runtime;

use crate::rlang_runner::r_script;

// Valid states
//
pub struct IdleNoServer;
pub struct Idle;
pub struct Initialization;
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
impl Meshetar<Idle> {}
impl Meshetar<IdleNoServer> {
    pub async fn start_server(self) -> Result<Meshetar<Idle>, Meshetar<IdleNoServer>> {
        let mut message_log = self.message_log.clone();
        match crate::start_rocket() {
            Ok(_) => Ok(Meshetar::<Idle> {
                message_log: {
                    message_log.push("Started rocket server.".into());
                    message_log
                },
                selected_pair: Pair::from(self.selected_pair),
                state: PhantomData,
            }),
            Err(e) => Err(Self {
                message_log: {
                    message_log.push(format!("Error starting Rocket server: {:?}", e).into());
                    message_log
                },
                selected_pair: Pair::from(self.selected_pair),
                state: PhantomData,
            }),
        }
    }
}
impl Meshetar<CriticalError> {}
impl Meshetar<Initialization> {
    pub async fn initialize() -> Result<Meshetar<IdleNoServer>, Meshetar<CriticalError>> {
        // Create new log
        let mut message_log: Vec<String> = Vec::new();
        message_log.push("Starting initialization.".into());

        // Install R dependencies - we cant do much without them
        //
        match r_script("renv_install.R", None) {
            // Initialize our database
            //
            Ok(_) => match crate::database::initialize().await {
                // Ready to rumble
                //
                Ok(_) => Ok(Meshetar::<IdleNoServer> {
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
            },
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

impl Meshetar<Running> {}
