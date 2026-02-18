use crate::csv::Account as CsvAccount;
use payments_engine::models::Account;

impl From<&Account> for CsvAccount {
    fn from(account: &Account) -> Self {
        Self {
            client_id: account.client_id(),
            available: account.available_balance().to_string(),
            held: account.held_balance().to_string(),
            total: account.total_balance().to_string(),
            locked: account.is_locked(),
        }
    }
}
