use double_methods::{DOUBLE_MERKLE_GUEST_ELF, DOUBLE_MERKLE_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::{Digest, Sha256};

const DEPTH: usize = 3;

const LEAVES_A: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
const LEAVES_B: [u64; 8] = [100, 200, 300, 420, 500, 600, 700, 800];
const PROVE_INDEX: u32 = 3;

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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let levels_a = build_tree(&LEAVES_A);
    let root_a = levels_a[DEPTH][0];
    let siblings_a = get_siblings(&levels_a, PROVE_INDEX);

    let levels_b = build_tree(&LEAVES_B);
    let root_b = levels_b[DEPTH][0];
    let siblings_b = get_siblings(&levels_b, PROVE_INDEX);

    println!("Double Merkle Tree Membership Proof (RISC Zero zkVM)");
    println!("=====================================================");
    println!("Tree A leaves: {:?}", LEAVES_A);
    println!("Tree B leaves: {:?}", LEAVES_B);
    println!("Proving index {} in both trees", PROVE_INDEX);
    println!();

    let env = ExecutorEnv::builder()
        // Tree A
        .write(&LEAVES_A[PROVE_INDEX as usize]).unwrap()
        .write(&PROVE_INDEX).unwrap()
        .write(&siblings_a).unwrap()
        // Tree B
        .write(&LEAVES_B[PROVE_INDEX as usize]).unwrap()
        .write(&PROVE_INDEX).unwrap()
        .write(&siblings_b).unwrap()
        .build()
        .unwrap();

    println!("Generating proof...");
    let prover = default_prover();
    let prove_info = prover.prove(env, DOUBLE_MERKLE_GUEST_ELF).unwrap();
    let receipt = prove_info.receipt;

    let (committed_root_a, committed_root_b): ([u8; 32], [u8; 32]) =
        receipt.journal.decode().unwrap();

    assert_eq!(committed_root_a, root_a, "Root A mismatch!");
    assert_eq!(committed_root_b, root_b, "Root B mismatch!");
    println!("Both roots match!");

    receipt.verify(DOUBLE_MERKLE_GUEST_ID).unwrap();
    println!("Receipt verified successfully!");

    let receipt_bytes = bincode::serialize(&receipt).unwrap();
    std::fs::write("proof.bin", &receipt_bytes).unwrap();
    println!("Proof saved to proof.bin ({} bytes)", receipt_bytes.len());
}
