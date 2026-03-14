#![no_main]
sp1_zkvm::entrypoint!(main);

use merkle_lib::{compute_root, MerkleProofInput};

pub fn main() {
    // Read private inputs from the host
    let input: MerkleProofInput = sp1_zkvm::io::read();

    // Compute the Merkle root from the private inputs
    let root = compute_root(input.leaf_value, input.leaf_index, &input.siblings);

    // Commit the computed root as public output
    sp1_zkvm::io::commit_slice(&root);
}
