use ark_serialize::CanonicalSerialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Instant;
use tracing::info;

const LEAVES: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
const PROVE_INDEX: u32 = 3; // proving membership of value 42
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

    // Level 0: leaf hashes
    let leaf_hashes: Vec<[u8; 32]> = leaves.iter().map(|v| hash_leaf(*v)).collect();
    levels.push(leaf_hashes);

    // Build internal levels
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

    // Build the Merkle tree on the host
    let levels = build_tree(&LEAVES);
    let expected_root = levels[DEPTH][0];
    let siblings = get_siblings(&levels, PROVE_INDEX);

    println!("Merkle Tree Membership Proof (Jolt zkVM)");
    println!("=========================================");
    println!("Leaves: {:?}", LEAVES);
    println!(
        "Proving membership of value {} at index {}",
        LEAVES[PROVE_INDEX as usize], PROVE_INDEX
    );
    println!("Root: 0x{}", hex::encode(expected_root));
    println!();

    // --- Native execution (sanity check) ---
    let native_root = guest::merkle_verify(
        LEAVES[PROVE_INDEX as usize],
        PROVE_INDEX,
        siblings[0],
        siblings[1],
        siblings[2],
    );
    assert_eq!(native_root, expected_root, "Native root mismatch!");
    info!("Native execution matches expected root");

    // --- Compile and preprocess ---
    let target_dir = "/tmp/jolt-guest-targets";
    let mut program = guest::compile_merkle_verify(target_dir);

    let shared_preprocessing = guest::preprocess_shared_merkle_verify(&mut program);
    let prover_preprocessing =
        guest::preprocess_prover_merkle_verify(shared_preprocessing.clone());
    let verifier_setup = prover_preprocessing.generators.to_verifier_setup();
    let verifier_preprocessing =
        guest::preprocess_verifier_merkle_verify(shared_preprocessing, verifier_setup, None);

    let prove_merkle = guest::build_prover_merkle_verify(program, prover_preprocessing);
    let verify_merkle = guest::build_verifier_merkle_verify(verifier_preprocessing);

    // --- Prove ---
    println!("Generating proof...");
    let now = Instant::now();
    let (output, proof, program_io) = prove_merkle(
        LEAVES[PROVE_INDEX as usize],
        PROVE_INDEX,
        siblings[0],
        siblings[1],
        siblings[2],
    );
    let prove_time = now.elapsed();
    info!("Prover runtime: {:.2} s", prove_time.as_secs_f64());

    // Check output
    println!("Committed root: 0x{}", hex::encode(output));
    assert_eq!(output, expected_root, "Root mismatch!");
    println!("Root matches expected value!");

    // Serialize and save proof before verification (proof is moved into verify)
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).expect("Failed to serialize proof");
    fs::write("proof.bin", &proof_bytes).expect("Failed to write proof");
    println!("Proof size: {} bytes ({:.1} KB)", proof_bytes.len(), proof_bytes.len() as f64 / 1024.0);

    // --- Verify ---
    let is_valid = verify_merkle(
        LEAVES[PROVE_INDEX as usize],
        PROVE_INDEX,
        siblings[0],
        siblings[1],
        siblings[2],
        output,
        program_io.panic,
        proof,
    );
    assert!(is_valid, "Proof verification failed!");
    println!("Proof verified successfully!");

    println!();
    println!("Public outputs: root hash");
    println!(
        "Private inputs: leaf value ({}), index ({}), siblings",
        LEAVES[PROVE_INDEX as usize], PROVE_INDEX
    );
}
