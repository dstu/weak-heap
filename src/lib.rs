#![feature(duration_from_micros)]
#![feature(test)]

#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

#[cfg(test)] mod bench;

use std::cmp::Ord;
use std::fmt;
use std::ptr;

#[derive(Debug)]
struct HeapEntry<T: fmt::Debug> {
  valence: bool,
  value: T,
}

#[derive(Debug)]
pub struct WeakHeap<T: fmt::Debug + Ord> {
  data: Vec<HeapEntry<T>>,
}

impl<T: fmt::Debug + Ord> WeakHeap<T> {
  pub fn new() -> Self {
    WeakHeap {
      data: Vec::new(),
    }
  }

  pub fn with_capacity(cap: usize) -> Self {
    WeakHeap {
      data: Vec::with_capacity(cap),
    }
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn peek(&self) -> Option<&T> {
    self.data.first().map(|x| &x.value)
  }

  pub fn push(&mut self, value: T) {
    let offset = self.len();
    self.data.push(HeapEntry { valence: false, value: value, });
    unsafe { self.data.get_unchecked_mut(offset / 2).valence |= offset % 2 == 0; }
    self.sift_up(offset);
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      let result = Some(self.data.swap_remove(0).value);
      if !self.is_empty() {
        unsafe { self.data.get_unchecked_mut(0).valence = false; }
        self.sift_down();
      }
      result
    }
  }

  fn child_offset(&self, offset: usize) -> usize {
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(unsafe { self.data.get_unchecked(offset).valence } as usize))
      .expect("child offset computation overflow")
  }

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

  fn sift_up(&mut self, mut offset: usize) {
    unsafe {
      let element = ptr::read(&self.data.get_unchecked(offset).value);
      let mut ancestor_offset;
      while offset > 0 {
        ancestor_offset = self.distinguished_ancestor_offset(offset);
        if self.data.get_unchecked(ancestor_offset).value >= element {
          break;
        }
        ptr::copy_nonoverlapping(
          &self.data.get_unchecked(ancestor_offset).value,
          &mut self.data.get_unchecked_mut(offset).value,
          1);
        self.data.get_unchecked_mut(offset).valence = !self.data.get_unchecked(offset).valence;
        offset = ancestor_offset;
      }
      ptr::write(&mut self.data.get_unchecked_mut(offset).value, element);
    }
  }

  fn sift_down(&mut self) {
    let mut child_offset = 1;
    if child_offset >= self.data.len() {
      return;
    }
    let mut next_child_offset = self.child_offset(child_offset);
    while next_child_offset < self.data.len() {
      child_offset = next_child_offset;
      next_child_offset = self.child_offset(child_offset);
    }
    while child_offset > 0 {
      if unsafe { self.data.get_unchecked(0).value < self.data.get_unchecked(child_offset).value } {
        self.data.swap(0, child_offset);
        // Swap valences back, reversing child offset valence.
        unsafe {
          let new_child_valence = !self.data.get_unchecked(0).valence;
          self.data.get_unchecked_mut(0).valence = self.data.get_unchecked(child_offset).valence;
          self.data.get_unchecked_mut(child_offset).valence = new_child_valence;
        }
      }
      child_offset /= 2;
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
