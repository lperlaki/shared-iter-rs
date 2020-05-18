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

use std::collections::HashMap;

#[derive(Debug)]
struct SharedIterCore<I: Iterator> {
    iter: Mutex<I>,
    buff: Mutex<HashMap<usize, (I::Item, usize)>>,
}

impl<I: Iterator> SharedIterCore<I> {
    fn new(iter: I) -> Self {
        Self {
            iter: Mutex::new(iter),
            buff: Mutex::new(HashMap::with_capacity(8)),
        }
    }
}

impl<I: Iterator> SharedIterCore<I>
where
    I::Item: Copy,
{
    fn get(this: &Arc<Self>, index: usize) -> Option<I::Item> {
        let mut buff = this.buff.lock().unwrap();
        if let Some(v) = buff.get_mut(&index) {
            v.1 -= 1;
            let val = v.0;
            if v.1 == 0 {
                buff.remove(&index);
            }
            Some(val)
        } else {
            let r = this.iter.lock().unwrap().next()?;
            buff.insert(index, (r, Arc::strong_count(this) - 1));
            Some(r)
        }
    }
}

/// # SharedIterator
pub struct SharedIter<I: Iterator> {
    core: Arc<SharedIterCore<I>>,
    index: usize,
}

impl<I: Iterator> std::fmt::Debug for SharedIter<I>
where
    I: std::fmt::Debug,
    I::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedIter")
            .field("core", &self.core)
            .field("index", &self.index)
            .finish()
    }
}

impl<I: Iterator> Iterator for SharedIter<I>
where
    I::Item: Copy,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let v = SharedIterCore::get(&self.core, self.index);
        self.index += 1;
        v
    }
}

impl<I: Iterator> Clone for SharedIter<I> {
    fn clone(&self) -> Self {
        Self {
            core: Arc::clone(&self.core),
            index: self.index,
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
            core: Arc::new(SharedIterCore::new(self)),
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
    fn test_multi_threaded() {
        use std::thread;
        let iter = (1..).share();
        let threads = (0..5)
            .map(|_| iter.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .map(|liter| thread::spawn(move || liter.take(10).collect::<Vec<_>>()))
            .collect::<Vec<_>>();

        let r = iter.take(10).collect::<Vec<_>>();
        for t in threads {
            assert_eq!(t.join().unwrap(), r);
        }
    }
}
