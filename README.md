# Zero-Knowledge-Proof-Acceleration

The shared NTT and Merkle baseline is in [whir-p3-metal](whir-p3-metal/).
It was imported from [WHIR-P3-Metal](https://github.com/privacy-ethereum/whir-p3-metal/tree/25b33abef525fb5a2cad3c70f9da8156928aa5f2) at commit `25b33abef525fb5a2cad3c70f9da8156928aa5f2`.
The upstream MIT and Apache licences are included in that directory, and the Plonky3 revision is pinned in its Cargo manifest.

From the repository root, run the CPU tests with Rust nightly. On an Apple Silicon Mac with Xcode Command Line Tools, also run the Metal tests.

```sh
cd whir-p3-metal
cargo +nightly test
cargo +nightly test --features gpu-metal
```

The shared Criterion benchmark is [benches/dft_gpu.rs](whir-p3-metal/benches/dft_gpu.rs).

## Metal NTT pilot

On Apple Silicon, run the CPU, full Metal call, and GPU command-buffer measurements for the same BabyBear input.

```sh
cd whir-p3-metal
cargo +nightly bench --features gpu-metal --bench dft_gpu -- 'raw_dft/(cpu|metal_gpu|metal_gpu_execution)_14x16$'
```

To sweep 4, 8, 12, and 16 columns at the same height, use the filter
`'raw_dft/(cpu|metal_gpu|metal_gpu_execution)_14x(4|8|12|16)$'`.

The GPU execution benchmark checks the result against the CPU before timing.
It sums Metal's GPU start-to-end command-buffer durations.
The full-call benchmark also includes host-side work and waiting for completion.

The [pilot samples](results/ntt/metal-pilot-14x16.csv) were collected on an Apple M2 Max (Mac14,5, 64 GiB, macOS 13.7.8, AC power) at commit `c853717` using Rust 1.102.0-nightly, the release profile, and default Rayon threading. Temporary instrumentation collected 500 paired samples after a three-second warm-up using seed 0. All samples used the zero-copy path and matched the CPU output.

| Measurement | Median | 25th–75th percentile |
|---|---:|---:|
| Full transform call | 245.08 µs | 235.65–253.24 µs |
| GPU command-buffer execution | 79.62 µs | 76.00–80.62 µs |
| Per-call difference | 166.19 µs | 156.92–176.44 µs |
