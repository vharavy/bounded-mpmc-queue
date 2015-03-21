Bounded MPMC Queue in Rust
==========================

In this repository I am attempting to create a multiple producers multiple consumers queue for
inter-thread communication in [Rust](https://github.com/rust-lang/rust) programming
language. The idea is based on the following articles:

  1. [Bounded MPMC Queue in C++ by Dmitry Vyukov](http://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue)
  2. [Bounded MPMC Queue in Java by David Dice](https://blogs.oracle.com/dave/entry/ptlqueue_a_scalable_bounded_capacity)

The usage of [Rust](https://github.com/rust-lang/rust) allows to develop safe and
very efficient API.
