use num_bigint::{BigUint, ToBigUint};
use rand::{Rng, SeedableRng};
use rand::distributions::Standard;
use rand::rngs::StdRng;
use rand::distributions::Distribution;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::mem;
use std::ops::Add;
use std::sync::atomic;

#[derive(Clone, Copy, Debug)]
pub struct ComparisonCountedI32(i32);

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

pub struct StandardComparisonCountedI32Distribution {}

impl Distribution<ComparisonCountedI32> for StandardComparisonCountedI32Distribution {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ComparisonCountedI32 {
    return rng.sample::<i32, rand::distributions::Standard>(rand::distributions::Standard).into()
  }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct ComparisonCounts {
  pub partial_ord: usize,
  pub ord: usize,
  pub eq: usize,
  pub neq: usize,
}

impl ComparisonCounts {
  pub fn now() -> Self {
    ComparisonCounts {
      partial_ord: PARTIAL_ORD_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      ord: ORD_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      eq: PARTIAL_EQ_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
      neq: PARTIAL_NEQ_COMPARISON_COUNTER.load(atomic::Ordering::SeqCst),
    }
  }

  pub fn take_difference(&mut self) {
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

pub trait HasDistribution: Sized {
  type Dist: rand::distributions::Distribution<Self>;

  fn distribution() -> Self::Dist;
}

impl HasDistribution for i32 {
  type Dist = Standard;

  fn distribution() -> Standard { Standard }
}

impl HasDistribution for i128 {
  type Dist = Standard;

  fn distribution() -> Standard { Standard }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct U32768 {
  value: BigUint,
}

pub struct U32768Distribution {}

impl rand::distributions::Distribution<U32768> for U32768Distribution {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> U32768 {
    let mut result = BigUint::default();
    for _ in 0..256 {
      result <<= 128;
      result += rng.sample::<u128, Standard>(Standard).to_biguint().unwrap();
    }
    U32768 { value: result, }
  }
}

impl HasDistribution for U32768 {
  type Dist = U32768Distribution;

  fn distribution() -> U32768Distribution { U32768Distribution{} }
}

/// Returns a sequence of `size` elements generated deterministically from an
/// RNG with a fixed seed.
pub fn get_values<T: HasDistribution>(size: usize) -> Vec<T> {
  let seed: [u8; 32] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
  let mut rng: StdRng = SeedableRng::from_seed(seed);
  rng.sample_iter(&T::distribution()).take(size).collect()
}

// /// Runs benchmarks over sequences of size `$sizes`. For example:
// ///
// /// ```rust,noexec
// /// fn bench_something(c: &mut Criterion) {
// ///   do_comparison_counted_i32_bench!(c, WeakHeap::new, 1..127);
// /// }
// /// ```
// ///
// /// Benchmarks will be run using a customized version of `i32` that counts the
// /// number of comparison operations performed.
// #[macro_export]
// macro_rules! do_comparison_counted_i32_bench {
//   ($bencher: ident, $heap_factory: expr, $sizes: expr) => ({
//     use $crate::benches::{ComparisonCountedI32/*, ComparisonCounts*/};
//     use std::collections::HashMap;
//     let mut all_values = HashMap::new();
//     let mut all_sorted = HashMap::new();
//     for size in $sizes {
//       let values: Vec<ComparisonCountedI32> =
//         $crate::benches::get_values::<i32>(size).into_iter().map(|x| x.into()).collect();
//       let sorted = {
//         let mut v = values.clone();
//         v.sort_by(|x, y| y.cmp(x));
//         v
//       };
//       all_values.insert(size, values);
//       all_sorted.insert(size, sorted);
//     }

//     $bencher.bench_function_over_inputs(
//       &format!("sort (counting comparisons) {}", stringify!($heap_factory)),
//       move |b: &mut criterion::Bencher, size: &usize| {
//         // let mut all_counts = Vec::new();
//         // let mut iterations = 0;
//         let values = all_values.get(size).unwrap();
//         let sorted = all_sorted.get(size).unwrap();
//         b.iter(|| {
//           // iterations += 1;
//           // let mut counts = ComparisonCounts::now();
//           let mut heap = $heap_factory();
//           for v in values {
//             heap.push(*v);
//           }
//           let mut heap_sorted = Vec::with_capacity(heap.len());
//           while let Some(x) = heap.pop() {
//             heap_sorted.push(x);
//           }
//           // counts.take_difference();
//           // all_counts.push(counts);
//           assert_eq!(heap_sorted, *sorted);
//         });
//         // let total_counts = all_counts.into_iter().fold(
//         //   ComparisonCounts::default(), |acc, x| acc + x);
//         // println!("\ndo_comparison_counted_bench({}, {}) partial_ord {:.2}, ord {:.2}, eq {:.2}, ne {:.2}",
//         //          stringify!($heap_factory), size,
//         //          (total_counts.partial_ord as f32) / ((size * iterations) as f32),
//         //          (total_counts.ord as f32) / ((size * iterations) as f32),
//         //          (total_counts.eq as f32) / ((size * iterations) as f32),
//         //          (total_counts.neq as f32) / ((size * iterations) as f32));
//       }, $sizes);
//   })
// }

/// Runs benchmarks over sequences of size `$sizes`.
#[macro_export]
macro_rules! do_bench {
  ($bencher: ident, $type: ty, $heap_factory: expr, $sizes: expr) => ({
    use std::collections::HashMap;
    let mut all_values = HashMap::new();
    let mut all_sorted = HashMap::new();
    for size in $sizes {
      let values: Vec<$type> = $crate::benches::get_values(size);
      let sorted = {
        let mut v = values.clone();
        v.sort_by(|x, y| y.cmp(x));
        v
      };
      all_values.insert(size, values);
      all_sorted.insert(size, sorted);
    }

    $bencher.bench_function_over_inputs(
      &format!("sort ({}) {}", stringify!($type), stringify!($heap_factory)),
      move |b: &mut criterion::Bencher, size: &usize| {
        let values = all_values.get(&size).unwrap();
        let sorted = all_sorted.get(size).unwrap();
        b.iter(|| {
          let mut heap = $heap_factory();
          for v in values {
            heap.push(v.clone());
          }
          let mut heap_sorted = Vec::with_capacity(heap.len());
          while let Some(x) = heap.pop() {
            heap_sorted.push(x);
          }
          assert_eq!(heap_sorted, *sorted);
        });
      }, $sizes);
  })
}

pub mod tiny;
// pub mod small;
// pub mod medium;
// pub mod large;
