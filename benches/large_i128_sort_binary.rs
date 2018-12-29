use criterion::criterion_main;
use weak_heap::benches::large;

criterion_main!(large::i128_sort_binary);
