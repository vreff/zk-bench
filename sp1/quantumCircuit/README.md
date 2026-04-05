# Quantum Circuit Proof Verification

Reproducible verification of [Google Quantum AI's](https://zenodo.org/records/19196956) Groth16 proofs for quantum circuits that compute secp256k1 elliptic curve point addition.

## What the proofs demonstrate

Two quantum circuit designs each correctly compute secp256k1 point addition, verified across 9,024 random test cases:

| Variant | Qubits | Non-Clifford Gates | Proof |
|---------|-------:|-------------------:|-------|
| Low-Qubit   | 1,175 | ≤ 2.7M | `proofs/low_qubits/proof_9024.bin` |
| Low-Toffoli | 1,425 | ≤ 2.1M | `proofs/low_toffoli/proof_9024.bin` |

Both proofs are Groth16 SNARKs over SP1 zkVM execution traces — the quantum circuit simulations ran inside a zero-knowledge virtual machine, and the validity of the circuits can be verified in seconds without re-executing them.

Anyone who trusts the quantum simulator in [`../vendor/19196956/lib/src/sim.rs`](../vendor/19196956/lib/src/sim.rs) can be convinced that the prover possesses quantum circuits meeting the claimed qubit and gate counts for secp256k1 point addition.

## Prerequisites

- **Docker** — the build runs inside `ghcr.io/succinctlabs/sp1:v6.0.2` for reproducibility
  ```bash
  # Ubuntu/Debian
  sudo apt-get update && sudo apt-get install -y docker.io

  # macOS
  brew install --cask docker  # then launch Docker Desktop
  ```
- **Rust** — host toolchain for compiling the verifier binary
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
  ```
- **SP1 toolchain** — install with:
  ```bash
  curl -L https://sp1.succinct.xyz | bash && sp1up
  ```

## Usage

```bash
./verify_quantum_proof.sh
```

The script performs four steps:

1. **Preflight** — checks Docker, `cargo-prove`, and verifies `Cargo.lock` integrity against the zenodo archive (auto-restores if modified)
2. **Deterministic build** — compiles the zkVM guest program inside SP1's pinned Docker image with `--locked` to guarantee bit-exact output
3. **ELF verification** — SHA-256 comparison of the freshly built ELF against the vendored binary
4. **Proof verification** — runs both Groth16 proofs through the SP1 verifier, checking the verification key, qubit counts, and gate counts against the paper's claims

## Key artifacts

| File | Description |
|------|-------------|
| `../vendor/19196956/proofs/zkp_ecc-program` | Vendored ELF (the program whose execution is proven) |
| `../vendor/19196956/proofs/vkey.bin` | Serialized verification key |
| `../vendor/19196956/proofs/low_qubits/proof_9024.bin` | Groth16 proof — low-qubit variant |
| `../vendor/19196956/proofs/low_toffoli/proof_9024.bin` | Groth16 proof — low-toffoli variant |

## Expected output

```
╔══════════════════════════════════════════════════════════════════════════╗
║                                                                          ║
║   ALL VERIFICATIONS PASSED                                               ║
║                                                                          ║
║   Deterministic build reproduced the exact ELF binary.                   ║
║   Both Groth16 SNARK proofs verified against the built program.          ║
║   Vkey: 0x00ca4af6cb15dbd83ec3eaab3a066402...                            ║
║                                                                          ║
╚══════════════════════════════════════════════════════════════════════════╝
```

## Verification key

```
0x00ca4af6cb15dbd83ec3eaab3a0664023828d90a98e650d2d340712f5f3eb0d4
```

This is a Poseidon2 hash derived deterministically from the compiled ELF — it serves as a cryptographic fingerprint of the proven program. Reproducing the Docker build yields the same ELF, which yields the same vkey, which the proofs are bound to.

## References

- **Paper:** [*How to use a quantum computer to speed up elliptic curve cryptography*](https://quantumai.google/static/site-assets/downloads/cryptocurrency-whitepaper.pdf) — Google Quantum AI
- **Code & proofs:** [zenodo.org/records/19196956](https://zenodo.org/records/19196956) — supplementary artifacts vendored in `../vendor/19196956/`
