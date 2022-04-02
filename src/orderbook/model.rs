use super::types::big_uint::MyBigUint;
use std::borrow::Cow;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Duration;
use async_graphql::{connection::{Connection, Edge, EmptyFields, query}, Context, ContextSelectionSet, Enum, FieldResult, Interface, Object, Positioned, ServerResult};
use async_graphql::parser::types::Field;
use async_graphql::registry::Registry;
use async_graphql::*;
use async_graphql::futures_util::StreamExt;
use chrono::{DateTime, FixedOffset, Utc};
use futures_core::Stream;
use tokio_stream::wrappers::IntervalStream;
use uuid::Uuid;
use std::fmt;
use std::fmt::Formatter;
use std::cmp::Ordering;
use slab::Slab;
use crate::orderbook::database::{HISTORY_CAPACITY, HISTORY_STATE, ORDERBOOK_STATE};
use crate::orderbook::simple_broker::SimpleBroker;
use crate::orderbook::types::date_time::MyDateTime;
use crate::orderbook::types::slice_display::sorted_slice;
use crate::orderbook::types::uuid::MyUuid;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderKind {
    Buy,
    Sell,
}

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    pub(crate) async fn orderbook(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<OrderBook> {
        Ok(ORDERBOOK_STATE.lock().unwrap().orderbook.clone())
    }
    pub(crate) async fn history(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<VecDeque<Deal>> {
        Ok(HISTORY_STATE.lock().unwrap().deals_history.clone())
    }

}

#[derive(Clone, Debug, SimpleObject)]
pub(crate) struct Deal {
    pub(crate) price: MyBigUint,
    pub(crate) quantity: usize,
    pub(crate) id: MyUuid,
    pub(crate) created_at: MyDateTime<FixedOffset>,
    pub(crate) kind: OrderType,
}

impl Deal {
    pub(crate) fn new(price: MyBigUint, quantity: usize, kind: OrderType) -> Self {
        Self {
            price,
            quantity,
            id: MyUuid(Uuid::new_v4()),
            created_at: MyDateTime(Utc::now().with_timezone(&FixedOffset::east(0))),
            kind,
        }
    }
}

pub(crate) fn deal(d: Deal) {
    let dh = &mut HISTORY_STATE.lock().unwrap().deals_history;
    // keep max size
    if dh.len() >= HISTORY_CAPACITY {
        (0..(dh.len() - HISTORY_CAPACITY)).for_each(|_| {
            dh.pop_back();
        });
    }
    dh.push_front(d.clone());
    SimpleBroker::publish(d.clone());
}



pub(crate) struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn deals(&self) -> impl Stream<Item = Deal> {
        SimpleBroker::<Deal>::subscribe()
    }
    async fn new_orders(&self) -> impl Stream<Item = OrderAdded> {
        SimpleBroker::<OrderAdded>::subscribe()
    }
    async fn removed_orders(&self) -> impl Stream<Item = OrderRemoved> {
        SimpleBroker::<OrderRemoved>::subscribe()
    }
}

#[derive(Hash, Clone, SimpleObject)]
pub(crate) struct OrderAdded {
    pub(crate) order: Order
}

#[derive(Hash, Clone, SimpleObject)]
pub(crate) struct OrderRemoved {
    pub(crate) order: Order
}

pub(crate) fn publish_order_add(order: &Order) {
    SimpleBroker::publish(OrderAdded { order: order.clone() });
}

pub(crate) fn publish_order_remove(order: &Order) {
    SimpleBroker::publish(OrderRemoved { order: order.clone() });
}

#[derive(Hash, Clone, Eq, PartialEq, Debug, SimpleObject)]
pub(crate) struct OrderCommons {
    pub(crate) quantity: usize,
    pub(crate) price: MyBigUint
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
pub(crate) struct Order {
    pub(crate) id: usize, // we may want to stricten it to newtype
    pub(crate) data: OrderCommons,
    pub(crate) kind: OrderType,
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
pub(crate) struct OrderBook {
    pub(crate) bids: BinaryHeap<Order>,
    pub(crate) asks: BinaryHeap<Order>,
    pub(crate) bid_map: Slab<OrderCommons>,
    pub(crate) ask_map: Slab<OrderCommons>,
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

#[derive(PartialEq, Hash, Eq, Clone, Copy, Debug, Enum, strum_macros::Display)]
pub(crate) enum OrderType {
    Buy,
    Sell,
}
