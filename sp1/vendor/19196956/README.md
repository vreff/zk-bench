### VENDOR NOTICE START
All contents in this folder and within this README are vendored from: https://zenodo.org/records/19196956

### VENDOR NOTICE END

# Zero-Knowledge Proofs for Quantum Elliptic Curve Cryptography (ZKP ECC)

This directory contains the Zero-Knowledge Proof (ZKP) infrastructure for verifying that we possess a quantum circuit that correctly implements Elliptic Curve Point Addition on the `secp256k1` curve and satisfies certain resource constraints.

If you're unfamiliar with zero knowledge proofs or quantum circuits, we recommend reading [docs/getting_started.md](docs/getting_started.md).
It contains a guided walkthrough of proving and verifying the function of a simpler circuit (a 64 qubit adder). 
It also contains instructions to install the [SP1 zkVM](https://github.com/succinctlabs/sp1) and the necessary dependencies needed to run the verification script.

We use the [SP1 zkVM](https://github.com/succinctlabs/sp1) to generate a Groth16 Succinct Non-Interactive Argument of Knowledge (SNARK) that attest to the correctness and efficiency of the input quantum circuit (provided as a `.kmx` file).
For details on the circuit format, see [docs/kickmix_format.md](docs/kickmix_format.md) and [docs/kickmix_instructions.md](docs/kickmix_instructions.md). 


## The ZKP Statements
### Verification Key (Hex):
The verification key is the cryptographic hash of the compiled RISC-V ELF binary of our program logic. The compiled RISC-V ELF binary is provided at `proofs/zkp_ecc-program` and the verification key is provided at `proofs/vkey.bin`.
```
0x00ca4af6cb15dbd83ec3eaab3a0664023828d90a98e650d2d340712f5f3eb0d4
```

### Statement 1 (Low-Qubit Variant)
We possess quantum kickmix circuit $C_{\text{low-qubit}}$ (uniquely committed to via its cryptographic hash) with resource counts of at-most:
- **2,700,000** non-Clifford gates (CCX+CCZ)
- **1175** logical qubits
- **17,000,000** total operations

that correctly computes point addition on the elliptic curve `secp256k1` across all 9024 pseudo-random inputs deterministically derived from the circuit's own hash.
#### Circuit SHA256 Hash
```
0xcc8f532ffea1583ceed3c9af75de3263ebaddd5fdf3cddfb3dea848b94d0396a
```
#### Groth16 Proof
```
0x0e78f4db0000000000000000000000000000000000000000000000000000000000000000008cd56e10c2fe24795cff1e1d1f40d3a324528d315674da45d26afb376e86700000000000000000000000000000000000000000000000000000000000000000215c7fe4fc597b861d82370ab556684ae36e98cf073e7f754f2788ad58721dbd012927516f316e7b4f3effb1dbd567732611cb0334f2d75e529c5e3becd0629c17605c7ff87c6f23324328744454bdec0df425a4a63e3358c10079c85ef757412ae86ae1f85bf47ef6980852d6f65423be2d90adb5b29896493324128b1cda0a0042f7138c850a1ca441210ba770a2eee39d56f6f90bf68b7a346e1658c6529715334621b6e1a63b85875b8c8a610e0d885662879755803027dad57d97140afb2498bbb63215b236575f95b0019f2b9713bc810e1e044d47ab360e92b899c46512fc97460609186bf1fe01c892a8015fb00e7fdea11b08f88c6adb79b1243518
```

### Statement 2 (Low-Gate Variant)
We possess quantum kickmix circuit $C_{\text{low-gate}}$ (uniquely committed to via its cryptographic hash) with resource counts of at-most:
- **2,100,000** non-Clifford gates (CCX+CCZ)
- **1425** logical qubits
- **17,000,000** total operations

that correctly computes point addition on the elliptic curve `secp256k1` across 9024 pseudo-random inputs deterministically derived from the circuit's own hash.

#### Circuit SHA256 Hash
```
0x24f5758f2216aa87aa2806af32a0db788767b873cf6869510cca3d893b3f8a69
```

#### Groth16 Proof
```
0x0e78f4db0000000000000000000000000000000000000000000000000000000000000000008cd56e10c2fe24795cff1e1d1f40d3a324528d315674da45d26afb376e867000000000000000000000000000000000000000000000000000000000000000000a11fe07d3afe9d5e9b5af9fdb37fc38bd529d09b92e08350556a3a38ad03f1b2ed337741ecfeae1a65849d1927cdfc3ea4d211734cd747fc4a5534449ebfd1e2130fde87661e0e0fba6ec2055c130d875c7fa3358e25e2236e928520eddfa992a9e6510d0635161c62e0e29f4c28921f56126a908b286c4d910089780441a5811799d5c7dbf293ac3e6d5f51267efbf95cf8643cb28c5f7c2bac8ee9d4b55c830475b328ff9f9b257f2383e7934aaab12616e04645bf6a2b9820cafba4fd3830655d676b7ff376817bbd18a178cf091ad4f4e53b2e322a1d75b3e1400d9b66e1feb401eae0df274d7a774f0bd2fc471ce574348daeaac3ee288dcd282456a33
```
### Measuring non-Clifford gates (CCX+CCZ)

Note we sometimes say "Toffoli count", instead of "non-Clifford count", despite including CCX and CCZ gates in the count (technically Toffoli refers specifically to a CCX).

In our quantum circuits, some instructions do not execute because they are conditioned upon classical bits.
Because the exact number of executed CCX and CCZ gates depends on the runtime input, we report the **average executed Toffoli count** as measured across the 9024 evaluated test cases.

## High-Level Overview of the Directory Structure

```
.
├── docs/*
├── lib
│   ├── Cargo.toml
│   ├── src
│   │   ├── circuit.rs
│   │   ├── lib.rs
│   │   ├── sim.rs
│   │   └── weierstrass_elliptic_curve.rs
│   └── tests/*
├── program
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── proofs
│   ├── low_qubits/proof_9024.bin
│   ├── low_toffoli/proof_9024.bin
│   ├── vkey.bin
│   └── zkp_ecc-program
├── prover
│   ├── build.rs
│   ├── Cargo.toml
│   └── prove.rs
├── verifier
│   ├── Cargo.toml
│   └── verifier.rs
├── Cargo.lock
├── Cargo.toml
├── README.md
├── run_proofs.sh
├── rust-toolchain
```

- [**docs/**](docs/): Documentation.
  - [docs/example_data/](docs/example_data/): Example circuits, and other data used in documentation.
  - [docs/tools/](docs/tools/): Example tools used in documentation.
- [**lib/**](lib/): Contains core Rust libraries shared between the prover host and the zkVM guest. This includes structures for parsing the `.kmx` circuitry, counting operations, simulating the quantum operations in a highly-parallel 64-shot manner (`Simulator`), and performing classical `secp256k1` mathematical operations.
- [**program/**](program/): Contains the code that actually runs *inside* the SP1 zkVM guest (`src/main.rs`). This performs the Fiat-Shamir test generation, iterates over the circuit simulation in batches of 64 shots, checks all arithmetic logic, counts Toffoli operations, and commits all resulting data as public values.
- [**prover/**](prover/): Contains the host logic that sits outside the zkVM.
  - [prove.rs](prover/prove.rs): Orchestrates proof generation by passing the `.kmx` circuit as private input to the SP1 prover.
- [**verifier/**](verifier/): A lightweight, standalone crate used for verifying a generated proof.
  - [verifier.rs](verifier/verifier.rs): The verification binary. Accepts the proof file using `--proof <path-to-proof>`. Also accepts optional parameters for the verification key using `--vkey <path-to-vkey>` OR the ELF file using `--elf <path-to-elf>`. If neither the verification key nor the ELF file is provided, the verification key is generated by compiling the `program/` in a standardised docker environment.
- [**run_proofs.sh**](run_proofs.sh): A comprehensive bash script used to automate the execution of proof generation runs.

## How We Generate the Proof

We use sp1's multi-gpu proving mode to generate proofs. See [docs/multi_gpu_proving.md](docs/multi_gpu_proving.md) for more details on how to setup the sp1 cluster. The `./run_proofs.sh` script is invoked as follows to start proof generation:

```bash
./run_proofs.sh \
  --num-tests "9024" \
  --kmx "./testdata/iadd_elliptic/low_toffoli.kmx" \
  --qubit-counts 1425 \
  --toffoli-counts 2100000 \
  --total-ops 17000000 \
  --proving-mode "multi-gpu"
```

and for the low-qubit variant:

```bash
./run_proofs.sh \
  --num-tests "9024" \
  --kmx "./testdata/iadd_elliptic/low_qubits.kmx" \
  --qubit-counts 1175 \
  --toffoli-counts 2700000 \
  --total-ops 17000000 \
  --proving-mode "multi-gpu"
```

1. **Compilation**: The script invokes `prover/prove.rs`. Using `sp1-build`, this compiles `program/` into an ELF native to the RISC-V zkVM architecture.
2. **Private Input Injection**: The `.kmx` operations are read from disk by the host and passed as an array of private inputs into the zkVM `stdin`.
3. **Execution**: The SP1 prover natively executes the ELF, which simulates the quantum circuit. It tracks memory access, assertions, bounded limits, and computes the test evaluations.
4. **STARK Proof Generation**: The host generates a Groth16 proof and saves it to disk inside the `proofs/` directory (e.g. `proofs/low_toffoli/proof_64.bin`). The host also saves the verification key (eg: `proofs/vkey.bin`) that represents a cryptographic commitment of the exact RISC-V program that was executed in order to generate the proof.


## How to Verify a Proof

After a proof is successfully created, it can be verified by a third-party observer using the standalone `verifier` binary. 

The verifier can use an explicitly provided verification key (eg: `proofs/vkey.bin`) via the `--vkey` flag, or deterministically derive the verification key from the proving ELF (eg: `proofs/zkp_ecc-program`) passed via the `--elf` flag, or the verifier can omit both flags and deterministically rebuild the ELF via Docker and derive the verification key from that.

```bash
# Verify using an explicitly exported vkey file
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin \
    --vkey proofs/vkey.bin
```

Alternatively, you can generate the verification key deterministically on-the-fly if you provide the ELF binary that was used to create the proof:

```bash
# Verify by hashing the given ELF binary
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin \
    --elf proofs/zkp_ecc-program
```

Finally, you can simply point the verifier at a proof and it will automatically construct an isolated Docker environment to deterministically rebuild the proving ELF and derive the verification key:

```bash
# Verify by using Docker to rebuild the original program
cargo run --release -p verifier -- \
    --proof proofs/low_toffoli/proof_9024.bin
```

Upon a successful invocation, the verifier prints useful information like:
1. The verification key corresponding to the ELF binary. 
2. For Groth16 or Plonk proofs, the verifier also prints bytes of the proof itself.
3. The SHA256 hash of the secret quantum circuit that was executed.
4. The demanded resource counts that the secret quantum circuit satisfies.
5. The number of test cases executed for verifying the correctness of the circuit.
6. Whether the proof is valid
