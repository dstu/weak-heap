/// Criterion group definitions for small tests.

use criterion::{Criterion, criterion_group};
use std::collections::BinaryHeap;
use std::ops::Range;
use crate::WeakHeap;
use crate::benches::U32768;

fn sizes() -> Range<usize> { 128..256 }

fn bench_i32_sort_binary(c: &mut Criterion) {
  do_bench!(c, i32, BinaryHeap::new, sizes())
}

criterion_group!(i32_sort_binary, bench_i32_sort_binary);

fn bench_i32_sort_weak(c: &mut Criterion) {
  do_bench!(c, i32, WeakHeap::new, sizes())
}

criterion_group!(i32_sort_weak, bench_i32_sort_weak);

fn bench_i128_sort_binary(c: &mut Criterion) {
  do_bench!(c, i128, BinaryHeap::new, sizes())
}

criterion_group!(i128_sort_binary, bench_i128_sort_binary);

fn bench_i128_sort_weak(c: &mut Criterion) {
  do_bench!(c, i128, WeakHeap::new, sizes())
}

criterion_group!(i128_sort_weak, bench_i128_sort_weak);

fn bench_u32768_sort_binary(c: &mut Criterion) {
  do_bench!(c, U32768, BinaryHeap::new, sizes())
}

criterion_group!(u32768_sort_binary, bench_u32768_sort_binary);

fn bench_u32768_sort_weak(c: &mut Criterion) {
  do_bench!(c, U32768, WeakHeap::new, sizes())
}

criterion_group!(u32768_sort_weak, bench_u32768_sort_weak);
