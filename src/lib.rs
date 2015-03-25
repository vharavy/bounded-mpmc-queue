use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;

struct Node<T> {
    ticket: AtomicUsize,
    data: UnsafeCell<Option<T>>
}

impl<T> Node<T> {
    fn new(ticket: usize) -> Node<T> {
        Node {
            ticket: AtomicUsize::new(ticket),
            data: UnsafeCell::new(None)
        }
    }
}

pub struct Queue<T> {
    nodes: Vec<Node<T>>,
    mask: usize,
    enqueue_index: AtomicUsize,
    dequeue_index: AtomicUsize
}

unsafe impl<T: Send> Send for Queue<T> { }
unsafe impl<T: Send> Sync for Queue<T> { }

impl<T> Queue<T> {
    pub fn new(bound: usize) -> Queue<T> {
        assert!(bound >= 2);
        assert_eq!(bound & (bound - 1), 0);

        let mut nodes = Vec::with_capacity(bound);
        for i in 0..bound {
            nodes.push(Node::new(i));
        }

        Queue {
            nodes: nodes,
            mask: bound - 1,
            enqueue_index: AtomicUsize::new(0),
            dequeue_index: AtomicUsize::new(0)
        }
    }

    pub fn try_enqueue(&self, item: T) -> Option<T> {
        let mut index = self.enqueue_index.load(Ordering::Relaxed);
        loop {
            let node = &self.nodes[index & self.mask];
            let ticket = node.ticket.load(Ordering::Acquire);
            if ticket == index {
                if index == self.enqueue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    unsafe {
                        *node.data.get() = Some(item);
                    }
                    node.ticket.store(index + 1, Ordering::Release);
                    return None;
                }
            } else if ticket < index {
                return Some(item);
            } else {
                index = self.enqueue_index.load(Ordering::Relaxed);
            }
        }
    }

    pub fn try_dequeue(&self) -> Option<T> {
        let mut index = self.dequeue_index.load(Ordering::Relaxed);
        loop {
            let node = &self.nodes[index & self.mask];
            let ticket = node.ticket.load(Ordering::Acquire);
            if ticket == index + 1 {
                if index == self.dequeue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    let data = unsafe {
                        (*node.data.get()).take()
                    };
                    node.ticket.store(index + self.mask + 1, Ordering::Release);
                    return data;
                }
            } else if ticket < index + 1 {
                return None;
            } else {
                index = self.dequeue_index.load(Ordering::Relaxed);
            }
        }
    }

    pub fn enqueue(&self, item: T) {
        let mut value = item;
        loop {
            match self.try_enqueue(value) {
                Some(v) => value = v,
                None => return
            }
        }
    }

    pub fn dequeue(&self) -> T {
        loop {
            match self.try_dequeue() {
                Some(value) => return value,
                None => {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;
    use std::thread;
    use std::sync::{Arc, Barrier};

    static QUEUE_SIZE: usize = 0x1000_usize;
    static THREAD_COUNT: usize = 2;
    static MESSAGE_COUNT: u64 = 0x100_0000_u64;

    fn consumer(queue: &Queue<u64>) {
        let mut sum = 0u64;

        for _ in 0..MESSAGE_COUNT as u64 {
            sum += queue.dequeue();
        }

        println!("Consumer: {}", sum);
    }

    fn producer(queue: &Queue<u64>) {
        let mut sum = 0u64;
        for i in 0..MESSAGE_COUNT as u64 {
            sum += i;
            queue.enqueue(i);
        }

        println!("Producer: {}", sum);
    }

    #[test]
    fn multiple_threads() {
        let queue = Queue::new(QUEUE_SIZE);

        let mut consumer_threads: Vec<_> = Vec::with_capacity(THREAD_COUNT);
        let mut producer_threads: Vec<_> = Vec::with_capacity(THREAD_COUNT);

        let barrier = Arc::new(Barrier::new(2 * THREAD_COUNT + 1));

        for _ in 0..THREAD_COUNT {
            let b = barrier.clone();
            let q = &queue;
            consumer_threads.push(thread::scoped(move || {
                b.wait();
                consumer(q);
            }));
        }

        for _ in 0..THREAD_COUNT {
            let b = barrier.clone();
            let q = &queue;
            producer_threads.push(thread::scoped(move || {
                b.wait();
                producer(q);
            }));
        }

        barrier.wait();

        for producer_thread in producer_threads {
            producer_thread.join();
        }

        for consumer_thread in consumer_threads {
            consumer_thread.join();
        }
    }
}
