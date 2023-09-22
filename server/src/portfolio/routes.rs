use rocket::{
    http::Status,
    response::status::{Accepted, Custom},
    serde::json::Json,
};

#[get("/balance_sheet")]
pub async fn balance_sheet() -> Result<Accepted<Json<BalanceSheetWithBalances>>, Custom<String>> {
    match portfolio::get_balance_sheet().await {
        Ok(balance_sheet) => Ok(Accepted(Some(Json(balance_sheet.clone())))),
        Err(e) => Err(Custom(Status::NotFound, format!("{:?}", e))),
    }
}
