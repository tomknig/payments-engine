use super::client::ClientId;
use super::money::Money;

pub enum Transaction {
    Deposit(DepositTransaction),
    Withdrawal(WithdrawalTransaction),
    Dispute(DisputeTransaction),
    Resolve(ResolveTransaction),
    Chargeback(ChargebackTransaction),
}

pub type TransactionId = u32;

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
