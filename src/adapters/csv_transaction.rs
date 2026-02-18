use std::convert::TryFrom;

use crate::csv::{Transaction as CsvTransaction, TransactionType as CsvTransactionType};
use anyhow::Context;
use payments_engine::models::{
    ChargebackTransaction, DepositTransaction, DisputeTransaction, ResolveTransaction, Transaction,
    WithdrawalTransaction,
};

impl TryFrom<CsvTransaction> for Transaction {
    type Error = anyhow::Error;

    fn try_from(csv_transaction: CsvTransaction) -> Result<Self, Self::Error> {
        let transaction = match csv_transaction.transaction_type {
            CsvTransactionType::Deposit => Transaction::Deposit(DepositTransaction {
                client_id: csv_transaction.client_id,
                id: csv_transaction.transaction_id,
                amount: csv_transaction
                    .amount
                    .try_into()
                    .context("Failed to parse amount")?,
            }),
            CsvTransactionType::Withdrawal => Transaction::Withdrawal(WithdrawalTransaction {
                client_id: csv_transaction.client_id,
                id: csv_transaction.transaction_id,
                amount: csv_transaction
                    .amount
                    .try_into()
                    .context("Failed to parse amount")?,
            }),
            CsvTransactionType::Dispute => Transaction::Dispute(DisputeTransaction {
                client_id: csv_transaction.client_id,
                original_transaction_id: csv_transaction.transaction_id,
            }),
            CsvTransactionType::Resolve => Transaction::Resolve(ResolveTransaction {
                client_id: csv_transaction.client_id,
                original_transaction_id: csv_transaction.transaction_id,
            }),
            CsvTransactionType::Chargeback => Transaction::Chargeback(ChargebackTransaction {
                client_id: csv_transaction.client_id,
                original_transaction_id: csv_transaction.transaction_id,
            }),
        };

        Ok(transaction)
    }
}
