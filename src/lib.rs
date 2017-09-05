
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

trait AtomicCounter {
    type PrimitiveType;

    fn inc(&self);
    fn add(&self, amount: Self::PrimitiveType);
    fn reset(&self) -> Self::PrimitiveType;
}

impl AtomicCounter for AtomicUsize {
    type PrimitiveType = usize;

    fn inc(&self) {
        self.add(1)
    }

    fn add(&self, amount: usize) {
        self.fetch_add(amount, Relaxed);
    }

    fn reset(&self) -> usize {
        self.swap(0, Relaxed)
    }
}

#[cfg(test)]
mod tests {

    use std::thread;
    use std::sync::Arc;
    use std::ops::Deref;

    use super::*;

    const NUM_THREADS: usize = 29;
    const NUM_ITERATIONS: usize = 7_000_000;

    #[test]
    fn test_inc() {
        let mut join_handles = Vec::new();
        let counter = Arc::new(AtomicUsize::new(0));
        println!("test_inc: Spawning {} threads, each with {} iterations...", NUM_THREADS, NUM_ITERATIONS);
        for _ in 0..NUM_THREADS {
            let counter_ref = counter.clone();
            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &AtomicUsize = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.inc();
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let count = counter.reset();
        println!("test_inc: Got count: {}", count);
        assert_eq!(NUM_THREADS * NUM_ITERATIONS, count);
    }

    #[test]
    fn test_add() {
        let mut join_handles = Vec::new();
        let counter = Arc::new(AtomicUsize::new(0));
        println!("test_add: Spawning {} threads, each with {} iterations...", NUM_THREADS, NUM_ITERATIONS);
        let mut expected_count = 0;
        for to_add in 0..NUM_THREADS {
            let counter_ref = counter.clone();
            expected_count += to_add * NUM_ITERATIONS;
            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &AtomicUsize = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.add(to_add);
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let count = counter.reset();
        println!("test_add: Expected count: {}, got count: {}", expected_count, count);
        assert_eq!(expected_count, count);
    }

    #[test]
    fn test_add_reset() {
        let mut join_handles = Vec::new();
        let counter = Arc::new(AtomicUsize::new(0));
        println!("test_add_reset: Spawning {} threads, each with {} iterations...", NUM_THREADS, NUM_ITERATIONS);
        let mut expected_count = 0;
        for to_add in 0..NUM_THREADS {
            expected_count += to_add * NUM_ITERATIONS;
        }

        // setup thread that `reset()`s all the time
        let counter_ref = counter.clone();
        let reset_handle = thread::spawn(move || {
            // Usually, you would check for some better termination condition.
            // I don't want to pollute my test with thread synchronization
            // operations outside of AtomicCounter, hence this approach.
            let mut total_count = 0;
            let counter: &AtomicUsize = counter_ref.deref();
            while total_count < expected_count {
                total_count += counter.reset();
            }
            // Ok, now we got the total_count but this could just be lucky.
            // Better do some more resets to be sure... ;)
            for _ in 0..NUM_ITERATIONS {
                total_count += counter.reset();
            }
            total_count
        });

        for to_add in 0..NUM_THREADS {
            let counter_ref = counter.clone();

            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &AtomicUsize = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.add(to_add);
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let actual_count = reset_handle.join().unwrap();
        println!("test_add_reset: Expected count: {}, got count: {}", expected_count, actual_count);
        assert_eq!(expected_count, actual_count);
    }

}
