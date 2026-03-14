# Merkle Tree Membership Proof — powdrVM

A zero-knowledge proof of Merkle tree membership using [powdr](https://github.com/powdr-labs/powdr), a modular zkVM stack that compiles Rust programs to RISC-V and proves execution with Plonky3 (STARK-based prover with zk-continuations for unbounded computation).

## Overview

- **Program**: Standard Rust code running inside powdrVM (RISC-V guest)
- **Hash**: SHA-256 (via `sha2` crate)
- **Proving system**: Plonky3 (FRI-based STARK with zk-continuations)
- **Tree**: Depth 3, 8 leaves `[10, 20, 30, 42, 50, 60, 70, 80]`
- **Claim**: Value `42` exists at index `3`

## Architecture

powdr uses a **host/guest** model with channel-based I/O:

- **Guest** (`guest/src/main.rs`): Runs inside the zkVM. Reads the leaf value, index, siblings, and expected root from numbered channels, computes the Merkle root via SHA-256, and asserts it matches.
- **Host** (`src/main.rs`): Runs outside the zkVM. Builds the Merkle tree, writes inputs to channels, runs the dry execution, and generates the ZK proof.

Data flows between host and guest via numbered channels: `session.write(channel, &data)` on the host side, `powdr_riscv_runtime::io::read(channel)` on the guest side. Any `serde`-serializable type can be used.

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
