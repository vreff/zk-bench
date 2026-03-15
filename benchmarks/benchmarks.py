#!/usr/bin/env python3
"""
ZK Merkle Proof Benchmark Suite

Runs proof-generation benchmarks for all 9 ZK implementations,
parses /usr/bin/time -l output, and produces:
  - results table (printed + saved as results.txt)
  - bar chart of wall times (chart_results_wall_time.png)
  - bar chart of peak RAM (chart_results_peak_ram.png)
  - bar chart of proof sizes (chart_results_proof_size.png)

Usage:
    python3 benchmarks.py              # run all benchmarks
    python3 benchmarks.py --skip-run   # skip running, just plot from last results
    python3 benchmarks.py circom noir  # run only specific frameworks
"""

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BENCH_DIR = Path(__file__).resolve().parent

# Ensure tool paths are available
_extra_paths = [
    Path.home() / ".zokrates" / "bin",
]
for p in _extra_paths:
    if p.is_dir() and str(p) not in os.environ.get("PATH", ""):
        os.environ["PATH"] = os.environ.get("PATH", "") + os.pathsep + str(p)

# ---------------------------------------------------------------------------
# Framework definitions
# ---------------------------------------------------------------------------

@dataclass
class Framework:
    name: str
    display: str
    proving_system: str
    workdir: str               # relative to ROOT
    bench_cmd: str             # command wrapped with /usr/bin/time -l
    pre_cmds: list = field(default_factory=list)  # commands to run once before benchmarking
    pre_each_cmds: list = field(default_factory=list)  # commands to run before each bench iteration
    proof_files: list = field(default_factory=list)  # relative to workdir
    proof_size_override: int | None = None  # manual override in bytes
    variant: str = "single"    # "single" or "double"


FRAMEWORKS = [
    Framework(
        name="circom",
        display="Circom",
        proving_system="Groth16 (snarkjs)",
        workdir="circom/merkle",
        pre_cmds=[],
        bench_cmd=(
            "snarkjs groth16 prove "
            "build/merkle_final.zkey build/witness.wtns "
            "build/proof.json build/public.json"
        ),
        proof_files=["build/proof.json"],
    ),
    Framework(
        name="circom_plonk",
        display="Circom (PLONK)",
        proving_system="PLONK (snarkjs)",
        workdir="circom/merkle",
        pre_cmds=[],
        bench_cmd=(
            "snarkjs plonk prove "
            "build/merkle_plonk.zkey build/witness.wtns "
            "build/proof_plonk.json build/public_plonk.json"
        ),
        proof_files=["build/proof_plonk.json"],
    ),
    Framework(
        name="noir",
        display="Noir",
        proving_system="UltraHonk",
        workdir="noirlang/merkle",
        bench_cmd="bb prove -b ./target/merkle.json -w ./target/merkle.gz -o ./target",
        proof_files=["target/proof"],
    ),
    Framework(
        name="zokrates",
        display="ZoKrates",
        proving_system="Groth16 (bellman)",
        workdir="zokrates/merkle",
        bench_cmd=(
            "zokrates generate-proof "
            "-i build/merkle -b bellman -s g16 -p build/proving.key "
            "-w build/witness -j build/proof.json"
        ),
        proof_files=["build/proof.json"],
    ),
    Framework(
        name="zokrates_ark",
        display="ZoKrates (ark)",
        proving_system="Groth16 (arkworks)",
        workdir="zokrates/merkle",
        bench_cmd=(
            "zokrates generate-proof "
            "-i build/merkle -b ark -s g16 -p build/proving_ark.key "
            "-w build/witness -j build/proof_ark.json"
        ),
        proof_files=["build/proof_ark.json"],
    ),
    Framework(
        name="leo",
        display="Leo",
        proving_system="Marlin (snarkVM)",
        workdir="leo/merkle",
        bench_cmd=(
            'env PRIVATE_KEY="APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH" '
            "leo execute --network testnet "
            '--endpoint "https://api.explorer.provable.com/v1" --yes '
            "verify "
            "3795873241443991455451735146226102458893119113405484212358614283425718189900field "
            "42field 3u32 "
            "5032677853915026442484505200337051980545600190313243825534151256332463055896field "
            "2025782052806597445336394462093422610260230542964192141256089645210002703802field "
            "6518303460776629079511004668974420229885492538691518135386352722012076854807field"
        ),
        proof_files=["build/transaction.execution.json"],
    ),
    Framework(
        name="risc0",
        display="RISC Zero",
        proving_system="STARK (FRI)",
        workdir="risc0/merkle",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/host",
        proof_files=["proof.bin"],
    ),
    Framework(
        name="sp1",
        display="SP1",
        proving_system="STARK (Plonky3)",
        workdir="sp1/merkle/script",
        pre_cmds=["cargo build --release"],
        bench_cmd="cargo run --release -- --prove",
        proof_files=["../proof.bin"],
    ),
    Framework(
        name="jolt",
        display="Jolt",
        proving_system="Lasso (Dory PCS)",
        workdir="jolt/merkle",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/merkle",
        proof_files=["proof.bin"],
    ),
    Framework(
        name="powdr",
        display="powdr",
        proving_system="STARK (Plonky3)",
        workdir="powdr/merkle",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/merkle",
        proof_files=["powdr-target/chunk_0/guest_proof.bin"],
    ),
    Framework(
        name="cairo",
        display="Cairo",
        proving_system="STARK (Stwo)",
        workdir="cairo/merkle",
        pre_each_cmds=[
            "rm -rf target/execute",
            "sleep 5",
            "scarb execute --arguments-file input.json --output standard",
        ],
        bench_cmd="scarb prove --execution-id 1",
        proof_files=["target/execute/merkle/execution1/proof/proof.json"],
    ),
]

# ---------------------------------------------------------------------------
# Double Merkle frameworks (2x computation)
# ---------------------------------------------------------------------------

DOUBLE_FRAMEWORKS = [
    Framework(
        name="cairo_double",
        display="Cairo (2x)",
        proving_system="STARK (Stwo)",
        workdir="cairo/doubleMerkle",
        variant="double",
        pre_cmds=[
            # Generate input.json from test output
            (
                "scarb test 2>&1 | python3 -c \""
                "import sys, json; lines = sys.stdin.read().split('\\n'); "
                "d = {l.split(':')[0]: l.split(':')[1] for l in lines if ':' in l and l.startswith(('LEAF','INDEX','SIB','ROOT'))}; "
                "vals = [d.get(k,'0') for k in ['LEAF_A','INDEX_A','SIB_A0','SIB_A1','SIB_A2','ROOT_A','LEAF_B','INDEX_B','SIB_B0','SIB_B1','SIB_B2','ROOT_B']]; "
                "json.dump([hex(int(v)) for v in vals], open('input.json','w'))"
                "\""
            ),
        ],
        pre_each_cmds=[
            "rm -rf target/execute",
            "sleep 5",
            "scarb execute --arguments-file input.json --output standard",
        ],
        bench_cmd="scarb prove --execution-id 1",
        proof_files=["target/execute/double_merkle/execution1/proof/proof.json"],
    ),
    Framework(
        name="circom_double",
        display="Circom (2x)",
        proving_system="Groth16 (snarkjs)",
        workdir="circom/merkle",
        variant="double",
        pre_cmds=[
            "circom circuits/doubleMerkle.circom --r1cs --wasm --sym -o build",
            "node scripts/generate_double_input.js",
            (
                "node build/doubleMerkle_js/generate_witness.js "
                "build/doubleMerkle_js/doubleMerkle.wasm input_double.json "
                "build/witness_double.wtns"
            ),
            (
                "snarkjs groth16 setup build/doubleMerkle.r1cs pot12_final.ptau "
                "build/doubleMerkle_0000.zkey"
            ),
            (
                "snarkjs zkey contribute build/doubleMerkle_0000.zkey "
                'build/doubleMerkle_final.zkey --name="C1" -e="entropy"'
            ),
        ],
        bench_cmd=(
            "snarkjs groth16 prove "
            "build/doubleMerkle_final.zkey build/witness_double.wtns "
            "build/proof_double.json build/public_double.json"
        ),
        proof_files=["build/proof_double.json"],
    ),
    Framework(
        name="circom_plonk_double",
        display="Circom PLONK (2x)",
        proving_system="PLONK (snarkjs)",
        workdir="circom/merkle",
        variant="double",
        pre_cmds=[
            "circom circuits/doubleMerkle.circom --r1cs --wasm --sym -o build",
            "node scripts/generate_double_input.js",
            (
                "node build/doubleMerkle_js/generate_witness.js "
                "build/doubleMerkle_js/doubleMerkle.wasm input_double.json "
                "build/witness_double.wtns"
            ),
            (
                "snarkjs plonk setup build/doubleMerkle.r1cs pot13_final.ptau "
                "build/doubleMerkle_plonk.zkey"
            ),
        ],
        bench_cmd=(
            "snarkjs plonk prove "
            "build/doubleMerkle_plonk.zkey build/witness_double.wtns "
            "build/proof_double_plonk.json build/public_double_plonk.json"
        ),
        proof_files=["build/proof_double_plonk.json"],
    ),
    Framework(
        name="noir_double",
        display="Noir (2x)",
        proving_system="UltraHonk",
        workdir="noirlang/doubleMerkle",
        variant="double",
        pre_cmds=[
            "nargo test --show-output 2>&1 | sed -n '/PROVER_TOML_START/,/PROVER_TOML_END/p' | grep -v PROVER_TOML > Prover.toml",
            "nargo compile",
            "nargo execute",
            "bb write_vk -b ./target/double_merkle.json -o ./target",
        ],
        bench_cmd="bb prove -b ./target/double_merkle.json -w ./target/double_merkle.gz -o ./target",
        proof_files=["target/proof"],
    ),
    Framework(
        name="zokrates_double",
        display="ZoKrates (2x)",
        proving_system="Groth16 (bellman)",
        workdir="zokrates/merkle",
        variant="double",
        pre_cmds=[
            "zokrates compile -i circuits/doubleMerkle.zok -o build/doubleMerkle",
            "node scripts/generate_double_input.js",
            "bash build/double_witness_cmd.sh",
            (
                "zokrates setup -i build/doubleMerkle -b bellman -s g16 "
                "-p build/proving_double.key -v build/verification_double.key"
            ),
        ],
        bench_cmd=(
            "zokrates generate-proof "
            "-i build/doubleMerkle -b bellman -s g16 -p build/proving_double.key "
            "-w build/witness_double -j build/proof_double.json"
        ),
        proof_files=["build/proof_double.json"],
    ),
    Framework(
        name="zokrates_ark_double",
        display="ZoKrates ark (2x)",
        proving_system="Groth16 (arkworks)",
        workdir="zokrates/merkle",
        variant="double",
        pre_cmds=[
            "zokrates compile -i circuits/doubleMerkle.zok -o build/doubleMerkle",
            "node scripts/generate_double_input.js",
            "bash build/double_witness_cmd.sh",
            (
                "zokrates setup -i build/doubleMerkle -b ark -s g16 "
                "-p build/proving_double_ark.key -v build/verification_double_ark.key"
            ),
        ],
        bench_cmd=(
            "zokrates generate-proof "
            "-i build/doubleMerkle -b ark -s g16 -p build/proving_double_ark.key "
            "-w build/witness_double -j build/proof_double_ark.json"
        ),
        proof_files=["build/proof_double_ark.json"],
    ),
    Framework(
        name="risc0_double",
        display="RISC Zero (2x)",
        proving_system="STARK (FRI)",
        workdir="risc0/doubleMerkle",
        variant="double",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/double_host",
        proof_files=["proof.bin"],
    ),
    Framework(
        name="sp1_double",
        display="SP1 (2x)",
        proving_system="STARK (Plonky3)",
        workdir="sp1/doubleMerkle/script",
        variant="double",
        pre_cmds=["cargo build --release"],
        bench_cmd="cargo run --release -- --prove",
        proof_files=["../proof.bin"],
    ),
    Framework(
        name="jolt_double",
        display="Jolt (2x)",
        proving_system="Lasso (Dory PCS)",
        workdir="jolt/doubleMerkle",
        variant="double",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/double_merkle",
        proof_files=["proof.bin"],
    ),
    Framework(
        name="powdr_double",
        display="powdr (2x)",
        proving_system="STARK (Plonky3)",
        workdir="powdr/doubleMerkle",
        variant="double",
        pre_cmds=["cargo build --release"],
        bench_cmd="./target/release/double_merkle",
        proof_files=["powdr-target/chunk_0/guest_proof.bin"],
    ),
]

ALL_FRAMEWORKS = FRAMEWORKS + DOUBLE_FRAMEWORKS

FRAMEWORK_MAP = {f.name: f for f in ALL_FRAMEWORKS}

# ---------------------------------------------------------------------------
# Runner
# ---------------------------------------------------------------------------

@dataclass
class BenchResult:
    name: str
    display: str
    proving_system: str
    wall_time_s: float = 0.0
    peak_ram_bytes: int = 0
    proof_size_bytes: int = 0
    success: bool = False
    error: str = ""
    variant: str = "single"


def parse_time_output(stderr: str) -> tuple[float, int]:
    """Parse macOS /usr/bin/time -l stderr output for wall time and peak RSS."""
    wall = 0.0
    rss = 0

    # wall time: "  1.23 real  ..." or "1.23 real"
    m = re.search(r"([\d.]+)\s+real", stderr)
    if m:
        wall = float(m.group(1))

    # peak RSS: "  12345678  maximum resident set size"
    m = re.search(r"(\d+)\s+maximum resident set size", stderr)
    if m:
        rss = int(m.group(1))

    return wall, rss


def measure_proof_size(fw: Framework) -> int:
    """Return total proof size in bytes."""
    if fw.proof_size_override is not None:
        return fw.proof_size_override

    total = 0
    workdir = ROOT / fw.workdir
    for pf in fw.proof_files:
        p = workdir / pf
        if p.exists():
            total += p.stat().st_size
    return total


def run_framework(fw: Framework, num_runs: int = 3) -> BenchResult:
    """Run a single framework benchmark num_runs times and average results."""
    result = BenchResult(
        name=fw.name,
        display=fw.display,
        proving_system=fw.proving_system,
        variant=fw.variant,
    )
    workdir = ROOT / fw.workdir

    if not workdir.exists():
        result.error = f"Directory not found: {workdir}"
        return result

    # Run pre-commands (once)
    for cmd in fw.pre_cmds:
        print(f"  [{fw.display}] pre: {cmd}")
        proc = subprocess.run(
            cmd, shell=True, cwd=workdir,
            capture_output=True, text=True, timeout=600,
        )
        if proc.returncode != 0:
            result.error = f"Pre-command failed: {proc.stderr[:200]}"
            return result

    # Run the benchmark num_runs times
    wall_times = []
    peak_rams = []
    for i in range(num_runs):
        # Run per-iteration pre-commands
        for cmd in fw.pre_each_cmds:
            print(f"  [{fw.display}] pre-each: {cmd}")
            proc = subprocess.run(
                cmd, shell=True, cwd=workdir,
                capture_output=True, text=True, timeout=600,
            )
            if proc.returncode != 0:
                result.error = f"Pre-each command failed: {proc.stderr[:200]}"
                return result

        run_label = f"run {i + 1}/{num_runs}" if num_runs > 1 else "bench"
        iter_bench_cmd = fw.bench_cmd.replace("{run}", str(i + 1))
        iter_full_cmd = f"/usr/bin/time -l {iter_bench_cmd}"
        print(f"  [{fw.display}] {run_label}: {iter_bench_cmd}")
        try:
            proc = subprocess.run(
                iter_full_cmd, shell=True, cwd=workdir,
                capture_output=True, text=True, timeout=1200,
            )
        except subprocess.TimeoutExpired:
            result.error = "Timed out (>20 min)"
            return result

        if proc.returncode != 0:
            result.error = f"Exit code {proc.returncode}: {proc.stderr[:300]}"
            return result

        wall, rss = parse_time_output(proc.stderr)
        wall_times.append(wall)
        peak_rams.append(rss)

    result.wall_time_s = sum(wall_times) / len(wall_times)
    result.peak_ram_bytes = int(sum(peak_rams) / len(peak_rams))
    result.proof_size_bytes = measure_proof_size(fw)
    result.success = True
    return result


# ---------------------------------------------------------------------------
# Formatting helpers
# ---------------------------------------------------------------------------

def fmt_bytes(b: int) -> str:
    if b == 0:
        return "N/A"
    if b < 1024:
        return f"{b} B"
    if b < 1024 * 1024:
        return f"{b / 1024:.1f} KB"
    return f"{b / (1024 * 1024):.1f} MB"


def fmt_ram(b: int) -> str:
    if b == 0:
        return "N/A"
    mb = b / (1024 * 1024)
    if mb >= 1000:
        return f"{mb:,.0f} MB"
    return f"{mb:.0f} MB"


def fmt_time(s: float) -> str:
    if s == 0:
        return "N/A"
    if s < 1:
        return f"{s:.2f} s"
    if s < 10:
        return f"{s:.1f} s"
    return f"{s:.1f} s"


# ---------------------------------------------------------------------------
# Output: Table
# ---------------------------------------------------------------------------

def print_results_table(results: list[BenchResult]):
    # Sort by wall time
    ok = sorted([r for r in results if r.success], key=lambda r: r.wall_time_s)
    fail = [r for r in results if not r.success]

    header = f"{'Framework':<14} {'Proving System':<22} {'Peak RAM':>10} {'Wall Time':>11} {'Proof Size':>12}"
    sep = "-" * len(header)

    lines = [
        "ZK Merkle Proof Benchmarks",
        "=" * 26,
        "",
        header,
        sep,
    ]
    for r in ok:
        lines.append(
            f"{r.display:<14} {r.proving_system:<22} {fmt_ram(r.peak_ram_bytes):>10} "
            f"{fmt_time(r.wall_time_s):>11} {fmt_bytes(r.proof_size_bytes):>12}"
        )
    for r in fail:
        lines.append(f"{r.display:<14} {'FAILED':<22} {'—':>10} {'—':>11} {'—':>12}")
        lines.append(f"  Error: {r.error}")

    text = "\n".join(lines)
    print("\n" + text + "\n")

    out = BENCH_DIR / "bench_results.txt"
    out.write_text(text + "\n")
    print(f"Saved table to {out}")


# ---------------------------------------------------------------------------
# Output: Charts
# ---------------------------------------------------------------------------

def save_charts(results: list[BenchResult]):
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        import matplotlib.ticker as ticker
    except ImportError:
        print("matplotlib not installed — skipping charts. Install with: pip install matplotlib")
        return

    ok = sorted([r for r in results if r.success], key=lambda r: r.wall_time_s)
    if not ok:
        return

    names = [r.display for r in ok]

    # Color palette: group by proving-system family
    def color_for(r: BenchResult) -> str:
        ps = r.proving_system.lower()
        if "groth16" in ps:
            return "#4C72B0"
        if "ultrahonk" in ps:
            return "#55A868"
        if "marlin" in ps:
            return "#C44E52"
        if "lasso" in ps:
            return "#8172B2"
        if "plonky3" in ps:
            return "#CCB974"
        if "stwo" in ps:
            return "#DA8BC3"
        if "fri" in ps:
            return "#64B5CD"
        return "#999999"

    colors = [color_for(r) for r in ok]

    # --- Wall Time ---
    fig, ax = plt.subplots(figsize=(10, 5))
    times = [r.wall_time_s for r in ok]
    bars = ax.barh(names, times, color=colors, edgecolor="white", linewidth=0.5)
    ax.set_xlabel("Wall Time (seconds)")
    ax.set_title("ZK Proof Generation — Wall Time")
    ax.invert_yaxis()
    for bar, t in zip(bars, times):
        ax.text(bar.get_width() + max(times) * 0.01, bar.get_y() + bar.get_height() / 2,
                fmt_time(t), va="center", fontsize=9)
    ax.set_xlim(0, max(times) * 1.18)
    plt.tight_layout()
    wall_path = BENCH_DIR / "chart_results_wall_time.png"
    fig.savefig(wall_path, dpi=150)
    plt.close(fig)
    print(f"Saved {wall_path}")

    # --- Peak RAM ---
    fig, ax = plt.subplots(figsize=(10, 5))
    ram_mb = [r.peak_ram_bytes / (1024 * 1024) for r in ok]
    bars = ax.barh(names, ram_mb, color=colors, edgecolor="white", linewidth=0.5)
    ax.set_xlabel("Peak RSS (MB)")
    ax.set_title("ZK Proof Generation — Peak Memory")
    ax.invert_yaxis()
    for bar, mb in zip(bars, ram_mb):
        ax.text(bar.get_width() + max(ram_mb) * 0.01, bar.get_y() + bar.get_height() / 2,
                fmt_ram(int(mb * 1024 * 1024)), va="center", fontsize=9)
    ax.set_xlim(0, max(ram_mb) * 1.18)
    plt.tight_layout()
    ram_path = BENCH_DIR / "chart_results_peak_ram.png"
    fig.savefig(ram_path, dpi=150)
    plt.close(fig)
    print(f"Saved {ram_path}")

    # --- Proof Size (log scale) ---
    fig, ax = plt.subplots(figsize=(10, 5))
    sizes = [r.proof_size_bytes for r in ok if r.proof_size_bytes > 0]
    ok_with_size = [r for r in ok if r.proof_size_bytes > 0]
    names_s = [r.display for r in ok_with_size]
    colors_s = [color_for(r) for r in ok_with_size]
    sizes_kb = [s / 1024 for s in sizes]
    bars = ax.barh(names_s, sizes_kb, color=colors_s, edgecolor="white", linewidth=0.5)
    ax.set_xlabel("Proof Size (KB, log scale)")
    ax.set_title("ZK Proof Generation — Proof Size")
    ax.set_xscale("log")
    ax.invert_yaxis()
    for bar, sz in zip(bars, [r.proof_size_bytes for r in ok_with_size]):
        ax.text(bar.get_width() * 1.15, bar.get_y() + bar.get_height() / 2,
                fmt_bytes(sz), va="center", fontsize=9)
    plt.tight_layout()
    size_path = BENCH_DIR / "chart_results_proof_size.png"
    fig.savefig(size_path, dpi=150)
    plt.close(fig)
    print(f"Saved {size_path}")


def save_scaling_charts(results: list[BenchResult]):
    """Generate grouped bar charts comparing single vs double Merkle benchmarks."""
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        import numpy as np
    except ImportError:
        print("matplotlib not installed — skipping scaling charts.")
        return

    single = {r.name: r for r in results if r.variant == "single" and r.success}
    double = {r.name.replace("_double", ""): r for r in results if r.variant == "double" and r.success}

    # Find frameworks that have both single and double results
    # Map double base name to single name (strip _plonk, _ark suffixes for matching)
    paired = []
    for base_name, d_result in sorted(double.items()):
        if base_name in single:
            paired.append((single[base_name], d_result))

    if not paired:
        print("No paired single/double results for scaling charts.")
        return

    labels = [s.display for s, _ in paired]
    x = np.arange(len(labels))
    width = 0.35

    # --- Wall Time Scaling ---
    fig, ax = plt.subplots(figsize=(12, 6))
    single_times = [s.wall_time_s for s, _ in paired]
    double_times = [d.wall_time_s for _, d in paired]

    bars1 = ax.bar(x - width/2, single_times, width, label="Single Merkle", color="#4C72B0", edgecolor="white")
    bars2 = ax.bar(x + width/2, double_times, width, label="Double Merkle", color="#C44E52", edgecolor="white")

    ax.set_ylabel("Wall Time (seconds)")
    ax.set_title("Scaling: Single vs Double Merkle — Wall Time")
    ax.set_xticks(x)
    ax.set_xticklabels(labels, rotation=30, ha="right")
    ax.legend()

    # Add multiplier labels on double bars
    for i, (s, d) in enumerate(zip(single_times, double_times)):
        if s > 0:
            mult = d / s
            ax.text(x[i] + width/2, d + max(double_times) * 0.02,
                    f"{mult:.1f}x", ha="center", fontsize=8, fontweight="bold")

    ax.set_ylim(0, max(double_times) * 1.15)
    plt.tight_layout()
    path = BENCH_DIR / "chart_scaling_time.png"
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"Saved {path}")

    # --- Peak RAM Scaling ---
    fig, ax = plt.subplots(figsize=(12, 6))
    single_ram = [s.peak_ram_bytes / (1024 * 1024) for s, _ in paired]
    double_ram = [d.peak_ram_bytes / (1024 * 1024) for _, d in paired]

    bars1 = ax.bar(x - width/2, single_ram, width, label="Single Merkle", color="#4C72B0", edgecolor="white")
    bars2 = ax.bar(x + width/2, double_ram, width, label="Double Merkle", color="#C44E52", edgecolor="white")

    ax.set_ylabel("Peak RSS (MB)")
    ax.set_title("Scaling: Single vs Double Merkle — Peak Memory")
    ax.set_xticks(x)
    ax.set_xticklabels(labels, rotation=30, ha="right")
    ax.legend()

    for i, (s, d) in enumerate(zip(single_ram, double_ram)):
        if s > 0:
            mult = d / s
            ax.text(x[i] + width/2, d + max(double_ram) * 0.02,
                    f"{mult:.1f}x", ha="center", fontsize=8, fontweight="bold")

    ax.set_ylim(0, max(double_ram) * 1.15)
    plt.tight_layout()
    path = BENCH_DIR / "chart_scaling_ram.png"
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"Saved {path}")

    # --- Proof Size Scaling ---
    paired_with_size = [(s, d) for s, d in paired if s.proof_size_bytes > 0 and d.proof_size_bytes > 0]
    if paired_with_size:
        labels_s = [s.display for s, _ in paired_with_size]
        x_s = np.arange(len(labels_s))
        single_sizes = [s.proof_size_bytes / 1024 for s, _ in paired_with_size]
        double_sizes = [d.proof_size_bytes / 1024 for _, d in paired_with_size]

        fig, ax = plt.subplots(figsize=(12, 6))
        bars1 = ax.bar(x_s - width/2, single_sizes, width, label="Single Merkle", color="#4C72B0", edgecolor="white")
        bars2 = ax.bar(x_s + width/2, double_sizes, width, label="Double Merkle", color="#C44E52", edgecolor="white")

        ax.set_ylabel("Proof Size (KB)")
        ax.set_title("Scaling: Single vs Double Merkle — Proof Size")
        ax.set_xticks(x_s)
        ax.set_xticklabels(labels_s, rotation=30, ha="right")
        ax.set_yscale("log")
        ax.legend()

        for i, (s, d) in enumerate(zip(single_sizes, double_sizes)):
            if s > 0:
                mult = d / s
                ax.text(x_s[i] + width/2, d * 1.3,
                        f"{mult:.1f}x", ha="center", fontsize=8, fontweight="bold")

        plt.tight_layout()
        path = BENCH_DIR / "chart_scaling_proof_size.png"
        fig.savefig(path, dpi=150)
        plt.close(fig)
        print(f"Saved {path}")


# ---------------------------------------------------------------------------
# Results persistence
# ---------------------------------------------------------------------------

RESULTS_FILE = BENCH_DIR / "bench_results.json"

def save_results_json(results: list[BenchResult]):
    data = []
    for r in results:
        data.append({
            "name": r.name,
            "display": r.display,
            "proving_system": r.proving_system,
            "wall_time_s": r.wall_time_s,
            "peak_ram_bytes": r.peak_ram_bytes,
            "proof_size_bytes": r.proof_size_bytes,
            "success": r.success,
            "error": r.error,
            "variant": r.variant,
        })
    RESULTS_FILE.write_text(json.dumps(data, indent=2) + "\n")
    print(f"Saved raw data to {RESULTS_FILE}")


def load_results_json() -> list[BenchResult]:
    if not RESULTS_FILE.exists():
        print(f"No previous results at {RESULTS_FILE}")
        sys.exit(1)
    data = json.loads(RESULTS_FILE.read_text())
    return [BenchResult(**d) for d in data]


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="ZK Merkle Proof Benchmark Suite")
    parser.add_argument("frameworks", nargs="*",
                        help="Run only these frameworks (default: all single)")
    parser.add_argument("--skip-run", action="store_true",
                        help="Skip running benchmarks; just plot from last results")
    parser.add_argument("--double", action="store_true",
                        help="Also run double Merkle benchmarks")
    parser.add_argument("--double-only", action="store_true",
                        help="Run only double Merkle benchmarks")
    parser.add_argument("--runs", type=int, default=3,
                        help="Number of benchmark runs per framework (default: 3)")
    parser.add_argument("--list", action="store_true",
                        help="List available frameworks and exit")
    args = parser.parse_args()

    if args.list:
        print("Single Merkle:")
        for f in FRAMEWORKS:
            print(f"  {f.name:<20} {f.display} — {f.proving_system}")
        print("\nDouble Merkle:")
        for f in DOUBLE_FRAMEWORKS:
            print(f"  {f.name:<20} {f.display} — {f.proving_system}")
        return

    if args.skip_run:
        results = load_results_json()
    else:
        if args.frameworks:
            targets = []
            for name in args.frameworks:
                if name not in FRAMEWORK_MAP:
                    print(f"Unknown framework: {name}. Use --list to see options.")
                    sys.exit(1)
                targets.append(FRAMEWORK_MAP[name])
        elif args.double_only:
            targets = DOUBLE_FRAMEWORKS
        elif args.double:
            targets = ALL_FRAMEWORKS
        else:
            targets = FRAMEWORKS

        results = []
        for fw in targets:
            print(f"\n{'=' * 50}")
            print(f"  Benchmarking: {fw.display} ({fw.proving_system})")
            print(f"{'=' * 50}")
            r = run_framework(fw, num_runs=args.runs)
            results.append(r)
            if r.success:
                print(f"  -> {fmt_time(r.wall_time_s)}, "
                      f"{fmt_ram(r.peak_ram_bytes)}, "
                      f"{fmt_bytes(r.proof_size_bytes)}")
            else:
                print(f"  -> FAILED: {r.error}")

        save_results_json(results)

    print_results_table(results)

    single_results = [r for r in results if r.variant == "single"]
    double_results = [r for r in results if r.variant == "double"]

    if single_results:
        save_charts(single_results)
    if single_results and double_results:
        save_scaling_charts(results)


if __name__ == "__main__":
    main()
