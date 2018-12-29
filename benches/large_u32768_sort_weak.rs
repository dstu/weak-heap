use criterion::criterion_main;
use weak_heap::benches::large;

criterion_main!(large::u32768_sort_weak);
