use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use origin_exercise::*;

// Test type that is a wrapper around `u32`, and implementing `Orderable`.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct U32(u32);
impl Orderable for U32 {
    fn index(&self) -> u32 {
        self.0
    }
}

// Generates some mostly ordered sequences where the `disorder_window`
// is the size of the chucks that are reversed, in order to simulate mostly ordered sequences.
fn mostly_ordered_sequence(len: usize, disorder_window: usize) -> Vec<U32> {
    let mut seq: Vec<U32> = (0..len as u32).map(U32).collect();
    for chunk in seq.chunks_mut(disorder_window) {
        chunk.reverse();
    }
    seq
}

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderer_push");

    for size in [10_000, 100_000, 1_000_000, 10_000_000] {
        group.throughput(Throughput::Elements(size as u64));

        // benchmark that runs on ordered sequences.
        //      -> this one should be running fast and not use the buffer at all.
        group.bench_with_input(BenchmarkId::new("ordered", size), &size, |b, &size| {
            b.iter(|| {
                let mut orderer: Orderer<U32> = Orderer::new();
                for i in 0..size as u32 {
                    black_box(orderer.push(U32(i)));
                }
            });
        });

        // benchmark that runs on mostly ordered sequences with a chunk size of 16.
        //      -> this is the realistic case according to the README.md.
        let seq = mostly_ordered_sequence(size, 16);
        group.bench_with_input(BenchmarkId::new("mostly_ordered_w16", size), &seq, |b, seq| {
            b.iter(|| {
                let mut orderer: Orderer<U32> = Orderer::new();
                for v in seq.iter().cloned() {
                    black_box(orderer.push(v));
                }
            });
        });

        // benchmark that runs on reversed sequences.
        //      -> this is the worse case because everything goes
        //         to the buffer waiting to the last number of the sequence.
        let reversed: Vec<U32> = (0..size as u32).rev().map(U32).collect();
        group.bench_with_input(BenchmarkId::new("reversed", size), &reversed, |b, seq| {
            b.iter(|| {
                let mut orderer: Orderer<U32> = Orderer::new();
                for v in seq.iter().cloned() {
                    black_box(orderer.push(v));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_push);
criterion_main!(benches);