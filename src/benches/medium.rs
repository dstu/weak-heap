/// Criterion group definitions for medium tests.

use std::collections::BinaryHeap;
use crate::WeakHeap;

fn sizes() -> Range<usize> { 256..1024 }

fn bench_i32_sort_binary(c: &mut Criterion) {
  do_bench!(c, i32, BinaryHeap::new, sizes())
}

criterion_group!(i32_sort_binary, bench_i32_sort_binary);

fn bench_i32_sort_weak(c: &mut Criterion) {
  do_bench!(c, i32, WeakHeap::new, sizes())
}

criterion_group!(i32_sort_weak, bench_i32_sort_weak);

fn bench_comparison_counted_i32_sort_binary(c: &mut Criterion) {
  do_comparison_counted_bench!(c, BinaryHeap::new, sizes())
}

criterion_group!(comparison_counted_i32_sort_weak,
                 bench_comparison_counted_i32_sort_weak);

fn bench_comparison_counted_i32_sort_weak(c: &mut Criterion) {
  do_comparison_counted_bench!(c, BinaryHeap::new, sizes())
}

criterion_group!(comparison_counted_i32_sort_weak,
                 bench_comparison_counted_i32_sort_weak);
