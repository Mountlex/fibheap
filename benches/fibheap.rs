use std::cmp::Ordering;

use binary_heap_plus::BinaryHeap;
use compare::Compare;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fibheap::FibHeap;
use rand::{distributions::Uniform, Rng};

fn fibheap(n: usize) {
    let mut heap = FibHeap::new();

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, n);

    let vals: Vec<(usize, usize)> = (0..n).map(|i: usize| (i, rng.sample(&range))).collect();

    for (item, key) in vals {
        heap.insert(item, key);
    }

    while let Some(_) = heap.pop_min() {}
}

fn binheap(n: usize) {
    let mut heap = BinaryHeap::new_min();

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, n);

    let vals: Vec<(usize, usize)> = (0..n).map(|i: usize| (i, rng.sample(&range))).collect();

    for (item, _) in vals.iter() {
        heap.push(item);
    }

    while let Some(_) = heap.pop() {}
}

fn fibheap_decrease(n: usize) {
    let mut heap = FibHeap::new();

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, n);

    let vals: Vec<(usize, usize)> = (0..n).map(|i: usize| (i, rng.sample(&range))).collect();

    for (item, _) in vals.iter() {
        heap.insert(*item, usize::MAX);
    }

    for (item, key) in vals {
        heap.decrease_key(&item, key);
    }

    while let Some(_) = heap.pop_min() {}
}

struct Comparator {
    vals: Vec<usize>,
}

impl Compare<usize> for Comparator {
    fn compare(&self, l: &usize, r: &usize) -> Ordering {
        self.vals[*l].cmp(&self.vals[*r])
    }
}

fn binheap_decrease(n: usize) {
    let mut cmp_vals = vec![usize::MAX; n];

    let mut heap = BinaryHeap::from_vec_cmp(
        vec![],
        Comparator {
            vals: cmp_vals.clone(),
        },
    );

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, n);

    let vals: Vec<(usize, usize)> = (0..n).map(|i: usize| (i, rng.sample(&range))).collect();

    for (item, _) in vals.iter() {
        heap.push(*item);
    }
    for (item, key) in vals.iter() {
        cmp_vals[*item] = *key;
        heap.replace_cmp(Comparator {
            vals: cmp_vals.clone(),
        });
    }

    while let Some(_) = heap.pop() {}
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 1000", |b| b.iter(|| fibheap(black_box(1000))));
    c.bench_function("bin 1000", |b| b.iter(|| binheap(black_box(1000))));

    c.bench_function("fib_dec 1000", |b| {
        b.iter(|| fibheap_decrease(black_box(1000)))
    });
    c.bench_function("bin_dec 1000", |b| {
        b.iter(|| binheap_decrease(black_box(1000)))
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = criterion_benchmark
}
criterion_main!(benches);
