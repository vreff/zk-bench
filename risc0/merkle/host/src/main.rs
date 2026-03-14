use methods::{MERKLE_GUEST_ELF, MERKLE_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::{Digest, Sha256};

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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Build the Merkle tree
    let levels = build_tree(&LEAVES);
    let expected_root = levels[DEPTH][0];
    let siblings = get_siblings(&levels, PROVE_INDEX);

    println!("Merkle Tree Membership Proof (RISC Zero zkVM)");
    println!("==============================================");
    println!("Leaves: {:?}", LEAVES);
    println!("Proving membership of value {} at index {}", LEAVES[PROVE_INDEX as usize], PROVE_INDEX);
    println!("Root: 0x{}", hex::encode(expected_root));
    println!();

    // Set up the executor environment with private inputs
    let env = ExecutorEnv::builder()
        .write(&LEAVES[PROVE_INDEX as usize])
        .unwrap()
        .write(&PROVE_INDEX)
        .unwrap()
        .write(&siblings)
        .unwrap()
        .build()
        .unwrap();

    // Prove execution in the zkVM
    println!("Generating proof...");
    let prover = default_prover();
    let prove_info = prover.prove(env, MERKLE_GUEST_ELF).unwrap();
    let receipt = prove_info.receipt;

    // Read the committed root from the journal (public output)
    let committed_root: [u8; 32] = receipt.journal.decode().unwrap();
    println!("Committed root: 0x{}", hex::encode(committed_root));

    // Verify the root matches
    assert_eq!(committed_root, expected_root, "Root mismatch!");
    println!("Root matches expected value!");

    // Verify the receipt (cryptographic proof verification)
    receipt.verify(MERKLE_GUEST_ID).unwrap();
    println!("Receipt verified successfully!");
    println!();
    println!("Public outputs (journal): root hash");
    println!("Private inputs: leaf value ({}), index ({}), siblings", LEAVES[PROVE_INDEX as usize], PROVE_INDEX);

    // Save the receipt for benchmarking
    let receipt_bytes = bincode::serialize(&receipt).unwrap();
    std::fs::write("proof.bin", &receipt_bytes).unwrap();
    println!();
    println!("Proof saved to proof.bin ({} bytes)", receipt_bytes.len());
}
