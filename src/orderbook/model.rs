use std::borrow::Cow;
use std::collections::VecDeque;
use std::time::Duration;
use async_graphql::{connection::{query, Connection, Edge, EmptyFields}, Context, ContextSelectionSet, Enum, FieldResult, Interface, Object, Positioned, ServerResult};
use async_graphql::parser::types::Field;
use async_graphql::registry::Registry;
use crate::orderbook::{MyBigUint, Order, OrderBook};
use async_graphql::*;
use async_graphql::futures_util::StreamExt;
use chrono::{DateTime, FixedOffset, Utc};
use futures_core::Stream;
use tokio_stream::wrappers::IntervalStream;
use uuid::Uuid;
use crate::orderbook::database::{HISTORY_CAPACITY, HISTORY_STATE, ORDERBOOK_STATE};
use crate::orderbook::date_time::MyDateTime;
use crate::orderbook::simple_broker::SimpleBroker;
use crate::orderbook::uuid::MyUuid;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderKind {
    Buy,
    Sell,
}

pub struct QueryRoot;

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
}

impl Deal {
    pub(crate) fn new(price: MyBigUint, quantity: usize) -> Self {
        Self {
            price,
            quantity,
            id: MyUuid(Uuid::new_v4()),
            created_at: MyDateTime(Utc::now().with_timezone(&FixedOffset::east(0))),
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