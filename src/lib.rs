#![feature(duration_from_micros)]
#![feature(inclusive_range_syntax)]
#![feature(test)]

#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

#[cfg(test)] mod bench;

use std::cmp::Ord;
use std::fmt;
use std::ptr;

#[derive(Debug)]
pub struct WeakHeap<T: fmt::Debug + Ord> {
  valences: Vec<bool>,
  data: Vec<T>,
}

impl<T: fmt::Debug + Ord> WeakHeap<T> {
  pub fn new() -> Self {
    WeakHeap {
      valences: Vec::new(),
      data: Vec::new(),
    }
  }

  pub fn with_capacity(cap: usize) -> Self {
    WeakHeap {
      valences: Vec::with_capacity(cap),
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
    self.data.first()
  }

  pub fn push(&mut self, value: T) {
    let offset = self.len();
    self.data.push(value);
    self.valences.push(false);
    self.valences[offset / 2] = self.valences[offset / 2] || offset % 2 == 0;
    self.sift_up(offset);
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.is_empty() {
      None
    } else {
      let result = Some(self.data.swap_remove(0));
      self.sift_down(0);
      result
    }
  }

  fn child_offset(&self, offset: usize) -> usize {
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(self.valences[offset] as usize))
      .expect("child offset computation overflow")
  }

  fn sibling_offset(&self, offset: usize) -> usize {
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(1 - self.valences[offset] as usize))
      .expect("sibling computation overflow")
  }

  fn distinguished_ancestor_offset(&self, mut offset: usize) -> usize {
    if offset == 0 {
      panic!("root has no ancestor");
    } else {
      let mut parent_offset = offset / 2;
      let mut parent_valence = self.valences[parent_offset];
      while (offset & 1) == (parent_valence as usize) {
        offset = parent_offset;
        parent_offset /= 2;
        parent_valence = self.valences[parent_offset];
      }
      parent_offset
    }
  }

  fn sift_up(&mut self, mut offset: usize) {
    unsafe {
      let element = ptr::read(&self.data[offset]);
      let mut ancestor_offset;
      while offset > 0 {
        ancestor_offset = self.distinguished_ancestor_offset(offset);
        if self.data[ancestor_offset] >= element {
          break;
        }
        ptr::copy_nonoverlapping(&self.data[ancestor_offset], &mut self.data[offset], 1);
        self.valences[offset] = !self.valences[offset];
        offset = ancestor_offset;
      }
      ptr::write(&mut self.data[offset], element);
    }
  }

  fn sift_down(&mut self, offset: usize) {
    let mut child_offset = self.sibling_offset(offset);
    if child_offset >= self.data.len() {
      return;
    }
    let mut next_child_offset = self.child_offset(child_offset);
    while next_child_offset < self.data.len() {
      child_offset = next_child_offset;
      next_child_offset = self.child_offset(child_offset);
    }
    while child_offset != offset {
      if self.data[offset] < self.data[child_offset] {
        self.data.swap(offset, child_offset);
        self.valences[child_offset] = !self.valences[child_offset];
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
