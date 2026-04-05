#!/usr/bin/env bash
set -euo pipefail

# Setup all ZK benchmark frameworks from scratch on Linux x86_64.
# This needs to run before benchmarks.py can execute.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="$HOME/.local/bin:$HOME/.nargo/bin:$HOME/.risc0/bin:$HOME/.sp1/bin:$HOME/.cargo/bin:$PATH"

ok()   { echo -e "\033[32m  ✓ $1\033[0m"; }
fail() { echo -e "\033[31m  ✗ $1\033[0m"; }
step() { echo -e "\n\033[1m━━━ $1 ━━━\033[0m"; }

# ──────────────────────────────────────────────────────────────────────────
step "1/9  Circom (Groth16 + PLONK)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/circom/merkle"
npm install --silent 2>/dev/null

mkdir -p build
circom circuits/merkle.circom --r1cs --wasm --sym -o build

# Generate witness
node scripts/generate_input.js > /dev/null
node build/merkle_js/generate_witness.js build/merkle_js/merkle.wasm input.json build/witness.wtns

# Powers of Tau (download a pre-computed one to save time)
if [[ ! -f pot12_final.ptau ]]; then
    echo "  Downloading powers of tau (pot12)..."
    curl -sL "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_12.ptau" -o pot12_final.ptau
fi
if [[ ! -f pot13_final.ptau ]]; then
    echo "  Downloading powers of tau (pot13 for PLONK)..."
    curl -sL "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_13.ptau" -o pot13_final.ptau
fi

# Groth16 setup
snarkjs groth16 setup build/merkle.r1cs pot12_final.ptau build/merkle_0000.zkey
snarkjs zkey contribute build/merkle_0000.zkey build/merkle_final.zkey --name="C1" -e="entropy"
snarkjs zkey export verificationkey build/merkle_final.zkey build/verification_key.json

# PLONK setup
snarkjs plonk setup build/merkle.r1cs pot13_final.ptau build/merkle_plonk.zkey

ok "Circom single-merkle ready"

# Double merkle setup
circom circuits/doubleMerkle.circom --r1cs --wasm --sym -o build
node scripts/generate_double_input.js > /dev/null
node build/doubleMerkle_js/generate_witness.js build/doubleMerkle_js/doubleMerkle.wasm input_double.json build/witness_double.wtns
snarkjs groth16 setup build/doubleMerkle.r1cs pot12_final.ptau build/doubleMerkle_0000.zkey
snarkjs zkey contribute build/doubleMerkle_0000.zkey build/doubleMerkle_final.zkey --name="C1" -e="entropy"
snarkjs plonk setup build/doubleMerkle.r1cs pot13_final.ptau build/doubleMerkle_plonk.zkey

ok "Circom double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "2/9  Noir (UltraHonk)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/noirlang/merkle"
nargo compile
nargo execute
bb write_vk -b ./target/merkle.json -o ./target
ok "Noir single-merkle ready"

cd "$ROOT/noirlang/doubleMerkle"
# Generate Prover.toml from test output
nargo test --show-output 2>&1 | sed -n '/PROVER_TOML_START/,/PROVER_TOML_END/p' | grep -v PROVER_TOML > Prover.toml || true
nargo compile
nargo execute
bb write_vk -b ./target/double_merkle.json -o ./target
ok "Noir double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "3/9  ZoKrates (Groth16 bellman + arkworks)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/zokrates/merkle"
npm install --silent 2>/dev/null

mkdir -p build
zokrates compile -i circuits/merkle.zok -o build/merkle

# Generate witness arguments from the JS script
WITNESS_ARGS=$(node scripts/generate_input.js 2>/dev/null | grep "^zokrates" | head -1 | sed 's/zokrates compute-witness.*-a //')
zokrates compute-witness -i build/merkle -o build/witness -a $WITNESS_ARGS

# Setup for bellman backend
zokrates setup -i build/merkle -b bellman -s g16 -p build/proving.key -v build/verification.key
# Setup for arkworks backend
zokrates setup -i build/merkle -b ark -s g16 -p build/proving_ark.key -v build/verification_ark.key

ok "ZoKrates single-merkle ready"

# Double merkle
zokrates compile -i circuits/doubleMerkle.zok -o build/doubleMerkle
node scripts/generate_double_input.js > /dev/null 2>&1
if [[ -f build/double_witness_cmd.sh ]]; then
    bash build/double_witness_cmd.sh
fi
zokrates setup -i build/doubleMerkle -b bellman -s g16 -p build/proving_double.key -v build/verification_double.key
zokrates setup -i build/doubleMerkle -b ark -s g16 -p build/proving_double_ark.key -v build/verification_double_ark.key

ok "ZoKrates double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "4/9  Leo (Marlin / snarkVM)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/leo/merkle"
leo build 2>&1 || true
ok "Leo ready"

# ──────────────────────────────────────────────────────────────────────────
step "5/9  RISC Zero (STARK / FRI)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/risc0/merkle"
cargo build --release 2>&1 | tail -3
ok "RISC Zero single-merkle ready"

cd "$ROOT/risc0/doubleMerkle"
cargo build --release 2>&1 | tail -3
ok "RISC Zero double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "6/9  SP1 (STARK / Plonky3)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/sp1/merkle/script"
cargo build --release 2>&1 | tail -3
ok "SP1 single-merkle ready"

cd "$ROOT/sp1/doubleMerkle/script"
cargo build --release 2>&1 | tail -3
ok "SP1 double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "7/9  Jolt (Lasso / Dory PCS)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/jolt/merkle"
cargo build --release 2>&1 | tail -3
ok "Jolt single-merkle ready"

cd "$ROOT/jolt/doubleMerkle"
cargo build --release 2>&1 | tail -3
ok "Jolt double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "8/9  powdr (STARK / Plonky3)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/powdr/merkle"
cargo build --release 2>&1 | tail -3
ok "powdr single-merkle ready"

cd "$ROOT/powdr/doubleMerkle"
cargo build --release 2>&1 | tail -3
ok "powdr double-merkle ready"

# ──────────────────────────────────────────────────────────────────────────
step "9/9  Cairo (STARK / Stwo)"
# ──────────────────────────────────────────────────────────────────────────
cd "$ROOT/cairo/merkle"
scarb build 2>&1 | tail -3
ok "Cairo single-merkle ready"

cd "$ROOT/cairo/doubleMerkle"
scarb build 2>&1 | tail -3
ok "Cairo double-merkle ready"

echo ""
echo -e "\033[1m\033[32m━━━ All frameworks set up successfully ━━━\033[0m"
