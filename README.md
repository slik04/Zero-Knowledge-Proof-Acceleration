# Zero-Knowledge-Proof-Acceleration

The shared NTT and Merkle baseline is in [whir-p3-metal](whir-p3-metal/).
It was imported from [WHIR-P3-Metal](https://github.com/privacy-ethereum/whir-p3-metal/tree/25b33abef525fb5a2cad3c70f9da8156928aa5f2) at commit `25b33abef525fb5a2cad3c70f9da8156928aa5f2`.
The upstream MIT and Apache licences are included in that directory, and the Plonky3 revision is pinned in its Cargo manifest.

The baseline pins stable Rust **1.93.0** in `rust-toolchain.toml` and commits
`Cargo.lock` for reproducible dependency resolution. From the repository root,
run the CPU library tests. On an Apple Silicon Mac with Xcode Command Line
Tools, also run the Metal library and Merkle integration tests.

```sh
cd whir-p3-metal
cargo test --locked --release --lib
cargo test --locked --release --features gpu-metal --lib
cargo test --locked --release --features gpu-metal --test merkle_metal -- --test-threads=1
```

Use [benches/merkle.rs](whir-p3-metal/benches/merkle.rs) for Merkle commit
comparisons. Follow the [Merkle benchmark runbook](whir-p3-metal/docs/merkle-benchmark.md)
for timing boundaries, correctness checks, CPU thread counts, Metal transfer
modes, and repeatable measurement commands. The older shared
[benches/dft_gpu.rs](whir-p3-metal/benches/dft_gpu.rs) remains available for diagnostics.
