#![allow(dead_code)]

use std::sync::atomic::AtomicUsize;
use std::cell::UnsafeCell;

struct Node<T> {
    sequence: AtomicUsize,
    data: Option<UnsafeCell<T>>
}

impl<T> Node<T> {
    fn new(sequence: usize) -> Node<T> {
        Node {
            sequence: AtomicUsize::new(sequence),
            data: None
        }
    }
}

pub struct Queue<T> {
    buffer: Vec<Node<T>>,
    mask: usize,
    enqueue_index: AtomicUsize,
    dequeue_index: AtomicUsize
}

impl<T> Queue<T> {
    fn new(bound: usize) -> Queue<T> {
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

    pub fn enqueue(&mut self, item: T) -> bool {
        false
    }

    pub fn dequeue(&mut self) -> Option<T> {
        None
    }
}
