use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;
use std::str::FromStr;
use async_graphql::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql::*;
use chrono::{DateTime, FixedOffset, ParseError, TimeZone};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) struct MyDateTime<T: TimeZone>(pub(crate) DateTime<T>);

impl <T: TimeZone> fmt::Display for MyDateTime<T> where T::Offset: fmt::Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_rfc3339())
    }
}

impl FromStr for MyDateTime<FixedOffset> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MyDateTime(DateTime::parse_from_rfc3339(s)?))
    }
}

#[Scalar]
impl ScalarType for MyDateTime<FixedOffset> {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(MyDateTime::from_str(value)?)
        } else {
            // If the type does not match
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}