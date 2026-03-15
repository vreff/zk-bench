use ark_serialize::CanonicalSerialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Instant;
use tracing::info;

const LEAVES_A: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
const LEAVES_B: [u64; 8] = [100, 200, 300, 420, 500, 600, 700, 800];
const PROVE_INDEX: u32 = 3;
const DEPTH: usize = 3;

fn hash_leaf(value: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.finalize().into()
}

fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

fn build_tree(leaves: &[u64]) -> Vec<Vec<[u8; 32]>> {
    let mut levels: Vec<Vec<[u8; 32]>> = Vec::new();
    let leaf_hashes: Vec<[u8; 32]> = leaves.iter().map(|v| hash_leaf(*v)).collect();
    levels.push(leaf_hashes);
    for level in 0..DEPTH {
        let prev = &levels[level];
        let mut next = Vec::new();
        for i in (0..prev.len()).step_by(2) {
            next.push(hash_pair(&prev[i], &prev[i + 1]));
        }
        levels.push(next);
    }
    levels
}

fn get_siblings(levels: &[Vec<[u8; 32]>], index: u32) -> Vec<[u8; 32]> {
    let mut siblings = Vec::new();
    let mut idx = index as usize;
    for level in 0..DEPTH {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        siblings.push(levels[level][sibling_idx]);
        idx /= 2;
    }
    siblings
}

pub fn main() {
    tracing_subscriber::fmt::init();

    let levels_a = build_tree(&LEAVES_A);
    let root_a = levels_a[DEPTH][0];
    let siblings_a = get_siblings(&levels_a, PROVE_INDEX);

    let levels_b = build_tree(&LEAVES_B);
    let root_b = levels_b[DEPTH][0];
    let siblings_b = get_siblings(&levels_b, PROVE_INDEX);

    println!("Double Merkle Tree Membership Proof (Jolt zkVM)");
    println!("================================================");
    println!("Tree A leaves: {:?}", LEAVES_A);
    println!("Tree B leaves: {:?}", LEAVES_B);
    println!("Proving index {} in both trees", PROVE_INDEX);
    println!();

    // Native execution check
    let (native_root_a, native_root_b) = guest::double_merkle_verify(
        LEAVES_A[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_a[0], siblings_a[1], siblings_a[2],
        LEAVES_B[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_b[0], siblings_b[1], siblings_b[2],
    );
    assert_eq!(native_root_a, root_a, "Native root A mismatch!");
    assert_eq!(native_root_b, root_b, "Native root B mismatch!");
    info!("Native execution matches expected roots");

    // Compile and preprocess
    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_double_merkle_verify(target_dir);

    let shared_preprocessing = guest::preprocess_shared_double_merkle_verify(&mut program);
    let prover_preprocessing =
        guest::preprocess_prover_double_merkle_verify(shared_preprocessing.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing =
        guest::preprocess_verifier_double_merkle_verify(shared_preprocessing, verifier_setup, None);

    let prove_fn = guest::build_prover_double_merkle_verify(program, prover_preprocessing);
    let verify_fn = guest::build_verifier_double_merkle_verify(verifier_preprocessing);

    // Prove
    println!("Generating proof...");
    let now = Instant::now();
    let (output, proof, program_io) = prove_fn(
        LEAVES_A[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_a[0], siblings_a[1], siblings_a[2],
        LEAVES_B[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_b[0], siblings_b[1], siblings_b[2],
    );
    let prove_time = now.elapsed();
    info!("Prover runtime: {:.2} s", prove_time.as_secs_f64());

    assert_eq!(output.0, root_a, "Root A mismatch!");
    assert_eq!(output.1, root_b, "Root B mismatch!");
    println!("Both roots match!");

    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).expect("Failed to serialize proof");
    fs::write("proof.bin", &proof_bytes).expect("Failed to write proof");
    println!("Proof size: {} bytes ({:.1} KB)", proof_bytes.len(), proof_bytes.len() as f64 / 1024.0);

    // Verify
    let is_valid = verify_fn(
        LEAVES_A[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_a[0], siblings_a[1], siblings_a[2],
        LEAVES_B[PROVE_INDEX as usize], PROVE_INDEX,
        siblings_b[0], siblings_b[1], siblings_b[2],
        output,
        program_io.panic,
        proof,
    );
    assert!(is_valid, "Proof verification failed!");
    println!("Proof verified successfully!");
}
