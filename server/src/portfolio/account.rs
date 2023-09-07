use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Account {
    #[serde(rename = "makerCommission")]
    maker_commission: i64,
    #[serde(rename = "takerCommission")]
    taker_commission: i64,
    #[serde(rename = "buyerCommission")]
    buyer_commission: i64,
    #[serde(rename = "sellerCommission")]
    seller_commission: i64,
    #[serde(rename = "canTrade")]
    can_trade: bool,
    #[serde(rename = "canWithdraw")]
    can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    can_deposit: bool,
    brokered: bool,
    #[serde(rename = "requireSelfTradePrevention")]
    require_self_rade_prevention: bool,
    #[serde(rename = "preventSor")]
    prevent_sor: bool,
    #[serde(rename = "updateTime")]
    update_time: i64,
    #[serde(rename = "accountType")]
    account_type: String,
    // permissions: Vec<String>,
    uid: i64,
}
