# circom-merkle

Merkle tree membership proof circuit in [circom](https://docs.circom.io/). Proves that a value exists in a Poseidon-based Merkle tree (depth 3, 8 leaves) without revealing the value or its position.

Based on [mjerkov/membership](https://github.com/mjerkov/membership/blob/main/zk/circuits/merkle.circom).

## Prerequisites

- **circom** ≥ 2.0.0 — [install guide](https://docs.circom.io/getting-started/installation/)
- **Node.js** ≥ 18
- **snarkjs** — `npm install -g snarkjs`

## Setup

```bash
npm install
```

## Commands

### 1. Compile the circuit

```bash
circom circuits/merkle.circom --r1cs --wasm --sym -o build
```

Outputs `build/merkle.r1cs`, `build/merkle.sym`, and `build/merkle_js/` (WASM witness generator).

### 2. Generate test inputs

```bash
node scripts/generate_input.js
```

Writes `input.json` with a valid Merkle proof for value `42` in an 8-leaf tree.

### 3. Powers of Tau ceremony (SRS)

Generate a local powers of tau file for testing:

```bash
snarkjs powersoftau new bn128 12 pot12_0000.ptau
snarkjs powersoftau contribute pot12_0000.ptau pot12_0001.ptau --name="First contribution" -e="random entropy"
snarkjs powersoftau prepare phase2 pot12_0001.ptau pot12_final.ptau
```

### 4. Groth16 trusted setup (phase 2)

```bash
snarkjs groth16 setup build/merkle.r1cs pot12_final.ptau build/merkle_0000.zkey
snarkjs zkey contribute build/merkle_0000.zkey build/merkle_final.zkey --name="Contributor 1" -e="some random entropy"
snarkjs zkey export verificationkey build/merkle_final.zkey build/verification_key.json
```

### 5. Generate witness

```bash
node build/merkle_js/generate_witness.js build/merkle_js/merkle.wasm input.json build/witness.wtns
```

### 6. Generate proof

```bash
snarkjs groth16 prove build/merkle_final.zkey build/witness.wtns build/proof.json build/public.json
```

### 7. Verify proof

```bash
snarkjs groth16 verify build/verification_key.json build/public.json build/proof.json
```

Expected output: `[INFO] snarkJS: OK!`

### 8. Visualize the tree and proof path

```bash
node scripts/visualize_tree.js [leafIndex]
```

Prints an ASCII tree with color-coded proof path (green), sibling witnesses (yellow), and other nodes (dim). Default leaf index is 3 (value 42). Pass a different index (0–7) to prove a different leaf.

### 9. (Optional) Export Solidity verifier

```bash
snarkjs zkey export solidityverifier build/merkle_final.zkey build/Verifier.sol
```

## Circuit overview

| Signal | Visibility | Description |
|---|---|---|
| `key` | private | Leaf index (bit-decomposed to determine path) |
| `value` | private | Leaf value (hashed with Poseidon before tree insertion) |
| `root` | **public** | Expected Merkle root |
| `siblings[3]` | private | Sibling hashes along the authentication path |

The circuit hashes `value` with `Poseidon(1)`, then walks up the tree using `Switcher` + `Poseidon(2)` at each level, and constrains the result to equal `root`.
