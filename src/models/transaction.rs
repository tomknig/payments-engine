use super::money::Money;
use super::client::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub type TransactionId = u32;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    client_id: ClientId,
    transaction_id: TransactionId,
    amount: Money,
}
