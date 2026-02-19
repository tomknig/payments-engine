use serde::{Deserialize, Serialize};
use std::fmt;

use super::client::ClientId;
use super::money::Money;

pub enum Transaction {
    Deposit(DepositTransaction),
    Withdrawal(WithdrawalTransaction),
    Dispute(DisputeTransaction),
    Resolve(ResolveTransaction),
    Chargeback(ChargebackTransaction),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransactionId(u32);

impl TransactionId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<u32> for TransactionId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

pub struct DepositTransaction {
    pub client_id: ClientId,
    pub id: TransactionId,
    pub amount: Money,
}

impl DepositTransaction {
    pub fn new(client_id: ClientId, id: TransactionId, amount: Money) -> Self {
        Self {
            client_id,
            id,
            amount,
        }
    }
}

pub struct WithdrawalTransaction {
    pub client_id: ClientId,
    pub id: TransactionId,
    pub amount: Money,
}

impl WithdrawalTransaction {
    pub fn new(client_id: ClientId, id: TransactionId, amount: Money) -> Self {
        Self {
            client_id,
            id,
            amount,
        }
    }
}

pub struct DisputeTransaction {
    pub client_id: ClientId,
    pub original_transaction_id: TransactionId,
}

impl DisputeTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }
}

pub struct ResolveTransaction {
    pub client_id: ClientId,
    pub original_transaction_id: TransactionId,
}

impl ResolveTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }
}

pub struct ChargebackTransaction {
    pub client_id: ClientId,
    pub original_transaction_id: TransactionId,
}

impl ChargebackTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }
}
