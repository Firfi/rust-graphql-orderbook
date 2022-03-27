use num_bigint::BigUint;
use rand::prelude::ThreadRng;
use rand::{Rng, RngCore};
use tokio::time::Interval;
use crate::orderbook::{BuyOrder, SellOrder};

const MARGIN: usize = 6;
const BIDDER_CROWD: usize = 10;

pub struct Reporter {
    rng: ThreadRng,
    n: u64,
}

fn price_law(k: u64) -> BigUint {
    BigUint::from(((((k as f64) * 0.1).sin() + (2 as f64)) * (100 as f64)).floor() as u64)
} // no 0 price;


#[derive(Hash, Clone)]
pub(crate) struct OrderScaffold {
    pub(crate) price: BigUint,
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
        let price_from_step = price_law(n) + BigUint::from(self.price_fluctuation());
        let scaffolds = vec![0, (self.rng.gen_range(0..1) * BIDDER_CROWD)]
            .into_iter()
            .map(|_| OrderScaffold {
                price: price_from_step.clone(), // + price_fluc
                quantity: self.rng.gen_range(0..100),
            })
            .collect::<Vec<OrderScaffold>>();
        // prices.sort(); // in case we add fluctuation
        let middle = (scaffolds.len() / 2);
        let (bids, asks) = scaffolds.split_at(middle);
        self.n = self.n.wrapping_add(1);
        ReportedScaffolds {
            bids: Vec::from(bids),
            asks: Vec::from(asks),
        }
    }

}