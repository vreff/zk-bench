# noir-merkle

Merkle tree membership proof circuit in [Noir](https://noir-lang.org/). Proves that a value exists in a Poseidon2-based Merkle tree (depth 3, 8 leaves) without revealing the value or its position.

Equivalent to the circom version in `../circom/`, but using Noir's Poseidon2 hash and the UltraHonk proving system (via [Barretenberg](https://github.com/AztecProtocol/barretenberg)).

## Prerequisites

- **nargo** ≥ 1.0.0-beta — [install](https://noir-lang.org/docs/getting_started/quick_start#installing-noirup)
- **bb** (barretenberg) — [install](https://noir-lang.org/docs/getting_started/quick_start#installing-barretenberg)
- **Node.js** ≥ 18 (for visualization script)

## Commands

### 1. Check/compile the circuit

```bash
nargo check
```

### 2. Run tests

```bash
nargo test --show-output
```

Runs `test_membership` (verifies proof logic) and `test_print_tree_data` (prints all tree node hashes).

### 3. Execute the circuit (compile + witness)

```bash
nargo execute
```

Reads inputs from `Prover.toml`, compiles the circuit, and generates the witness at `target/merkle.gz`.

### 4. Generate proof

```bash
bb prove -b ./target/merkle.json -w ./target/merkle.gz --write_vk -o ./target
```

Generates `target/proof`, `target/vk` (verification key), and `target/public_inputs`.

### 5. Verify proof

```bash
bb verify -p ./target/proof -k ./target/vk
```

Expected output: `Proof verified successfully`

### 6. Visualize the tree and proof path

```bash
node scripts/visualize_tree.js [leafIndex]
```

Prints an ASCII tree with color-coded proof path (green), sibling witnesses (yellow), and other nodes (dim). Default leaf index is 3 (value 42). Pass a different index (0–7) to prove a different leaf.

This script calls `nargo test` internally to compute the Poseidon2 hashes.

## Circuit overview

| Signal | Visibility | Description |
|---|---|---|
| `leaf` | private | The leaf value to prove membership of |
| `index` | private | Leaf position in the tree (bit-decomposed for path selection) |
| `siblings[3]` | private | Sibling hashes along the authentication path |
| `root` | **public** | Expected Merkle root |

The circuit walks up the tree from the leaf using `Poseidon2::hash([left, right], 2)` at each level, selecting left/right based on the index bits, and asserts the computed root equals the provided root.

## Differences from the circom version

| | Circom | Noir |
|---|---|---|
| Hash function | Poseidon | Poseidon2 |
| Proving system | Groth16 (snarkjs) | UltraHonk (Barretenberg) |
| Leaf handling | Hashes value with `Poseidon(1)` before tree insertion | Uses raw values directly as leaves |
| Setup | Requires trusted setup ceremony (Powers of Tau + phase 2) | No trusted setup required |
| Proof size | ~256 bytes | Larger (UltraHonk) |
