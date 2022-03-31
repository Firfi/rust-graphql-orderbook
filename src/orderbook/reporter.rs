use std::cmp::max;
use num_bigint::BigUint;
use rand::prelude::ThreadRng;
use rand::{Rng, RngCore};
use rand::seq::SliceRandom;
use tokio::time::Interval;
use num_traits::cast::ToPrimitive;
use crate::orderbook::database::ORDERBOOK_STATE;
use crate::orderbook::model::Order;
use crate::orderbook::types::big_uint::MyBigUint;

const MARGIN: usize = 6;
const BIDDER_CROWD: usize = 10;

pub(crate) struct Reporter {
    rng: ThreadRng,
    n: u64,
}

fn price_law(k: u64) -> BigUint {
    BigUint::from(((((k as f64) * 0.1).sin() + (2 as f64)) * (100 as f64)).floor() as u64)
} // no 0 price;


#[derive(Hash, Clone)]
pub(crate) struct OrderScaffold {
    pub(crate) price: MyBigUint,
    pub(crate) quantity: usize,
}

pub(crate) struct ReportedScaffolds {
    pub(crate) bids: Vec<OrderScaffold>,
    pub(crate) asks: Vec<OrderScaffold>,
}

impl Reporter {
    pub fn new() -> Self {
        Reporter {
            rng: rand::thread_rng(),
            n: 0,
        }
    }
    fn price_fluctuation(&mut self) -> usize {
        self.rng.gen_range(0..1) * MARGIN + 1 // no 0 price
    }
    pub(crate) fn step(&mut self) -> ReportedScaffolds {
        let n = self.n;

        let mut scaffolds = vec![0, (self.rng.gen_range(0..1) * BIDDER_CROWD)]
            .into_iter()
            .map(|i| OrderScaffold {
                price: MyBigUint(price_law(n + i as u64) + BigUint::from(self.price_fluctuation())),
                quantity: self.rng.gen_range(1..100),
            })
            .collect::<Vec<OrderScaffold>>();
        // scaffolds.shuffle(&mut self.rng);
        // prices.sort(); // in case we add fluctuation
        let middle = (scaffolds.len() / 2);

        let (bids_, asks_) = scaffolds.split_at(middle);
        let state = &mut ORDERBOOK_STATE.lock().unwrap().orderbook;
        let diff = state.bids.len() as i32 - state.asks.len() as i32;
        let bids_with_diff_bias = bids_.iter().map(|s| OrderScaffold {
            price: MyBigUint(BigUint::from(max(1, s.price.0.to_i32().unwrap() - diff.to_i32().unwrap()) as u64)),
            quantity: s.quantity,
        }).collect::<Vec<OrderScaffold>>();
        let asks_with_diff_bias = asks_.iter().map(|s| OrderScaffold {
            price: MyBigUint(BigUint::from(max(1, s.price.0.to_i32().unwrap() + diff.to_i32().unwrap()) as u64)),
            quantity: s.quantity,
        }).collect::<Vec<OrderScaffold>>();
        let bids = bids_with_diff_bias.iter().map(|s| OrderScaffold {
            // buy if orderbook too big artificiall
            price: if state.asks.len() < 50 {s.price.clone()} else { state.asks.peek().unwrap().data.price.clone() },
            quantity: s.quantity,
        }).collect::<Vec<OrderScaffold>>();
        let asks = asks_with_diff_bias.iter().map(|s| OrderScaffold {
            // sell if orderbook too big artificiall
            price: if state.bids.len() < 50 {s.price.clone()} else { state.bids.peek().unwrap().data.price.clone() },
            quantity: s.quantity,
        }).collect::<Vec<OrderScaffold>>();
        self.n = self.n.wrapping_add(1);
        // self.diff = self.diff.wrapping_add(bids.iter().map(|s| s.quantity as i32 * &s.price.0.to_i32().unwrap()).sum::<i32>() - asks.iter().map(|s| s.quantity as i32 * &s.price.0.to_i32().unwrap()).sum::<i32>());
        ReportedScaffolds {
            bids: Vec::from(bids),
            asks: Vec::from(asks),
        }
    }

}