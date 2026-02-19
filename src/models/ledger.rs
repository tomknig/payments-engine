use super::account::Account;
use super::client::ClientId;
use super::transaction::{
    ChargebackTransaction, DepositTransaction, DisputeTransaction, ResolveTransaction, Transaction,
    TransactionId, WithdrawalTransaction,
};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("deposit of client {0} failed with reason: {1}")]
    DepositError(ClientId, String),
    #[error("withdrawal of client {0} failed with reason: {1}")]
    WithdrawalError(ClientId, String),
    #[error("no account found for client {0}")]
    AccountNotFound(ClientId),
    #[error("transaction {0} of client {1} has already been disputed")]
    TransactionAlreadyDisputed(TransactionId, ClientId),
    #[error("opening a dispute for transaction {0} of client {1} failed with reason: {2}")]
    DisputeError(TransactionId, ClientId, String),
    #[error("no transaction found for id {0} of client {1}")]
    TransactionNotFound(TransactionId, ClientId),
    #[error("resolving the dispute for transaction {0} of client {1} failed with reason: {2}")]
    ResolutionError(TransactionId, ClientId, String),
    #[error("dispute not found for transaction {0} of client {1}")]
    DisputeNotFound(TransactionId, ClientId),
    #[error("chargeback for transaction {0} of client {1} failed with reason: {2}")]
    ChargebackError(TransactionId, ClientId, String),
    #[error("transaction {0} has already been processed")]
    TransactionAlreadyProcessed(TransactionId),
}

pub struct Ledger {
    accounts: HashMap<ClientId, Account>,
    deposits: HashMap<TransactionId, DepositTransaction>,
    open_disputes: HashSet<TransactionId>,
    withdrawal_ids: HashSet<TransactionId>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: HashMap::new(),
            deposits: HashMap::new(),
            open_disputes: HashSet::new(),
            withdrawal_ids: HashSet::new(),
        }
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    fn is_transaction_disputed(&self, transaction_id: TransactionId) -> bool {
        self.open_disputes.contains(&transaction_id)
    }

    fn has_transaction_been_processed(&self, transaction_id: &TransactionId) -> bool {
        self.withdrawal_ids.contains(transaction_id) || self.deposits.contains_key(transaction_id)
    }

    fn get_deposit_transaction(
        &self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> Result<&DepositTransaction, LedgerError> {
        let deposit_transaction = self
            .deposits
            .get(&transaction_id)
            .ok_or(LedgerError::TransactionNotFound(transaction_id, client_id))?;

        if deposit_transaction.client_id != client_id {
            return Err(LedgerError::TransactionNotFound(transaction_id, client_id));
        }

        Ok(deposit_transaction)
    }

    fn handle_deposit(&mut self, transaction: DepositTransaction) -> Result<(), LedgerError> {
        if self.has_transaction_been_processed(&transaction.id) {
            return Err(LedgerError::TransactionAlreadyProcessed(transaction.id));
        }

        let client_id = transaction.client_id;
        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account
            .deposit(transaction.amount)
            .map_err(|e| LedgerError::DepositError(client_id, e.to_string()))?;

        self.deposits.insert(transaction.id, transaction);
        Ok(())
    }

    fn handle_withdrawal(&mut self, transaction: WithdrawalTransaction) -> Result<(), LedgerError> {
        if self.has_transaction_been_processed(&transaction.id) {
            return Err(LedgerError::TransactionAlreadyProcessed(transaction.id));
        }

        let client_id = transaction.client_id;
        let account = self
            .accounts
            .get_mut(&client_id)
            .ok_or(LedgerError::AccountNotFound(client_id))?;

        account
            .withdraw(transaction.amount)
            .map_err(|e| LedgerError::WithdrawalError(client_id, e.to_string()))?;

        self.withdrawal_ids.insert(transaction.id);
        Ok(())
    }

    fn handle_dispute(&mut self, dispute: DisputeTransaction) -> Result<(), LedgerError> {
        let (client_id, transaction_id) = (dispute.client_id, dispute.original_transaction_id);
        let deposit = self.get_deposit_transaction(client_id, transaction_id)?;

        if self.is_transaction_disputed(transaction_id) {
            return Err(LedgerError::TransactionAlreadyDisputed(
                transaction_id,
                client_id,
            ));
        }

        let amount = deposit.amount;
        let account = self
            .accounts
            .get_mut(&client_id)
            .ok_or(LedgerError::AccountNotFound(client_id))?;

        account
            .open_dispute(amount)
            .map_err(|e| LedgerError::DisputeError(transaction_id, client_id, e.to_string()))?;

        self.open_disputes.insert(transaction_id);

        Ok(())
    }

    fn handle_dispute_resolution(
        &mut self,
        resolution: ResolveTransaction,
    ) -> Result<(), LedgerError> {
        let (client_id, transaction_id) =
            (resolution.client_id, resolution.original_transaction_id);
        let deposit = self.get_deposit_transaction(client_id, transaction_id)?;

        if !self.is_transaction_disputed(transaction_id) {
            return Err(LedgerError::DisputeNotFound(transaction_id, client_id));
        }

        let amount = deposit.amount;
        let account = self
            .accounts
            .get_mut(&client_id)
            .ok_or(LedgerError::AccountNotFound(client_id))?;

        account
            .resolve_dispute(amount)
            .map_err(|e| LedgerError::ResolutionError(transaction_id, client_id, e.to_string()))?;

        self.open_disputes.remove(&transaction_id);

        Ok(())
    }

    fn handle_chargeback(&mut self, chargeback: ChargebackTransaction) -> Result<(), LedgerError> {
        let (client_id, transaction_id) =
            (chargeback.client_id, chargeback.original_transaction_id);
        let deposit = self.get_deposit_transaction(client_id, transaction_id)?;

        if !self.is_transaction_disputed(transaction_id) {
            return Err(LedgerError::DisputeNotFound(transaction_id, client_id));
        }

        let amount = deposit.amount;
        let account = self
            .accounts
            .get_mut(&chargeback.client_id)
            .ok_or(LedgerError::AccountNotFound(chargeback.client_id))?;

        account
            .handle_chargeback(amount)
            .map_err(|e| LedgerError::ChargebackError(transaction_id, client_id, e.to_string()))?;

        self.open_disputes.remove(&transaction_id);

        Ok(())
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        match transaction {
            Transaction::Deposit(transaction) => self.handle_deposit(transaction),
            Transaction::Withdrawal(transaction) => self.handle_withdrawal(transaction),
            Transaction::Dispute(transaction) => self.handle_dispute(transaction),
            Transaction::Resolve(transaction) => self.handle_dispute_resolution(transaction),
            Transaction::Chargeback(transaction) => self.handle_chargeback(transaction),
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

    mod deposits {
        use super::*;

        #[test]
        fn test_process_a_single_deposit_transaction() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![Transaction::Deposit(
                DepositTransaction::new(client_1, 9_001, Money::parse_unchecked("100")),
            )]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("100")
            );
        }

        #[test]
        fn test_skip_repeated_transaction_id() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("100")
            );
        }

        #[test]
        fn test_process_multiple_deposit_transactions_of_one_user() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("300")
            );
        }

        #[test]
        fn test_process_multiple_deposit_transactions_of_multiple_users() {
            let client_1 = 1_001;
            let client_2 = 1_002;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_2,
                    9_003,
                    Money::parse_unchecked("300"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_2,
                    9_004,
                    Money::parse_unchecked("400"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_005,
                    Money::parse_unchecked("500"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_2,
                    9_006,
                    Money::parse_unchecked("600"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("800")
            );
            assert_eq!(
                ledger.accounts.get(&client_2).unwrap().total_balance(),
                Money::parse_unchecked("1300")
            );
        }
    }

    mod withdrawals {
        use super::*;

        #[test]
        fn test_withdrawal_succeeds_if_funds_are_available() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("50")
            );
        }

        #[test]
        fn test_withdrawal_fails_if_transaction_id_has_been_used_by_deposit() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("50"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("100")
            );
        }

        #[test]
        fn test_withdrawal_fails_if_transaction_id_has_been_used_by_withdrawal() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("50")
            );
        }

        #[test]
        fn test_withdrawal_fails_if_client_does_not_exist() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![Transaction::Withdrawal(
                WithdrawalTransaction::new(client_1, 9_002, Money::parse_unchecked("50")),
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
                    .starts_with("no account found for client 1001"),
                "unexpected error: {first_error}"
            );
        }

        #[test]
        fn test_withdrawal_fails_if_amount_exceeds_balance() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
            ]);

            assert_eq!(
                ledger.accounts.get(&client_1).unwrap().total_balance(),
                Money::parse_unchecked("100")
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

        #[test]
        fn test_withdrawal_after_a_dispute_has_been_opened() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_001)),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_003,
                    Money::parse_unchecked("250"),
                )),
            ]);

            let error = results
                .get(3)
                .expect("expect a result for the withdrawal transaction")
                .as_ref()
                .expect_err("expected the withdrawal transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("withdrawal of client 1001 failed with reason"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("200")
            );
            assert_eq!(
                client_1_account.held_balance(),
                Money::parse_unchecked("100")
            );
        }
    }

    mod open_disputes {
        use super::*;

        #[test]
        fn test_open_dispute_with_existing_deposit() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_001)),
            ]);

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("0")
            );
            assert_eq!(
                client_1_account.held_balance(),
                Money::parse_unchecked("100")
            );
        }

        #[test]
        fn test_open_dispute_for_already_disputed_deposit() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_001)),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_001)),
            ]);

            let error = results
                .get(3)
                .expect("expect a result for the second dispute transaction")
                .as_ref()
                .expect_err("expected second dispute transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("transaction 9001 of client 1001 has already been disputed"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(
                client_1_account.held_balance(),
                Money::parse_unchecked("50")
            );
        }

        #[test]
        fn test_open_dispute_for_transaction_of_another_client() {
            let client_1 = 1_001;
            let client_2 = 1_002;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_2,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_2, 9_001)),
            ]);

            let error = results
                .get(2)
                .expect("expect a result for the dispute transaction")
                .as_ref()
                .expect_err("expected the dispute transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("no transaction found for id 9001 of client 1002"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));

            let client_2_account = ledger.accounts.get(&client_2).unwrap();
            assert_eq!(
                client_2_account.total_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(
                client_2_account.available_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(client_2_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_open_dispute_for_existing_but_incorrect_transaction() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(2)
                .expect("expect a result for the dispute transaction")
                .as_ref()
                .expect_err("expected the dispute transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("no transaction found for id 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_open_dispute_for_non_existing_transaction() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(1)
                .expect("expect a result for the dispute transaction")
                .as_ref()
                .expect_err("expected the dispute transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("no transaction found for id 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_open_dispute_for_existing_transaction_but_insufficient_balance() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Withdrawal(WithdrawalTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("50"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_001)),
            ]);

            let error = results
                .get(2)
                .expect("expect a result for the dispute transaction")
                .as_ref()
                .expect_err("expected the dispute transaction to fail");

            assert!(
                error.to_string().starts_with(
                    "opening a dispute for transaction 9001 of client 1001 failed with reason"
                ),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("50")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }
    }

    mod resolve_disputes {
        use super::*;

        #[test]
        fn test_resolve_an_open_and_unresolved_dispute() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
                Transaction::Resolve(ResolveTransaction::new(client_1, 9_002)),
            ]);

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_resolve_a_resolved_dispute() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
                Transaction::Resolve(ResolveTransaction::new(client_1, 9_002)),
                Transaction::Resolve(ResolveTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(4)
                .expect("expect a result for the second dispute resolution transaction")
                .as_ref()
                .expect_err("expected the second dispute resolution transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("dispute not found for transaction 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_resolve_without_an_opened_dispute() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Resolve(ResolveTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(2)
                .expect("expect a result for the dispute resolution transaction")
                .as_ref()
                .expect_err("expected the dispute resolution transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("dispute not found for transaction 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }
    }

    mod chargebacks {
        use super::*;

        #[test]
        fn test_issue_a_chargeback_for_an_open_and_unresolved_dispute() {
            let client_1 = 1_001;

            let (ledger, _) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
                Transaction::Chargeback(ChargebackTransaction::new(client_1, 9_002)),
            ]);

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_issue_a_chargeback_for_a_resolved_dispute() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
                Transaction::Chargeback(ChargebackTransaction::new(client_1, 9_002)),
                Transaction::Chargeback(ChargebackTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(4)
                .expect("expect a result for the second chargeback transaction")
                .as_ref()
                .expect_err("expected the second chargeback transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("dispute not found for transaction 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_issue_a_chargeback_without_an_opened_dispute() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Chargeback(ChargebackTransaction::new(client_1, 9_002)),
            ]);

            let error = results
                .get(2)
                .expect("expect a result for the chargeback transaction")
                .as_ref()
                .expect_err("expected the chargeback transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("dispute not found for transaction 9002 of client 1001"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("300")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }

        #[test]
        fn test_a_successful_chargeback_freezes_the_affected_account() {
            let client_1 = 1_001;

            let (ledger, results) = ledger_with_transactions(vec![
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_001,
                    Money::parse_unchecked("100"),
                )),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_002,
                    Money::parse_unchecked("200"),
                )),
                Transaction::Dispute(DisputeTransaction::new(client_1, 9_002)),
                Transaction::Chargeback(ChargebackTransaction::new(client_1, 9_002)),
                Transaction::Deposit(DepositTransaction::new(
                    client_1,
                    9_003,
                    Money::parse_unchecked("300"),
                )),
            ]);

            let error = results
                .get(4)
                .expect("expect a result for the last deposit transaction")
                .as_ref()
                .expect_err("expected the last deposit transaction to fail");

            assert!(
                error
                    .to_string()
                    .starts_with("deposit of client 1001 failed with reason: account is frozen"),
                "unexpected error: {error}"
            );

            let client_1_account = ledger.accounts.get(&client_1).unwrap();
            assert_eq!(
                client_1_account.total_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(
                client_1_account.available_balance(),
                Money::parse_unchecked("100")
            );
            assert_eq!(client_1_account.held_balance(), Money::parse_unchecked("0"));
        }
    }
}
