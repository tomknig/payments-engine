use std::collections::HashMap;
use super::client::ClientId;
use super::account::Account;
use super::transaction::{TransactionId, Transaction, TransactionType};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("balance insufficient for withdrawal")]
    WithdrawableBalanceExceeded(String),
    #[error("unknown ledger error")]
    Unknown,
}

pub struct Ledger {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    fn deposit(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn withdrawal(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn dispute(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn resolve(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        match transaction.transaction_type {
            TransactionType::Deposit => self.deposit(transaction),
            TransactionType::Withdrawal => self.withdrawal(transaction),
            TransactionType::Dispute => self.dispute(transaction),
            TransactionType::Resolve => self.resolve(transaction),
            TransactionType::Chargeback => self.chargeback(transaction),
        }
    }
}
