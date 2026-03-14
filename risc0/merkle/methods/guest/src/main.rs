use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

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

fn main() {
    // Read private inputs from the host
    let leaf_value: u64 = env::read();
    let leaf_index: u32 = env::read();
    let siblings: Vec<[u8; 32]> = env::read();

    // Compute leaf hash
    let mut current = hash_leaf(leaf_value);

    // Walk up the tree using siblings
    let mut idx = leaf_index;
    for sibling in &siblings {
        if idx % 2 == 0 {
            current = hash_pair(&current, sibling);
        } else {
            current = hash_pair(sibling, &current);
        }
        idx /= 2;
    }

    // Commit the computed root as public output
    env::commit(&current);
}
