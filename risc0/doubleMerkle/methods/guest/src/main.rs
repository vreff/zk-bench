use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

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

fn compute_root(leaf_value: u64, leaf_index: u32, siblings: &[[u8; 32]]) -> [u8; 32] {
    let mut current = hash_leaf(leaf_value);
    let mut idx = leaf_index;
    for sibling in siblings.iter().take(DEPTH) {
        if idx % 2 == 0 {
            current = hash_pair(&current, sibling);
        } else {
            current = hash_pair(sibling, &current);
        }
        idx /= 2;
    }
    current
}

fn main() {
    // Tree A
    let leaf_a: u64 = env::read();
    let index_a: u32 = env::read();
    let siblings_a: Vec<[u8; 32]> = env::read();

    // Tree B
    let leaf_b: u64 = env::read();
    let index_b: u32 = env::read();
    let siblings_b: Vec<[u8; 32]> = env::read();

    let root_a = compute_root(leaf_a, index_a, &siblings_a);
    let root_b = compute_root(leaf_b, index_b, &siblings_b);

    // Commit both roots as public output
    env::commit(&(root_a, root_b));
}
