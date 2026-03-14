// Merkle tree membership proof in Cairo.
// Proves that a value exists in a Poseidon-based Merkle tree (depth 3, 8 leaves)
// without revealing the value or its position.

use core::poseidon::PoseidonTrait;
use core::hash::HashStateTrait;

const DEPTH: u32 = 3;

fn hash_pair(left: felt252, right: felt252) -> felt252 {
    PoseidonTrait::new().update(left).update(right).finalize()
}

fn compute_root(leaf: felt252, index: u32, siblings: @Array<felt252>) -> felt252 {
    let mut node = leaf;
    let mut idx = index;
    let mut i: u32 = 0;
    while i < DEPTH {
        let sibling = *siblings.at(i);
        let bit = idx % 2;
        node = if bit == 1 {
            hash_pair(sibling, node)
        } else {
            hash_pair(node, sibling)
        };
        idx = idx / 2;
        i += 1;
    };
    node
}

#[executable]
fn main(
    leaf: felt252,
    index: felt252,
    sibling_0: felt252,
    sibling_1: felt252,
    sibling_2: felt252,
    root: felt252,
) -> Array<felt252> {
    let idx: u32 = index.try_into().unwrap();
    let mut siblings: Array<felt252> = array![];
    siblings.append(sibling_0);
    siblings.append(sibling_1);
    siblings.append(sibling_2);

    let computed_root = compute_root(leaf, idx, @siblings);
    assert(computed_root == root, 'Not a member of the Merkle tree');

    // Return the public input (root) as output
    array![root]
}

#[cfg(test)]
mod tests {
    use super::{hash_pair, compute_root};

    #[test]
    fn test_membership() {
        // Build tree from the same 8 leaves
        let leaves: Array<felt252> = array![10, 20, 30, 42, 50, 60, 70, 80];

        // Level 1: hash pairs of leaves
        let n01 = hash_pair(*leaves.at(0), *leaves.at(1));
        let n23 = hash_pair(*leaves.at(2), *leaves.at(3));
        let n45 = hash_pair(*leaves.at(4), *leaves.at(5));
        let n67 = hash_pair(*leaves.at(6), *leaves.at(7));

        // Level 2
        let n0123 = hash_pair(n01, n23);
        let n4567 = hash_pair(n45, n67);

        // Level 3: root
        let root = hash_pair(n0123, n4567);

        // Prove membership for leaf 42 at index 3
        let mut siblings: Array<felt252> = array![];
        siblings.append(*leaves.at(2)); // sibling at level 0 = 30
        siblings.append(n01);           // sibling at level 1
        siblings.append(n4567);         // sibling at level 2

        let computed_root = compute_root(42, 3, @siblings);
        assert(computed_root == root, 'Root mismatch');

        // Print tree data for input generation
        println!("TREE_DATA_START");
        println!("NODE:0:0:{}", *leaves.at(0));
        println!("NODE:0:1:{}", *leaves.at(1));
        println!("NODE:0:2:{}", *leaves.at(2));
        println!("NODE:0:3:{}", *leaves.at(3));
        println!("NODE:0:4:{}", *leaves.at(4));
        println!("NODE:0:5:{}", *leaves.at(5));
        println!("NODE:0:6:{}", *leaves.at(6));
        println!("NODE:0:7:{}", *leaves.at(7));
        println!("NODE:1:0:{}", n01);
        println!("NODE:1:1:{}", n23);
        println!("NODE:1:2:{}", n45);
        println!("NODE:1:3:{}", n67);
        println!("NODE:2:0:{}", n0123);
        println!("NODE:2:1:{}", n4567);
        println!("NODE:3:0:{}", root);
        println!("SIB:0:{}", *leaves.at(2));
        println!("SIB:1:{}", n01);
        println!("SIB:2:{}", n4567);
        println!("TREE_DATA_END");
    }
}
