use criterion::criterion_main;
use weak_heap::benches::medium;

criterion_main!(medium::u32768_sort_binary);
