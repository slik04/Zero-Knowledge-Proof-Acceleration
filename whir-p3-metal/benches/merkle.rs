use std::cell::OnceCell;
use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use p3_baby_bear::{BabyBear, Poseidon2BabyBear};
use p3_commit::Mmcs;
use p3_field::Field;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use whir_p3::gpu_dft::MetalBabyBearDft;

type Perm = Poseidon2BabyBear<16>;
type Hash = PaddingFreeSponge<Perm, 16, 8, 8>;
type Compress = TruncatedPermutation<Perm, 2, 8, 16>;
type Packed = <BabyBear as Field>::Packing;
type CpuMmcs = MerkleTreeMmcs<Packed, Packed, Hash, Compress, 2, 8>;
type MatrixData = RowMajorMatrix<BabyBear>;

#[derive(Clone, Copy, Debug)]
struct Case {
    log_leaves: u32,
    width: usize,
}

impl Case {
    fn matrix(self) -> MatrixData {
        let mut rng = SmallRng::seed_from_u64(0);
        let elements = (1usize << self.log_leaves) * self.width;
        RowMajorMatrix::new((0..elements).map(|_| rng.random()).collect(), self.width)
    }

    fn name(self) -> String {
        format!("log{}_w{}", self.log_leaves, self.width)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputTransfer {
    Copy,
    NoCopy,
}

impl InputTransfer {
    fn for_matrix(matrix: &MatrixData) -> Self {
        // Mirrors the production make_source_buffer predicate. Check each actual
        // setup allocation: clones need not have the master's alignment.
        const PAGE_BYTES: usize = 16_384;
        let address = matrix.values.as_ptr() as usize;
        let bytes = matrix.values.len() * size_of::<BabyBear>();
        if address.is_multiple_of(PAGE_BYTES) && bytes.is_multiple_of(PAGE_BYTES) {
            Self::NoCopy
        } else {
            Self::Copy
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Copy => "metal_copy",
            Self::NoCopy => "metal_no_copy",
        }
    }
}

fn benchmark_merkle(c: &mut Criterion) {
    let perm = Perm::new_from_rng_128(&mut SmallRng::seed_from_u64(1));
    let cpu = CpuMmcs::new(Hash::new(perm.clone()), Compress::new(perm), 0);
    let gpu = OnceCell::new();
    // Initializes Rayon's pool before either backend is measured.
    let threads = rayon::current_num_threads();
    eprintln!("Merkle CPU worker threads: {threads}");

    let cases = [10, 12, 14, 16, 18, 20]
        .into_iter()
        .flat_map(|log_leaves| [8, 16, 64].map(|width| Case { log_leaves, width }))
        .chain([Case {
            log_leaves: 21,
            width: 16,
        }]);
    let mut group = c.benchmark_group("merkle_commit");

    for case in cases {
        let data = case.matrix();
        let bytes = data.values.len() * size_of::<BabyBear>();
        // Logical input throughput, not measured DRAM bandwidth.
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_function(
            BenchmarkId::new(format!("cpu_{threads}t"), case.name()),
            |b| {
                b.iter_batched(
                    || data.clone(),
                    |matrix| black_box(cpu.commit_matrix(black_box(matrix))),
                    BatchSize::PerIteration,
                );
            },
        );

        let mut preflight_input = Some(data.clone());
        let transfer = InputTransfer::for_matrix(preflight_input.as_ref().unwrap());
        group.bench_function(BenchmarkId::new(transfer.label(), case.name()), |b| {
            let gpu = gpu.get_or_init(|| {
                let gpu = MetalBabyBearDft::default();
                eprintln!("Merkle Metal device: {}", gpu.device().name());
                gpu
            });
            if let Some(matrix) = preflight_input.take() {
                let expected_cap = cpu.commit_matrix(matrix.clone()).0;
                let (cap, _tree) = gpu.gpu_commit_matrix(matrix);
                assert_eq!(cap, expected_cap, "CPU/GPU root mismatch for {case:?}");
            }

            b.iter_batched(
                || {
                    let matrix = data.clone();
                    assert_eq!(
                        InputTransfer::for_matrix(&matrix),
                        transfer,
                        "input transfer changed for {case:?}; do not mix copy/no-copy timings",
                    );
                    matrix
                },
                |matrix| black_box(gpu.gpu_commit_matrix(black_box(matrix))),
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5));
    targets = benchmark_merkle
}
criterion_main!(benches);
