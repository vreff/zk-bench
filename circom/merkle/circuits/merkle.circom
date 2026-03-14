// Used for reference: https://github.com/mjerkov/membership/blob/main/zk/circuits/merkle.circom.
// Repurposed with added documentation.

pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/switcher.circom";
include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/bitify.circom";

// Computes the parent hash from a leaf/intermediate node and its sibling,
// using `selector` to determine left/right ordering.
template Mkt2VerifierLevel() {
    signal input sibling;
    signal input low;
    signal input selector;
    signal output root;

    component sw = Switcher();
    component hash = Poseidon(2);

    sw.sel <== selector;
    sw.L <== low;
    sw.R <== sibling;

    hash.inputs[0] <== sw.outL;
    hash.inputs[1] <== sw.outR;

    root <== hash.out;
}

// Verifies that a value belongs to a Merkle tree of depth `nLevels`.
//
// Inputs:
//   key       – the leaf index (encodes the path from root to leaf)
//   value     – the leaf value (pre-image; gets hashed with Poseidon)
//   root      – the expected Merkle root (public)
//   siblings  – the sibling hashes along the path
template Mkt2Verifier(nLevels) {
    signal input key;
    signal input value;
    signal input root;
    signal input siblings[nLevels];

    // Hash the leaf value
    component hashV = Poseidon(1);
    hashV.inputs[0] <== value;

    // Decompose key into bits to determine left/right at each level
    component n2b = Num2Bits(nLevels);
    component levels[nLevels];

    n2b.in <== key;

    for (var i = nLevels - 1; i >= 0; i--) {
        levels[i] = Mkt2VerifierLevel();
        levels[i].sibling <== siblings[i];
        levels[i].selector <== n2b.out[i];

        if (i == nLevels - 1) {
            levels[i].low <== hashV.out;
        } else {
            levels[i].low <== levels[i + 1].root;
        }
    }

    // Constrain the computed root to match the public root
    root === levels[0].root;
}

// Merkle tree with 3 levels → 8 leaves (2^3)
component main { public [root] } = Mkt2Verifier(3);
