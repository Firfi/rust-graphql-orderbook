use std::borrow::Cow;
use std::collections::VecDeque;
use async_graphql::{connection::{query, Connection, Edge, EmptyFields}, Context, ContextSelectionSet, Enum, FieldResult, Interface, Object, OutputType, Positioned, ServerResult};
use async_graphql::parser::types::Field;
use async_graphql::registry::Registry;
use crate::orderbook::{MyBigUint, Order, OrderBook};
use async_graphql::*;
use crate::orderbook::database::{HISTORY_CAPACITY, HISTORY_STATE, ORDERBOOK_STATE};

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
}

pub(crate) fn deal(d: Deal) {
    let dh = &mut HISTORY_STATE.lock().unwrap().deals_history;
    // keep max size
    if dh.len() >= HISTORY_CAPACITY {
        (0..(dh.len() - HISTORY_CAPACITY)).for_each(|_| {
            dh.pop_back();
        });
    }
    dh.push_front(d);
}
