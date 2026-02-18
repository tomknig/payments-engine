use super::account::Account;
use super::client::ClientId;
use super::transaction::{Transaction, TransactionId, TransactionType};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("deposit of client {0} failed with reason: {1}")]
    DepositError(ClientId, String),
    #[error("withdrawal of client {0} failed with reason: {1}")]
    WithdrawalError(ClientId, String),
    #[error("no account found for client id {0}")]
    AccountNotFound(ClientId),
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

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    fn handle_deposit(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        let client_id = transaction.client_id;
        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account
            .deposit(transaction.amount)
            .map_err(|e| LedgerError::DepositError(client_id, e.to_string()))?;

        self.transactions.insert(transaction.id, transaction);
        Ok(())
    }

    fn handle_withdrawal(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        let client_id = transaction.client_id;

        let account = self
            .accounts
            .get_mut(&client_id)
            .ok_or(LedgerError::AccountNotFound(client_id))?;

        account
            .withdraw(transaction.amount)
            .map_err(|e| LedgerError::WithdrawalError(client_id, e.to_string()))?;

        self.transactions.insert(transaction.id, transaction);
        Ok(())
    }

    fn handle_dispute(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn handle_dispute_resolution(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    fn handle_chargeback(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        todo!()
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        match transaction.transaction_type {
            TransactionType::Deposit => self.handle_deposit(transaction),
            TransactionType::Withdrawal => self.handle_withdrawal(transaction),
            TransactionType::Dispute => self.handle_dispute(transaction),
            TransactionType::Resolve => self.handle_dispute_resolution(transaction),
            TransactionType::Chargeback => self.handle_chargeback(transaction),
        }
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::money::Money;
    use super::*;

    fn ledger_with_transactions(
        transactions: Vec<Transaction>,
    ) -> (Ledger, Vec<Result<(), LedgerError>>) {
        let mut ledger = Ledger::new();
        let mut results = Vec::with_capacity(transactions.len());

        for transaction in transactions {
            let result = ledger.process_transaction(transaction);
            results.push(result);
        }

        (ledger, results)
    }

    #[test]
    fn test_process_a_single_deposit_transaction() {
        let client_1 = 1_001;

        let (ledger, _) = ledger_with_transactions(vec![Transaction::new(
            TransactionType::Deposit,
            client_1,
            9_001,
            Money::new("100"),
        )]);

        assert_eq!(
            ledger.accounts.get(&client_1).unwrap().total(),
            Money::new("100")
        );
    }

    #[test]
    fn test_process_multiple_deposit_transactions_of_one_user() {
        let client_1 = 1_001;

        let (ledger, _) = ledger_with_transactions(vec![
            Transaction::new(TransactionType::Deposit, client_1, 9_001, Money::new("100")),
            Transaction::new(TransactionType::Deposit, client_1, 9_002, Money::new("200")),
        ]);

        assert_eq!(
            ledger.accounts.get(&client_1).unwrap().total(),
            Money::new("300")
        );
    }

    #[test]
    fn test_process_multiple_deposit_transactions_of_multiple_users() {
        let client_1 = 1_001;
        let client_2 = 1_002;

        let (ledger, _) = ledger_with_transactions(vec![
            Transaction::new(TransactionType::Deposit, client_1, 9_001, Money::new("100")),
            Transaction::new(TransactionType::Deposit, client_1, 9_002, Money::new("200")),
            Transaction::new(TransactionType::Deposit, client_2, 9_003, Money::new("300")),
            Transaction::new(TransactionType::Deposit, client_2, 9_004, Money::new("400")),
            Transaction::new(TransactionType::Deposit, client_1, 9_005, Money::new("500")),
            Transaction::new(TransactionType::Deposit, client_2, 9_006, Money::new("600")),
        ]);

        assert_eq!(
            ledger.accounts.get(&client_1).unwrap().total(),
            Money::new("800")
        );
        assert_eq!(
            ledger.accounts.get(&client_2).unwrap().total(),
            Money::new("1300")
        );
    }

    #[test]
    fn test_withdrawal_succeeds_if_funds_are_available() {
        let client_1 = 1_001;

        let (ledger, _) = ledger_with_transactions(vec![
            Transaction::new(TransactionType::Deposit, client_1, 9_001, Money::new("100")),
            Transaction::new(
                TransactionType::Withdrawal,
                client_1,
                9_002,
                Money::new("50"),
            ),
        ]);

        assert_eq!(
            ledger.accounts.get(&client_1).unwrap().total(),
            Money::new("50")
        );
    }

    #[test]
    fn test_withdrawal_fails_if_client_does_not_exist() {
        let client_1 = 1_001;

        let (ledger, results) = ledger_with_transactions(vec![Transaction::new(
            TransactionType::Withdrawal,
            client_1,
            9_001,
            Money::new("50"),
        )]);

        assert_eq!(ledger.accounts.len(), 0);

        let first_error = results
            .first()
            .expect("expected one transaction result")
            .as_ref()
            .expect_err("expected first transaction to fail");

        assert!(
            first_error
                .to_string()
                .starts_with("no account found for client id 1001"),
            "unexpected error: {first_error}"
        );
    }

    #[test]
    fn test_withdrawal_fails_if_amount_exceeds_balance() {
        let client_1 = 1_001;

        let (ledger, results) = ledger_with_transactions(vec![
            Transaction::new(TransactionType::Deposit, client_1, 9_001, Money::new("100")),
            Transaction::new(
                TransactionType::Withdrawal,
                client_1,
                9_002,
                Money::new("200"),
            ),
        ]);

        assert_eq!(
            ledger.accounts.get(&client_1).unwrap().total(),
            Money::new("100")
        );

        let first_error = results
            .get(1)
            .expect("expected a transaction result")
            .as_ref()
            .expect_err("expected first transaction to fail");

        assert!(
            first_error
                .to_string()
                .starts_with("withdrawal of client 1001 failed with reason"),
            "unexpected error: {first_error}"
        );
    }
}
