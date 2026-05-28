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
- `sorted`: easy case, no use of the buffer, the data added to the structure is already sorted.
- `mostly_sorted`: use case described in the README, chunks of 16 sorted data items arrive, then arrive in reverse order, and so on.
- `reversed`: worst-case scenario, the data always goes into the buffer before being flushed entirely into the structure.

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


#### Usage of VeqDeque instead of BTreeSet

This optimization is not in the source code because it is an improvement I tested, but it was not particularly satisfactory.
We perform an insertion into the buffer, which is a `VecDeque`, and this moves all the elements in memory because the `VecDeque` structure is contiguous.
To prevent values from being moved in memory, I wanted to replace the vector with a `BTreeSet`. This tree structure supports fast insertions and does not require moving all values, since the values are scattered throughout memory.
However, the benchmarks showed a regression across most tests.
As memory loads data in blocks, the vec is loaded in its entirety, whereas with the `BTreeSet`, iterating over the data in `flush_sequential` is more expensive because the data must be loaded into memory several times as it is scattered across memory, whereas the `VecDeque` is loaded only once.

#### Add immediate return if a lower index is provided
Since `next_expected_index` is always increasing, storing a lower index in the buffer wastes space.
I therefore chose to handle this specific case by doing nothing silently. My program could have used `panic!()` at this point, but that wasn’t very easy to manage in the benchmark code.

In the benchmarks, it is not straightforward to explain the huge gains (e.g., the +5943% in flow rate), which is strange. I would say this is noise, as it occurs in the worst-case scenario, where everything goes into the buffer.

| Benchmark | Size | Time | Throughput | Time gain | Flow rate gain |
|-----------|------|------|------------|-----------|----------------|
| ordered | 10k | 21.0 µs | 476 Melem/s | ~0% — | ~0% — |
| ordered | 100k | 213 µs | 469 Melem/s | -2.0% ✓ | +2.1% ✓ |
| ordered | 1M | 2.19 ms | 456 Melem/s | ~0% — | ~0% — |
| ordered | 10M | 22.1 ms | 453 Melem/s | ~0% — | ~0% — |
| mostly_ordered_w16 | 10k | 76.7 µs | 130 Melem/s | -59.6% ✓ | +147% ✓ |
| mostly_ordered_w16 | 100k | 785 µs | 127 Melem/s | -67.1% ✓ | +204% ✓ |
| mostly_ordered_w16 | 1M | 8.12 ms | 123 Melem/s | -71.1% ✓ | +247% ✓ |
| mostly_ordered_w16 | 10M | 83.4 ms | 120 Melem/s | -75.2% ✓ | +303% ✓ |
| reversed | 10k | 4.26 µs | 2.35 Gelem/s | -97.0% ✓ | +3225% ✓ |
| reversed | 100k | 43.5 µs | 2.30 Gelem/s | -97.7% ✓ | +4226% ✓ |
| reversed | 1M | 443 µs | 2.26 Gelem/s | -98.1% ✓ | +5210% ✓ |
| reversed | 10M | 4.76 ms | 2.10 Gelem/s | -98.3% ✓ | +5943% ✓ |


#### Use `flush_sequential` only when needed
Previously, `flush_sequential` was called at the end of every `push` call, whether in the fast or slow case.
Rather than doing this systematically, I have added a `min_buffer_index` variable to the `Orderer` structure to keep track of the smallest element in the buffer.
If the element added to the structure is the expected one, `next_expected_index` is incremented, and if `min_buffer_index` is equal to `next_expected_index`, then `flush_sequential` is useful. Otherwise, we add to the buffer, potentially out of order, and update `min_buffer_index` if the new value is lower. There is therefore no longer any need to keep the buffer sorted at all times, we now sort it only during flushes. La structure du buffer est transformee en `Vec` plutot que `VeqDeque` afin de pouvoir utilsier la fonction `sort_unstable`.
To ensure that the sorting and removal of duplicates are not performed multiple times in a row in specific cases where we have, for example, the sequence `0, 1, 4, 7, 6, 8, 3, 2, 5`, which places the two sequences 3, 4 and 6, 7, 8 into the buffer, and waits for the 2, then the 5, to trigger `flush_sequential` twice in a row. Since, in the meantime, no values have been added to the buffer between the 2 and the 5, it remains sorted. Thus, the boolean `is_ordered` is important because it prevents an unnecessary call to the `sort_and_remove_duplicates` function.

The benchmarks appear consistent because `ordered` improves significantly as we never enter `flush_sequential` anymore since the elements arrive in order, `mostly_ordered` is also consistent with a smaller gain, as we do enter it a little. `reversed` is not much more efficient because all the values still end up in the buffer anyway.

| Benchmark | Size | Time | Throughput | Time gain | Flow rate gain |
|-----------|------|------|------------|-----------|----------------|
| ordered | 10k | 4.69 µs | 2.13 Gelem/s | -77.5% ✓ | +345% ✓ |
| ordered | 100k | 48.6 µs | 2.06 Gelem/s | -77.2% ✓ | +338% ✓ |
| ordered | 1M | 508 µs | 1.97 Gelem/s | -76.8% ✓ | +331% ✓ |
| ordered | 10M | 5.39 ms | 1.85 Gelem/s | -75.6% ✓ | +309% ✓ |
| mostly_ordered_w16 | 10k | 43.8 µs | 228 Melem/s | -43.1% ✓ | +75.7% ✓ |
| mostly_ordered_w16 | 100k | 406 µs | 246 Melem/s | -48.2% ✓ | +93.1% ✓ |
| mostly_ordered_w16 | 1M | 3.97 ms | 252 Melem/s | -51.1% ✓ | +104% ✓ |
| mostly_ordered_w16 | 10M | 40.7 ms | 245 Melem/s | -51.1% ✓ | +105% ✓ |
| reversed | 10k | 4.21 µs | 2.38 Gelem/s | -1.5% ✓ | +1.5% ✓ |
| reversed | 100k | 42.4 µs | 2.36 Gelem/s | -2.5% ✓ | +2.6% ✓ |
| reversed | 1M | 434 µs | 2.30 Gelem/s | -2.4% ✓ | +2.5% ✓ |
| reversed | 10M | 4.63 ms | 2.16 Gelem/s | -2.7% ✓ | +2.8% ✓ |
