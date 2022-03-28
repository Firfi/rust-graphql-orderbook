mod model;
mod reporter;
mod database;
mod matcher;

use async_graphql::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::cmp;
use std::iter;
use std::fmt;

use tokio::time;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use slab::Slab;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::iter::Rev;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use num_bigint::{BigUint, ParseBigIntError};

pub use model::QueryRoot;
use crate::orderbook::database::ORDERBOOK_STATE;
use crate::orderbook::matcher::{Matcher, OrderType};
use crate::orderbook::reporter::{OrderScaffold, Reporter};

pub type OrderBookSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub(crate) struct MyBigUint(BigUint);

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

#[derive(Hash, Clone, Eq, PartialEq, Debug, SimpleObject)]
pub struct OrderCommons {
    quantity: usize,
    price: MyBigUint
}

impl fmt::Display for OrderCommons {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "OrderCommons {{ quantity: {}, price: {} }}", self.quantity, self.price)
    }
}

#[derive(Hash, Clone, SimpleObject)]
pub struct Order {
    id: usize, // we may want to stricten it to newtype
    data: OrderCommons,
    kind: OrderType,
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.kind == other.kind
    }
}

impl Eq for Order {}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.kind != other.kind { // incomparable
            panic!("Incomparable orders (Buy and Sell)");
        }
        return if self.kind == OrderType::Sell {
            other.data.price.0.cmp(&self.data.price.0)
        } else {
            self.data.price.0.cmp(&other.data.price.0)
        }

    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct OrderBook {
    bids: BinaryHeap<Order>,
    asks: BinaryHeap<Order>,
    bid_map: Slab<OrderCommons>,
    ask_map: Slab<OrderCommons>,
}

const DEFAULT_LIMIT: usize = 100;

fn sorted_slice<T: Ord + std::clone::Clone>(h: &BinaryHeap<T>, limit: Option<usize>) -> Vec<T> {
    let mut v = h.clone().into_sorted_vec();
    v.reverse();
    let limit_ = cmp::min(limit.unwrap_or(DEFAULT_LIMIT), v.len());
    v[0..limit_].to_vec()
}

#[Object]
impl OrderBook {
    async fn bids_total(&self) -> usize {
        self.bids.len()
    }
    async fn bids(&self, limit: Option<usize>) -> Vec<Order> {
        sorted_slice(&self.bids, limit)
    }
    async fn asks_total(&self) -> usize {
        self.asks.len()
    }
    async fn asks(&self, limit: Option<usize>) -> Vec<Order> {
        sorted_slice(&self.asks, limit)
    }
}

pub async fn run_reporter_poll() {
    let mut interval_sec = time::interval(Duration::from_secs(1));
    let mut reporter = Reporter::new();
    loop {
        interval_sec.tick().await;
        let scaffolds = reporter.step();
        scaffolds.bids.iter().map(|x| (x, OrderType::Buy)).chain(scaffolds.asks.iter().map(|x| (x, OrderType::Sell))).for_each(move |(x, order_type)| {
            Matcher::run(order_type, &OrderCommons {
                quantity: x.quantity,
                price: x.price.clone(),
            })
        });
    }
}

