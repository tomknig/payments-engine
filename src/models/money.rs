use serde::{Deserialize, Serialize};
use std::{fmt, ops};

const PRECISION: usize = 4;

#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    pub value: u64,
}

impl Money {
    pub fn new(value: u64) -> Self {
        Money { value }
    }
}

impl From<&str> for Money {
    fn from(value: &str) -> Self {
        let trimmed_value = value.trim();

        let (integer_slice, fraction_slice) = match trimmed_value.split_once('.') {
            Some((i, f)) => (i, f),
            None => (trimmed_value, ""),
        };

        let integer_part = integer_slice.parse::<u64>().unwrap_or(0) * 10u64.pow(PRECISION as u32);
        let mut fraction_part = 0u64;

        for (i, &b) in fraction_slice.as_bytes().iter().enumerate() {
            if PRECISION - i == 0 {
                break;
            }
            fraction_part += (b - b'0') as u64 * 10u64.pow((PRECISION - i - 1) as u32);
        }

        Money {
            value: integer_part + fraction_part,
        }
    }
}

impl ops::Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Self::Output {
        Money {
            value: self.value + other.value,
        }
    }
}

impl ops::Sub for Money {
    type Output = Money;

    fn sub(self, other: Money) -> Self::Output {
        Money {
            value: self.value - other.value,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let integer_part = self.value / 10u64.pow(PRECISION as u32);
        let fraction_part = self.value % 10u64.pow(PRECISION as u32);

        write!(f, "{}.{:0PRECISION$}", integer_part, fraction_part)
    }
}

impl fmt::Debug for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let money = Money::from("123");
        assert_eq!(money.value, 123_0000);
    }

    #[test]
    fn test_parse_fraction() {
        let money = Money::from("123.4567");
        assert_eq!(money.value, 123_4567);
    }

    #[test]
    fn test_parse_fraction_with_leading_zeros() {
        let money = Money::from("123.0007");
        assert_eq!(money.value, 123_0007);
    }

    #[test]
    fn test_parse_fraction_with_trailing_zeros() {
        let money = Money::from("123.4560");
        assert_eq!(money.value, 123_4560);
    }

    #[test]
    fn test_parse_fraction_with_leading_and_trailing_zeros() {
        let money = Money::from("123.0560");
        assert_eq!(money.value, 123_0560);
    }

    #[test]
    fn test_display_zero_fraction() {
        let money = Money { value: 123_0000 };
        assert_eq!(money.to_string(), "123.0000");
    }

    #[test]
    fn test_display_one_decimal_place() {
        let money = Money { value: 123_4000 };
        assert_eq!(money.to_string(), "123.4000");
    }

    #[test]
    fn test_display_leading_zeros_in_fraction() {
        let money = Money { value: 123_0007 };
        assert_eq!(money.to_string(), "123.0007");
    }

    #[test]
    fn test_display_full_precision() {
        let money = Money { value: 123_4567 };
        assert_eq!(money.to_string(), "123.4567");
    }
}
