use criterion::criterion_main;
use weak_heap::benches::medium;

criterion_main!(medium::i128_sort_weak);
