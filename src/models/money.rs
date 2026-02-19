use serde::{Deserialize, Serialize};
use std::{fmt, ops};
use thiserror::Error;

const PRECISION: usize = 4;

#[derive(Error, Debug, PartialEq)]
pub enum MoneyError {
    #[error("unable to parse money: {0}")]
    ParseError(String),
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    pub value: u64,
}

impl Money {
    pub fn new() -> Self {
        Money::default()
    }

    pub fn parse_unchecked(s: &str) -> Self {
        Self::parse(s).expect("valid money literal")
    }

    pub fn parse(s: &str) -> Result<Self, MoneyError> {
        let trimmed_value = s.trim();
        let mut parts = trimmed_value.split('.');
        let integer_slice = parts.next().unwrap_or("");
        let fraction_slice = parts.next().unwrap_or("");

        if parts.next().is_some() {
            return Err(MoneyError::ParseError(
                "multiple dots are not allowed".to_string(),
            ));
        }

        if fraction_slice.len() > PRECISION {
            return Err(MoneyError::ParseError(
                "Too many fractional digits".to_string(),
            ));
        }

        let integer_part = integer_slice
            .parse::<u64>()
            .map_err(|e| MoneyError::ParseError(e.to_string()))?;
        let integer_part = integer_part * 10u64.pow(PRECISION as u32);
        let mut fraction_part = 0u64;

        for (i, &b) in fraction_slice.as_bytes().iter().enumerate() {
            if PRECISION - i == 0 {
                break;
            }

            if !b.is_ascii_digit() {
                return Err(MoneyError::ParseError(format!(
                    "Invalid digit at position {}",
                    i
                )));
            }

            let literal = (b - b'0') as u64;
            fraction_part += literal * 10u64.pow((PRECISION - i - 1) as u32);
        }

        Ok(Money {
            value: integer_part + fraction_part,
        })
    }
}

impl TryFrom<&str> for Money {
    type Error = MoneyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Money::parse(value)
    }
}

impl TryFrom<String> for Money {
    type Error = MoneyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Money::parse(value.as_str())
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

    mod parse {
        use super::*;

        #[test]
        fn test_parse_integer() {
            let money = Money::parse("1");
            assert!(money.is_ok());
            assert_eq!(money.unwrap().value, 1_0000);
        }

        #[test]
        fn test_parse_valid_decimal() {
            let money = Money::parse("1.0");
            assert!(money.is_ok());
            assert_eq!(money.unwrap().value, 1_0000);
        }

        #[test]
        fn test_reject_too_many_decimal_places() {
            let money = Money::parse("1.23456");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_char_in_fraction() {
            let money = Money::parse("1.0a0");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_negative_number() {
            let money = Money::parse("-1.0");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_negative_number_with_char_in_fraction() {
            let money = Money::parse("-1.0a0");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_char_in_integer() {
            let money = Money::parse("a.0a0");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_multiple_dots() {
            let money = Money::parse("1.0.");
            assert!(money.is_err());
        }

        #[test]
        fn test_parse_special_characters_in_fraction() {
            let money = Money::parse("1./");
            assert!(money.is_err());
        }
    }

    mod parse_unchecked {
        use super::*;

        #[test]
        fn test_parse_integer() {
            let money = Money::parse_unchecked("123");
            assert_eq!(money.value, 123_0000);
        }

        #[test]
        fn test_parse_one_fractional_digit() {
            let money = Money::parse_unchecked("123.4");
            assert_eq!(money.value, 123_4000);
        }

        #[test]
        fn test_parse_two_fractional_digits() {
            let money = Money::parse_unchecked("123.45");
            assert_eq!(money.value, 123_4500);
        }

        #[test]
        fn test_parse_three_fractional_digits() {
            let money = Money::parse_unchecked("123.456");
            assert_eq!(money.value, 123_4560);
        }

        #[test]
        fn test_parse_four_fractional_digits() {
            let money = Money::parse_unchecked("123.4567");
            assert_eq!(money.value, 123_4567);
        }

        #[test]
        fn test_parse_fraction_with_leading_zeros() {
            let money = Money::parse_unchecked("123.0007");
            assert_eq!(money.value, 123_0007);
        }

        #[test]
        fn test_parse_fraction_with_trailing_zeros() {
            let money = Money::parse_unchecked("123.4560");
            assert_eq!(money.value, 123_4560);
        }

        #[test]
        fn test_parse_fraction_with_leading_and_trailing_zeros() {
            let money = Money::parse_unchecked("123.0560");
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
}
