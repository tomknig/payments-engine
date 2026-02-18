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
            available: Money::new("0"),
            held: Money::new("0"),
            locked: false,
        }
    }

    pub fn available_balance(&self) -> Money {
        self.available
    }

    pub fn held_balance(&self) -> Money {
        self.held
    }

    pub fn total_balance(&self) -> Money {
        self.available + self.held
    }

    pub fn deposit(&mut self, amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        self.available = self.available + amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if self.available < amount {
            return Err(AccountError::WithdrawableBalanceExceeded(
                self.available,
                amount,
            ));
        }

        self.available = self.available - amount;
        Ok(())
    }

    pub fn open_dispute(&mut self, dispute_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if self.available < dispute_amount {
            return Err(AccountError::DisputableBalanceExceeded(
                self.available,
                dispute_amount,
            ));
        }

        self.available = self.available - dispute_amount;
        self.held = self.held + dispute_amount;
        Ok(())
    }

    pub fn resolve_dispute(&mut self, resolution_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if self.held < resolution_amount {
            return Err(AccountError::HeldBalanceExceeded(
                self.held,
                resolution_amount,
            ));
        }

        self.held = self.held - resolution_amount;
        self.available = self.available + resolution_amount;
        Ok(())
    }

    pub fn handle_chargeback(&mut self, chargeback_amount: Money) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::AccountFrozen);
        }

        if self.held < chargeback_amount {
            return Err(AccountError::HeldBalanceExceeded(
                self.held,
                chargeback_amount,
            ));
        }

        self.held = self.held - chargeback_amount;
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
            let mut account = Account::new(1);
            account.available = Money::new("100");
            account.held = Money::new("10");

            assert_eq!(account.total_balance(), Money::new("110"));
            assert_eq!(account.available, Money::new("100"));
            assert_eq!(account.held, Money::new("10"));
            assert!(!account.locked);
        }
    }

    mod deposits {
        use super::*;

        #[test]
        fn test_deposit_money() {
            let mut account = Account::new(1);
            account.available = Money::new("0");

            let deposit_result = account.deposit(Money::new("42"));
            assert!(deposit_result.is_ok());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_deposit_to_locked_account() {
            let mut account = Account::new(1);
            account.available = Money::new("0");
            account.locked = true;

            let deposit_result = account.deposit(Money::new("42"));
            assert_eq!(deposit_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::new("0"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("0"));
            assert!(account.locked);
        }
    }

    mod withdrawals {
        use super::*;

        #[test]
        fn test_withdraw_available_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let withdrawal_result = account.withdraw(Money::new("2"));
            assert!(withdrawal_result.is_ok());

            assert_eq!(account.total_balance(), Money::new("40"));
            assert_eq!(account.available, Money::new("40"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_all_available_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let withdrawal_result = account.withdraw(Money::new("42"));
            assert!(withdrawal_result.is_ok());

            assert_eq!(account.total_balance(), Money::new("0"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_unavailable_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let withdrawal_result = account.withdraw(Money::new("50"));
            assert!(withdrawal_result.is_err());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_from_empty_account() {
            let mut account = Account::new(1);

            assert_eq!(account.total_balance(), Money::new("0"));

            let withdrawal_result = account.withdraw(Money::new("1"));
            assert!(withdrawal_result.is_err());

            assert_eq!(account.total_balance(), Money::new("0"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_withdraw_from_locked_account() {
            let mut account = Account::new(1);
            account.available = Money::new("42");
            account.locked = true;

            let withdrawal_result = account.withdraw(Money::new("1"));
            assert_eq!(withdrawal_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(account.locked);
        }
    }

    mod open_disputes {
        use super::*;

        #[test]
        fn test_open_dispute() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("20"));
            assert!(dispute_result.is_ok());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("22"));
            assert_eq!(account.held, Money::new("20"));
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_for_all_available_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("42"));
            assert!(dispute_result.is_ok());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("42"));
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_with_insufficient_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("50"));
            assert!(dispute_result.is_err());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_on_empty_account() {
            let mut account = Account::new(1);

            assert_eq!(account.total_balance(), Money::new("0"));

            let dispute_result = account.open_dispute(Money::new("42"));
            assert!(dispute_result.is_err());

            assert_eq!(account.total_balance(), Money::new("0"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_open_dispute_on_locked_account() {
            let mut account = Account::new(1);
            account.available = Money::new("42");
            account.locked = true;

            let dispute_result = account.open_dispute(Money::new("2"));
            assert_eq!(dispute_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(account.locked);
        }
    }

    mod resolve_disputes {
        use super::*;

        #[test]
        fn test_resolve_dispute() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("10"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("10"));

            let resolve_result = account.resolve_dispute(Money::new("2"));
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("34"));
            assert_eq!(account.held, Money::new("8"));
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_of_all_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("42"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("42"));

            let resolve_result = account.resolve_dispute(Money::new("42"));
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_with_insufficient_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("2"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("40"));
            assert_eq!(account.held, Money::new("2"));

            let resolve_result = account.resolve_dispute(Money::new("4"));
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("40"));
            assert_eq!(account.held, Money::new("2"));
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_with_no_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));

            let resolve_result = account.resolve_dispute(Money::new("2"));
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_resolve_dispute_on_locked_account() {
            let mut account = Account::new(1);
            account.available = Money::new("42");
            account.held = Money::new("2");
            account.locked = true;

            let resolve_result = account.resolve_dispute(Money::new("2"));
            assert_eq!(resolve_result, Err(AccountError::AccountFrozen));

            assert_eq!(account.total_balance(), Money::new("44"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("2"));
            assert!(account.locked);
        }
    }

    mod chargebacks {
        use super::*;

        #[test]
        fn test_chargeback() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("10"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("10"));

            let resolve_result = account.handle_chargeback(Money::new("2"));
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("40"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("8"));
            assert!(account.locked);
        }

        #[test]
        fn test_chargeback_of_all_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("42"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("42"));

            let resolve_result = account.handle_chargeback(Money::new("42"));
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("0"));
            assert_eq!(account.available, Money::new("0"));
            assert_eq!(account.held, Money::new("0"));
            assert!(account.locked);
        }

        #[test]
        fn test_chargeback_with_insufficient_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("2"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("40"));
            assert_eq!(account.held, Money::new("2"));
            assert!(!account.locked);

            let resolve_result = account.handle_chargeback(Money::new("4"));
            assert!(resolve_result.is_err());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("40"));
            assert_eq!(account.held, Money::new("2"));
            assert!(!account.locked);
        }

        #[test]
        fn test_chargeback_with_no_held_funds() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);

            let resolve_result = account.handle_chargeback(Money::new("2"));
            assert!(resolve_result.is_err());

            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("42"));
            assert_eq!(account.held, Money::new("0"));
            assert!(!account.locked);
        }

        #[test]
        fn test_chargeback_with_locked_account() {
            let mut account = Account::new(1);
            account.available = Money::new("42");

            let dispute_result = account.open_dispute(Money::new("10"));
            assert!(dispute_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("42"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("10"));

            let resolve_result = account.handle_chargeback(Money::new("2"));
            assert!(resolve_result.is_ok());
            assert_eq!(account.total_balance(), Money::new("40"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("8"));
            assert!(account.locked);

            let resolve_result = account.handle_chargeback(Money::new("8"));
            assert_eq!(resolve_result, Err(AccountError::AccountFrozen));
            assert_eq!(account.total_balance(), Money::new("40"));
            assert_eq!(account.available, Money::new("32"));
            assert_eq!(account.held, Money::new("8"));
            assert!(account.locked);
        }
    }
}
