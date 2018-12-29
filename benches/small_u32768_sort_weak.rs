use criterion::criterion_main;
use weak_heap::benches::small;

criterion_main!(small::u32768_sort_weak);
