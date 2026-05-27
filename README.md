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


#### Usage of VeqDeque instead of BTreeSet

This optimization is not in the source code because it is an improvement I tested, but it was not particularly satisfactory.
We perform an insertion into the buffer, which is a `VecDeque`, and this moves all the elements in memory because the `VecDeque` structure is contiguous.
To prevent values from being moved in memory, I wanted to replace the vector with a `BTreeSet`. This tree structure supports fast insertions and does not require moving all values, since the values are scattered throughout memory.
However, the benchmarks showed a regression across most tests.
As memory loads data in blocks, the vec is loaded in its entirety, whereas with the `BTreeSet`, iterating over the data in `flush_sequential` is more expensive because the data must be loaded into memory several times as it is scattered across memory, whereas the `VecDeque` is loaded only once.

```
Benchmarking orderer_push/ordered/100: Collecting 100 samples in estimated orderer_push/ordered/100
                        time:   [95.955 ns 96.100 ns 96.258 ns]
                        thrpt:  [1.0389 Gelem/s 1.0406 Gelem/s 1.0422 Gelem/s]
                 change:
                        time:   [-52.738% -52.473% -52.188%] (p = 0.00 < 0.05)
                        thrpt:  [+109.15% +110.41% +111.59%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking orderer_push/mostly_ordered_w16/100: Collecting 100 samples inorderer_push/mostly_ordered_w16/100
                        time:   [1.0730 µs 1.0743 µs 1.0756 µs]
                        thrpt:  [92.969 Melem/s 93.085 Melem/s 93.196 Melem/s]
                 change:
                        time:   [-29.804% -29.531% -29.276%] (p = 0.00 < 0.05)
                        thrpt:  [+41.395% +41.907% +42.459%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking orderer_push/reversed/100: Collecting 100 samples in estimatedorderer_push/reversed/100
                        time:   [1.0281 µs 1.0296 µs 1.0310 µs]
                        thrpt:  [96.998 Melem/s 97.129 Melem/s 97.271 Melem/s]
                 change:
                        time:   [+10.024% +11.151% +11.852%] (p = 0.00 < 0.05)
                        thrpt:  [-10.596% -10.032% -9.1104%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking orderer_push/ordered/1000: Collecting 100 samples in estimatedorderer_push/ordered/1000
                        time:   [877.37 ns 878.69 ns 880.28 ns]
                        thrpt:  [1.1360 Gelem/s 1.1381 Gelem/s 1.1398 Gelem/s]
                 change:
                        time:   [-54.785% -54.621% -54.403%] (p = 0.00 < 0.05)
                        thrpt:  [+119.31% +120.37% +121.17%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking orderer_push/mostly_ordered_w16/1000: Collecting 100 samples iorderer_push/mostly_ordered_w16/1000
                        time:   [22.899 µs 22.976 µs 23.115 µs]
                        thrpt:  [43.261 Melem/s 43.523 Melem/s 43.669 Melem/s]
                 change:
                        time:   [+12.078% +12.514% +12.916%] (p = 0.00 < 0.05)
                        thrpt:  [-11.438% -11.122% -10.776%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking orderer_push/reversed/1000: Collecting 100 samples in estimateorderer_push/reversed/1000
                        time:   [16.538 µs 16.558 µs 16.589 µs]
                        thrpt:  [60.280 Melem/s 60.392 Melem/s 60.468 Melem/s]
                 change:
                        time:   [+52.318% +53.894% +57.152%] (p = 0.00 < 0.05)
                        thrpt:  [-36.368% -35.020% -34.348%]
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking orderer_push/ordered/10000: Collecting 100 samples in estimateorderer_push/ordered/10000
                        time:   [8.7906 µs 8.7953 µs 8.8005 µs]
                        thrpt:  [1.1363 Gelem/s 1.1370 Gelem/s 1.1376 Gelem/s]
                 change:
                        time:   [-54.971% -54.816% -54.675%] (p = 0.00 < 0.05)
                        thrpt:  [+120.63% +121.32% +122.08%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking orderer_push/mostly_ordered_w16/10000: Collecting 100 samples orderer_push/mostly_ordered_w16/10000
                        time:   [398.94 µs 399.19 µs 399.47 µs]
                        thrpt:  [25.033 Melem/s 25.051 Melem/s 25.066 Melem/s]
                 change:
                        time:   [+101.75% +103.24% +104.40%] (p = 0.00 < 0.05)
                        thrpt:  [-51.075% -50.798% -50.433%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking orderer_push/reversed/10000: Collecting 100 samples in estimatorderer_push/reversed/10000
                        time:   [179.33 µs 179.46 µs 179.59 µs]
                        thrpt:  [55.682 Melem/s 55.723 Melem/s 55.763 Melem/s]
                 change:
                        time:   [+32.511% +32.970% +33.429%] (p = 0.00 < 0.05)
                        thrpt:  [-25.054% -24.795% -24.535%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
```

