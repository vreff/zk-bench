#!/usr/bin/env bash
set -euo pipefail

# ╔══════════════════════════════════════════════════════════════════════════════╗
# ║                                                                            ║
# ║   🔬  ZKP-ECC: Quantum Circuit Proof Verification Pipeline                ║
# ║                                                                            ║
# ║   Reproduces and verifies Google Quantum AI's Groth16 proofs for           ║
# ║   elliptic curve point addition circuits on secp256k1.                     ║
# ║                                                                            ║
# ║   Source: https://zenodo.org/records/19196956                              ║
# ║                                                                            ║
# ╚══════════════════════════════════════════════════════════════════════════════╝

BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

CHECKMARK="✅"
CROSS="❌"
ARROW="▸"
GEAR="⚙️"
LOCK="🔒"
MICROSCOPE="🔬"
ROCKET="🚀"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENDOR_DIR="$SCRIPT_DIR/../vendor/19196956"
ELF_OUTPUT="$VENDOR_DIR/target/elf-compilation/docker/riscv64im-succinct-zkvm-elf/release/zkp_ecc-program"
VENDORED_ELF="$VENDOR_DIR/proofs/zkp_ecc-program"
EXPECTED_VKEY="0x00ca4af6cb15dbd83ec3eaab3a0664023828d90a98e650d2d340712f5f3eb0d4"

divider() {
    echo -e "${DIM}──────────────────────────────────────────────────────────────────${NC}"
}

header() {
    echo ""
    divider
    echo -e "  ${BOLD}${CYAN}$1${NC}"
    divider
}

step() {
    echo -e "  ${YELLOW}${ARROW}${NC} $1"
}

success() {
    echo -e "  ${GREEN}${CHECKMARK} $1${NC}"
}

fail() {
    echo -e "  ${RED}${CROSS} $1${NC}"
    exit 1
}

info() {
    echo -e "  ${DIM}$1${NC}"
}

value() {
    echo -e "  ${MAGENTA}${BOLD}$1${NC}  $2"
}

# ─── Preflight ────────────────────────────────────────────────────────────────

header "${GEAR}  Preflight Checks"

step "Checking vendor directory..."
[[ -d "$VENDOR_DIR" ]] || fail "Vendor directory not found: $VENDOR_DIR"
success "Vendor directory found"

step "Checking Docker..."
docker info > /dev/null 2>&1 || fail "Docker is not running"
success "Docker is available"

step "Checking cargo-prove..."
if command -v cargo-prove &> /dev/null; then
    PROVE_VERSION=$(cargo-prove prove --version 2>&1 | head -1)
elif [[ -x "$HOME/.sp1/bin/cargo-prove" ]]; then
    export PATH="$HOME/.sp1/bin:$PATH"
    PROVE_VERSION=$(cargo-prove prove --version 2>&1 | head -1)
else
    fail "cargo-prove not found. Install SP1: curl -L https://sp1.succinct.xyz | bash && sp1up"
fi
success "cargo-prove: ${DIM}$PROVE_VERSION${NC}"

step "Checking Cargo.lock integrity..."
ORIGINAL_LOCK_HASH="81883f36f3fd02df41bb78744390b10cc7a3dd84c53a0de57d39717ab4d916df"
CURRENT_LOCK_HASH=$(sha256sum "$VENDOR_DIR/Cargo.lock" | awk '{print $1}')
if [[ "$CURRENT_LOCK_HASH" == "$ORIGINAL_LOCK_HASH" ]]; then
    success "Cargo.lock matches zenodo original"
else
    echo -e "  ${YELLOW}⚠️  Cargo.lock has been modified (expected: ${ORIGINAL_LOCK_HASH:0:16}...)${NC}"
    step "Restoring original Cargo.lock from zenodo..."
    ZENODO_ZIP="/tmp/zkp_ecc_zenodo_restore.zip"
    curl -sL -o "$ZENODO_ZIP" "https://zenodo.org/api/records/19196956/files/zkp_ecc_zenodo.zip/content"
    unzip -o -q "$ZENODO_ZIP" Cargo.lock -d "/tmp/zenodo_lock_restore"
    cp "/tmp/zenodo_lock_restore/Cargo.lock" "$VENDOR_DIR/Cargo.lock"
    rm -rf "$ZENODO_ZIP" "/tmp/zenodo_lock_restore"
    success "Cargo.lock restored from zenodo"
fi

# ─── Deterministic Build ──────────────────────────────────────────────────────

header "${LOCK}  Deterministic Build (Docker + --locked)"

step "Building zkp_ecc-program inside Docker..."
info "Image: ghcr.io/succinctlabs/sp1:v6.0.2"
info "Target: riscv64im-succinct-zkvm-elf"
echo ""

cd "$VENDOR_DIR"
cargo prove build --packages zkp_ecc-program --docker --locked 2>&1 | while IFS= read -r line; do
    # Show only key progress lines
    if [[ "$line" == *"Compiling zkp_ecc"* ]] || \
       [[ "$line" == *"Finished"* ]] || \
       [[ "$line" == *"Downloading"* && "$line" == *"crates"* ]] || \
       [[ "$line" == *"error"* ]]; then
        echo -e "    ${DIM}${line##*\] }${NC}"
    fi
done

echo ""

[[ -f "$ELF_OUTPUT" ]] || fail "Build failed — ELF not found at $ELF_OUTPUT"
success "Build complete"

# ─── ELF Verification ────────────────────────────────────────────────────────

header "${MICROSCOPE}  ELF Binary Verification"

BUILT_HASH=$(sha256sum "$ELF_OUTPUT" | awk '{print $1}')
VENDORED_HASH=$(sha256sum "$VENDORED_ELF" | awk '{print $1}')
BUILT_SIZE=$(stat --printf="%s" "$ELF_OUTPUT" 2>/dev/null || stat -f%z "$ELF_OUTPUT")

value "Built ELF:" "$(echo "$BUILT_HASH" | head -c 24)..."
value "Vendored ELF:" "$(echo "$VENDORED_HASH" | head -c 24)..."
value "Size:" "$BUILT_SIZE bytes"
echo ""

if [[ "$BUILT_HASH" == "$VENDORED_HASH" ]]; then
    success "ELF hashes match — reproducible build confirmed"
else
    fail "ELF mismatch! Built: ${BUILT_HASH:0:16}... vs Vendored: ${VENDORED_HASH:0:16}..."
fi

# ─── Proof Verification ──────────────────────────────────────────────────────

verify_proof() {
    local label="$1"
    local proof_path="$2"
    local expected_qubits="$3"
    local expected_nonclifford="$4"

    header "${ROCKET}  Verifying: $label"

    step "Running SP1 Groth16 verifier..."
    echo ""

    OUTPUT=$(cd "$VENDOR_DIR" && cargo run --release -p verifier -- \
        --proof "$proof_path" \
        --elf "$ELF_OUTPUT" 2>&1)

    # Check for success
    if echo "$OUTPUT" | grep -q "Successfully verified"; then
        PROOF_TYPE=$(echo "$OUTPUT" | grep "Successfully verified" | sed 's/Successfully verified //' | sed 's/ proof\.//')
        success "Proof valid ($PROOF_TYPE)"
    else
        echo "$OUTPUT"
        fail "Verification failed!"
    fi

    # Extract values
    VKEY=$(echo "$OUTPUT" | grep "Verifying Key" | awk '{print $NF}')
    CIRCUIT_HASH=$(echo "$OUTPUT" | grep "Circuit hash" | awk '{print $NF}')
    NUM_TESTS=$(echo "$OUTPUT" | grep "Number of tests" | awk '{print $NF}')
    QUBITS=$(echo "$OUTPUT" | grep "Qubit count" | awk '{print $NF}')
    NONCLIFFORD=$(echo "$OUTPUT" | grep "non-Clifford" | awk '{print $NF}')
    TOTAL_OPS=$(echo "$OUTPUT" | grep "Total ops" | awk '{print $NF}')

    echo ""
    value "Verification Key:" "$VKEY"
    value "Circuit Hash:" "$CIRCUIT_HASH"
    value "Tests:" "$NUM_TESTS"
    value "Qubits:" "$QUBITS"
    value "Non-Clifford Gates:" "$NONCLIFFORD"
    value "Total Operations:" "$TOTAL_OPS"
    echo ""

    # Validate expected values
    if [[ "$VKEY" == "$EXPECTED_VKEY" ]]; then
        success "Verification key matches expected"
    else
        fail "Verification key mismatch!"
    fi

    if [[ "$QUBITS" == "$expected_qubits" ]]; then
        success "Qubit count matches claim ($expected_qubits)"
    else
        fail "Qubit count mismatch: got $QUBITS, expected $expected_qubits"
    fi

    if [[ "$NONCLIFFORD" -le "$expected_nonclifford" ]]; then
        success "Non-Clifford gate count ≤ $expected_nonclifford"
    else
        fail "Non-Clifford gate count exceeds claim: $NONCLIFFORD > $expected_nonclifford"
    fi
}

verify_proof \
    "Statement 1 — Low-Qubit Variant (1,175 qubits, ≤2.7M non-Clifford)" \
    "proofs/low_qubits/proof_9024.bin" \
    "1175" \
    "2700000"

verify_proof \
    "Statement 2 — Low-Toffoli Variant (1,425 qubits, ≤2.1M non-Clifford)" \
    "proofs/low_toffoli/proof_9024.bin" \
    "1425" \
    "2100000"

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo ""

# Box with fixed-width content (no ANSI-length guessing)
B="${BOLD}"
R="${NC}"
G="${GREEN}${BOLD}"
D="${DIM}"
C="${CYAN}"

echo -e "${B}╔══════════════════════════════════════════════════════════════════════════╗${R}"
echo -e "${B}║${R}                                                                          ${B}║${R}"
echo -e "${B}║${R}   ${G}ALL VERIFICATIONS PASSED${R}                                               ${B}║${R}"
echo -e "${B}║${R}                                                                          ${B}║${R}"
echo -e "${B}║${R}   ${D}Deterministic build reproduced the exact ELF binary.${R}                   ${B}║${R}"
echo -e "${B}║${R}   ${D}Both Groth16 SNARK proofs verified against the built program.${R}          ${B}║${R}"
echo -e "${B}║${R}   ${D}Vkey: 0x00ca4af6cb15dbd83ec3eaab3a066402...${R}                            ${B}║${R}"
echo -e "${B}║${R}                                                                          ${B}║${R}"
echo -e "${B}║${R}   ${D}What was proven:${R}                                                       ${B}║${R}"
echo -e "${B}║${R}    ${C}1.${R} Quantum circuit: ${B}1,175 qubits${R}, ${B}<= 2.7M${R} non-Clifford gates          ${B}║${R}"
echo -e "${B}║${R}       computes secp256k1 point addition (9,024 tests)                    ${B}║${R}"
echo -e "${B}║${R}    ${C}2.${R} Quantum circuit: ${B}1,425 qubits${R}, ${B}<= 2.1M${R} non-Clifford gates          ${B}║${R}"
echo -e "${B}║${R}       computes secp256k1 point addition (9,024 tests)                    ${B}║${R}"
echo -e "${B}║${R}                                                                          ${B}║${R}"
echo -e "${B}╚══════════════════════════════════════════════════════════════════════════╝${R}"
echo ""
