#![cfg_attr(feature = "guest", no_std)]

#[jolt::provable(heap_size = 32768, max_trace_length = 65536)]
fn merkle_verify(
    leaf_value: u64,
    leaf_index: u32,
    sib0: [u8; 32],
    sib1: [u8; 32],
    sib2: [u8; 32],
) -> [u8; 32] {
    // Hash the leaf: SHA-256(value as little-endian u64 bytes)
    let mut current = jolt_inlines_sha2::Sha256::digest(&leaf_value.to_le_bytes());

    // Walk up the tree using siblings
    let siblings = [sib0, sib1, sib2];
    let mut idx = leaf_index;

    for i in 0..3 {
        let mut data = [0u8; 64];
        if idx % 2 == 0 {
            data[..32].copy_from_slice(&current);
            data[32..].copy_from_slice(&siblings[i]);
        } else {
            data[..32].copy_from_slice(&siblings[i]);
            data[32..].copy_from_slice(&current);
        }
        current = jolt_inlines_sha2::Sha256::digest(&data);
        idx /= 2;
    }

    current
}
