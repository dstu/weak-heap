use super::WeakHeap;
use super::tests;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::mem;
use std::ops::Add;
use std::sync::atomic;
use test::Bencher;

static TINY_SIZES: &[usize] = &[1, 2, 7, 8, 9, 10, 20, 30, 31, 32, 33];
static SMALL_SIZES: &[usize] = &[100, 127, 128, 129, 150, 200, 254, 255, 256, 257, 258];
static MEDIUM_SIZES: &[usize] = &[10000, 20000, 32767, 32768, 32769, 50000];
static LARGE_SIZES: &[usize] = &[1048575, 1048576, 1048577];

#[derive(Clone, Copy, Debug)]
struct ComparisonCountedI32(i32);

static PARTIAL_ORD_COMPARISON_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
impl PartialOrd for ComparisonCountedI32 {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    PARTIAL_ORD_COMPARISON_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    self.0.partial_cmp(&other.0)
  }
}

static ORD_COMPARISON_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
impl Ord for ComparisonCountedI32 {
  fn cmp(&self, other: &Self) -> Ordering {
    ORD_COMPARISON_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    self.0.cmp(&other.0)
  }
}

static PARTIAL_EQ_COMPARISON_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
static PARTIAL_NEQ_COMPARISON_COUNTER: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
impl PartialEq for ComparisonCountedI32 {
  fn eq(&self, other: &Self) -> bool {
    PARTIAL_EQ_COMPARISON_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    self.0.eq(&other.0)
  }

  fn ne(&self, other: &Self) -> bool {
    PARTIAL_NEQ_COMPARISON_COUNTER.fetch_add(1, atomic::Ordering::SeqCst);
    self.0.ne(&other.0)
  }
}

impl Eq for ComparisonCountedI32 {}

impl From<i32> for ComparisonCountedI32 {
  fn from(x: i32) -> Self {
    ComparisonCountedI32(x)
  }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
struct ComparisonCounts {
  partial_ord: usize,
  ord: usize,
  eq: usize,
  neq: usize,
}

impl ComparisonCounts {
  fn now() -> Self {
    ComparisonCounts {
      partial_ord: PARTIAL_ORD_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      ord: ORD_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      eq: PARTIAL_EQ_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      neq: PARTIAL_NEQ_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
    }
  }

  fn take_difference(&mut self) {
    let mut now = ComparisonCounts::now();
    assert!(now.partial_ord >= self.partial_ord);
    now.partial_ord -= self.partial_ord;
    assert!(now.ord >= self.ord);
    now.ord -= self.ord;
    assert!(now.eq >= self.eq);
    now.eq -= self.eq;
    assert!(now.neq >= self.neq);
    now.neq -= self.neq;
    mem::swap(&mut now, self);
  }
}

impl Add for ComparisonCounts {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    ComparisonCounts {
      partial_ord: self.partial_ord + rhs.partial_ord,
      ord: self.ord + rhs.ord,
      eq: self.eq + rhs.eq,
      neq: self.neq + rhs.neq,
    }
  }
}

macro_rules! do_bench {
  ($bencher: ident, $heap_factory: expr, $size: expr) => ({
      let size: usize = $size;
      let values: Vec<ComparisonCountedI32> =
        tests::get_values(size).into_iter().map(|x| x.into()).collect();
      let sorted = {
        let mut v = values.clone();
        v.sort_by(|x, y| y.cmp(x));
        v
      };

      let mut all_counts = Vec::new();
      let mut iterations = 0;
      $bencher.iter(|| {
        iterations += 1;
        let mut counts = ComparisonCounts::now();
        let mut heap = $heap_factory();
        for v in &values {
          heap.push(*v);
        }
        let mut heap_sorted = Vec::with_capacity(heap.len());
        while let Some(x) = heap.pop() {
          heap_sorted.push(x);
        }
        counts.take_difference();
        all_counts.push(counts);
        assert_eq![heap_sorted, sorted];
      });
    let total_counts = all_counts.into_iter().fold(
      ComparisonCounts::default(), |acc, x| acc + x);
    println!("\ndo_bench({}, {}) partial_ord {:.2}, ord {:.2}, eq {:.2}, ne {:.2}",
             stringify!($heap_factory), size,
             (total_counts.partial_ord as f32) / ((size * iterations) as f32),
             (total_counts.ord as f32) / ((size * iterations) as f32),
             (total_counts.eq as f32) / ((size * iterations) as f32),
             (total_counts.neq as f32) / ((size * iterations) as f32));
    total_counts
  })
}

macro_rules! do_bench_inserts {
  ($bencher: ident, $heap_factory: expr, $size: expr) => ({
    let size: usize = $size;
    let values: Vec<ComparisonCountedI32> =
      tests::get_values(size).into_iter().map(|x| x.into()).collect();

    let mut all_counts = Vec::new();
    let mut iterations = 0;
    $bencher.iter(|| {
      iterations += 1;
      let mut counts = ComparisonCounts::now();
      let mut heap = $heap_factory();
      for v in &values {
        heap.push(*v);
      }
      counts.take_difference();
      all_counts.push(counts);
    });
    let total_counts = all_counts.into_iter().fold(
      ComparisonCounts::default(), |acc, x| acc + x);
    println!("\ndo_bench_inserts({}, {}) partial_ord {:.2}, ord {:.2}, eq {:.2}, ne {:.2}",
             stringify!($heap_factory), size,
             (total_counts.partial_ord as f32) / ((size * iterations) as f32),
             (total_counts.ord as f32) / ((size * iterations) as f32),
             (total_counts.eq as f32) / ((size * iterations) as f32),
             (total_counts.neq as f32) / ((size * iterations) as f32));
    total_counts
  })
}

macro_rules! do_bench_removals {
  ($bencher: ident, $heap_factory: expr, $size: expr) => ({
    let size: usize = $size;
    let values: Vec<ComparisonCountedI32> =
      tests::get_values(size).into_iter().map(|x| x.into()).collect();
    let sorted = {
      let mut v = values.clone();
      v.sort_by(|x, y| y.cmp(x));
      v
    };

    let mut all_counts = Vec::new();
    let mut iterations = 0;
    $bencher.iter(|| {
      iterations += 1;
      let mut heap = $heap_factory();
      for v in &values {
        heap.push(*v);
      }
      let mut receptacle = Vec::with_capacity(size);
      let mut counts = ComparisonCounts::now();
      while let Some(x) = heap.pop() {
        receptacle.push(x);
      }
      counts.take_difference();
      assert_eq!(receptacle, sorted);
      all_counts.push(counts);
    });
    let total_counts = all_counts.into_iter().fold(
      ComparisonCounts::default(), |acc, x| acc + x);
    println!("\ndo_bench_removals({}, {}) partial_ord {:.2}, ord {:.2}, eq {:.2}, ne {:.2}",
             stringify!($heap_factory), size,
             (total_counts.partial_ord as f32) / ((size * iterations) as f32),
             (total_counts.ord as f32) / ((size * iterations) as f32),
             (total_counts.eq as f32) / ((size * iterations) as f32),
             (total_counts.neq as f32) / ((size * iterations) as f32));
    total_counts
  })
}

#[bench]
fn bench_00_weak_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_01_weak_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_02_weak_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_03_weak_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_00_weak_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench_inserts!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_01_weak_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench_inserts!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_02_weak_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench_inserts!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_03_weak_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench_inserts!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_removals_00_weak_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench_removals!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_removals_01_weak_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench_removals!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_removals_02_weak_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench_removals!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_removals_03_weak_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench_removals!(b, WeakHeap::new, *size);
  }
}

#[bench]
fn bench_00_builtin_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_01_builtin_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_02_builtin_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_03_builtin_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_00_binary_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench_inserts!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_01_binary_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench_inserts!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_02_binary_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench_inserts!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_inserts_03_builtin_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench_inserts!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_removals_00_binary_tiny(b: &mut Bencher) {
  for size in TINY_SIZES {
    do_bench_removals!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_removals_01_binary_small(b: &mut Bencher) {
  for size in SMALL_SIZES {
    do_bench_removals!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_removals_02_binary_medium(b: &mut Bencher) {
  for size in MEDIUM_SIZES {
    do_bench_removals!(b, BinaryHeap::new, *size);
  }
}

#[bench]
fn bench_removals_03_builtin_large(b: &mut Bencher) {
  for size in LARGE_SIZES {
    do_bench_removals!(b, BinaryHeap::new, *size);
  }
}
