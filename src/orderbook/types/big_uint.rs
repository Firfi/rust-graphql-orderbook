
use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;
use std::str::FromStr;
use async_graphql::{InputValueError, InputValueResult, ScalarType, Value};
use num_bigint::{BigUint, ParseBigIntError};
use async_graphql::*;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) struct MyBigUint(pub(crate) BigUint);

impl fmt::Display for MyBigUint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Add for MyBigUint {
    type Output = MyBigUint;

    fn add(self, rhs: Self) -> Self::Output {
        MyBigUint(self.0.add(rhs.0))
    }
}

impl FromStr for MyBigUint {
    type Err = ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MyBigUint(BigUint::from_str(s)?))
    }
}

#[Scalar]
impl ScalarType for MyBigUint {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(MyBigUint::from_str(value)?)
        } else {
            // If the type does not match
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_str_radix(10))
    }
}