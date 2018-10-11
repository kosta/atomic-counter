 # AtomicCounter \
   [![docs.rs](https://docs.rs/atomic-counter/badge.svg)](https://docs.rs/atomic-counter) \
   [![Build Status](https://travis-ci.com/kosta/atomic-counter.svg?branch=master)](https://travis-ci.com/kosta/atomic-counter)

 Atomic (thread-safe) counters for Rust.

 This crate contains an [`AtomicCounter`](trait.AtomicCounter.html) trait
 that can safely be shared across threads.

 This crate provides two implementations:

 * [`RelaxedCounter`](struct.RelaxedCounter.html) which is suitable for
     e.g. collecting metrics or generate IDs, but which does not provide
     ["Sequential Consistency"](https://doc.rust-lang.org/nomicon/atomics.html#sequentially-consistent).
     `RelaxedCounter` uses [`Relaxed`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.Relaxed)
     memory ordering.

 * [`ConsistentCounter`](struct.ConsistentCounter.html) which provides the
     same interface but is sequentially consistent. Use this counter if the
     order of update from multiple threads is important.
     `ConsistentCounter` uses [`Sequentially Consistent`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.SeqCst)
     memory ordering.

 Both implementations are lock-free. Both are a very thin layer over
 [`AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html)
 which is more powerful but might be harder to use correctly.

 ## Which counter to use

 * If you are just collecting metrics, the `RelaxedCounter` is probably right choice.

 * If you are generating IDs, but don't make strong assumptions (like allocating
     memory based on the ID count), `RelaxedCounter` is probably the right choice.

 * If you are generating multiple IDs where you maintain an ordering
     invariant (e.g. ID `a` is always greater than ID `b`), you need "Sequential
     Consistency" and thus need to use `ConsistentCounter`. The same is true
     for all use cases where the _ordering_ of incrementing the counter is
     important.

 ## No updates are lost - It's just about the ordering!

 Note that in both implementations, _no count is lost_ and all operations are atomic.
 The difference is _only_ in how the order of operations are observed by different
 threads.

 ## Example:
 Assume `a` is 5 and `b` is 4. You always want to maintain `a > b`.

 Thread 1 executes this code:

 ```rust,ignore

 a.inc();
 b.inc();
 ```

 Thread 2 gets counts:

 ```rust,ignore

 let a_local = a.get();
 let b_local = b.get();
 ```

 What are the values for `a_local` and `b_local`? That depends on the order
 in which thread 1 and 2 have run:

 * `a_local` could still be 5 and `b_local` is still be 4 (e.g. if thread 2 ran before thread 1)
 * `a_local` could be increment to 6 while `b_local` is still at 4 (e.g. if thread 1 and 2 ran in parallel)
 * `a_local` could be increment to 6 and `b_local` be incremented to 5 (e.g. if thread 2 ran after thread 1).
 * Additionally, if at least one counter is a `RelaxedCounter`, we cannot make
     assumption on the order of `a.inc()` and `b.inc()`. Thus, in this case
     thread 2 can also observe `a_local` to be 5 (not incremented yet) but
     `b_local` to be incremented to 5, _breaking the invariant_ `a > b`.
     Note that if thread 2 (or any other thread) `get()` the counts
     again, at some point they will observe both values to be incremented.
     No operations will be lost. It is only the _ordering_ of the operations
     that cannot be assumed if `Ordering` is `Relaxed`.

 So in order to maintain invariants such as `a > b` across multiple threads,
 use `ConsistentCounter`.
