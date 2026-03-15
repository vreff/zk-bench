#![no_main]
sp1_zkvm::entrypoint!(main);

use double_merkle_lib::{compute_root, DoubleMerkleProofInput};

pub fn main() {
    let input: DoubleMerkleProofInput = sp1_zkvm::io::read();

    let root_a = compute_root(input.leaf_value_a, input.leaf_index_a, &input.siblings_a);
    let root_b = compute_root(input.leaf_value_b, input.leaf_index_b, &input.siblings_b);

    // Commit both roots as public output
    sp1_zkvm::io::commit_slice(&root_a);
    sp1_zkvm::io::commit_slice(&root_b);
}
