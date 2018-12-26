//! This crate provides an implementation of the [weak
//! heap](https://en.wikipedia.org/wiki/Weak_heap), a variant heap data
//! structure.
//!
//! ## Runtime costs
//!
//! Nominal operation time bounds are comparable to those of the more popular
//! binary heap:
//! * find-min: O(1)
//! * delete-min: O(log n)
//! * insert: O(log n)
//!
//! In a more realistic model of computation that does more than count the
//! number of comparisons being made, the runtime cost of this heap's operations
//! requires a more nuanced analysis.
//!
//! In simple benchmarks, the time it takes to perform bulk operations on this
//! heap when it contains a simple data type that it is very cheap to compare
//! (like integers) has greater expectation and variance than a comparable set
//! of operations on `std::collections::BinaryHeap`. Once a weak heap reaches a
//! modest size, however, the number of comparisons that are required to perform
//! an operation is less than the number of comparisons that a binary heap
//! makes. If the cost of making a comparison truly dominates running time, then
//! `WeakHeap` may be faster than `BinaryHeap` in practice.

use std::cmp::Ord;
use std::fmt;
use std::ptr;

/// An entry in the heap, consisting of a bit that indicates whether the roles
/// of its left and right children are swapped, and the actual value being
/// stored in the heap.
#[derive(Debug)]
struct HeapEntry<T: fmt::Debug> {
  /// The actual value at this heap entry.
  value: T,
  /// If `true`, then siblings of this heap entry (when viewing the heap as an
  /// N-ary tree) are in its left-hand subtree (when viewing the heap as a
  /// binary tree), and its children are in its right-hand subtree. If `false`,
  /// the left- and right-hand subtrees are flipped.
  valence: bool,
}

/// A weak heap data structure. This implementation is a max-heap.
///
/// ```rust
/// # use weak_heap::WeakHeap;
/// # fn main() {
/// let mut heap: WeakHeap<i32> = WeakHeap::new();
/// heap.push(5);
/// heap.push(10);
/// heap.push(7);
/// let mut ordered = Vec::new();
/// while let Some(n) = heap.pop() {
///   ordered.push(n);
/// }
/// assert_eq!(ordered, vec![10, 7, 5]);
/// # }
/// ```
#[derive(Debug)]
pub struct WeakHeap<T: fmt::Debug + Ord> {
  data: Vec<HeapEntry<T>>,
}

impl<T: fmt::Debug + Ord> WeakHeap<T> {
  /// Creates a new heap with a default capacity.
  pub fn new() -> Self {
    WeakHeap {
      data: Vec::new(),
    }
  }

  /// Creates a new heap with capacity for at least `cap` elements.
  pub fn with_capacity(cap: usize) -> Self {
    WeakHeap {
      data: Vec::with_capacity(cap),
    }
  }

  /// Returns the number of elements in the heap.
  pub fn len(&self) -> usize {
    self.data.len()
  }

  /// Returns `true` iff the heap is empty.
  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  /// Returns a reference to the top element on the heap, or `None` if the heap
  /// is empty.
  pub fn peek(&self) -> Option<&T> {
    self.data.first().map(|x| &x.value)
  }

  /// Pushes `value` onto the heap.
  pub fn push(&mut self, value: T) {
    let offset = self.len();
    self.data.push(HeapEntry { valence: false, value: value, });
    unsafe { self.data.get_unchecked_mut(offset / 2).valence |= offset % 2 == 0; }
    self.sift_up(offset);
  }

  /// Removes the top element from the heap and returns it, or returns `None` if
  /// the heap is empty.
  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      let result = Some(self.data.swap_remove(0).value);
      if !self.is_empty() {
        unsafe { self.data.get_unchecked_mut(0).valence = false; }
        self.sift_down(1);
      }
      result
    }
  }

  /// Returns the offset into `self.data` for the child of the element at
  /// `offset`.
  fn child_offset(&self, offset: usize) -> usize {
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(unsafe { self.data.get_unchecked(offset).valence } as usize))
      .expect("child offset computation overflow")
  }

  /// Returns the offset into `self.data` for the distinguished ancestor of the
  /// element at `offset`. The distinguished ancestor of an element is its
  /// immediate parent when viewing the heap as an N-ary tree.
  fn distinguished_ancestor_offset(&self, mut offset: usize) -> usize {
    debug_assert!(offset > 0);
    let mut parent_offset = offset / 2;
    let mut parent_valence = unsafe { self.data.get_unchecked(parent_offset).valence };
    while (offset & 1) == (parent_valence as usize) {
      offset = parent_offset;
      parent_offset /= 2;
      parent_valence = unsafe { self.data.get_unchecked(parent_offset).valence };
    }
    parent_offset
  }

  /// Sifts the element at `offset` up in the heap so that the heap invariants
  /// are satisfied. This is done by repeatedly swapping an item with its
  /// distinguished ancestor until its distinguished ancestor is greater than it
  /// is.
  fn sift_up(&mut self, mut offset: usize) {
    unsafe {
      let element = ptr::read(&self.data.get_unchecked(offset).value);
      let mut ancestor_offset;
      while offset > 0 {
        ancestor_offset = self.distinguished_ancestor_offset(offset);
        let ancestor_value: *const _ = &self.data.get_unchecked(ancestor_offset).value;
        if *ancestor_value >= element {
          break;
        }
        let offset_item = self.data.get_unchecked_mut(offset);
        ptr::copy_nonoverlapping(ancestor_value, &mut offset_item.value, 1);
        offset_item.valence = !offset_item.valence;
        offset = ancestor_offset;
      }
      ptr::write(&mut self.data.get_unchecked_mut(offset).value, element);
    }
  }

  /// Sifts the top of the heap down so that the heap invariants are
  /// satisfied. This is done by recursively swapping descendants of the element
  /// at `child_offset` with the top of the heap (starting with a descenant at
  /// the lowest level of the tree possible and proceeding up then through each
  /// preceding level, until finally the element at `child_offset` is compared
  /// against).
  ///
  /// When this recursion begins, the top of the heap may be in violation of the
  /// heap invariant (i.e., it may be less than one of its children).
  fn sift_down(&mut self, child_offset: usize) {
    if child_offset >= self.data.len() {
      return;
    }
    let next_child_offset = self.child_offset(child_offset);
    self.sift_down(next_child_offset);
    unsafe {
      let head_value: *mut T = &mut self.data.get_unchecked_mut(0).value;
      let child = &mut self.data.get_unchecked_mut(child_offset);
      if *head_value < child.value {
        ptr::swap_nonoverlapping(head_value, &mut child.value, 1);
        child.valence = !child.valence;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::WeakHeap;
  use rand::{Rng, SeedableRng, StdRng};

  pub fn get_values(size: usize) -> Vec<i32> {
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    (0..size).map(|_| rng.gen::<i32>()).map(|x| x.into()).collect()
  }

  #[test]
  fn instantiate_empty_heap() {
    WeakHeap::<usize>::new();
  }

  #[test]
  fn singleton_heap() {
    let mut t = WeakHeap::new();
    assert_eq!(t.len(), 0);
    assert!(t.is_empty());
    t.push(23);
    assert_eq!(t.len(), 1);
    assert!(!t.is_empty());
    assert_eq!(t.peek(), Some(&23i32));
    assert_eq!(t.pop(), Some(23i32));
    assert_eq!(t.len(), 0);
    assert!(t.is_empty());
  }

  #[test]
  fn multiple_pop() {
    let mut t = WeakHeap::new();
    let values = [0usize, 1, 2, 3, 4, 5, 6];
    for x in &values {
      t.push(*x);
      assert_eq!(t.peek().map(|n| *n), Some(*x));
    }
    assert_eq!(values.len(), t.len());
    let mut values_iter = values.iter().rev();
    while let Some(x) = t.pop() {
      assert_eq!(x, *values_iter.next().unwrap());
    }
    assert!(t.is_empty());
  }

  #[test]
  fn correct_ordering() {
    let values = get_values(60);
    let sorted = {
      let mut sorted = values.clone();
      sorted.sort_by(|x, y| y.cmp(x));
      sorted
    };
    let mut heap_sorted = Vec::new();
    let mut heap = WeakHeap::new();
    for x in &values {
      heap.push(*x);
    }
    while let Some(x) = heap.pop() {
      heap_sorted.push(x);
    }
    assert_eq!(heap_sorted, sorted);
  }
}
