use powdr::Session;
use sha2::{Digest, Sha256};

const LEAVES_A: [u64; 8] = [10, 20, 30, 42, 50, 60, 70, 80];
const LEAVES_B: [u64; 8] = [100, 200, 300, 420, 500, 600, 700, 800];
const PROVE_INDEX: u32 = 3;
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

    let levels_a = build_tree(&LEAVES_A);
    let root_a = levels_a[DEPTH][0];
    let siblings_a = get_siblings(&levels_a, PROVE_INDEX);

    let levels_b = build_tree(&LEAVES_B);
    let root_b = levels_b[DEPTH][0];
    let siblings_b = get_siblings(&levels_b, PROVE_INDEX);

    println!("Double Merkle Tree Membership Proof (powdrVM)");
    println!("=============================================");
    println!("Tree A leaves: {:?}", LEAVES_A);
    println!("Tree B leaves: {:?}", LEAVES_B);
    println!("Proving index {} in both trees", PROVE_INDEX);
    println!();

    let mut session = Session::builder()
        .guest_path("./guest")
        .out_path("powdr-target")
        .chunk_size_log2(18)
        .build()
        // Tree A
        .write(1, &LEAVES_A[PROVE_INDEX as usize])
        .write(2, &PROVE_INDEX)
        .write(3, &siblings_a)
        .write(4, &root_a)
        // Tree B
        .write(5, &LEAVES_B[PROVE_INDEX as usize])
        .write(6, &PROVE_INDEX)
        .write(7, &siblings_b)
        .write(8, &root_b);

    println!("Running dry run...");
    session.run();
    println!("Dry run successful!");
    println!();

    println!("Generating proof...");
    session.prove();
    println!("Proof generated and verified successfully!");
}
