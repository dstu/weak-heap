use criterion::criterion_main;
use weak_heap::benches::tiny;

criterion_main!(tiny::i128_sort_binary);
