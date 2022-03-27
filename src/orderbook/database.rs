use std::collections::BinaryHeap;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use slab::Slab;
use crate::orderbook::OrderBook;

const CAPACITY: usize = 10_000;
pub struct OrderBookData {
    pub orderbook: OrderBook,
}

impl OrderBookData {
    pub fn new() -> Self {
        OrderBookData {
            orderbook: OrderBook {
                bids: BinaryHeap::with_capacity(CAPACITY),
                asks: BinaryHeap::with_capacity(CAPACITY),
                bid_map: Slab::with_capacity(CAPACITY),
                ask_map: Slab::with_capacity(CAPACITY),
            },
        }
    }
}

pub static STATE: Lazy<Mutex<OrderBookData>> = Lazy::new(|| Mutex::new(OrderBookData::new()));