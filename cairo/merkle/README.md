# cairo-merkle

Merkle tree membership proof in [Cairo](https://www.cairo-lang.org/). Proves that a value exists in a Poseidon-based Merkle tree (depth 3, 8 leaves) without revealing the value or its position.

Equivalent to the circom, Noir, and ZoKrates versions, using Cairo's native Poseidon hash and the [Stwo](https://github.com/starkware-libs/stwo) STARK prover.

## Prerequisites

- **Scarb** ≥ 2.16 — `curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh | sh`
- **Node.js** ≥ 18 (for visualization script)

## Commands

### 1. Build

```bash
scarb build
```

### 2. Run tests

```bash
scarb test
```

Runs `test_membership` which verifies the proof logic and prints all tree node hashes.

### 3. Execute the program

```bash
scarb execute --arguments-file input.json --print-program-output --output standard
```

Reads hex-encoded inputs from `input.json`, executes the Merkle proof program, and saves the execution trace for proving. The program outputs the root as its public output.

### 4. Generate proof (Stwo STARK)

```bash
scarb prove --execution-id 1
```

Generates a STARK proof of the execution using the Stwo prover. The proof is saved to `target/execute/merkle/execution1/proof/proof.json`.

Alternatively, combine execute + prove in one step:

```bash
scarb prove --execute --arguments-file input.json --output standard
```

### 5. Verify proof

```bash
scarb verify --execution-id 1
```

Expected output: `Verified proof successfully`

### 6. Visualize the tree and proof path

```bash
node scripts/visualize_tree.js [leafIndex]
```

Prints an ASCII tree with color-coded proof path (green), sibling witnesses (yellow), and other nodes (dim). Default leaf index is 3 (value 42). This script calls `scarb test` internally to compute the Starknet Poseidon hashes.

## Program overview

| Input | Visibility | Description |
|---|---|---|
| `leaf` | private | The leaf value to prove membership of |
| `index` | private | Leaf position (bit-decomposed for path selection) |
| `sibling_0..2` | private | Sibling hashes along the authentication path |
| `root` | **public** | Expected Merkle root (returned as program output) |

The program decomposes `index` into bits, walks up the tree hashing with `PoseidonTrait::new().update(left).update(right).finalize()` at each level, and asserts the computed root equals the provided root.

## Comparison with other implementations

| | Circom | Noir | ZoKrates | Cairo |
|---|---|---|---|---|
| Hash function | Poseidon (circomlib) | Poseidon2 | Poseidon (circomlib) | Poseidon (Starknet) |
| Proving system | Groth16 (snarkjs) | UltraHonk (Barretenberg) | Groth16 (built-in) | STARK (Stwo) |
| Proof type | zk-SNARK | zk-SNARK | zk-SNARK | zk-STARK |
| Trusted setup | Yes (ceremony) | No | Yes (single command) | No |
| Proof size | ~256 bytes | Medium | ~256 bytes | Larger (STARK) |
| Verification | Fast (pairing) | Fast | Fast (pairing) | Fast (hash-based) |
