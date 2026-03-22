# zk-bench

A comparative benchmark suite implementing the **same Merkle tree membership proof** across 10 zero-knowledge proof frameworks, covering DSL-based SNARKs, general-purpose zkVMs, and STARK provers.

This project serves two purposes:

1. **Learning resource** — Each implementation is written for **maximum readability**, serving as a reference for ZK developers to see how the same circuit looks across different DSLs and VMs. If you're evaluating frameworks or learning a new one, start by reading the Merkle proof implementation in your framework of interest.

2. **Benchmarking** — Following the philosophy of [zk-Bench (Ernstberger et al., 2023)](https://eprint.iacr.org/2023/1503.pdf), we benchmark proof generation across frameworks using a standardized workload. Where zk-Bench focuses on isolated primitive operations (field arithmetic, hashing, etc.), this repo uses **complete, readable programs for common use cases** — making the benchmarks more representative of real-world developer experience.

## What It Proves

Every implementation proves the same statement: **value `42` exists at index 3 in a depth-3 Merkle tree** with 8 leaves `[10, 20, 30, 42, 50, 60, 70, 80]`.

A "double" variant (2x) proves membership in **two independent Merkle trees** to measure how each framework scales with computation size.

## Frameworks

| Framework | Language | Proving System | Trusted Setup | Hash |
|---|---|---|---|---|
| [Circom](circom/merkle/) | Circom DSL | Groth16 / PLONK (snarkjs) | Per-circuit (Groth16) / Universal (PLONK) | Poseidon |
| [Noir](noirlang/merkle/) | Noir DSL | UltraHonk (Barretenberg) | Universal SRS | Poseidon |
| [ZoKrates](zokrates/merkle/) | ZoKrates DSL | Groth16 (bellman / arkworks) | Per-circuit | Poseidon |
| [Leo](leo/merkle/) | Leo DSL | Marlin (snarkVM) | Universal | Poseidon |
| [Cairo](cairo/merkle/) | Cairo DSL | Circle STARK (Stwo) | None | Poseidon |
| [RISC Zero](risc0/merkle/) | Rust | FRI-STARK | None | SHA-256 |
| [SP1](sp1/merkle/) | Rust | FRI-STARK (Plonky3) | None | SHA-256 |
| [Jolt](jolt/merkle/) | Rust | Lasso + Dory PCS | None | SHA-256 |
| [powdr](powdr/merkle/) | Rust | FRI-STARK (Plonky3) | None | SHA-256 |

## Project Structure

```
zk-examples/
├── benchmarks/          # Benchmark runner, results, and charts
│   ├── benchmarks.py    # Automated benchmark suite
│   ├── BENCHMARKS.md    # Detailed analysis and methodology
│   ├── bench_results.*  # Latest results (txt + json)
│   └── chart_*.png      # Generated visualizations
├── circom/merkle/       # Circom (Groth16 + PLONK)
├── noirlang/merkle/     # Noir (UltraHonk)
├── zokrates/merkle/     # ZoKrates (bellman + arkworks)
├── leo/merkle/          # Leo (Marlin)
├── cairo/merkle/        # Cairo (Stwo STARK)
├── risc0/merkle/        # RISC Zero zkVM
├── sp1/merkle/          # SP1 zkVM
├── jolt/merkle/         # Jolt zkVM
├── powdr/merkle/        # powdr zkVM
└── <framework>/doubleMerkle/  # 2x variants for scaling analysis
```

## Benchmark Results

Measured on Apple M1 (8 GB RAM) with `/usr/bin/time -l`. Only proof generation is timed.

| Framework | Proving System | Peak RAM | Wall Time | Proof Size |
|---|---|---|---|---|
| ZoKrates | Groth16 (bellman) | 10 MB | 0.06 s | 849 B |
| ZoKrates (ark) | Groth16 (arkworks) | 16 MB | 0.04 s | 849 B |
| Noir | UltraHonk | 11 MB | 0.17 s | 15.9 KB |
| Circom | Groth16 (snarkjs) | 246 MB | 0.46 s | 805 B |
| Circom (PLONK) | PLONK (snarkjs) | 410 MB | 2.0 s | 2.2 KB |
| Jolt | Lasso (Dory PCS) | 191 MB | 2.5 s | 77.5 KB |
| Leo | Marlin (snarkVM) | 422 MB | 8.5 s | 7.3 KB |
| RISC Zero | STARK (FRI) | 1,206 MB | 17.4 s | 238.8 KB |
| Cairo | STARK (Stwo) | 5,119 MB | 28.6 s | 10.3 MB |
| powdr | STARK (Plonky3) | 3,445 MB | 26.6 s | 1.9 MB |
| SP1 | STARK (Plonky3) | 4,422 MB | 42.3 s | 2.6 MB |

See [benchmarks/BENCHMARKS.md](benchmarks/BENCHMARKS.md) for full analysis including arithmetic backends, scaling charts, and tradeoff discussion.

## Running Benchmarks

```bash
# Prerequisites: Python 3.10+, matplotlib
pip install matplotlib

# Run all single-circuit benchmarks (3 runs averaged)
python3 benchmarks/benchmarks.py

# Run single + double (scaling comparison)
python3 benchmarks/benchmarks.py --double

# Run only double-circuit benchmarks
python3 benchmarks/benchmarks.py --double-only

# Run specific frameworks
python3 benchmarks/benchmarks.py circom noir zokrates

# Adjust number of runs
python3 benchmarks/benchmarks.py --runs 5

# Replot from last results without re-running
python3 benchmarks/benchmarks.py --skip-run

# List all available frameworks
python3 benchmarks/benchmarks.py --list
```

## Key Takeaways

- **Fastest proving**: ZoKrates Groth16 (0.04–0.06s) — native Rust field arithmetic on BN254
- **Smallest proofs**: Groth16 (~800 B) — constant size regardless of circuit complexity
- **Best all-around**: Noir UltraHonk — fast proving (0.17s), low memory (11 MB), no per-circuit trusted setup
- **Best zkVM**: Jolt — 2.5s proving, 191 MB RAM, 77.5 KB proofs; much lighter than STARK-based VMs
- **Post-quantum**: RISC Zero, SP1, powdr, Cairo — STARK-based systems don't rely on elliptic curve hardness

The fundamental tradeoff: **SNARKs** (Groth16, PLONK, UltraHonk) produce small proofs fast but require elliptic curve assumptions. **STARKs** (RISC Zero, SP1, Cairo, powdr) offer transparency and post-quantum security at the cost of larger proofs and higher resource usage.

## Per-Framework Setup

Each subdirectory has its own README with installation and usage instructions. Generally:

- **Circom**: `npm install`, compile circuit, generate witness, prove
- **Noir**: `nargo compile && nargo execute && bb prove`
- **ZoKrates**: `zokrates compile && zokrates setup && zokrates generate-proof`
- **Leo**: `leo execute`
- **Cairo**: `scarb execute && scarb prove`
- **RISC Zero / SP1 / Jolt / powdr**: `cargo build --release && ./target/release/<binary>`
