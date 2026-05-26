## Exercise

In `src/lib.rs`, the `Orderer` struct implements some logic for buffering and ordering elements that implement the `Orderable` trait.
We want you to optimize this code for performance, modifying the implemention and, if justified, the interface of the service as well.
In this case, understand as performance the maximum supported call rate for `push` assuming `flush` is called every few seconds.
You can also assume the elements will be mostly ordered (but not completely!).

Some directives:
* Justify all of your choices.
* Consider the time spent on allocations.
* Follow software development practices as you would for a regular, professional project (except for part where you ship it as a zip file...).
* The exercise is designed to not need any dependencies, if you need to use one, please justify it.

Note that this is an exercise, and we do not intend you to spend more than a couple of hours on it.
If you get stuck please feel free to take some assumptions/simplify the problem, but state the changes clearly and why you made them.

### My changes - Charlène Gros

I have added the `criterion` crate, which provides fairly accurate benchmarks and exactly the data we need: namely, the execution time, which must remain consistent, and the number of calls per second achievable.
I have 12 benchmarks, 3 data set types, and 4 call rates. The data sets are as follows:
- `sorted`: easy case, no use of the buffer; the data added to the structure is already sorted.
- `mostly_sorted`: use case described in the README; chunks of 16 sorted data items arrive, then arrive in reverse order, and so on.
- `reversed`: worst-case scenario; the data always goes into the buffer before being flushed entirely into the structure.

Each of these datasets runs the function to be optimized, `push`, 10k, 100k, 1M, and 10M times.
These values are quite arbitrary, but I chose this range to reach `ms`.

The benchmarks for the initial code are as follows:

| Benchmark | Size | Time | Throughput | Time gain | Flow rate gain |
|-----------|------|------|------------|-----------|----------------|
| ordered | 10k | 127 µs | 78 Melem/s | — | — |
| ordered | 100k | 1.23 ms | 81 Melem/s | — | — |
| ordered | 1M | 12.35 ms | 81 Melem/s | — | — |
| ordered | 10M | 122 ms | 82 Melem/s | — | — |
| mostly_ordered_w16 | 10k | 231 µs | 43 Melem/s | — | — |
| mostly_ordered_w16 | 100k | 2.70 ms | 37 Melem/s | — | — |
| mostly_ordered_w16 | 1M | 31.6 ms | 32 Melem/s | — | — |
| mostly_ordered_w16 | 10M | 368 ms | 27 Melem/s | — | — |
| reversed | 10k | 184 µs | 54 Melem/s | — | — |
| reversed | 100k | 2.26 ms | 44 Melem/s | — | — |
| reversed | 1M | 27.1 ms | 37 Melem/s | — | — |
| reversed | 10M | 319 ms | 31 Melem/s | — | — |

This exercise also made me use the crate `flamegraph`, which confirmed my hypothesis of where the CPU takes time in my program.

#### `push` should not create a new `vec` for each call
The `push` function always returns a new vector, which is often empty in the `mostly_sorted` case, the most common.
To still return the information if required by the caller, I have added a closure parameter, `pipe`, to the `push` and `flush_sequential` functions, changing the return type to `void`.
This optimization avoids many unnecessary memory allocations.

The benchmark below shows improvements in both time and flow rate.
This is because creating an array with every `push` takes time, particularly for allocation, even if the array is empty.

| Benchmark | Size | Time | Throughput | Time gain | Flow rate gain |
|-----------|------|------|------------|-----------|----------------|
| ordered | 10k | 21.0 µs | 477 Melem/s | -83.6% ✓ | +509% ✓ |
| ordered | 100k | 218 µs | 459 Melem/s | -82.4% ✓ | +469% ✓ |
| ordered | 1M | 2.20 ms | 454 Melem/s | -82.2% ✓ | +461% ✓ |
| ordered | 10M | 22.2 ms | 450 Melem/s | -81.9% ✓ | +451% ✓ |
| mostly_ordered_w16 | 10k | 190 µs | 53 Melem/s | -18.1% ✓ | +22.1% ✓ |
| mostly_ordered_w16 | 100k | 2.38 ms | 42 Melem/s | -11.8% ✓ | +13.3% ✓ |
| mostly_ordered_w16 | 1M | 28.1 ms | 36 Melem/s | -11.0% ✓ | +12.4% ✓ |
| mostly_ordered_w16 | 10M | 336 ms | 30 Melem/s | -8.9% ✓ | +9.8% ✓ |
| reversed | 10k | 142 µs | 70 Melem/s | -23.0% ✓ | +29.8% ✓ |
| reversed | 100k | 1.89 ms | 53 Melem/s | -16.5% ✓ | +19.7% ✓ |
| reversed | 1M | 23.6 ms | 42 Melem/s | -12.9% ✓ | +14.7% ✓ |
| reversed | 10M | 287 ms | 35 Melem/s | -9.9% ✓ | +11.1% ✓ |


