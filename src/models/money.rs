use rust_decimal::{Decimal, dec};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;

const PRECISION: u32 = 4;

// We support a maximum of 24 integer and 4 fractional digits
const MAX: Decimal = dec!(999999999999999999999999.9999);
const MIN: Decimal = dec!(-999999999999999999999999.9999);

#[derive(Error, Debug, PartialEq)]
pub enum MoneyError {
    #[error("unable to parse money: {0}")]
    ParseError(String),
    #[error("adding {0} to {1} is out of representable bounds")]
    AdditionOutOfBounds(Money, Money),
    #[error("subtracting {0} from {1} is out of representable bounds")]
    SubtractionOutOfBounds(Money, Money),
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Money(Decimal);

impl Money {
    pub fn new() -> Self {
        Money::default()
    }

    pub fn parse(s: &str) -> Result<Self, MoneyError> {
        let mut value =
            Decimal::from_str(s.trim()).map_err(|e| MoneyError::ParseError(e.to_string()))?;

        if value.scale() > PRECISION {
            return Err(MoneyError::ParseError(
                "too many fractional digits".to_string(),
            ));
        }

        value.rescale(PRECISION);

        if value > MAX || value < MIN {
            return Err(MoneyError::ParseError(
                "value out of representable bounds".to_string(),
            ));
        }

        Ok(Money(value))
    }

    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative()
    }

    pub fn checked_add(self, other: Money) -> Result<Self, MoneyError> {
        let result = match self.0.checked_add(other.0) {
            None => Err(MoneyError::AdditionOutOfBounds(other, self)),
            Some(result) => Ok(Money(result)),
        }?;

        if result.0 > MAX || result.0 < MIN {
            return Err(MoneyError::AdditionOutOfBounds(other, self));
        }

        Ok(result)
    }

    pub fn checked_sub(self, other: Money) -> Result<Self, MoneyError> {
        let result = match self.0.checked_sub(other.0) {
            None => Err(MoneyError::SubtractionOutOfBounds(other, self)),
            Some(result) => Ok(Money(result)),
        }?;

        if result.0 > MAX || result.0 < MIN {
            return Err(MoneyError::SubtractionOutOfBounds(other, self));
        }

        Ok(result)
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

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.4}", self.0)
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
            let money = Money::parse("1").unwrap();
            assert_eq!(money.to_string(), "1.0000");
        }

        #[test]
        fn test_parse_valid_decimal() {
            let money = Money::parse("1.0").unwrap();
            assert_eq!(money.to_string(), "1.0000");
        }

        #[test]
        fn test_parse_negative_number() {
            let money = Money::parse("-1.0").unwrap();
            assert_eq!(money.to_string(), "-1.0000");
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

        #[test]
        fn test_largest_number_parses_successfully() {
            let max_representable = MAX.to_string();
            let money = Money::parse(&max_representable).unwrap();
            assert_eq!(money.to_string(), "999999999999999999999999.9999");
        }

        #[test]
        fn test_smallest_number_parses_successfully() {
            let min_representable = MIN.to_string();
            let money = Money::parse(&min_representable).unwrap();
            assert_eq!(money.to_string(), "-999999999999999999999999.9999");
        }

        #[test]
        fn test_reject_number_above_maximum() {
            let max_representable = MAX.to_string();
            let larger_than_representable = format!("9{}", max_representable);
            let money = Money::parse(&larger_than_representable);
            assert!(money.is_err());
        }

        #[test]
        fn test_reject_number_below_minimum() {
            let max_representable = MAX.to_string();
            let smaller_than_representable = format!("-9{}", max_representable);
            let money = Money::parse(&smaller_than_representable);
            assert!(money.is_err());
        }
    }

    mod arithmetic {
        use super::*;

        #[test]
        fn test_checked_addition() {
            let money = Money::parse("1.25").unwrap();
            let more_money = Money::parse("0.5").unwrap();
            let sum = money.checked_add(more_money).unwrap();
            assert_eq!(sum.to_string(), "1.7500");
        }

        #[test]
        fn test_checked_subtraction_can_be_negative() {
            let money = Money::parse("1").unwrap();
            let more_money = Money::parse("2").unwrap();
            let diff = money.checked_sub(more_money).unwrap();
            assert_eq!(diff.to_string(), "-1.0000");
        }

        #[test]
        fn test_checked_add_fails_when_out_of_bounds() {
            let max_representable = MAX.to_string();
            let money = Money::parse(&max_representable).unwrap();
            let more_money = Money::parse(&max_representable).unwrap();
            let result = money.checked_add(more_money);

            let error = result.as_ref().expect_err("expected addition to fail");

            assert!(error.to_string().starts_with("adding 999999999999999999999999.9999 to 999999999999999999999999.9999 is out of representable bounds"));
        }

        #[test]
        fn test_checked_sub_fails_when_out_of_bounds() {
            let min_representable = MIN.to_string();
            let max_representable = MAX.to_string();
            let money = Money::parse(&min_representable).unwrap();
            let more_money = Money::parse(&max_representable).unwrap();
            let result = money.checked_sub(more_money);

            let error = result.as_ref().expect_err("expected subtraction to fail");

            assert!(error.to_string().starts_with("subtracting 999999999999999999999999.9999 from -999999999999999999999999.9999 is out of representable bounds"));
        }
    }
}
