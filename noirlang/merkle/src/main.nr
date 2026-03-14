// Merkle tree membership proof in Noir.
// Proves that a value exists in a Poseidon2-based Merkle tree (depth 3, 8 leaves)
// without revealing the value or its position.

use poseidon::poseidon2::Poseidon2;

// Tree depth = 3, leaves = 2^3 = 8
global DEPTH: u32 = 3;

fn hash_pair(left: Field, right: Field) -> Field {
    Poseidon2::hash([left, right], 2)
}

fn compute_root(leaf: Field, index: Field, siblings: [Field; DEPTH]) -> Field {
    let mut node = leaf;
    let bits: [u1; DEPTH] = index.to_le_bits();
    for i in 0..DEPTH {
        let (left, right) = if bits[i] == 1 {
            (siblings[i], node)
        } else {
            (node, siblings[i])
        };
        node = hash_pair(left, right);
    }
    node
}

fn main(
    leaf: Field,           // private: the leaf value to prove membership of
    index: Field,          // private: the leaf's position in the tree
    siblings: [Field; 3],  // private: sibling hashes along the path
    root: pub Field,       // public: the expected Merkle root
) {
    let computed_root = compute_root(leaf, index, siblings);
    assert(computed_root == root, "Leaf is not a member of the Merkle tree");
}

#[test]
fn test_membership() {
    // Build the tree from the same 8 leaves used in the circom example
    let leaves: [Field; 8] = [10, 20, 30, 42, 50, 60, 70, 80];

    // Level 1: hash pairs of leaves
    let n01 = hash_pair(leaves[0], leaves[1]); // H(10, 20)
    let n23 = hash_pair(leaves[2], leaves[3]); // H(30, 42)
    let n45 = hash_pair(leaves[4], leaves[5]); // H(50, 60)
    let n67 = hash_pair(leaves[6], leaves[7]); // H(70, 80)

    // Level 2: hash pairs of level-1 nodes
    let n0123 = hash_pair(n01, n23);
    let n4567 = hash_pair(n45, n67);

    // Level 3: root
    let root = hash_pair(n0123, n4567);

    // Prove membership for leaf 42 at index 3
    // Index 3 in little-endian bits: [1, 1, 0]
    // Level 0 sibling: leaves[2] = 30
    // Level 1 sibling: n01 = H(10, 20)
    // Level 2 sibling: n4567 = H(H(50,60), H(70,80))
    let siblings = [leaves[2], n01, n4567];

    let computed_root = compute_root(42, 3, siblings);
    assert(computed_root == root);
}

#[test]
fn test_print_tree_data() {
    let leaves: [Field; 8] = [10, 20, 30, 42, 50, 60, 70, 80];

    // Level 1
    let n01 = hash_pair(leaves[0], leaves[1]);
    let n23 = hash_pair(leaves[2], leaves[3]);
    let n45 = hash_pair(leaves[4], leaves[5]);
    let n67 = hash_pair(leaves[6], leaves[7]);

    // Level 2
    let n0123 = hash_pair(n01, n23);
    let n4567 = hash_pair(n45, n67);

    // Level 3
    let root = hash_pair(n0123, n4567);

    // Bind leaves to locals for fstring
    let l0 = leaves[0];
    let l1 = leaves[1];
    let l2 = leaves[2];
    let l3 = leaves[3];
    let l4 = leaves[4];
    let l5 = leaves[5];
    let l6 = leaves[6];
    let l7 = leaves[7];

    println(f"TREE_DATA_START");
    println(f"NODE:0:0:{l0}");
    println(f"NODE:0:1:{l1}");
    println(f"NODE:0:2:{l2}");
    println(f"NODE:0:3:{l3}");
    println(f"NODE:0:4:{l4}");
    println(f"NODE:0:5:{l5}");
    println(f"NODE:0:6:{l6}");
    println(f"NODE:0:7:{l7}");
    println(f"NODE:1:0:{n01}");
    println(f"NODE:1:1:{n23}");
    println(f"NODE:1:2:{n45}");
    println(f"NODE:1:3:{n67}");
    println(f"NODE:2:0:{n0123}");
    println(f"NODE:2:1:{n4567}");
    println(f"NODE:3:0:{root}");
    println(f"TREE_DATA_END");
}
