use core::cmp::Ord;
use core::{cmp, fmt};
use core::option::Option;
use core::result::Result::Ok;
use std::collections::BinaryHeap;

const DEFAULT_LIMIT: usize = 100;

pub struct SliceDisplay<'a, T: 'a>(&'a [T]);

impl<'a, T: fmt::Display + 'a> fmt::Display for SliceDisplay<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for item in self.0 {
            if !first {
                write!(f, ", {}", item)?;
            } else {
                write!(f, "{}", item)?;
            }
            first = false;
        }
        Ok(())
    }
}

pub fn sorted_slice<T: Ord + std::clone::Clone + fmt::Display>(h: &BinaryHeap<T>, limit: Option<usize>) -> Vec<T> {
    let mut v = h.clone().into_sorted_vec();
    v.reverse();
    v[0..cmp::min(limit.unwrap_or(DEFAULT_LIMIT), v.len())].to_vec()
}
