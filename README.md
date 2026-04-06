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

> **Why different hash functions?** DSL circuits use **[Poseidon](https://eprint.iacr.org/2019/458)**, an algebraic hash designed to be efficient inside arithmetic circuits — orders of magnitude cheaper than SHA-256 in constraint count. zkVMs use **SHA-256** because they execute normal CPU code and several (RISC Zero, SP1) include built-in SHA-256 accelerator precompiles. Each framework uses the hash a real developer would naturally reach for, keeping the benchmark representative.

## Project Structure

```
zk-bench/
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

Measured on AMD Ryzen 9 5900X (24 threads, 32 GB RAM) with 2× NVIDIA RTX 3090. Only proof generation is timed (3-run median).

### Single Merkle Proof

| Framework | Proving System | Peak RAM | Wall Time | Proof Size |
|---|---|---|---|---|
| ZoKrates (ark) | Groth16 (arkworks) | 23 MB | 0.03 s | 849 B |
| ZoKrates | Groth16 (bellman) | 18 MB | 0.03 s | 849 B |
| Noir | UltraHonk | 39 MB | 0.06 s | 15.6 KB |
| Circom | Groth16 (snarkjs) | 444 MB | 0.46 s | 809 B |
| **RISC Zero (GPU)** | **STARK (FRI, CUDA)** | **361 MB** | **0.89 s** | **238.8 KB** |
| Jolt | Lasso (Dory PCS) | 184 MB | 1.6 s | 77.5 KB |
| Circom (PLONK) | PLONK (snarkjs) | 650 MB | 1.9 s | 2.2 KB |
| Leo | Marlin (snarkVM) | 590 MB | 3.7 s | 7.3 KB |
| RISC Zero | STARK (FRI) | 1,189 MB | 9.2 s | 238.8 KB |
| SP1 (GPU) | STARK (Plonky3, CUDA) | 117 MB | 11.6 s | 2.6 MB |
| Cairo | STARK (Stwo) | 14,188 MB | 12.0 s | 10.3 MB |
| powdr | STARK (Plonky3) | 5,880 MB | 26.8 s | 1.9 MB |
| SP1 | STARK (Plonky3) | 9,791 MB | 36.2 s | 2.6 MB |

### Double Merkle Proof (2×)

| Framework | Proving System | Peak RAM | Wall Time | Proof Size |
|---|---|---|---|---|
| ZoKrates (2x) | Groth16 (bellman) | 19 MB | 0.05 s | 923 B |
| ZoKrates ark (2x) | Groth16 (arkworks) | 29 MB | 0.05 s | 923 B |
| Noir (2x) | UltraHonk | 39 MB | 0.07 s | 15.6 KB |
| Circom (2x) | Groth16 (snarkjs) | 518 MB | 0.50 s | 806 B |
| **RISC Zero GPU (2x)** | **STARK (FRI, CUDA)** | **394 MB** | **1.1 s** | **250.5 KB** |
| Jolt (2x) | Lasso (Dory PCS) | 345 MB | 2.4 s | 78.4 KB |
| Circom PLONK (2x) | PLONK (snarkjs) | 925 MB | 3.3 s | 2.2 KB |
| Leo (2x) | Marlin (snarkVM) | 1,016 MB | 5.2 s | 9.0 KB |
| Cairo (2x) | STARK (Stwo) | 14,175 MB | 10.2 s | 10.3 MB |
| SP1 GPU (2x) | STARK (Plonky3, CUDA) | 117 MB | 11.7 s | 2.6 MB |
| RISC Zero (2x) | STARK (FRI) | 2,348 MB | 18.4 s | 250.5 KB |
| powdr (2x) | STARK (Plonky3) | 6,713 MB | 32.4 s | 1.9 MB |
| SP1 (2x) | STARK (Plonky3) | 9,801 MB | 36.5 s | 2.6 MB |

### GPU Acceleration

| Framework | CPU Time | GPU Time | Speedup |
|---|---|---|---|
| RISC Zero | 9.2 s | 0.89 s | **~10×** |
| RISC Zero (2x) | 18.4 s | 1.1 s | **~17×** |
| SP1 | 36.2 s | 11.6 s | **~3.1×** |
| SP1 (2x) | 36.5 s | 11.7 s | **~3.1×** |

## Running Benchmarks

```bash
# Prerequisites: Python 3.10+, matplotlib
pip install matplotlib

# Run all single-circuit benchmarks (3 runs averaged)
python3 benchmarks/benchmarks.py

# Run single + double (scaling comparison)
python3 benchmarks/benchmarks.py --double

# Run GPU-accelerated benchmarks (requires NVIDIA GPU + CUDA)
python3 benchmarks/benchmarks.py --gpu

# Run all (CPU + GPU + double)
python3 benchmarks/benchmarks.py --double --gpu

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

- **Fastest proving**: ZoKrates Groth16 (0.03s) — native Rust field arithmetic on BN254
- **Smallest proofs**: Groth16 (~800 B) — constant size regardless of circuit complexity
- **Best all-around**: Noir UltraHonk — fast proving (0.06s), low memory (39 MB), no per-circuit trusted setup
- **Best zkVM (CPU)**: Jolt — 1.6s proving, 184 MB RAM, 77.5 KB proofs; much lighter than STARK-based VMs
- **Best zkVM (GPU)**: RISC Zero — 0.89s with CUDA, ~10× speedup over CPU
- **GPU acceleration**: RISC Zero benefits most from GPU (~10–17× speedup); SP1 gets ~3.1×
- **Post-quantum**: RISC Zero, SP1, powdr, Cairo — STARK-based systems don't rely on elliptic curve hardness

The fundamental tradeoff: **SNARKs** (Groth16, PLONK, UltraHonk) produce small proofs fast but require elliptic curve assumptions. **STARKs** (RISC Zero, SP1, Cairo, powdr) offer transparency and post-quantum security at the cost of larger proofs and higher resource usage. **GPU acceleration** narrows this gap significantly for STARK-based VMs.

## Per-Framework Setup

Each subdirectory has its own README with installation and usage instructions. Generally:

- **Circom**: `npm install`, compile circuit, generate witness, prove
- **Noir**: `nargo compile && nargo execute && bb prove`
- **ZoKrates**: `zokrates compile && zokrates setup && zokrates generate-proof`
- **Leo**: `leo execute`
- **Cairo**: `scarb execute && scarb prove`
- **RISC Zero / SP1 / Jolt / powdr**: `cargo build --release && ./target/release/<binary>`
