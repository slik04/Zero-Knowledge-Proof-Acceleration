#![cfg(all(feature = "gpu-metal", any(target_os = "macos", target_os = "ios")))]

use p3_baby_bear::{BabyBear, Poseidon2BabyBear};
use p3_commit::Mmcs;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_matrix::{Matrix, dense::RowMajorMatrix};
use p3_merkle_tree::{MerkleTree, MerkleTreeMmcs};
use p3_symmetric::{MerkleCap, PaddingFreeSponge, TruncatedPermutation};
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use whir_p3::gpu_dft::{GpuMmcs, MetalBabyBearDft};

type Perm = Poseidon2BabyBear<16>;
type Hash = PaddingFreeSponge<Perm, 16, 8, 8>;
type Compress = TruncatedPermutation<Perm, 2, 8, 16>;
type Packed = <BabyBear as Field>::Packing;
type CpuMmcs = MerkleTreeMmcs<Packed, Packed, Hash, Compress, 2, 8>;
type MatrixData = RowMajorMatrix<BabyBear>;
type Tree = MerkleTree<BabyBear, BabyBear, MatrixData, 2, 8>;
type Cap = MerkleCap<BabyBear, [BabyBear; 8]>;

fn cpu_mmcs() -> CpuMmcs {
    // MetalBabyBearDft uses the same permutation seed and a root-only cap.
    let perm = Perm::new_from_rng_128(&mut SmallRng::seed_from_u64(1));
    CpuMmcs::new(Hash::new(perm.clone()), Compress::new(perm), 0)
}

fn random_matrix(height: usize, width: usize) -> MatrixData {
    let mut rng = SmallRng::seed_from_u64(42);
    RowMajorMatrix::new((0..height * width).map(|_| rng.random()).collect(), width)
}

fn assert_matches_cpu(cpu: &CpuMmcs, matrix: &MatrixData, (cap, tree): (Cap, Tree)) {
    let (expected_cap, expected_tree) = cpu.commit_matrix(matrix.clone());
    let dimensions = [matrix.dimensions()];
    let height = matrix.height();
    let width = matrix.width();

    assert_eq!(cap, expected_cap, "cap mismatch for {height}x{width}");
    assert_eq!(tree.num_layers(), expected_tree.num_layers());

    let indices: Vec<usize> = if height <= 64 {
        (0..height).collect()
    } else {
        vec![0, 1, height / 2, height - 1]
    };
    for index in indices {
        let expected = cpu.open_batch(index, &expected_tree);
        let opening = cpu.open_batch(index, &tree);
        assert_eq!(
            opening.opened_values, expected.opened_values,
            "opened values mismatch at {index} for {height}x{width}"
        );
        assert_eq!(
            opening.opening_proof, expected.opening_proof,
            "proof mismatch at {index} for {height}x{width}"
        );
        cpu.verify_batch(&expected_cap, &dimensions, index, (&opening).into())
            .expect("GPU tree opening must verify against the CPU commitment");
    }

    let mut tampered = cpu.open_batch(0, &tree);
    tampered.opened_values[0][0] += BabyBear::ONE;
    assert!(
        cpu.verify_batch(&expected_cap, &dimensions, 0, (&tampered).into())
            .is_err(),
        "changed leaf must be rejected for {height}x{width}"
    );
    tampered.opened_values[0][0] -= BabyBear::ONE;
    if let Some(sibling) = tampered.opening_proof.first_mut() {
        sibling[0] += BabyBear::ONE;
        assert!(
            cpu.verify_batch(&expected_cap, &dimensions, 0, (&tampered).into())
                .is_err(),
            "changed sibling must be rejected for {height}x{width}"
        );
    }
}

#[test]
fn direct_gpu_commits_match_cpu() {
    let cpu = cpu_mmcs();
    let gpu = MetalBabyBearDft::default();

    // Force Metal even for small trees, covering all three leaf dispatches
    // and absorption boundaries around the sponge's eight-element rate.
    for height in [1, 2, 4, 8, 64] {
        for width in [1, 7, 8, 9, 16, 64] {
            let matrix = random_matrix(height, width);
            let committed = gpu.gpu_commit_matrix(matrix.clone());
            assert_matches_cpu(&cpu, &matrix, committed);
        }
    }

    let matrix = random_matrix(4096, 16);
    let committed = gpu.gpu_commit_matrix(matrix.clone());
    assert_matches_cpu(&cpu, &matrix, committed);
}

#[test]
fn direct_gpu_commits_handle_field_edge_values() {
    let cpu = cpu_mmcs();
    let gpu = MetalBabyBearDft::default();
    let boundary_values = [
        BabyBear::ZERO,
        BabyBear::ONE,
        -BabyBear::ONE,
        -BabyBear::TWO,
    ];
    let height = 64;
    let width = 9;
    let inputs = [
        vec![BabyBear::ZERO; height * width],
        (0..height * width)
            .map(|index| boundary_values[index % boundary_values.len()])
            .collect(),
    ];

    for values in inputs {
        let matrix = RowMajorMatrix::new(values, width);
        let committed = gpu.gpu_commit_matrix(matrix.clone());
        assert_matches_cpu(&cpu, &matrix, committed);
    }
}

#[test]
fn mmcs_commits_match_cpu_across_gpu_dispatch_threshold() {
    let cpu = cpu_mmcs();
    let gpu_mmcs = GpuMmcs::new(cpu.clone(), MetalBabyBearDft::default());

    // 64, 128, and 256 KiB straddle the dispatcher's 128 KiB threshold.
    for height in [1024, 2048, 4096] {
        let matrix = random_matrix(height, 16);
        let committed = gpu_mmcs.commit_matrix(matrix.clone());
        assert_matches_cpu(&cpu, &matrix, committed);
    }
}
