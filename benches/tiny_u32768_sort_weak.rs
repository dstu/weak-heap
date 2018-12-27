use criterion::criterion_main;
use weak_heap::benches::tiny;

criterion_main!(tiny::u32768_sort_weak);
