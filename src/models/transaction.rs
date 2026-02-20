use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use super::client::ClientId;
use super::money::Money;

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("amount of transactions must be positive")]
    NegativeTransactionAmount,
}

pub enum Transaction {
    Deposit(DepositTransaction),
    Withdrawal(WithdrawalTransaction),
    Dispute(DisputeTransaction),
    Resolve(ResolveTransaction),
    Chargeback(ChargebackTransaction),
}

impl Transaction {
    pub fn client_id(&self) -> ClientId {
        match self {
            Transaction::Deposit(transaction) => transaction.client_id,
            Transaction::Withdrawal(transaction) => transaction.client_id,
            Transaction::Dispute(transaction) => transaction.client_id,
            Transaction::Resolve(transaction) => transaction.client_id,
            Transaction::Chargeback(transaction) => transaction.client_id,
        }
    }
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
    client_id: ClientId,
    id: TransactionId,
    amount: Money,
}

impl DepositTransaction {
    pub fn new(
        client_id: ClientId,
        id: TransactionId,
        amount: Money,
    ) -> Result<Self, TransactionError> {
        if amount.is_negative() {
            return Err(TransactionError::NegativeTransactionAmount);
        }

        Ok(Self {
            client_id,
            id,
            amount,
        })
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn amount(&self) -> Money {
        self.amount
    }
}

pub struct WithdrawalTransaction {
    client_id: ClientId,
    id: TransactionId,
    amount: Money,
}

impl WithdrawalTransaction {
    pub fn new(
        client_id: ClientId,
        id: TransactionId,
        amount: Money,
    ) -> Result<Self, TransactionError> {
        if amount.is_negative() {
            return Err(TransactionError::NegativeTransactionAmount);
        }

        Ok(Self {
            client_id,
            id,
            amount,
        })
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn amount(&self) -> Money {
        self.amount
    }
}

pub struct DisputeTransaction {
    client_id: ClientId,
    original_transaction_id: TransactionId,
}

impl DisputeTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn original_transaction_id(&self) -> TransactionId {
        self.original_transaction_id
    }
}

pub struct ResolveTransaction {
    client_id: ClientId,
    original_transaction_id: TransactionId,
}

impl ResolveTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn original_transaction_id(&self) -> TransactionId {
        self.original_transaction_id
    }
}

pub struct ChargebackTransaction {
    client_id: ClientId,
    original_transaction_id: TransactionId,
}

impl ChargebackTransaction {
    pub fn new(client_id: ClientId, original_transaction_id: TransactionId) -> Self {
        Self {
            client_id,
            original_transaction_id,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn original_transaction_id(&self) -> TransactionId {
        self.original_transaction_id
    }
}
