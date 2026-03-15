// Double Merkle tree membership proof using Poseidon2 hash.
// Proves membership in TWO independent depth-3 Merkle trees.
program double_merkle.aleo {
    @noupgrade
    async constructor() {}

    inline hash_leaf(value: field) -> field {
        return Poseidon2::hash_to_field(value);
    }

    struct Pair {
        left: field,
        right: field,
    }

    inline hash_pair(left: field, right: field) -> field {
        let p: Pair = Pair { left: left, right: right };
        return Poseidon2::hash_to_field(p);
    }

    // Verify a single Merkle proof: given leaf, index, 3 siblings, compute root.
    inline verify_one(
        leaf: field,
        index: u32,
        sibling0: field,
        sibling1: field,
        sibling2: field,
    ) -> field {
        let current: field = hash_leaf(leaf);

        // Level 0
        let bit0: u32 = index & 1u32;
        let left0: field = bit0 == 0u32 ? current : sibling0;
        let right0: field = bit0 == 0u32 ? sibling0 : current;
        current = hash_pair(left0, right0);

        // Level 1
        let idx1: u32 = index / 2u32;
        let bit1: u32 = idx1 & 1u32;
        let left1: field = bit1 == 0u32 ? current : sibling1;
        let right1: field = bit1 == 0u32 ? sibling1 : current;
        current = hash_pair(left1, right1);

        // Level 2
        let idx2: u32 = idx1 / 2u32;
        let bit2: u32 = idx2 & 1u32;
        let left2: field = bit2 == 0u32 ? current : sibling2;
        let right2: field = bit2 == 0u32 ? sibling2 : current;
        current = hash_pair(left2, right2);

        return current;
    }

    // Verify membership in two independent Merkle trees.
    transition verify(
        public root_a: field,
        leaf_a: field,
        index_a: u32,
        sibling_a0: field,
        sibling_a1: field,
        sibling_a2: field,
        public root_b: field,
        leaf_b: field,
        index_b: u32,
        sibling_b0: field,
        sibling_b1: field,
        sibling_b2: field,
    ) -> bool {
        let computed_a: field = verify_one(leaf_a, index_a, sibling_a0, sibling_a1, sibling_a2);
        assert_eq(computed_a, root_a);

        let computed_b: field = verify_one(leaf_b, index_b, sibling_b0, sibling_b1, sibling_b2);
        assert_eq(computed_b, root_b);

        return true;
    }
}
