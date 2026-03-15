// Double Merkle tree membership proof in Cairo.
// Proves membership in TWO independent Poseidon-based Merkle trees (depth 3)
// to roughly double the constraint count.

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
    // Tree A
    leaf_a: felt252,
    index_a: felt252,
    sibling_a0: felt252,
    sibling_a1: felt252,
    sibling_a2: felt252,
    root_a: felt252,
    // Tree B
    leaf_b: felt252,
    index_b: felt252,
    sibling_b0: felt252,
    sibling_b1: felt252,
    sibling_b2: felt252,
    root_b: felt252,
) -> Array<felt252> {
    // Verify Tree A
    let idx_a: u32 = index_a.try_into().unwrap();
    let mut siblings_a: Array<felt252> = array![];
    siblings_a.append(sibling_a0);
    siblings_a.append(sibling_a1);
    siblings_a.append(sibling_a2);
    let computed_root_a = compute_root(leaf_a, idx_a, @siblings_a);
    assert(computed_root_a == root_a, 'Not a member of tree A');

    // Verify Tree B
    let idx_b: u32 = index_b.try_into().unwrap();
    let mut siblings_b: Array<felt252> = array![];
    siblings_b.append(sibling_b0);
    siblings_b.append(sibling_b1);
    siblings_b.append(sibling_b2);
    let computed_root_b = compute_root(leaf_b, idx_b, @siblings_b);
    assert(computed_root_b == root_b, 'Not a member of tree B');

    // Return both roots as public output
    array![root_a, root_b]
}

#[cfg(test)]
mod tests {
    use super::{hash_pair, compute_root};

    #[test]
    fn test_double_membership() {
        // Tree A
        let leaves_a: Array<felt252> = array![10, 20, 30, 42, 50, 60, 70, 80];
        let n01_a = hash_pair(*leaves_a.at(0), *leaves_a.at(1));
        let n23_a = hash_pair(*leaves_a.at(2), *leaves_a.at(3));
        let n45_a = hash_pair(*leaves_a.at(4), *leaves_a.at(5));
        let n67_a = hash_pair(*leaves_a.at(6), *leaves_a.at(7));
        let n0123_a = hash_pair(n01_a, n23_a);
        let n4567_a = hash_pair(n45_a, n67_a);
        let root_a = hash_pair(n0123_a, n4567_a);

        let mut siblings_a: Array<felt252> = array![];
        siblings_a.append(*leaves_a.at(2));
        siblings_a.append(n01_a);
        siblings_a.append(n4567_a);
        let computed_a = compute_root(42, 3, @siblings_a);
        assert(computed_a == root_a, 'Root A mismatch');

        // Tree B
        let leaves_b: Array<felt252> = array![100, 200, 300, 420, 500, 600, 700, 800];
        let n01_b = hash_pair(*leaves_b.at(0), *leaves_b.at(1));
        let n23_b = hash_pair(*leaves_b.at(2), *leaves_b.at(3));
        let n45_b = hash_pair(*leaves_b.at(4), *leaves_b.at(5));
        let n67_b = hash_pair(*leaves_b.at(6), *leaves_b.at(7));
        let n0123_b = hash_pair(n01_b, n23_b);
        let n4567_b = hash_pair(n45_b, n67_b);
        let root_b = hash_pair(n0123_b, n4567_b);

        let mut siblings_b: Array<felt252> = array![];
        siblings_b.append(*leaves_b.at(2));
        siblings_b.append(n01_b);
        siblings_b.append(n4567_b);
        let computed_b = compute_root(420, 3, @siblings_b);
        assert(computed_b == root_b, 'Root B mismatch');

        // Print input.json format for scarb execute
        // Format: ["leaf_a", "index_a", "sib_a0", "sib_a1", "sib_a2", "root_a",
        //          "leaf_b", "index_b", "sib_b0", "sib_b1", "sib_b2", "root_b"]
        println!("INPUT_JSON_START");
        println!("LEAF_A:{}", 42);
        println!("INDEX_A:{}", 3);
        println!("SIB_A0:{}", *leaves_a.at(2));
        println!("SIB_A1:{}", n01_a);
        println!("SIB_A2:{}", n4567_a);
        println!("ROOT_A:{}", root_a);
        println!("LEAF_B:{}", 420);
        println!("INDEX_B:{}", 3);
        println!("SIB_B0:{}", *leaves_b.at(2));
        println!("SIB_B1:{}", n01_b);
        println!("SIB_B2:{}", n4567_b);
        println!("ROOT_B:{}", root_b);
        println!("INPUT_JSON_END");
    }
}
