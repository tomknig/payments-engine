use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub client_id: u16,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    pub amount: String,
}
