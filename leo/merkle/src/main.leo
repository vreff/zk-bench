// Merkle tree membership proof using Poseidon2 hash.
// Proves that a leaf value exists at a given index in a depth-3 Merkle tree.
program merkle.aleo {
    @noupgrade
    async constructor() {}

    // Hash a leaf value to a field element.
    inline hash_leaf(value: field) -> field {
        return Poseidon2::hash_to_field(value);
    }

    // Hash two child nodes into a parent node.
    struct Pair {
        left: field,
        right: field,
    }

    inline hash_pair(left: field, right: field) -> field {
        let p: Pair = Pair { left: left, right: right };
        return Poseidon2::hash_to_field(p);
    }

    // Compute the full Merkle tree from 8 leaves and return the root.
    // Used to derive the correct inputs for the verify transition.
    transition compute_root(
        l0: field, l1: field, l2: field, l3: field,
        l4: field, l5: field, l6: field, l7: field,
    ) -> field {
        // Leaf hashes (level 0)
        let h0: field = hash_leaf(l0);
        let h1: field = hash_leaf(l1);
        let h2: field = hash_leaf(l2);
        let h3: field = hash_leaf(l3);
        let h4: field = hash_leaf(l4);
        let h5: field = hash_leaf(l5);
        let h6: field = hash_leaf(l6);
        let h7: field = hash_leaf(l7);

        // Level 1
        let n10: field = hash_pair(h0, h1);
        let n11: field = hash_pair(h2, h3);
        let n12: field = hash_pair(h4, h5);
        let n13: field = hash_pair(h6, h7);

        // Level 2
        let n20: field = hash_pair(n10, n11);
        let n21: field = hash_pair(n12, n13);

        // Root
        let root: field = hash_pair(n20, n21);
        return root;
    }

    // Return the sibling hashes for leaf at the given index (0-7).
    // Returns (sibling0, sibling1, sibling2) and the root.
    transition compute_siblings(
        l0: field, l1: field, l2: field, l3: field,
        l4: field, l5: field, l6: field, l7: field,
        index: u32,
    ) -> (field, field, field, field) {
        // Leaf hashes
        let h0: field = hash_leaf(l0);
        let h1: field = hash_leaf(l1);
        let h2: field = hash_leaf(l2);
        let h3: field = hash_leaf(l3);
        let h4: field = hash_leaf(l4);
        let h5: field = hash_leaf(l5);
        let h6: field = hash_leaf(l6);
        let h7: field = hash_leaf(l7);

        // Level 1 nodes
        let n10: field = hash_pair(h0, h1);
        let n11: field = hash_pair(h2, h3);
        let n12: field = hash_pair(h4, h5);
        let n13: field = hash_pair(h6, h7);

        // Level 2 nodes
        let n20: field = hash_pair(n10, n11);
        let n21: field = hash_pair(n12, n13);

        // Root
        let root: field = hash_pair(n20, n21);

        // Find siblings based on index
        // Level 0 sibling: XOR with 1
        let sib0: field = index == 0u32 ? h1 :
                          index == 1u32 ? h0 :
                          index == 2u32 ? h3 :
                          index == 3u32 ? h2 :
                          index == 4u32 ? h5 :
                          index == 5u32 ? h4 :
                          index == 6u32 ? h7 : h6;

        // Level 1 sibling
        let sib1: field = index < 2u32 ? n11 :
                          index < 4u32 ? n10 :
                          index < 6u32 ? n13 : n12;

        // Level 2 sibling
        let sib2: field = index < 4u32 ? n21 : n20;

        return (sib0, sib1, sib2, root);
    }

    // Return all node hashes at each level for visualization.
    // Returns 15 values: h0..h7 (leaves), n10..n13 (level1), n20,n21 (level2), root
    struct TreeLevel0 {
        h0: field, h1: field, h2: field, h3: field,
        h4: field, h5: field, h6: field, h7: field,
    }
    struct TreeLevel1 {
        n0: field, n1: field, n2: field, n3: field,
    }
    struct TreeLevel2 {
        n0: field, n1: field, root: field,
    }

    transition compute_tree(
        l0: field, l1: field, l2: field, l3: field,
        l4: field, l5: field, l6: field, l7: field,
    ) -> (TreeLevel0, TreeLevel1, TreeLevel2) {
        let h0: field = hash_leaf(l0);
        let h1: field = hash_leaf(l1);
        let h2: field = hash_leaf(l2);
        let h3: field = hash_leaf(l3);
        let h4: field = hash_leaf(l4);
        let h5: field = hash_leaf(l5);
        let h6: field = hash_leaf(l6);
        let h7: field = hash_leaf(l7);
        let n10: field = hash_pair(h0, h1);
        let n11: field = hash_pair(h2, h3);
        let n12: field = hash_pair(h4, h5);
        let n13: field = hash_pair(h6, h7);
        let n20: field = hash_pair(n10, n11);
        let n21: field = hash_pair(n12, n13);
        let root: field = hash_pair(n20, n21);
        return (
            TreeLevel0 { h0: h0, h1: h1, h2: h2, h3: h3, h4: h4, h5: h5, h6: h6, h7: h7 },
            TreeLevel1 { n0: n10, n1: n11, n2: n12, n3: n13 },
            TreeLevel2 { n0: n20, n1: n21, root: root },
        );
    }

    // Verify Merkle membership: given a leaf, its index, and 3 sibling hashes,
    // compute the root and assert it matches the expected public root.
    transition verify(
        public root: field,
        leaf: field,
        index: u32,
        sibling0: field,
        sibling1: field,
        sibling2: field,
    ) -> bool {
        // Hash the leaf
        let current: field = hash_leaf(leaf);

        // Level 0: combine with sibling0
        let bit0: u32 = index & 1u32;
        let left0: field = bit0 == 0u32 ? current : sibling0;
        let right0: field = bit0 == 0u32 ? sibling0 : current;
        current = hash_pair(left0, right0);

        // Level 1: combine with sibling1
        let idx1: u32 = index / 2u32;
        let bit1: u32 = idx1 & 1u32;
        let left1: field = bit1 == 0u32 ? current : sibling1;
        let right1: field = bit1 == 0u32 ? sibling1 : current;
        current = hash_pair(left1, right1);

        // Level 2: combine with sibling2
        let idx2: u32 = idx1 / 2u32;
        let bit2: u32 = idx2 & 1u32;
        let left2: field = bit2 == 0u32 ? current : sibling2;
        let right2: field = bit2 == 0u32 ? sibling2 : current;
        current = hash_pair(left2, right2);

        // Assert computed root matches expected root
        assert_eq(current, root);

        return true;
    }
}
