use std::sync::atomic::{AtomicIsize, Ordering};

use criterion::{criterion_group, black_box, Criterion, BenchmarkId};
use fast_counter::{ make_new_padded_counter, CachePadded };

const TEST_SIZES: [usize; 5] = [1, 2, 4, 8, 16];

fn generate_test_data(count: usize) -> Vec<CachePadded<AtomicIsize>> {
    (0..count)
        .into_iter()
        .map(|_| make_new_padded_counter())
        .collect()
}

fn simple_sum(data: &Vec<CachePadded<AtomicIsize>>) -> isize {
    data.iter().map(|c| c.value.load(Ordering::Relaxed)).sum()
}


fn unroll_sum(data: &Vec<CachePadded<AtomicIsize>>) -> isize {
    let mut result = 0;
    let mut iter = data.chunks_exact(8);

    while let Some(chunk) = iter.next() {
        result += chunk.iter().map(|c| c.value.load(Ordering::Relaxed)).sum::<isize>();
    }

    let iter = iter.remainder();

    result += iter.iter().map(|c| c.value.load(Ordering::Relaxed)).sum::<isize>();

    result
}

fn unroll_four_sum(data: &Vec<CachePadded<AtomicIsize>>) -> isize {
    let mut result = 0;
    let mut iter = data.chunks_exact(4);

    while let Some(chunk) = iter.next() {
        result += chunk.iter().map(|c| c.value.load(Ordering::Relaxed)).sum::<isize>();
    }

    let iter = iter.remainder();

    result += iter.iter().map(|c| c.value.load(Ordering::Relaxed)).sum::<isize>();

    result
}

unsafe fn hand_unroll_sum(data: &Vec<CachePadded<AtomicIsize>>) -> isize {
    let mut result = 0;
    let mut i = 0;

    if i + 8 <= data.len() {
        loop {
            result += data.get_unchecked(i).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 1).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 2).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 3).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 4).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 5).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 6).value.load(Ordering::Relaxed);
            result += data.get_unchecked(i + 7).value.load(Ordering::Relaxed);

            i += 8;

            if i + 7 >= data.len() {
                return result;
            }
        }
    } 

    if i + 4 <= data.len() {
        result += data.get_unchecked(i).value.load(Ordering::Relaxed);
        result += data.get_unchecked(i + 1).value.load(Ordering::Relaxed);
        result += data.get_unchecked(i + 2).value.load(Ordering::Relaxed);
        result += data.get_unchecked(i + 3).value.load(Ordering::Relaxed);

        return result;
    } 

    while i < data.len() {
        result += data.get_unchecked(i).value.load(Ordering::Relaxed);
        i += 1;
    }

    result
}


fn unsafe_sum(data: &Vec<CachePadded<AtomicIsize>>) -> isize {
    let data = unsafe { std::mem::transmute::<&Vec<CachePadded<AtomicIsize>>, &Vec<CachePadded<isize>>>(data) };

    data.iter().map(|c| c.value).sum()
}

const ITER: usize = 10_000;

fn sum_atomics(c: &mut Criterion) {
    let mut counter_group = c.benchmark_group("atomic_sum");

    for cores in TEST_SIZES {
        let test_data = generate_test_data(cores);

        counter_group.bench_with_input(
            BenchmarkId::new("simple_loop", cores), 
            &test_data, 
            |b, to_count| {
                b.iter(|| {
                    for _ in 0..ITER {
                        simple_sum(black_box(to_count));
                    }
                })
            });

        counter_group.bench_with_input(
            BenchmarkId::new("unrolled_loop", cores), 
            &test_data, 
            |b, to_count| {
                b.iter(|| {
                    for _ in 0..ITER {
                        unroll_sum(black_box(to_count));
                    }
                })
            });

        counter_group.bench_with_input(
            BenchmarkId::new("hand_unrolled_loop", cores), 
            &test_data, 
            |b, to_count| {
                b.iter(|| {
                    for _ in 0..ITER {
                        unsafe { hand_unroll_sum(black_box(to_count)); }
                    }
                })
            });

        counter_group.bench_with_input(
            BenchmarkId::new("unsafe_loop", cores), 
            &test_data, 
            |b, to_count| {
                b.iter(|| {
                    for _ in 0..ITER {
                        unsafe_sum(black_box(to_count));
                    }
                })
            });
    }
    counter_group.finish();
}

criterion_group!(counting, sum_atomics);
