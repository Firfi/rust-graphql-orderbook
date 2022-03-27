mod model;
mod reporter;
mod database;

use async_graphql::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::cmp;

use tokio::time;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use slab::Slab;
use std::collections::HashMap;
use std::iter::Rev;
use std::str::FromStr;
use std::time::Duration;
use num_bigint::{BigUint, ParseBigIntError};

pub use model::QueryRoot;
use crate::orderbook::database::STATE;
use crate::orderbook::reporter::{OrderScaffold, Reporter};

pub type OrderBookSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct MyBigUint(BigUint);

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

#[derive(Hash, Clone, SimpleObject)]
pub struct BuyOrder {
    id: usize, // we may want to stricten it to newtype
    data: OrderCommons,
}

impl PartialEq for BuyOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BuyOrder {}

impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        other.data.price.0.cmp(&self.data.price.0)
    }
}

impl PartialOrd for BuyOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Hash, Clone, SimpleObject)]
pub struct SellOrder {
    id: usize, // we may want to stricten it to newtype
    data: OrderCommons,
}

impl PartialEq for SellOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SellOrder {}

impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.price.0.cmp(&other.data.price.0)
    }
}

impl PartialOrd for SellOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
pub struct OrderBook {
    bids: BinaryHeap<BuyOrder>,
    asks: BinaryHeap<SellOrder>,
    bid_map: Slab<OrderCommons>,
    ask_map: Slab<OrderCommons>,
}

const DEFAULT_LIMIT: usize = 100;

fn sorted_slice<T: Ord + std::clone::Clone>(h: &BinaryHeap<T>, limit: Option<usize>) -> Vec<T> {
    let v = h.clone().into_sorted_vec();
    let limit_ = cmp::min(limit.unwrap_or(DEFAULT_LIMIT), v.len());
    v[0..limit_].to_vec()
}

#[Object]
impl OrderBook {
    async fn bids(&self, limit: Option<usize>) -> Vec<BuyOrder> {
        sorted_slice(&self.bids, limit)
    }
    async fn asks(&self, limit: Option<usize>) -> Vec<SellOrder> {
        sorted_slice(&self.asks, limit)
    }
}

pub async fn run_reporter_poll() {
    let mut interval_sec = time::interval(Duration::from_secs(1));
    let mut reporter = Reporter::new();
    loop {
        interval_sec.tick().await;
        let scaffolds = reporter.step();
        let ob = &mut STATE.lock().unwrap().orderbook;
        let asks = &mut ob.asks;
        let bids = &mut ob.bids;
        let ask_map = &mut ob.ask_map;
        let bid_map = &mut ob.bid_map;
        // todo make it generic
        scaffolds.bids.iter().for_each(move |x| {
            let commons = OrderCommons {
                quantity: x.quantity.clone(),
                price: MyBigUint(x.price.clone())
            };
            let id = bid_map.insert(commons.clone());
            bids.push(BuyOrder {
                id,
                data: commons.clone(),
            });
        });
        // todo make it generic
        scaffolds.asks.iter().for_each(move |x| {
            let commons = OrderCommons {
                quantity: x.quantity.clone(),
                price: MyBigUint(x.price.clone())
            };
            let id = ask_map.insert(commons.clone());
            asks.push(SellOrder {
                id,
                data: commons.clone(),
            });
        });
    }
}

