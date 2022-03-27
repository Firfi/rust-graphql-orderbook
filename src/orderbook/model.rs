use std::borrow::Cow;
use async_graphql::{connection::{query, Connection, Edge, EmptyFields}, Context, ContextSelectionSet, Enum, FieldResult, Interface, Object, OutputType, Positioned, ServerResult};
use async_graphql::parser::types::Field;
use async_graphql::registry::Registry;
use crate::orderbook::{BuyOrder, OrderBook, SellOrder};
use async_graphql::*;
use crate::orderbook::database::STATE;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum OrderKind {
    Buy,
    Sell,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn orderbook(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<OrderBook> {
        Ok(STATE.lock().unwrap().orderbook.clone())
    }
}
