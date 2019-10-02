//! # Shared Iter
//!
//! Clone an Iterator and shared it accros threads
//!
//! ```rust
//! use shared_iter::ShareIterator;
//!
//! let iter1 = (1..20).share();
//! let iter2 = iter1.clone();
//!
//! assert_eq!(iter1.take(10).collect::<Vec<_>>(), iter2.take(10).collect::<Vec<_>>());
//! ```

use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct SharedIterCore<I: Iterator> {
    iter: I,
    buff: Vec<I::Item>,
}

impl<I: Iterator> SharedIterCore<I> {
    fn new(iter: I) -> Self {
        Self {
            iter,
            buff: Vec::new(),
        }
    }
}

impl<I: Iterator> SharedIterCore<I>
where
    I::Item: Copy,
{
    fn get(&mut self, index: usize) -> Option<I::Item> {
        let val = self.buff.get(index);
        if let Some(v) = val {
            Some(*v)
        } else {
            let r = self.iter.next()?;
            self.buff.push(r);
            Some(r)
        }
    }
}

/// # SharedIterator
pub struct SharedIter<I: Iterator> {
    core: Arc<Mutex<SharedIterCore<I>>>,
    index: usize,
}

impl<I: Iterator> Iterator for SharedIter<I>
where
    I::Item: Copy,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let mut core = self.core.lock().expect("Something went wrong");
        let v = core.get(self.index);
        self.index += 1;
        v
    }
}

impl<I: Iterator> Clone for SharedIter<I> {
    fn clone(&self) -> Self {
        Self {
            core: Arc::clone(&self.core),
            index: 0,
        }
    }
}

/// # ShareIterator
pub trait ShareIterator: Iterator + Sized {
    fn share(self) -> SharedIter<Self>;
}

impl<I: Iterator> ShareIterator for I {
    fn share(self) -> SharedIter<Self> {
        SharedIter {
            core: Arc::new(Mutex::new(SharedIterCore::new(self))),
            index: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    #[test]
    fn test_iter() {
        let iter = (1..20).share();
        let iter2 = iter.clone();
        assert_eq!(
            iter.take(10).collect::<Vec<_>>(),
            iter2.take(10).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_rand() {
        let iter = rand::thread_rng()
            .sample_iter::<u8, _>(rand::distributions::Standard)
            .share();
        let iter2 = iter.clone();
        assert_eq!(
            iter.take(10).collect::<Vec<_>>(),
            iter2.take(10).collect::<Vec<_>>()
        );
    }
}
