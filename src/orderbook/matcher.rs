use std::collections::BinaryHeap;
use crate::orderbook::database::ORDERBOOK_STATE;
use crate::orderbook::{Order, MyBigUint, OrderCommons};
use async_graphql::*;
use slab::Slab;
use std::fmt::Display;
use crate::orderbook::model::{Deal, deal};

pub(crate) struct Matcher {

}

#[derive(PartialEq, Hash, Eq, Clone, Copy, Enum, strum_macros::Display)]
pub(crate) enum OrderType {
    Buy,
    Sell,
}

struct CompareOrders {
    f: Box<dyn Fn(&MyBigUint, &MyBigUint) -> bool>,
}

impl CompareOrders {
    fn new<F>(f: F) -> CompareOrders
        where
            F: Fn(&MyBigUint, &MyBigUint) -> bool + 'static,
    {
        CompareOrders { f: Box::new(f) }
    }
}

struct DealPrice {
    f: Box<dyn Fn(&OrderCommons, &OrderCommons) -> MyBigUint>,
}

impl DealPrice {
    fn new<F>(f: F) -> DealPrice
        where
            F: Fn(&OrderCommons, &OrderCommons) -> MyBigUint + 'static,
    {
        DealPrice { f: Box::new(f) }
    }
}

impl Matcher {
    pub(crate) fn run(kind: OrderType, data: &OrderCommons) {
        let state = &mut ORDERBOOK_STATE.lock().unwrap().orderbook;
        let (retrieve_queue, add_queue, retrieve_map, add_map, comparison, deal_price) = match kind {
            OrderType::Buy => (&mut state.asks, &mut state.bids, &mut state.ask_map, &mut state.bid_map, CompareOrders::new(|p1: &MyBigUint, p2: &MyBigUint| p1.0 <= p2.0), DealPrice::new(|new_order: &OrderCommons, retrieved_order: &OrderCommons| retrieved_order.price.clone())),
            OrderType::Sell => (&mut state.bids, &mut state.asks, &mut state.bid_map, &mut state.ask_map, CompareOrders::new(|p1: &MyBigUint, p2: &MyBigUint| p1.0 >= p2.0), DealPrice::new(|new_order: &OrderCommons, retrieved_order: &OrderCommons| new_order.price.clone()))
        };
        // recursive
        fn _run(qty: usize, kind: OrderType, data: &OrderCommons, retrieve_queue: &mut BinaryHeap<Order>, add_queue: &mut BinaryHeap<Order>, retrieve_map: &mut Slab<OrderCommons>, add_map: &mut Slab<OrderCommons>, comparison: &CompareOrders, deal_price: &DealPrice) {
            fn make_order(d: &OrderCommons, kind: OrderType, add_queue: &mut BinaryHeap<Order>, add_map: &mut Slab<OrderCommons>) -> Order {
                let id = add_map.insert(d.clone());
                let order = Order { id, data: d.clone(), kind };
                add_queue.push(order.clone());
                return order.clone();
            };
            let peeked_order = retrieve_queue.peek();
            if peeked_order.is_some() && (comparison.f)(&peeked_order.unwrap().data.price, &data.price) {
                let retrieve_order = retrieve_queue.pop().unwrap(); // is_some already
                retrieve_map.remove(retrieve_order.id);
                let retrieved_qty = retrieve_order.data.quantity.clone();
                let diff = qty as i32 - retrieved_qty as i32;
                deal(Deal {
                    price: (deal_price.f)(&data, &retrieve_order.data),
                    quantity: retrieved_qty,
                });
                if diff == 0 {
                    // deal done
                } else if diff > 0 {
                    _run(diff as usize, kind, &data, retrieve_queue, add_queue, retrieve_map, add_map, &comparison, &deal_price);
                } else if diff < 0 {
                    // TODO just update order
                    let new_order = make_order(&OrderCommons {
                        price: retrieve_order.data.price.clone(),
                        quantity: diff.abs() as usize,
                    }, kind, add_queue, add_map);
                    //publishBidChange(new_order)
                }
            } else {

                let id = make_order(&data, kind, add_queue, add_map);
                // publish_order(id);
            }
        };
        _run(data.quantity, kind, &data, retrieve_queue, add_queue, retrieve_map, add_map, &comparison, &deal_price);
    }
}