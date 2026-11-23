# Merkle commit benchmark

Use `benches/merkle.rs` for CPU versus Metal comparisons on Apple Silicon.
An initial [M4 Pro validation run](merkle-m4-pro-baseline.md) records all
three repeated sessions, their uncertainty, and their execution conditions.
It measures one prepared, owned row-major matrix becoming a commitment cap
and a retained tree that can produce openings. Both implementations use
BabyBear, Poseidon2 width 16, an eight-element digest, a binary tree, and a
root-only cap. Fixed seeds make CPU and GPU inputs and hash parameters match.

## Timing contract

These are warmed, host-visible commit timings. They do not measure only GPU
kernel execution or include input generation and full proof generation.

Included: production tree allocation and hashing; for Metal, input-buffer
creation, any required input copy, command encoding, submission, completion
wait, and construction of the returned tree. GPU execution finishes before
the timed operation returns.

Excluded: random input generation, matrix cloning, CPU/GPU initialization,
shader compilation, correctness checks, and destruction of inputs/results.
Criterion uses `BatchSize::PerIteration` so large trees do not accumulate
across a batch. `black_box` consumes the returned cap and retained tree.

Metal returns a tree backed by shared GPU memory. This benchmark does not
copy every digest layer into separate CPU vectors. The older `merkle` group
in `benches/dft_gpu.rs` does that extra materialization; treat it as a
diagnostic, not the primary comparison.

The benchmark calls `gpu_commit_matrix` directly, forcing actual Metal work
without the small-input CPU fallback in `GpuMmcs`. Missing Metal or a root
mismatch fails the run. Integration tests check openings and dispatch edges.

## Copy versus no-copy inputs

Metal identifiers are `metal_copy` or `metal_no_copy`. The latter requires
both the owned matrix pointer and byte length to be multiples of 16,384,
matching the production buffer-selection rule.

Classification uses an actual cloned input, and every subsequent setup
allocation is checked. A change aborts the case rather than mixing transfer
paths under one label. Do not suppress this guard or combine the two labels
when reporting results. An aborted case needs investigation before reuse.

## Build and correctness checks

Run from `whir-p3-metal` on an Apple Silicon Mac with Xcode Command Line Tools.
`rust-toolchain.toml` pins stable Rust **1.93.0**; committed `Cargo.lock` pins
dependency resolution. Keep `--locked` on verification and benchmark commands.

```sh
cargo test --locked --release --lib
cargo test --locked --release --features gpu-metal --lib
cargo test --locked --release --features gpu-metal --test merkle_metal -- --test-threads=1
cargo bench --locked --features gpu-metal --bench merkle -- --test
```

The final command is a smoke check, not a performance measurement. Require
all checks to pass before collecting results. The integration tests force
the GPU for tiny trees, exercise partial sponge blocks and field edge
values, verify openings, and reject modified leaf values and proofs.

## Collect measurements

Defaults are **2 seconds of warmup**, **5 seconds of measurement**, and
**30 samples** per case. Criterion can extend measurement for slow cases.
Shapes are `2^10, 2^12, ..., 2^20` leaves with widths `8, 16, 64`, plus
`2^21` leaves with width `16`. Larger cases require sufficient free memory.

Use separate processes for one CPU worker, all CPU workers, and Metal.
Set `RAYON_NUM_THREADS` explicitly; the executable prints its actual worker
count and Metal device name. CPU identifiers include the worker count.
The temporary `caffeinate` wrapper prevents idle system sleep while each
process runs; it does not change global power settings.

```sh
set -o pipefail
merkle_cpu_threads="$(sysctl -n hw.logicalcpu)"
merkle_run_dir="$PWD/target/merkle-runs/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$merkle_run_dir"

CRITERION_HOME="$merkle_run_dir/cpu-1" RAYON_NUM_THREADS=1 \
  caffeinate -i cargo bench --locked --features gpu-metal --bench merkle -- 'merkle_commit/cpu_1t/' \
  2>&1 | tee "$merkle_run_dir/cpu-1.log"

CRITERION_HOME="$merkle_run_dir/cpu-all" RAYON_NUM_THREADS="$merkle_cpu_threads" \
  caffeinate -i cargo bench --locked --features gpu-metal --bench merkle -- 'merkle_commit/cpu_' \
  2>&1 | tee "$merkle_run_dir/cpu-all.log"

CRITERION_HOME="$merkle_run_dir/metal" RAYON_NUM_THREADS="$merkle_cpu_threads" \
  caffeinate -i cargo bench --locked --features gpu-metal --bench merkle -- 'merkle_commit/metal_' \
  2>&1 | tee "$merkle_run_dir/metal.log"
```

Repeat for **three independent sessions**, creating a new output directory
each time. Reverse the process order in the second session: Metal, all CPU
workers, then one worker. Allow the machine to return to comparable idle
conditions between sessions. Never run the benchmark processes concurrently.

Keep the machine plugged in with a consistent power mode and background
load. Record the chip, CPU/GPU core counts, RAM, macOS version, `rustc -Vv`,
`git rev-parse HEAD`, worktree changes, actual Rayon worker count, power mode,
and session order alongside the results. Note sustained load or thermal
changes. Archive the run directories and logs before cleaning `target`.

## Report results

- Report per-case latency with Criterion's **95% confidence interval**, and
  variation across the three sessions. Keep independent sessions separate.
- If extracting JSON medians, report the median of normalized sample times
  (`times / iterations`) with its matching median confidence interval. This
  is not an individual-call p50. Linear-sampling console output can instead
  show a regression slope; do not mix these estimators.
- Label CPU worker count and Metal copy/no-copy mode. Compare identical
  leaf count, leaf width, hash parameters, and timing boundaries.
- Compute speedup as CPU latency divided by GPU latency for matching cases;
  state which CPU worker count and session produced the ratio.
- Throughput is **logical input bytes per second**: `leaves × width × 4`
  divided by elapsed time. It is not measured DRAM bandwidth or total memory
  traffic. Retain Criterion data and command logs so results can be checked.
- Report failed correctness, transfer-mode, or smoke checks as failures;
  they do not produce valid performance results.
