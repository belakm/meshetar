use rocket::fs::NamedFile;
use rocket::get;
use rocket::http::Status;
use rocket::response::status::{Custom, NotFound};

static PATH: &str = "static/account_balance.svg";

#[get("/plot/account_balance_history")]
pub async fn account_balance_history() -> Result<NamedFile, ServerError> {
    let plot = crate::plot::plot_account_balance().await;
    match plot {
        Ok(_) => NamedFile::open(&PATH)
            .await
            .map_err(|e| MAP_TO_404(&e.to_string())),
        Err(err) => Err(err),
    }
}
pub static MAP_TO_500: fn(&str) -> ServerError =
    |err| ServerError::Custom(Custom(Status::InternalServerError, err.to_string()));

pub static MAP_TO_404: fn(&str) -> ServerError =
    |err| ServerError::NotFound(NotFound(err.to_string()));

#[derive(Debug, Responder)]
pub enum ServerError {
    Custom(Custom<String>),
    NotFound(NotFound<String>),
}
