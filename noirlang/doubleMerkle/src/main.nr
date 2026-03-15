// Double Merkle tree membership proof in Noir.
// Proves membership in TWO independent Poseidon2-based Merkle trees (depth 3)
// to roughly double the constraint count.

use poseidon::poseidon2::Poseidon2;

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
    // Tree A
    leaf_a: Field,
    index_a: Field,
    siblings_a: [Field; 3],
    root_a: pub Field,
    // Tree B
    leaf_b: Field,
    index_b: Field,
    siblings_b: [Field; 3],
    root_b: pub Field,
) {
    let computed_root_a = compute_root(leaf_a, index_a, siblings_a);
    assert(computed_root_a == root_a, "Leaf A is not a member of Merkle tree A");

    let computed_root_b = compute_root(leaf_b, index_b, siblings_b);
    assert(computed_root_b == root_b, "Leaf B is not a member of Merkle tree B");
}

#[test]
fn test_double_membership() {
    // Tree A: same leaves as single merkle
    let leaves_a: [Field; 8] = [10, 20, 30, 42, 50, 60, 70, 80];

    let n01_a = hash_pair(leaves_a[0], leaves_a[1]);
    let n23_a = hash_pair(leaves_a[2], leaves_a[3]);
    let n45_a = hash_pair(leaves_a[4], leaves_a[5]);
    let n67_a = hash_pair(leaves_a[6], leaves_a[7]);
    let n0123_a = hash_pair(n01_a, n23_a);
    let n4567_a = hash_pair(n45_a, n67_a);
    let root_a = hash_pair(n0123_a, n4567_a);

    let siblings_a = [leaves_a[2], n01_a, n4567_a];
    let computed_a = compute_root(42, 3, siblings_a);
    assert(computed_a == root_a);

    // Tree B: different leaves
    let leaves_b: [Field; 8] = [100, 200, 300, 420, 500, 600, 700, 800];

    let n01_b = hash_pair(leaves_b[0], leaves_b[1]);
    let n23_b = hash_pair(leaves_b[2], leaves_b[3]);
    let n45_b = hash_pair(leaves_b[4], leaves_b[5]);
    let n67_b = hash_pair(leaves_b[6], leaves_b[7]);
    let n0123_b = hash_pair(n01_b, n23_b);
    let n4567_b = hash_pair(n45_b, n67_b);
    let root_b = hash_pair(n0123_b, n4567_b);

    let siblings_b = [leaves_b[2], n01_b, n4567_b];
    let computed_b = compute_root(420, 3, siblings_b);
    assert(computed_b == root_b);

    // Print Prover.toml values
    println(f"PROVER_TOML_START");
    println(f"leaf_a = \"42\"");
    println(f"index_a = \"3\"");
    let sa0 = siblings_a[0];
    let sa1 = siblings_a[1];
    let sa2 = siblings_a[2];
    println(f"siblings_a = [\"{sa0}\", \"{sa1}\", \"{sa2}\"]");
    println(f"root_a = \"{root_a}\"");
    println(f"leaf_b = \"420\"");
    println(f"index_b = \"3\"");
    let sb0 = siblings_b[0];
    let sb1 = siblings_b[1];
    let sb2 = siblings_b[2];
    println(f"siblings_b = [\"{sb0}\", \"{sb1}\", \"{sb2}\"]");
    println(f"root_b = \"{root_b}\"");
    println(f"PROVER_TOML_END");
}
