#![cfg_attr(feature = "guest", no_std)]

#[jolt::provable(heap_size = 65536, max_trace_length = 131072)]
fn double_merkle_verify(
    // Tree A
    leaf_a: u64,
    index_a: u32,
    sib_a0: [u8; 32],
    sib_a1: [u8; 32],
    sib_a2: [u8; 32],
    // Tree B
    leaf_b: u64,
    index_b: u32,
    sib_b0: [u8; 32],
    sib_b1: [u8; 32],
    sib_b2: [u8; 32],
) -> ([u8; 32], [u8; 32]) {
    // Verify Tree A
    let mut current_a = jolt_inlines_sha2::Sha256::digest(&leaf_a.to_le_bytes());
    let siblings_a = [sib_a0, sib_a1, sib_a2];
    let mut idx_a = index_a;
    for i in 0..3 {
        let mut data = [0u8; 64];
        if idx_a % 2 == 0 {
            data[..32].copy_from_slice(&current_a);
            data[32..].copy_from_slice(&siblings_a[i]);
        } else {
            data[..32].copy_from_slice(&siblings_a[i]);
            data[32..].copy_from_slice(&current_a);
        }
        current_a = jolt_inlines_sha2::Sha256::digest(&data);
        idx_a /= 2;
    }

    // Verify Tree B
    let mut current_b = jolt_inlines_sha2::Sha256::digest(&leaf_b.to_le_bytes());
    let siblings_b = [sib_b0, sib_b1, sib_b2];
    let mut idx_b = index_b;
    for i in 0..3 {
        let mut data = [0u8; 64];
        if idx_b % 2 == 0 {
            data[..32].copy_from_slice(&current_b);
            data[32..].copy_from_slice(&siblings_b[i]);
        } else {
            data[..32].copy_from_slice(&siblings_b[i]);
            data[32..].copy_from_slice(&current_b);
        }
        current_b = jolt_inlines_sha2::Sha256::digest(&data);
        idx_b /= 2;
    }

    (current_a, current_b)
}
