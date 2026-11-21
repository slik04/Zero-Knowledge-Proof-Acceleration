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
