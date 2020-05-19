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

use std::sync::{
    mpsc::{channel, Receiver, SendError, Sender, TryRecvError},
    Arc, Mutex,
};

use slab::Slab;
#[derive(Debug)]
struct SharedIterCore<I: Iterator> {
    iter: I,
    sender: Slab<Sender<I::Item>>,
}

impl<I: Iterator> SharedIterCore<I> {
    fn new(iter: I) -> Self {
        Self {
            iter,
            sender: Slab::with_capacity(1),
        }
    }

    fn send(&mut self, val: I::Item) -> Result<(), SendError<I::Item>>
    where
        I::Item: Copy,
    {
        for (_, sender) in self.sender.iter() {
            sender.send(val)?;
        }
        Ok(())
    }

    fn next(&mut self)
    where
        I::Item: Copy,
    {
        if let Some(val) = self.iter.next() {
            self.send(val).expect("");
        }
    }

    fn new_recv(&mut self) -> (usize, Receiver<I::Item>) {
        let (sender, receiver) = channel();
        let id = self.sender.insert(sender);
        (id, receiver)
    }

    fn remove_recv(&mut self, id: usize) {
        self.sender.remove(id);
    }
}

#[derive(Debug)]
pub struct SharedIter<I: Iterator> {
    id: usize,
    inner: Arc<Mutex<SharedIterCore<I>>>,
    receiver: Receiver<I::Item>,
}

impl<I: Iterator> SharedIter<I> {
    fn new(iter: I) -> Self {
        let mut inner = SharedIterCore::new(iter);
        let (id, receiver) = inner.new_recv();
        Self {
            id,
            inner: Arc::new(Mutex::new(inner)),
            receiver,
        }
    }
}

impl<I: Iterator> Clone for SharedIter<I> {
    fn clone(&self) -> Self {
        let (id, receiver) = self.inner.lock().unwrap().new_recv();
        Self {
            inner: self.inner.clone(),
            receiver,
            id,
        }
    }
}

impl<I: Iterator> Iterator for SharedIter<I>
where
    I::Item: Copy,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        match self.receiver.try_recv() {
            Ok(val) => Some(val),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => {
                self.inner.lock().unwrap().next();
                self.receiver.try_recv().ok()
            }
        }
    }
}

impl<I: Iterator> Drop for SharedIter<I> {
    fn drop(&mut self) {
        self.inner.lock().unwrap().remove_recv(self.id);
    }
}

/// # ShareIterator
pub trait ShareIterator: Iterator + Sized {
    fn share(self) -> SharedIter<Self>;
}

impl<I: Iterator> ShareIterator for I {
    fn share(self) -> SharedIter<Self> {
        SharedIter::new(self)
    }
}

// impl<I: Iterator> ShareIterator for SharedIter<I> {
//     fn share(self) -> SharedIter<Self> {
//        self
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
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
