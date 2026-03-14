use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

pub const DEPTH: usize = 3;
pub const LEAVES: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
pub const PROVE_INDEX: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct MerkleProofInput {
    pub leaf_value: u64,
    pub leaf_index: u32,
    pub siblings: Vec<[u8; 32]>,
}

pub fn hash_leaf(value: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.finalize().into()
}

pub fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub fn build_tree(leaves: &[u64]) -> Vec<Vec<[u8; 32]>> {
    let mut levels: Vec<Vec<[u8; 32]>> = Vec::new();
    levels.push(leaves.iter().map(|v| hash_leaf(*v)).collect());
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

pub fn get_siblings(levels: &[Vec<[u8; 32]>], index: u32) -> Vec<[u8; 32]> {
    let mut siblings = Vec::new();
    let mut idx = index as usize;
    for level in 0..DEPTH {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        siblings.push(levels[level][sibling_idx]);
        idx /= 2;
    }
    siblings
}

pub fn compute_root(leaf_value: u64, leaf_index: u32, siblings: &[[u8; 32]]) -> [u8; 32] {
    let mut current = hash_leaf(leaf_value);
    let mut idx = leaf_index;
    for sibling in siblings {
        if idx % 2 == 0 {
            current = hash_pair(&current, sibling);
        } else {
            current = hash_pair(sibling, &current);
        }
        idx /= 2;
    }
    current
}
