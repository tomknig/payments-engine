use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientId(u16);

impl ClientId {
    pub fn new(id: u16) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

impl From<u16> for ClientId {
    fn from(id: u16) -> Self {
        Self(id)
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
