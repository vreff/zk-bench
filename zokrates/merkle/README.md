# zokrates-merkle

Merkle tree membership proof circuit in [ZoKrates](https://zokrates.github.io/). Proves that a value exists in a Poseidon-based Merkle tree (depth 3, 8 leaves) without revealing the value or its position.

Equivalent to the circom and Noir versions, using ZoKrates' stdlib Poseidon hash and the Groth16 proving system.

## Prerequisites

- **ZoKrates** ≥ 0.8.8 — `curl -LSfs get.zokrat.es | sh` then `export PATH=$PATH:$HOME/.zokrates/bin`
- **Node.js** ≥ 18 (for input generation and visualization scripts)

## Setup

```bash
npm install
```

## Commands

### 1. Compile the circuit

```bash
zokrates compile -i circuits/merkle.zok -o build/merkle
```

Outputs `build/merkle` (compiled circuit). 1072 constraints.

### 2. Generate test inputs

```bash
node scripts/generate_input.js [leafIndex]
```

Computes the Poseidon Merkle tree and prints the witness arguments needed for `compute-witness`. Default leaf index is 3 (value 42).

### 3. Compute witness

```bash
zokrates compute-witness -i build/merkle -o build/witness -a <args from step 2>
```

For the default (leaf 42 at index 3):

```bash
zokrates compute-witness -i build/merkle -o build/witness -a 42 3 30 18520321019059006606511285595387750999043784958310087972051959520693448686063 3420474571345144317592756791686222517137740774669488698617074013334764505433 14137441823196867098576785772577409116077936272054299522801628487819529363847
```

### 4. Trusted setup

```bash
zokrates setup -i build/merkle -p build/proving.key -v build/verification.key
```

### 5. Generate proof

```bash
zokrates generate-proof -i build/merkle -p build/proving.key -w build/witness -j build/proof.json
```

### 6. Verify proof

```bash
zokrates verify -v build/verification.key -j build/proof.json
```

Expected output: `PASSED`

### 7. Visualize the tree and proof path

```bash
node scripts/visualize_tree.js [leafIndex]
```

Prints an ASCII tree with color-coded proof path (green), sibling witnesses (yellow), and other nodes (dim). Default leaf index is 3 (value 42).

### 8. (Optional) Export Solidity verifier

```bash
zokrates export-verifier -i build/verification.key -o build/Verifier.sol
```

## Circuit overview

| Signal | Visibility | Description |
|---|---|---|
| `leaf` | private | The leaf value to prove membership of |
| `index` | private | Leaf position (`u32`, bit-decomposed for path selection) |
| `siblings[3]` | private | Sibling hashes along the authentication path |
| `root` | **public** | Expected Merkle root |

The circuit decomposes `index` into bits, then walks up the tree hashing with `poseidon([left, right])` at each level, selecting left/right based on the index bits. It asserts the computed root equals the provided root.

## Comparison with circom and Noir versions

| | Circom | Noir | ZoKrates |
|---|---|---|---|
| Hash function | Poseidon | Poseidon2 | Poseidon |
| Proving system | Groth16 (snarkjs) | UltraHonk (Barretenberg) | Groth16 (built-in) |
| Leaf handling | Hashes value before insertion | Raw values as leaves | Raw values as leaves |
| Setup | Separate ceremony steps | No trusted setup | Single `setup` command |
| Constraints | 951 | N/A (gates) | 1072 |
