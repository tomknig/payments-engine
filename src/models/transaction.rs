use super::client::ClientId;
use super::money::Money;
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
    pub id: TransactionId,
    pub transaction_type: TransactionType,
    pub client_id: ClientId,
    pub amount: Money,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        client_id: ClientId,
        id: TransactionId,
        amount: Money,
    ) -> Self {
        Transaction {
            id,
            transaction_type,
            client_id,
            amount,
        }
    }
}
