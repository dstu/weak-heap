#![feature(test)]

extern crate bit_vec;
#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

use bit_vec::BitVec;
use std::cmp::Ord;
use std::fmt;
use std::mem;

const USIZE_BITS: u32 = (mem::size_of::<usize>() * 8) as u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EdgeValence {
  Standard,
  Flipped,
}

impl From<bool> for EdgeValence {
  fn from(b: bool) -> EdgeValence {
    if b {
      EdgeValence::Flipped
    } else {
      EdgeValence::Standard
    }
  }
}

impl Into<bool> for EdgeValence {
  fn into(self) -> bool {
    match self {
      EdgeValence::Flipped => true,
      EdgeValence::Standard => false,
    }
  }
}

impl Into<usize> for EdgeValence {
  fn into(self) -> usize {
    match self {
      EdgeValence::Flipped => 1,
      EdgeValence::Standard => 0,
    }
  }
}

fn lg_ceil(n: usize) -> usize {
  if n == 0 {
    return 1;
  }
  let trailing_zeros = n.trailing_zeros();
  let leading_zeros = n.leading_zeros();
  if trailing_zeros + leading_zeros + 1 == USIZE_BITS {
    // Perfect power of 2.
    trailing_zeros as usize
  } else {
    (USIZE_BITS - leading_zeros + 1) as usize
  }
}

struct ValenceVec(BitVec);

impl ValenceVec {
  fn new() -> Self {
    ValenceVec(BitVec::new())
  }

  fn with_capacity(cap: usize) -> Self {
    ValenceVec(BitVec::with_capacity(cap))
  }

  fn get(&self, index: usize) -> Option<EdgeValence> {
    self.0.get(index).map(|b| b.into())
  }

  fn flip(&mut self, index: usize) {
    let value = self.0.get(index).unwrap();
    self.0.set(index, !value);
  }

  fn push(&mut self, value: EdgeValence) {
    self.0.grow(1, value.into());
  }

  fn pop(&mut self) -> Option<EdgeValence> {
    self.0.pop().map(|b| b.into())
  }
}

impl fmt::Debug for ValenceVec {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "ValenceVec[")?;
    let mut items = (0..self.0.len()).map(|n| self.0.get(n).unwrap());
    if let Some(b) = items.next() {
      write!(f, "{:?}", EdgeValence::from(b))?;
      while let Some(b) = items.next() {
        write!(f, ", {:?}", EdgeValence::from(b))?;
      }
    }
    write!(f, "]")
  }
}

#[derive(Debug)]
pub struct WeakHeap<T: fmt::Debug + Ord> {
  valences: ValenceVec,
  data: Vec<T>,
  insert_buffer: Vec<T>,
  max_insert_index: Option<usize>,
}

impl<T: fmt::Debug + Ord> WeakHeap<T> {
  pub fn new() -> Self {
    WeakHeap {
      valences: ValenceVec::new(),
      data: Vec::new(),
      insert_buffer: Vec::new(),
      max_insert_index: None,
    }
  }

  pub fn with_capacity(cap: usize) -> Self {
    WeakHeap {
      valences: ValenceVec::with_capacity(cap),
      data: Vec::with_capacity(cap),
      insert_buffer: Vec::with_capacity(cap),
      max_insert_index: None,
    }
  }

  pub fn len(&self) -> usize {
    self.data.len() + self.insert_buffer.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty() && self.insert_buffer.is_empty()
  }

  pub fn peek(&self) -> Option<&T> {
    match (self.max_insert_index.and_then(|n| self.insert_buffer.get(n)),
           self.data.first()) {
      (Some(d1), Some(d2)) =>
        if d1 > d2 {
          Some(d1)
        } else {
          Some(d2)
        },
      (Some(d), None) => Some(d),
      (None, Some(d)) => Some(d),
      (None, None) => None,
    }
  }

  pub fn push(&mut self, value: T) {
    // println!("pushing: {:?}", value);
    if self.insert_buffer.len() == 1 + lg_ceil(self.data.len()) {
      // println!("flushing on input value: {:?}", value);
      self.insert_buffer.push(value);
      self.flush_insert_buffer();
    } else {
      self.max_insert_index = match self.max_insert_index {
        None => Some(0),
        Some(n) =>
          if value > self.insert_buffer[n] {
            Some(self.insert_buffer.len())
          } else {
            Some(n)
          },
      };
      self.insert_buffer.push(value);
    }
  }

  pub fn flush_insert_buffer(&mut self) {
    if self.insert_buffer.is_empty() {
      return;
    }
    let mut right = self.data.len() + self.insert_buffer.len() - 2;
    let mut left = usize::max(self.data.len(), right / 2);
    for _ in 0..self.insert_buffer.len() {
      self.valences.push(EdgeValence::Standard);
    }
    self.data.append(&mut self.insert_buffer);
    self.max_insert_index = None;
    while right > left + 1 {
      left /= 2;
      right /= 2;
      for offset in left..(right + 1) {
        self.sift_down(offset);
      }
    }
    if left != 0 {
      let ancestor_offset = self.distinguished_ancestor_offset(left);
      self.sift_down(ancestor_offset);
      self.sift_up(ancestor_offset);
    }
    if right != 0 {
      let ancestor_offset = self.distinguished_ancestor_offset(right);
      self.sift_down(ancestor_offset);
      self.sift_up(ancestor_offset);
    }
  }

  pub fn pop(&mut self) -> Option<T> {
    if self.data.is_empty() {
      if let Some(n) = self.max_insert_index {
        let value = self.insert_buffer.swap_remove(n);
        self.update_max_insert_index();
        Some(value)
      } else {
        None
      }
    } else {
      if let Some(n) = self.max_insert_index {
        if self.insert_buffer[n] > *self.data.first().unwrap() {
          let value = self.insert_buffer.swap_remove(n);
          self.update_max_insert_index();
          return Some(value)
        }
      }
      if self.len() == 1 {
        self.valences.pop();
        self.data.pop()
      } else {
        let x = self.data.swap_remove(0);
        self.valences.pop();
        self.sift_down(0);
        Some(x)
      }
    }
  }

  fn update_max_insert_index(&mut self) {
    self.max_insert_index = self.insert_buffer.iter().enumerate()
      .max_by(|&(_, ref a), &(_, ref b)| a.cmp(b)).map(|(index, _)| index);
  }

  fn child_offset(&self, offset: usize) -> usize {
    let valence = self.valences.get(offset).expect("no valence available for child offset");
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(Into::<usize>::into(valence)))
      .expect("child offset computation overflow")
  }

  fn sibling_offset(&self, offset: usize) -> usize {
    let valence = self.valences.get(offset).expect("no valence available for sibling offset");
    offset.checked_mul(2)
      .and_then(|n| n.checked_add(1 - Into::<usize>::into(valence)))
      .expect("sibling offset computation overflow")
  }

  fn distinguished_ancestor_offset(&self, mut offset: usize) -> usize {
    if offset == 0 {
      panic!("root has no ancestor");
    } else {
      let mut parent_offset = offset / 2;
      let mut parent_valence = self.valences.get(parent_offset).expect("no parent valence available");
      while (offset & 1) == Into::<usize>::into(parent_valence) {
        offset = parent_offset;
        parent_offset /= 2;
        parent_valence = self.valences.get(parent_offset).expect("no parent valence available");
      }
      parent_offset
    }
  }

  fn sift_up(&mut self, mut offset: usize) {
    while offset > 0 {
      let ancestor_offset = self.distinguished_ancestor_offset(offset);
      if self.join(ancestor_offset, offset) {
        break;
      }
      offset = ancestor_offset;
    }
  }

  fn join(&mut self, a: usize, b: usize) -> bool {
    // println!("join({}, {})", a, b);
    // println!("join start: {:?}", self);
    if self.data[a] < self.data[b] {
      // println!("join: swapping offsets {} and {}", a, b);
      self.data.swap(a, b);
      self.valences.flip(b);
      // println!("join end: {:?}", self);
      false
    } else {
      // println!("join end: {:?}", self);
      true
    }
  }

  fn sift_down(&mut self, offset: usize) {
    let mut child_offset = self.sibling_offset(offset);
    if child_offset >= self.len() {
      return;
    }
    let mut next_child_offset = self.child_offset(child_offset);
    while next_child_offset < self.data.len() {
      // println!("sift_down: data.len = {}, valences.len = {}, child_offset = {}, next_child_offset = {}",
      //          self.data.len(), self.valences.0.len(), child_offset, next_child_offset);
      child_offset = next_child_offset;
      next_child_offset = self.child_offset(child_offset);
    }
    while child_offset != offset {
      self.join(offset, child_offset);
      child_offset /= 2;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::WeakHeap;

  use rand::{Rng, SeedableRng, StdRng};
  use std::collections::BinaryHeap;
  use test::Bencher;

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
      // println!("after push: {:?}", t);
      assert_eq!(t.peek().map(|n| *n), Some(*x));
    }
    assert_eq!(values.len(), t.len());
    let mut values_iter = values.iter().rev();
    while let Some(x) = t.pop() {
      // println!("after pop: {:?}", t);
      assert_eq!(x, *values_iter.next().unwrap());
    }
    assert!(t.is_empty());
  }

  fn get_values(size: usize) -> Vec<i32> {
    let seed: &[_] = &[1, 2, 3, 4];
    let mut rng: StdRng = SeedableRng::from_seed(seed);
    (0..size).map(|_| rng.gen::<i32>()).collect()
  }

  #[inline]
  fn bench_weak(b: &mut Bencher, size: usize) {
    let values = get_values(size);
    let sorted = {
      let mut v = values.clone();
      v.sort_by(|x, y| y.cmp(x));
      v
    };

    b.iter(|| {
      let mut heap = WeakHeap::new();
      for v in &values {
        heap.push(*v);
        // println!("pushed onto heap: {}", *v);
      }
      let mut heap_sorted = Vec::new();
      heap_sorted.reserve(heap.len());
      loop {
        match heap.pop() {
          None => break,
          Some(x) => {
            // println!("popped from heap: {}", x);
            heap_sorted.push(x);
          },
        }
      }
      assert_eq![heap_sorted, sorted];
    });
  }

  #[bench]
  fn bench_weak_tiny(b: &mut Bencher) {
    bench_weak(b, 10);
  }

  #[bench]
  fn bench_weak_small(b: &mut Bencher) {
    bench_weak(b, 100);
  }

  #[bench]
  fn bench_weak_medium(b: &mut Bencher) {
    bench_weak(b, 10000);
  }

  #[bench]
  fn bench_weak_large(b: &mut Bencher) {
      bench_weak(b, 100000);
  }

  // large and ludicrous tests commented out for now.

  // #[bench]
  // fn bench_weak_ludicrous(b: &mut Bencher) {
  //     bench_weak(b, 10000000000);
  // }

  #[inline]
  fn bench_builtin(b: &mut Bencher, size: usize) {
    let values = get_values(size);
    let sorted = {
      let mut v = values.clone();
      v.sort_by(|x, y| y.cmp(x));
      v
    };
    b.iter(|| {
      let mut heap = BinaryHeap::new();
      for v in &values {
        heap.push(*v);
      }
      let mut heap_sorted = Vec::new();
      heap_sorted.reserve(heap.len());
      loop {
        match heap.pop() {
          None => break,
          Some(x) => heap_sorted.push(x),
        }
      }
      assert_eq![heap_sorted, sorted];
    });
  }

  #[bench]
  fn bench_builtin_tiny(b: &mut Bencher) {
    bench_builtin(b, 10);
  }

  #[bench]
  fn bench_builtin_small(b: &mut Bencher) {
    bench_builtin(b, 100);
  }

  #[bench]
  fn bench_builtin_medium(b: &mut Bencher) {
    bench_builtin(b, 10000);
  }

  #[bench]
  fn bench_builtin_large(b: &mut Bencher) {
      bench_builtin(b, 100000);
  }

  // ludicrous tests commented out for now.

  // #[bench]
  // fn bench_builtin_ludicrous(b: &mut Bencher) {
  //     bench_builtin(b, 10000000000);
  // }
}
