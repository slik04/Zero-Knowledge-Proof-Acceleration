# Apple M4 Pro Merkle baseline — 2026-11-23

The dedicated benchmark completed three repeated measurement sessions on
one Apple M4 Pro: **19 shapes × 3 backends × 3 sessions = 171 case results**,
each with 30 normalized Criterion samples. This validates the benchmark
on this machine; it does not rank other libraries or hardware.

The measured executable was built from commit
`ba4848706209aff6b6607eed7b766b735a563042`. Benchmark code remained unchanged
throughout measurement. Only documentation was dirty at measurement start.
See the [runbook](merkle-benchmark.md) for the exact timing contract.

## Results

These are warmed host-visible commit timings: prepared owned matrix to
commitment cap plus retained tree. All observed Metal inputs used the
production **no-copy** route; every actual input allocation was checked.

Each cell below is the minimum–maximum of the three session medians in
milliseconds. These ranges are **not confidence intervals**. Ratios use
matching CPU/GPU cases within each session; above 1 means Metal is faster.

| Leaves × row width | CPU, 14 workers (ms) | Metal (ms) | CPU/Metal ratio |
|---|---:|---:|---:|
| 2^10 × 16 | 0.358–0.366 | 1.062–1.089 | 0.336–0.340× |
| 2^14 × 16 | 1.224–1.325 | 1.203–1.232 | 1.02–1.09× |
| 2^16 × 64 | 8.150–9.802 | 4.219–4.277 | 1.91–2.31× |
| 2^20 × 64 | 118.971–134.459 | 45.196–46.598 | 2.63–2.89× |
| 2^21 × 16 | 82.423–93.457 | 32.619–32.880 | 2.51–2.84× |

Small inputs favor the CPU. Around 2^14 leaves the result depends on row
width and is near the crossover. At every sampled size of 2^16 leaves or
larger, Metal's median was lower than the 14-worker CPU median in all
three sessions. This is not an exact or universal dispatch threshold.
The existing `GpuMmcs` threshold was correctness-tested, not retuned.

## Statistics and artifacts

- [CSV](results/merkle-m4-pro-2026-11-23.csv): every session/backend/shape,
  input bytes, actual transfer route, sample count, sampling mode, median
  ns/op, and that median's own 95% bootstrap confidence interval.
- [Manifest](results/merkle-m4-pro-2026-11-23.json): source revision, build
  flags, toolchain, dependency revision, workload parameters, process
  order, timestamps, conditions, exit status, and validation counts.
- Full Criterion samples, reports, and console logs remain locally under
  `target/merkle-runs/2026-11-23-baseline/`. They are ignored build artifacts,
  not versioned data; archive them before cleaning `target`.

The CSV median is the median of normalized sample times
(`sample duration / iterations`), not an individual-call p50. It was
checked against the original sample data for all 171 results. Linear
sampling's console slope is a different estimator. Confidence intervals
are within-session intervals, not cross-session or speedup intervals.

Across the three sessions, the largest median spread (`max/min - 1`) was
**20.3% for CPU 14 workers**, **10.1% for CPU 1 worker**, and **8.0% for
Metal**. Retain these repeatability limits when interpreting small gains;
do not present the best session or a pooled confidence interval.

## Conditions and validation

Hardware: Apple M4 Pro, 24 GiB RAM, 14 CPU cores (10 performance plus
4 efficiency) and 20 GPU cores. The Metal device reported Apple M4 Pro;
GPU inventory was captured after measurement. macOS 27.2.1, build 26B131;
Rust 1.93.0, LLVM 21.1.8;
native CPU instructions, thin LTO, one codegen unit, and locked dependencies.

Session order was CPU 1 → CPU 14 → Metal, then Metal → CPU 14 → CPU 1,
then CPU 1 → CPU 14 → Metal. Processes ran sequentially, with approximately
20 seconds between sessions. These were repeated desktop sessions, not
thermally reset experiments. The machine was on AC power with power mode
0. A temporary idle-sleep assertion began late in session 1's Metal run;
its timestamp is recorded in the manifest. Global power settings were
unchanged. Load and thermal-warning status were recorded at process
boundaries; continuous temperatures, frequencies, and background activity
were not controlled or measured. No thermal/performance warning was
recorded at those boundaries; this does not establish constant clocks or
absence of throttling. The CPU variation has no established cause.

Validation passed: 22 CPU library tests, 29 Metal-enabled library tests,
3 new real-GPU Merkle integration tests, and 38 executed benchmark smoke
cases. Each measured GPU shape also passed its CPU-root preflight. The
new tests first exposed a genuine partial-block sponge mismatch, which
was corrected before measurement. Formatting and Clippy checks passed for
the new benchmark/tests; inherited library lint warnings remain.
