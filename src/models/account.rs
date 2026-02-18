use super::money::Money;
use super::client::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Account {
    client_id: ClientId,
    available: Money,
    held: Money,
    total: Money,
    locked: bool,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: Money::new(0),
            held: Money::new(0),
            total: Money::new(0),
            locked: false,
        }
    }
}
