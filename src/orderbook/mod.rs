mod model;
mod reporter;
mod database;
mod matcher;
mod types;
mod simple_broker;

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
use model::{OrderCommons, OrderType};
use types::*;

// pub use model::QueryRoot;
use crate::orderbook::database::ORDERBOOK_STATE;
use crate::orderbook::matcher::Matcher;
pub(crate) use crate::orderbook::model::{QueryRoot, SubscriptionRoot};
use crate::orderbook::reporter::{OrderScaffold, Reporter};

pub(crate) type OrderBookSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

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

