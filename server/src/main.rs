// Main modules
//
mod book;
mod database;
mod formatting;
mod load_config;
mod meshetar;
mod prediction_model;
mod rlang_runner;
// mod api;
// mod plot;

use meshetar::Meshetar;
// Dependencies
//
use rocket::catch;
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::Request;

// Logging
use env_logger::{Builder, Env};
use log::LevelFilter;
use tokio::runtime::Runtime;

use crate::meshetar::Summary;

fn main() {
    // Set log level for libs that are too noisy
    //
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder
        .filter_module("sqlx", LevelFilter::Warn)
        .filter_module("rocket", LevelFilter::Warn)
        .init();

    // Create runtime
    //
    let runtime = Runtime::new().unwrap();
    let meshetar = Meshetar::new();
    let meshetar = runtime.block_on(async { meshetar.initialize().await });
    runtime.block_on(async {
        // Go into runtime
        //
        tokio::spawn(async {
            match meshetar {
                Ok(meshetar) => {}
                Err(meshetar) => {
                    println!("FAIL {:?}", meshetar.summerize())
                }
            }
        });
    });
    ignite_rocket();
}

fn ignite_rocket() -> Result<(), String> {
    match start_rocket() {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("ROCKET ERROR - {:?}", e);
            println!("RESTARTING SERVER");
            ignite_rocket()
        }
    }
}

// Rocket server
// macro needs to in the root crate
// ...
#[macro_use]
extern crate rocket;

#[catch(500)]
fn internal_error() -> &'static str {
    "Error 500; something is not clicking right."
}

#[catch(404)]
fn not_found() -> &'static str {
    "Error 404; nothing here fren."
}

#[catch(default)]
fn default(status: Status, req: &Request) -> String {
    format!("{} ({})", status, req.uri())
}

#[rocket::main]
async fn start_rocket() -> Result<(), String> {
    println!("Igniting rocket.");
    match rocket::build() // .mount("/", routes![api::account_balance_history])
        .mount("/", FileServer::new("static", Options::None).rank(1))
        .register("/", catchers![internal_error, not_found, default])
        .launch()
        .map_err(|e| e.to_string())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
