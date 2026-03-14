# Merkle Tree Membership Proof — Jolt zkVM

A zero-knowledge proof of Merkle tree membership using [Jolt](https://github.com/a16z/jolt), a16z's lookup-based zkVM that proves the correct execution of arbitrary Rust programs using the Lasso lookup argument.

## Overview

- **Program**: Standard Rust code running inside the Jolt zkVM (RISC-V guest)
- **Hash**: SHA-256 (via `jolt-inlines-sha2`)
- **Proving system**: Lookup-based SNARK (Lasso/Twist-and-Shout + Dory PCS)
- **Tree**: Depth 3, 8 leaves `[10, 20, 30, 42, 50, 60, 70, 80]`
- **Claim**: Value `42` exists at index `3`

## Architecture

Jolt uses a **guest/host** model:

- **Guest** (`guest/src/lib.rs`): Runs inside the zkVM. A function annotated with `#[jolt::provable]` takes the leaf value, index, and sibling hashes, computes the Merkle root via SHA-256, and returns it.
- **Host** (`src/main.rs`): Runs outside the zkVM. Builds the Merkle tree, calls the provable function to generate a proof, and verifies the proof.

The `#[jolt::provable]` macro automatically generates `compile_`, `preprocess_`, `build_prover_`, `build_verifier_`, and native execution functions.

## Build & Run

```bash
# Build
cargo build --release

# Run (prove + verify)
RUST_LOG=info ./target/release/merkle

# Benchmark
/usr/bin/time -l ./target/release/merkle
```

## Visualize

```bash
node scripts/visualize_tree.js      # default: prove leaf[3] = 42
node scripts/visualize_tree.js 0    # prove leaf[0] = 10
```

## Expected Output

```
Root: 0x16c01e0fba0dd14e15230925dfe865a34a6301118ac3af13c605d6c8887c58ab
```
