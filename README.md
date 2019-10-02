# Shared Iter

Clone an Iterator and shared it accros threads

```rust
use shared_iter::ShareIterator;

let iter1 = (1..20).share();
let iter2 = iter1.clone();

assert_eq!(iter1.take(10).collect::<Vec<_>>(), iter2.take(10).collect::<Vec<_>>());
```
