# ZK Proof Benchmarks — Merkle Tree Membership

All nine implementations prove the same thing: membership of value `42` at index 3
in a depth-3 Merkle tree with 8 leaves `[10, 20, 30, 42, 50, 60, 70, 80]`.

Five use Poseidon-family hashes; RISC Zero, SP1, Jolt, and powdr use SHA-256.

## System

- **CPU**: Apple M1 (arm64)
- **RAM**: 8 GB
- **OS**: macOS

## Toolchain Versions

| DSL | Compiler / Prover | Version |
|---|---|---|
| Circom | circom + snarkjs | circom 2.2.3, snarkjs 0.7.6 |
| Noir | nargo + bb (Barretenberg) | nargo 1.0.0-beta.19, bb 4.0.0-nightly |
| ZoKrates | zokrates | 0.8.8 |
| Cairo | scarb + Stwo | scarb 2.16.1, cairo 2.16.1 |
| Leo | leo + snarkVM | leo 3.5.0, snarkVM 4.5.0 |
| RISC Zero | cargo-risczero + r0vm | cargo-risczero 3.0.5, risc0-zkvm 3.0.5 |
| SP1 | cargo-prove + sp1-sdk | cargo-prove 6.0.2, sp1-sdk 6.0.1 |
| Jolt | jolt + jolt-sdk | jolt 0.1.0, jolt-sdk 0.1.0 |
| powdr | cargo-powdr + powdr | cargo-powdr 0.1.3, powdr 0.1.3 |

## Arithmetic & Proof Backends

Following the methodology of [zk-Bench (Ernstberger et al., 2023)](https://eprint.iacr.org/2023/1503),
we distinguish the **arithmetic backend** (finite field and elliptic curve library) from the
**circuit/proof backend** (the proof system). The arithmetic backend determines the cost of
primitive operations (field add/mul/inv, MSM, FFT) that dominate proving time.

| Framework | Finite Field | Bits | Arithmetic Library | Curve (PCS) | Proof System |
|---|---|---|---|---|---|
| [**Circom**](https://github.com/iden3/circom) | BN254 scalar | 254 | [ffjavascript](https://github.com/nicola/ffjavascript) (JS) | BN254 | Groth16 ([snarkjs](https://github.com/iden3/snarkjs)) |
| [**Noir**](https://github.com/noir-lang/noir) | BN254 scalar | 254 | [Barretenberg](https://github.com/AztecProtocol/barretenberg/tree/master/cpp/src/barretenberg/ecc/fields) (C++) | BN254 | UltraHonk |
| [**ZoKrates**](https://github.com/Zokrates/ZoKrates) | BN254 scalar | 254 | [bellman](https://github.com/zkcrypto/bellman) (Rust) | BN254 | Groth16 |
| [**Cairo**](https://github.com/starkware-libs/cairo) | M31 (Mersenne-31) | 31 | [Stwo](https://github.com/starkware-libs/stwo/tree/main/crates/prover/src/core/fields) (Rust, SIMD) | — | Circle STARK |
| [**Leo**](https://github.com/ProvableHQ/leo) | BLS12-377 scalar | 252 | [snarkVM](https://github.com/ProvableHQ/snarkVM/tree/mainnet/fields) (Rust) | BLS12-377 | Marlin |
| [**RISC Zero**](https://github.com/risc0/risc0) | Baby Bear | 31 | [risc0-zkp](https://github.com/risc0/risc0/tree/main/risc0/zkp/src/field) (Rust, Metal) | — | FRI-STARK |
| [**SP1**](https://github.com/succinctlabs/sp1) | Baby Bear | 31 | [Plonky3](https://github.com/Plonky3/Plonky3/tree/main/baby-bear) (Rust) | BN254† | FRI-STARK |
| [**Jolt**](https://github.com/a16z/jolt) | BN254 scalar | 254 | [arkworks](https://github.com/arkworks-rs/algebra/tree/master/curves/bn254) (Rust) | BN254 | Lasso + Dory |
| [**powdr**](https://github.com/powdr-labs/powdr) | Baby Bear | 31 | [Plonky3](https://github.com/Plonky3/Plonky3/tree/main/baby-bear) (Rust) | — | FRI-STARK |

**†** SP1 uses Baby Bear for STARK generation; BN254 is used only for optional recursive
compression to Groth16 (not benchmarked here).

**Key observations:**
- **254-bit fields** (BN254, BLS12-377) are used by pairing-based SNARKs that need elliptic curve
  pairings for polynomial commitment schemes (KZG, Dory). Larger field ⇒ higher per-operation cost.
- **31-bit fields** (Baby Bear, Mersenne-31) are used by STARK systems. Smaller field ⇒ SIMD-friendly
  (pack multiple field elements into a single register), but requires extension fields for cryptographic
  security (typically degree-4 extensions, giving ~124-bit security).
- **Arithmetic library matters**: Circom's JavaScript field arithmetic (ffjavascript) is ~10–50×
  slower per-operation than native Rust/C++ implementations, explaining its higher memory despite
  the same Groth16 algorithm as ZoKrates.

## Proof Generation Benchmarks

Measured with `/usr/bin/time -l` on macOS. Only the **proof generation** step is
benchmarked (compilation and witness generation excluded).

| DSL | Proving System | Peak RAM | Wall Time | Proof Size |
|---|---|---|---|---|
| **ZoKrates** | Groth16 (bellman) | **9 MB** | **0.04 s** | 849 B |
| **ZoKrates** | Groth16 (arkworks) | **16 MB** | **0.05 s** | 849 B |
| **Noir** | UltraHonk | **12 MB** | **0.37 s** | 15.9 KB |
| **Circom** | Groth16 (snarkjs) | **245 MB** | **0.42 s** | 802 B |
| **Circom** | PLONK (snarkjs) | **390 MB** | **2.1 s** | 2.2 KB |
| **Jolt** | Lasso (Dory PCS) | **188 MB** | **3.46 s** | 77.5 KB |
| **Leo** | Marlin (snarkVM) | **375 MB** | **9.62 s** | 1.1 KB |
| **RISC Zero** | STARK (FRI) | **857 MB** | **19.8 s** | 239 KB |
| **SP1** | STARK (Plonky3) | **3,017 MB** | **46.2 s** | 2.7 MB |
| **powdr** | STARK (Plonky3) | **2,882 MB** | **76.6 s** | 2.0 MB |
| **Cairo** | STARK (Stwo) | **4,719 MB** | **29.0 s** | 10.3 MB |

## Analysis

### Memory
- **ZoKrates** is the most memory-efficient (~9 MB with bellman backend), followed
  by **Noir** (~12 MB). Both are suitable for resource-constrained environments.
- **Jolt** uses ~188 MB — remarkably lightweight for a general-purpose zkVM. Jolt's
  lookup-based approach (Lasso) avoids the large polynomial evaluation tables that
  STARK-based zkVMs require.
- **Circom (snarkjs)** uses ~258 MB — the JavaScript-based prover is less memory-efficient
  than native implementations despite the same Groth16 algorithm.
- **Leo** uses ~375 MB — the Marlin prover in snarkVM is heavier than standalone
  Groth16 implementations but lighter than STARK provers.
- **RISC Zero** uses ~857 MB — the zkVM must emulate a full RISC-V CPU and then
  generate a STARK proof of the execution trace. Heavier than SNARKs but much lighter
  than Cairo's Stwo prover for this small circuit.
- **powdr** uses ~2.9 GB — similar to SP1, as both use Plonky3 under the hood.
  powdr's zk-continuations architecture splits execution into chunks, each proven
  independently, which bounds per-chunk memory but adds setup overhead.
- **SP1** uses ~3 GB — the Plonky3-based STARK prover has higher memory requirements
  than RISC Zero for this small program. SP1 is optimized for larger programs where
  its parallel proving architecture shines.
- **Cairo (Stwo)** uses ~4.7 GB — STARKs require significantly more memory than SNARKs
  for the same computation. This is inherent to the FRI-based proof system which operates
  over large polynomial evaluations.

### Speed
- **ZoKrates** is fastest (0.04s) using the bellman backend for Groth16.
- **Noir** and **Circom** are comparable (0.37s and 0.52s).
- **Jolt** takes 3.46s — the fastest of the three general-purpose zkVMs. Jolt's
  lookup-based proving (Lasso + Dory commitment scheme) avoids the heavy polynomial
  FFTs of STARK provers, making it significantly faster than RISC Zero and SP1 for
  small programs.
- **Leo** takes 9.62s — the Marlin universal setup and proof generation in snarkVM is
  more complex than Groth16 but avoids a per-circuit trusted setup.
- **RISC Zero** takes 19.8s — the zkVM executes a full RISC-V program trace then
  generates a FRI-based STARK proof. Competitive with Cairo despite running a
  general-purpose VM rather than a specialized algebraic circuit.
- **SP1** takes 46.2s — though SP1 is designed for large programs and distributed
  proving via the Succinct Prover Network. Small programs don't amortize the fixed
  overhead well.
- **powdr** takes 76.6s — the slowest in this benchmark. Much of the time is spent on
  one-time setup (fixed column generation ~26s, backend setup ~27s); with cached setup,
  subsequent runs complete in ~37s. powdr's automated constraint solver for witness
  generation is also slower than hand-optimized witness generators.
- **Cairo** is ~300× slower (29s) due to STARK proof generation complexity.

### Proof Size
- **Groth16** proofs (Circom, ZoKrates) are tiny (~800 bytes) — constant size regardless
  of circuit complexity.
- **Marlin** (Leo) is also compact (~1.1 KB) — slightly larger than Groth16 but uses a
  universal setup instead of per-circuit trusted setup.
- **UltraHonk** (Noir) is moderate (~16 KB).
- **Jolt** proofs are ~77.5 KB — smaller than all STARK-based zkVM proofs. Jolt's
  Dory polynomial commitment scheme produces compact proofs without requiring recursive
  compression.
- **RISC Zero STARK** proofs are ~239 KB — much smaller than Cairo's STARKs because
  RISC Zero uses recursive proof composition (Succinct) to compress the raw STARK.
- **powdr STARK** proofs are ~2.0 MB — comparable to SP1, as both use the Plonky3
  prover. Slightly smaller because powdr's continuations produce one proof per chunk
  (this program fits in a single chunk).
- **SP1 STARK** proofs are ~2.7 MB — larger than RISC Zero's compressed proofs but
  much smaller than Cairo's. SP1 can compress further with its `compress` and `groth16`
  proof modes.
- **STARK** (Cairo) proofs are much larger (~10 MB) — a known tradeoff for
  transparency (no trusted setup) and post-quantum security.

### Tradeoffs Summary

| Property | SNARKs (Circom, ZoKrates) | UltraHonk (Noir) | Marlin (Leo) | Lasso (Jolt) | STARK (RISC Zero) | STARK (SP1) | STARK (powdr) | STARK (Cairo) |
|---|---|---|---|---|---|---|---|---|
| Trusted Setup | Required | No | Universal (in snarkVM) | No | No | No | No | No |
| Proof Size | Tiny (~800 B) | Small (~16 KB) | Small (~1.1 KB) | Small (~78 KB) | Medium (~239 KB) | Medium (~2.7 MB) | Medium (~2.0 MB) | Large (~10 MB) |
| Prover Memory | Low | Low | Medium (~375 MB) | Low (~188 MB) | Medium (~857 MB) | High (~3 GB) | High (~2.9 GB) | Very High |
| Prover Speed | Fast | Fast | Medium | Fast–Medium | Medium | Slow | Slow | Slow |
| Post-Quantum | No | No | No | No | Yes | Yes | Yes | Yes |
| Verification | Fast (pairing) | Fast | Fast | Fast (pairing) | Fast (hash-based) | Fast (hash-based) | Fast (hash-based) | Fast (hash-based) |
| Language | DSL (Circom) | DSL (Noir) | DSL (Leo) | General (Rust) | General (Rust) | General (Rust) | General (Rust) | DSL (Cairo) |

## Reproducing

```bash
# Circom
cd circom/merkle
/usr/bin/time -l snarkjs groth16 prove build/merkle_final.zkey build/witness.wtns build/proof.json build/public.json

# Circom (PLONK)
cd circom/merkle
/usr/bin/time -l snarkjs plonk prove build/merkle_plonk.zkey build/witness.wtns build/proof_plonk.json build/public_plonk.json

# Noir
cd noirlang/merkle
/usr/bin/time -l bb prove -b ./target/merkle.json -w ./target/merkle.gz -o ./target

# ZoKrates (bellman)
cd zokrates/merkle
/usr/bin/time -l zokrates generate-proof -i build/merkle -b bellman -s g16 -p build/proving.key -w build/witness -j build/proof.json

# ZoKrates (arkworks)
cd zokrates/merkle
/usr/bin/time -l zokrates generate-proof -i build/merkle -b ark -s g16 -p build/proving_ark.key -w build/witness -j build/proof_ark.json

# Cairo
cd cairo/merkle
scarb execute --arguments-file input.json --output standard
/usr/bin/time -l scarb prove --execution-id 1 2>/tmp/cairo_time.txt; cat /tmp/cairo_time.txt

# Leo
cd leo/merkle
PRIVATE_KEY="APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH" \
/usr/bin/time -l leo execute --network testnet --endpoint "https://api.explorer.provable.com/v1" --yes \
  verify <root>field <leaf>field <index>u32 <sib0>field <sib1>field <sib2>field

# RISC Zero
cd risc0/merkle
cargo build --release
/usr/bin/time -l ./target/release/host

# SP1
cd sp1/merkle
cargo build --release
cd script && /usr/bin/time -l cargo run --release -- --prove

# Jolt
cd jolt/merkle
cargo build --release
/usr/bin/time -l ./target/release/merkle

# powdr
cd powdr/merkle
cargo build --release
/usr/bin/time -l ./target/release/merkle
```
