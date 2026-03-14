#!/usr/bin/env python3
"""
ZK Merkle Proof Benchmark Suite

Runs proof-generation benchmarks for all 9 ZK implementations,
parses /usr/bin/time -l output, and produces:
  - results table (printed + saved as results.txt)
  - bar chart of wall times (bench_wall_time.png)
  - bar chart of peak RAM (bench_peak_ram.png)
  - bar chart of proof sizes (bench_proof_size.png)

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

ROOT = Path(__file__).resolve().parent

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
    pre_cmds: list = field(default_factory=list)  # commands to run before benchmarking
    proof_files: list = field(default_factory=list)  # relative to workdir
    proof_size_override: int | None = None  # manual override in bytes


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
        name="cairo",
        display="Cairo",
        proving_system="STARK (Stwo)",
        workdir="cairo/merkle",
        pre_cmds=[
            "scarb execute --arguments-file input.json --output standard",
        ],
        bench_cmd="scarb prove --execution-id 1",
        proof_files=[],
        proof_size_override=None,  # Cairo proof size read from scarb output
    ),
    Framework(
        name="leo",
        display="Leo",
        proving_system="Marlin (snarkVM)",
        workdir="leo/merkle",
        bench_cmd=(
            'PRIVATE_KEY="APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH" '
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
]

FRAMEWORK_MAP = {f.name: f for f in FRAMEWORKS}

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


def run_framework(fw: Framework) -> BenchResult:
    """Run a single framework benchmark."""
    result = BenchResult(
        name=fw.name,
        display=fw.display,
        proving_system=fw.proving_system,
    )
    workdir = ROOT / fw.workdir

    if not workdir.exists():
        result.error = f"Directory not found: {workdir}"
        return result

    # Run pre-commands
    for cmd in fw.pre_cmds:
        print(f"  [{fw.display}] pre: {cmd}")
        proc = subprocess.run(
            cmd, shell=True, cwd=workdir,
            capture_output=True, text=True, timeout=600,
        )
        if proc.returncode != 0:
            result.error = f"Pre-command failed: {proc.stderr[:200]}"
            return result

    # Run the benchmark with /usr/bin/time -l
    full_cmd = f"/usr/bin/time -l {fw.bench_cmd}"
    print(f"  [{fw.display}] bench: {fw.bench_cmd}")
    try:
        proc = subprocess.run(
            full_cmd, shell=True, cwd=workdir,
            capture_output=True, text=True, timeout=1200,
        )
    except subprocess.TimeoutExpired:
        result.error = "Timed out (>20 min)"
        return result

    if proc.returncode != 0:
        result.error = f"Exit code {proc.returncode}: {proc.stderr[:300]}"
        return result

    # /usr/bin/time writes to stderr
    wall, rss = parse_time_output(proc.stderr)
    result.wall_time_s = wall
    result.peak_ram_bytes = rss
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

    out = ROOT / "bench_results.txt"
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
    wall_path = ROOT / "bench_wall_time.png"
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
    ram_path = ROOT / "bench_peak_ram.png"
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
    size_path = ROOT / "bench_proof_size.png"
    fig.savefig(size_path, dpi=150)
    plt.close(fig)
    print(f"Saved {size_path}")


# ---------------------------------------------------------------------------
# Results persistence
# ---------------------------------------------------------------------------

RESULTS_FILE = ROOT / "bench_results.json"

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
                        help="Run only these frameworks (default: all)")
    parser.add_argument("--skip-run", action="store_true",
                        help="Skip running benchmarks; just plot from last results")
    parser.add_argument("--list", action="store_true",
                        help="List available frameworks and exit")
    args = parser.parse_args()

    if args.list:
        for f in FRAMEWORKS:
            print(f"  {f.name:<10} {f.display} — {f.proving_system}")
        return

    if args.skip_run:
        results = load_results_json()
    else:
        targets = FRAMEWORKS
        if args.frameworks:
            targets = []
            for name in args.frameworks:
                if name not in FRAMEWORK_MAP:
                    print(f"Unknown framework: {name}. Use --list to see options.")
                    sys.exit(1)
                targets.append(FRAMEWORK_MAP[name])

        results = []
        for fw in targets:
            print(f"\n{'=' * 50}")
            print(f"  Benchmarking: {fw.display} ({fw.proving_system})")
            print(f"{'=' * 50}")
            r = run_framework(fw)
            results.append(r)
            if r.success:
                print(f"  -> {fmt_time(r.wall_time_s)}, "
                      f"{fmt_ram(r.peak_ram_bytes)}, "
                      f"{fmt_bytes(r.proof_size_bytes)}")
            else:
                print(f"  -> FAILED: {r.error}")

        save_results_json(results)

    print_results_table(results)
    save_charts(results)


if __name__ == "__main__":
    main()
