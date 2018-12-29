use criterion::criterion_main;
use weak_heap::benches::small;

criterion_main!(small::i128_sort_binary);
