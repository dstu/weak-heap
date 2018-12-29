use criterion::criterion_main;
use weak_heap::benches::small;

criterion_main!(small::i32_sort_weak);
