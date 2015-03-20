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
    buffer: Vec<Node<T>>,
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

        let mut buffer = Vec::with_capacity(bound);
        for i in 0..bound {
            buffer.push(Node::new(i));
        }

        Queue {
            buffer: buffer,
            mask: bound - 1,
            enqueue_index: AtomicUsize::new(0),
            dequeue_index: AtomicUsize::new(0)
        }
    }

    pub fn try_enqueue(&mut self, item: T) -> bool {
        let mut index = self.enqueue_index.load(Ordering::Relaxed);
        loop {
            let cell = &mut self.buffer[index & self.mask];
            let ticket = cell.ticket.load(Ordering::Acquire);
            if ticket == index {
                if index == self.enqueue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    unsafe {
                        *cell.data.get() = Some(item);
                    }
                    cell.ticket.store(index + 1, Ordering::Relaxed);
                    return true;
                }
            } else if ticket < index {
                return false;
            } else {
                index = self.enqueue_index.load(Ordering::Relaxed);
            }
        }
    }

    pub fn try_dequeue(&mut self) -> Option<T> {
        let mut index = self.dequeue_index.load(Ordering::Relaxed);
        loop {
            let cell = &self.buffer[index & self.mask];
            let ticket = cell.ticket.load(Ordering::Acquire);
            if ticket == index + 1 {
                if index == self.dequeue_index.compare_and_swap(index, index + 1, Ordering::Relaxed) {
                    let data = unsafe {
                        (*cell.data.get()).take()
                    };
                    cell.ticket.store(index + self.mask + 1, Ordering::Release);
                    return data;
                }
            } else if ticket < index + 1 {
                return None;
            } else {
                index = self.dequeue_index.load(Ordering::Relaxed);
            }
        }
    }

    /*
    pub fn enqueue(&mut self, item: T) {
        while (!self.try_enqueue(item)) {
        }
    }

    pub fn dequeue(&mut self) -> T {
        loop {
            let result = self.try_dequeue();
            if (result.is_some()) {
                return result.unwrap();
            }
        }
    }
    */
}

#[cfg(test)]
mod tests {
    use super::Queue;

    static QUEUE_SIZE: usize = 1024usize;

    #[test]
    fn test_single_thread() {
        let mut queue: Queue<usize> = Queue::new(QUEUE_SIZE);

        for i in 0..QUEUE_SIZE as usize {
            let success = queue.try_enqueue(i);
            assert!(success);
        }

        for i in 0..QUEUE_SIZE as usize {
            let value = queue.try_dequeue().unwrap();
            assert_eq!(value, i);
        }
    }
}
