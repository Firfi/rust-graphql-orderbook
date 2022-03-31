use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;
use std::str::FromStr;
use async_graphql::{InputValueError, InputValueResult, ScalarType, Value};
use async_graphql::*;
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) struct MyUuid(pub(crate) Uuid);

impl fmt::Display for MyUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for MyUuid {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MyUuid(Uuid::from_str(s)?))
    }
}

#[Scalar]
impl ScalarType for MyUuid {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            Ok(MyUuid::from_str(value)?)
        } else {
            // If the type does not match
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}