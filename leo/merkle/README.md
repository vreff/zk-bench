# Merkle Tree Membership Proof — Leo (Aleo)

A zero-knowledge Merkle tree membership proof implemented in **Leo**, the programming language for the Aleo blockchain. Uses the built-in **Poseidon2** hash function and the **Marlin** proving system (via snarkVM).

## Overview

- **Tree depth**: 3 (8 leaves)
- **Leaves**: `[10, 20, 30, 42, 50, 60, 70, 80]`
- **Proving**: leaf `42` exists at index `3`
- **Hash function**: Poseidon2 (built-in, rate 2)
- **Proving system**: Marlin (via Aleo's snarkVM)

## Prerequisites

```bash
# Install Leo via cargo
cargo install leo-lang
leo --version  # should print leo 3.5.0+
```

## Quick Start

### 1. Build the circuit

```bash
leo build
```

### 2. Compute the tree (helper — get root and siblings)

```bash
leo run compute_siblings \
  10field 20field 30field 42field \
  50field 60field 70field 80field 3u32
```

This outputs `(sibling0, sibling1, sibling2, root)`.

### 3. Run the proof

```bash
leo run verify \
  3795873241443991455451735146226102458893119113405484212358614283425718189900field \
  42field 3u32 \
  5032677853915026442484505200337051980545600190313243825534151256332463055896field \
  2025782052806597445336394462093422610260230542964192141256089645210002703802field \
  6518303460776629079511004668974420229885492538691518135386352722012076854807field
```

Expected output:
```
➡️  Output
 • true
```

### 4. Visualize the tree

```bash
node scripts/visualize_tree.js 3
```

## Circuit Details

The `verify` transition takes:
- **Public input**: `root` (field) — the Merkle root
- **Private inputs**: `leaf` (field), `index` (u32), `sibling0..2` (field) — the membership witness

It hashes the leaf with `Poseidon2::hash_to_field`, then walks up the tree combining with siblings using a `Pair` struct hashed via `Poseidon2::hash_to_field`, and asserts the computed root matches the public root.

Helper transitions `compute_root`, `compute_siblings`, and `compute_tree` are included for computing the correct input values.

## Project Structure

```
leo/merkle/
├── program.json          # Leo project manifest
├── src/
│   └── main.leo          # Merkle proof circuit
├── scripts/
│   └── visualize_tree.js # Tree visualization
└── build/                # Compiled Aleo instructions (after leo build)
```
