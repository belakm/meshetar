// Main modules
//
mod big_iron;
mod book;
mod database;
mod formatting;
mod load_config;
mod prediction_model;
mod rlang_runner;
// mod api;
// mod plot;

// Dependencies
//
use rocket::catch;
use rocket::fs::FileServer;
use rocket::fs::Options;
use rocket::futures::TryFutureExt;
use rocket::http::Status;
use rocket::Request;
use tokio::runtime::Runtime;

// Logging
use env_logger::{Builder, Env};
use log::LevelFilter;

fn main() {
    // Create

    // Set log level for libs that are too noisy
    //
    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));
    builder.filter_module("sqlx", LevelFilter::Off).init();

    // Init R libs and start the program
    //
    match rlang_runner::r_script("renv_install.R", None) {
        Ok(_) => {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Call your async functions in sequence
                let init1 = database::initialize().await;
                /*let init2 = book::fetch_history("BTCUSDT").await;

                // TODO: find better matching mechanism
                match (init1, init2) {
                    (Ok(_), Ok(_)) => {}
                    (Ok(_), Err(e)) => println!("{:?}", e),
                    (Err(e), Ok(_)) => println!("{:?}", e),
                    (Err(e1), Err(e2)) => println!("{:1?} {:2?}", e1, e2),
                }*/
            });
            rt.spawn(book::main());
            start_rocket().unwrap();
        }
        Err(e) => println!("Error in R: {:?}", e),
    }
}

// Rocket
//
// needs to in the root crate

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
