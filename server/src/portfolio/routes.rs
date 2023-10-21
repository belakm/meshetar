// use rocket::{
//     http::Status,
//     response::status::{Accepted, Custom},
//     serde::json::Json,
// };
//
// use super::balance::ExchangeBalanceSheet;
//
// #[get("/balance_sheet")]
// pub async fn balance_sheet() -> Result<Accepted<Json<ExchangeBalanceSheet>>, Custom<String>> {
//     match get_balance_sheet().await {
//         Ok(balance_sheet) => Ok(Accepted(Some(Json(balance_sheet.clone())))),
//         Err(e) => Err(Custom(Status::NotFound, format!("{:?}", e))),
//     }
// }
