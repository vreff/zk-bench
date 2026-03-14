use powdr::Session;
use sha2::{Digest, Sha256};

const LEAVES: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
const PROVE_INDEX: u32 = 3; // proving membership of value 42
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
    env_logger::init();

    // Build the Merkle tree on the host
    let levels = build_tree(&LEAVES);
    let expected_root = levels[DEPTH][0];
    let siblings = get_siblings(&levels, PROVE_INDEX);

    println!("Merkle Tree Membership Proof (powdrVM)");
    println!("======================================");
    println!("Leaves: {:?}", LEAVES);
    println!(
        "Proving membership of value {} at index {}",
        LEAVES[PROVE_INDEX as usize], PROVE_INDEX
    );
    println!("Root: 0x{}", hex::encode(expected_root));
    println!();

    // Create a powdr session
    let mut session = Session::builder()
        .guest_path("./guest")
        .out_path("powdr-target")
        .chunk_size_log2(18)
        .build()
        .write(1, &LEAVES[PROVE_INDEX as usize])
        .write(2, &PROVE_INDEX)
        .write(3, &siblings)
        .write(4, &expected_root);

    // Fast dry run to test execution
    println!("Running dry run...");
    session.run();
    println!("Dry run successful!");
    println!();

    // Generate proof
    println!("Generating proof...");
    session.prove();
    println!("Proof generated and verified successfully!");
    println!();
    println!("Public outputs: root hash (embedded in guest assertion)");
    println!(
        "Private inputs: leaf value ({}), index ({}), siblings",
        LEAVES[PROVE_INDEX as usize], PROVE_INDEX
    );
}
