use std::hint::black_box;
use origin_exercise::*;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct U32(u32);
impl Orderable for U32 {
    fn index(&self) -> u32 { self.0 }
}

fn mostly_ordered_sequence(len: usize, disorder_window: usize, start: u32) -> Vec<U32> {
    let mut seq: Vec<U32> = (start..start + len as u32).map(U32).collect();
    for chunk in seq.chunks_mut(disorder_window) {
        chunk.reverse();
    }
    seq
}

fn main() {
    let size = 1_000;
    let seq = mostly_ordered_sequence(size, 16, 1);

    // an iteration of 1000 that pushes values in the orderer structure to produce a flamegraph.
    for _ in 0..1000 {
        let mut orderer: Orderer<U32> = Orderer::new();
        orderer.push(U32(0), |e| { black_box(e); });
        for v in seq.iter().cloned() {
            orderer.push(v, |e| { black_box(e); });
        }
        black_box(&orderer);
    }
}