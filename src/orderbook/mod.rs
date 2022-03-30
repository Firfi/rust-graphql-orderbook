mod model;
mod reporter;
mod database;
mod matcher;
mod big_uint;
mod simple_broker;
mod uuid;
mod date_time;

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
use std::fmt::{Display, Formatter};
use std::iter::Rev;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use num_bigint::{BigUint, ParseBigIntError};

// pub use model::QueryRoot;
use crate::orderbook::database::ORDERBOOK_STATE;
use crate::orderbook::matcher::{Matcher, OrderType};
pub(crate) use crate::orderbook::model::{SubscriptionRoot, QueryRoot};
use crate::orderbook::reporter::{OrderScaffold, Reporter};
use crate::orderbook::big_uint::{MyBigUint};

pub(crate) type OrderBookSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;



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

impl fmt::Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Order {{ kind: {}, id: {}, data: {} }}", self.kind, self.id, self.data)
    }
}

#[derive(Hash, Clone, Debug, SimpleObject)]
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

struct SliceDisplay<'a, T: 'a>(&'a [T]);

impl<'a, T: fmt::Display + 'a> fmt::Display for SliceDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for item in self.0 {
            if !first {
                write!(f, ", {}", item)?;
            } else {
                write!(f, "{}", item)?;
            }
            first = false;
        }
        Ok(())
    }
}

fn sorted_slice<T: Ord + std::clone::Clone + fmt::Display>(h: &BinaryHeap<T>, limit: Option<usize>) -> Vec<T> {
    let mut v = h.clone().into_sorted_vec();
    v.reverse();
    v[0..cmp::min(limit.unwrap_or(DEFAULT_LIMIT), v.len())].to_vec()
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

