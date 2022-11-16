use criterion::criterion_main;

mod count;
mod incrementer;

criterion_main! {
    count::counting,
    incrementer::benches,
}
