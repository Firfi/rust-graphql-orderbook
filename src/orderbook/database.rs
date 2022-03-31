use std::collections::{BinaryHeap, VecDeque};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use slab::Slab;
use crate::orderbook::model::Deal;
use crate::orderbook::model::OrderBook;

pub const ORDERBOOK_CAPACITY: usize = 50;
pub const HISTORY_CAPACITY: usize = 10_000;
pub struct OrderBookData {
    pub(crate) orderbook: OrderBook,
}

impl OrderBookData {
    pub fn new() -> Self {
        OrderBookData {
            orderbook: OrderBook {
                bids: BinaryHeap::with_capacity(ORDERBOOK_CAPACITY),
                asks: BinaryHeap::with_capacity(ORDERBOOK_CAPACITY),
                bid_map: Slab::with_capacity(ORDERBOOK_CAPACITY),
                ask_map: Slab::with_capacity(ORDERBOOK_CAPACITY),
            },
        }
    }
}

pub struct HistoryData {
    pub(crate) deals_history: VecDeque<Deal>,
}

impl HistoryData {
    pub fn new() -> Self {
        HistoryData {
            deals_history: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }
}

pub static ORDERBOOK_STATE: Lazy<Mutex<OrderBookData>> = Lazy::new(|| Mutex::new(OrderBookData::new()));
pub static HISTORY_STATE: Lazy<Mutex<HistoryData>> = Lazy::new(|| Mutex::new(HistoryData::new()));