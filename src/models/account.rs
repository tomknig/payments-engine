use super::client::ClientId;
use super::money::Money;
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("available balance (`{0}`) is insufficient for withdrawal (`{1}`)")]
    WithdrawableBalanceExceeded(Money, Money),
    #[error("available balance (`{0}`) is insufficient for dispute (`{1}`)")]
    DisputableBalanceExceeded(Money, Money),
    #[error("held balance (`{0}`) is insufficient for resolution (`{1}`)")]
    HeldBalanceExceeded(Money, Money),
    #[error("money arithmetic failed: {0}")]
    MoneyArithmeticError(String),
    #[error("transaction amount must never be negative")]
    NegativeTransactionAmount,
    #[error("account is frozen")]
    AccountFrozen,
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    client_id: ClientId,
    available: Money,
    held: Money,
    locked: bool,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: Money::default(),
            held: Money::default(),
            locked: false,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn available_balance(&self) -> Money {
        self.available
    }

    pub fn held_balance(&self) -> Money {
        self.held
    }

    pub fn total_balance(&self) -> Money {
        self.available
            .checked_add(self.held)
            .expect("account invariant violated: total balance out of bounds")
    }

    pub fn deposit(&mut self, amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if amount.is_negative() {
            return Err(AccountError::NegativeTransactionAmount);
        }

        self.available = self
            .available
            .checked_add(amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if amount.is_negative() {
            return Err(AccountError::NegativeTransactionAmount);
        }

        if self.available < amount {
            return Err(AccountError::WithdrawableBalanceExceeded(
                self.available,
                amount,
            ));
        }

        self.available = self
            .available
            .checked_sub(amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;

        Ok(())
    }

    pub fn open_dispute(&mut self, dispute_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if dispute_amount.is_negative() {
            return Err(AccountError::NegativeTransactionAmount);
        }

        if self.available < dispute_amount {
            return Err(AccountError::DisputableBalanceExceeded(
                self.available,
                dispute_amount,
            ));
        }

        let new_available = self
            .available
            .checked_sub(dispute_amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;
        let new_held = self
            .held
            .checked_add(dispute_amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;
        self.available = new_available;
        self.held = new_held;
        Ok(())
    }

    pub fn resolve_dispute(&mut self, resolution_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if resolution_amount.is_negative() {
            return Err(AccountError::NegativeTransactionAmount);
        }

        if self.held < resolution_amount {
            return Err(AccountError::HeldBalanceExceeded(
                self.held,
                resolution_amount,
            ));
        }

        let new_held = self
            .held
            .checked_sub(resolution_amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;
        let new_available = self
            .available
            .checked_add(resolution_amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;
        self.held = new_held;
        self.available = new_available;
        Ok(())
    }

    pub fn handle_chargeback(&mut self, chargeback_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if chargeback_amount.is_negative() {
            return Err(AccountError::NegativeTransactionAmount);
        }

        if self.held < chargeback_amount {
            return Err(AccountError::HeldBalanceExceeded(
                self.held,
                chargeback_amount,
            ));
        }

        self.held = self
            .held
            .checked_sub(chargeback_amount)
            .map_err(|e| AccountError::MoneyArithmeticError(e.to_string()))?;
        self.locked = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod invariants {
        use super::*;

        #[test]
        fn test_total_money_invariant() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("100").unwrap();
            account.held = Money::parse("10").unwrap();

            assert_eq!(account.total_balance(), Money::parse("110").unwrap());
            assert_eq!(account.available, Money::parse("100").unwrap());
            assert_eq!(account.held, Money::parse("10").unwrap());
            assert!(!account.locked);
        }
    }

    mod deposits {
        use super::*;

        #[test]
        fn test_deposit_money() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("0").unwrap();

            let deposit_result = account.deposit(Money::parse("42").unwrap());
            assert!(deposit_result.is_ok());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_deposit_to_locked_account() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("0").unwrap();
            account.locked = true;

            let deposit_result = account.deposit(Money::parse("42").unwrap());
            assert_eq!(deposit_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(account.locked);
        }

        #[test]
        fn test_reject_negative_deposit() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("2").unwrap();

            let deposit_result = account.deposit(Money::parse("-1").unwrap());
            assert_eq!(deposit_result, Err(AccountError::NegativeTransactionAmount));

            assert_eq!(account.total_balance(), Money::parse("2").unwrap());
            assert_eq!(account.available, Money::parse("2").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }
    }

    mod withdrawals {
        use super::*;

        #[test]
        fn test_withdraw_available_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let withdrawal_result = account.withdraw(Money::parse("2").unwrap());
            assert!(withdrawal_result.is_ok());

            assert_eq!(account.total_balance(), Money::parse("40").unwrap());
            assert_eq!(account.available, Money::parse("40").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_all_available_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let withdrawal_result = account.withdraw(Money::parse("42").unwrap());
            assert!(withdrawal_result.is_ok());

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_unavailable_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let withdrawal_result = account.withdraw(Money::parse("50").unwrap());
            assert!(withdrawal_result.is_err());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_from_empty_account() {
            let mut account = Account::new(ClientId::new(1));

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());

            let withdrawal_result = account.withdraw(Money::parse("1").unwrap());
            assert!(withdrawal_result.is_err());

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_from_locked_account() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();
            account.locked = true;

            let withdrawal_result = account.withdraw(Money::parse("1").unwrap());
            assert_eq!(withdrawal_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(account.locked);
        }

        #[test]
        fn test_reject_negative_withdrawal() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let withdrawal_result = account.withdraw(Money::parse("-1").unwrap());
            assert_eq!(
                withdrawal_result,
                Err(AccountError::NegativeTransactionAmount)
            );

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }
    }

    mod open_disputes {
        use super::*;

        #[test]
        fn test_open_dispute() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("20").unwrap());
            assert!(dispute_result.is_ok());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("22").unwrap());
            assert_eq!(account.held, Money::parse("20").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_for_all_available_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("42").unwrap());
            assert!(dispute_result.is_ok());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("42").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_with_insufficient_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("50").unwrap());
            assert!(dispute_result.is_err());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_on_empty_account() {
            let mut account = Account::new(ClientId::new(1));

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());

            let dispute_result = account.open_dispute(Money::parse("42").unwrap());
            assert!(dispute_result.is_err());

            assert_eq!(account.total_balance(), Money::parse("0").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_on_locked_account() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();
            account.locked = true;

            let dispute_result = account.open_dispute(Money::parse("2").unwrap());
            assert_eq!(dispute_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(account.locked);
        }
    }

    mod resolve_disputes {
        use super::*;

        #[test]
        fn test_resolve_dispute() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("10").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("10").unwrap());

            let resolve_result = account.resolve_dispute(Money::parse("2").unwrap());
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("34").unwrap());
            assert_eq!(account.held, Money::parse("8").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_of_all_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("42").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("42").unwrap());

            let resolve_result = account.resolve_dispute(Money::parse("42").unwrap());
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_with_insufficient_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("2").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("40").unwrap());
            assert_eq!(account.held, Money::parse("2").unwrap());

            let resolve_result = account.resolve_dispute(Money::parse("4").unwrap());
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("40").unwrap());
            assert_eq!(account.held, Money::parse("2").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_with_no_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());

            let resolve_result = account.resolve_dispute(Money::parse("2").unwrap());
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_on_locked_account() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();
            account.held = Money::parse("2").unwrap();
            account.locked = true;

            let resolve_result = account.resolve_dispute(Money::parse("2").unwrap());
            assert_eq!(resolve_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::parse("44").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("2").unwrap());
            assert!(account.locked);
        }
    }

    mod chargebacks {
        use super::*;

        #[test]
        fn test_chargeback() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("10").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("10").unwrap());

            let resolve_result = account.handle_chargeback(Money::parse("2").unwrap());
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("40").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("8").unwrap());
            assert!(account.locked);
        }

        #[test]
        fn test_chargeback_of_all_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("42").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("42").unwrap());

            let resolve_result = account.handle_chargeback(Money::parse("42").unwrap());
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("0").unwrap());
            assert_eq!(account.available, Money::parse("0").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(account.locked);
        }

        #[test]
        fn test_chargeback_with_insufficient_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("2").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("40").unwrap());
            assert_eq!(account.held, Money::parse("2").unwrap());
            assert!(!account.locked);

            let resolve_result = account.handle_chargeback(Money::parse("4").unwrap());
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("40").unwrap());
            assert_eq!(account.held, Money::parse("2").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_chargeback_with_no_held_funds() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);

            let resolve_result = account.handle_chargeback(Money::parse("2").unwrap());
            assert!(resolve_result.is_err());

            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("42").unwrap());
            assert_eq!(account.held, Money::parse("0").unwrap());
            assert!(!account.locked);
        }

        #[test]
        fn test_chargeback_with_locked_account() {
            let mut account = Account::new(ClientId::new(1));
            account.available = Money::parse("42").unwrap();

            let dispute_result = account.open_dispute(Money::parse("10").unwrap());
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("42").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("10").unwrap());

            let resolve_result = account.handle_chargeback(Money::parse("2").unwrap());
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::parse("40").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("8").unwrap());
            assert!(account.locked);

            let resolve_result = account.handle_chargeback(Money::parse("8").unwrap());
            assert_eq!(resolve_result, Err(AccountError::AccountFrozen));
            assert_eq!(account.total_balance(), Money::parse("40").unwrap());
            assert_eq!(account.available, Money::parse("32").unwrap());
            assert_eq!(account.held, Money::parse("8").unwrap());
            assert!(account.locked);
        }
    }
}
