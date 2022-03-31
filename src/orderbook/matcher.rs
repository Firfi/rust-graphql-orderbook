use std::collections::BinaryHeap;
use crate::orderbook::database::ORDERBOOK_STATE;
use async_graphql::*;
use slab::Slab;
use std::fmt::Display;
use uuid::Uuid;
use crate::orderbook::model::{Deal, deal, Order, OrderCommons, OrderType, publish_order_add, publish_order_remove};
use crate::orderbook::types::big_uint::MyBigUint;

pub(crate) struct Matcher {

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
            fn create_order(d: &OrderCommons, kind: OrderType, add_queue: &mut BinaryHeap<Order>, add_map: &mut Slab<OrderCommons>) -> Order {
                let id = add_map.insert(d.clone());
                let order = Order { id, data: d.clone(), kind };
                add_queue.push(order.clone());
                publish_order_add(&order);
                return order.clone();
            };
            let peeked_order = retrieve_queue.peek();
            if peeked_order.is_some() && (comparison.f)(&peeked_order.unwrap().data.price, &data.price) {
                let retrieved_order = retrieve_queue.pop().unwrap(); // is_some already
                retrieve_map.remove(retrieved_order.id);
                publish_order_remove(&retrieved_order);
                let retrieved_qty = retrieved_order.data.quantity.clone();
                let diff = qty as i32 - retrieved_qty as i32;
                deal(Deal::new((deal_price.f)(&data, &retrieved_order.data), retrieved_qty));
                if diff == 0 {
                    // deal done
                } else if diff > 0 {
                    _run(diff as usize, kind, &data, retrieve_queue, add_queue, retrieve_map, add_map, &comparison, &deal_price);
                } else if diff < 0 {
                    // TODO just update the order
                    create_order(&OrderCommons {
                        price: retrieved_order.data.price.clone(),
                        quantity: diff.abs() as usize,
                    }, kind, add_queue, add_map);
                }
            } else {

                let id = create_order(&data, kind, add_queue, add_map);
                // publish_order(id);
            }
        };
        _run(data.quantity, kind, &data, retrieve_queue, add_queue, retrieve_map, add_map, &comparison, &deal_price);
    }
}