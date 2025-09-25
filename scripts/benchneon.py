#!/usr/bin/env python3

# benchneon.py
# Copyright (c) 2025 Nyoom Engineering
# SPDX-License-Identifier: MIT

# Example usage:
#   ./scripts/benchneon.py --iterations 2 --warmups 0 --tokens 1000 --color-keys 200
#   ./scripts/benchneon.py --iterations 4 --warmups 1 --tokens 200000 --color-keys 20000
#
# Options:
#   --tokens, --color-keys, --iterations, --warmups, --profile, --bin, --no-build, --keep-temp

import argparse
import os
import random
import statistics
import subprocess
import sys
import tempfile
import time
from typing import Optional

ACCENTS = [
    "#08bdba",
    "#33b1ff",
    "#3ddbd9",
    "#42be65",
    "#78a9ff",
    "#82cfff",
    "#a6c8ff",
    "#be95ff",
    "#ee5396",
    "#ff7eb6",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark scalar vs NEON monochrome ramp transforms",
    )
    parser.add_argument("--tokens", type=int, default=60000, help="generated tokenColors entries")
    parser.add_argument("--color-keys", type=int, default=6000, help="generated [colors] entries")
    parser.add_argument("--iterations", type=int, default=12, help="runs per variant")
    parser.add_argument("--warmups", type=int, default=2, help="warmup runs to discard")
    parser.add_argument("--profile", choices=("release", "dev"), default="release")
    parser.add_argument("--bin", help="existing oxocarbon-themec binary")
    parser.add_argument(
        "--keep-temp",
        action="store_true",
        help="preserve temporary directory for inspection",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="skip cargo build even if --bin missing (requires manual binary)",
    )
    return parser.parse_args()


def synth_theme(path: str, tokens: int, color_keys: int) -> None:
    with open(path, "w", encoding="utf-8") as fh:
        fh.write('name = "Benchmark Monochrome"\n')
        fh.write('type = "dark"\n\n')
        fh.write('[colors]\n')
        for idx in range(color_keys):
            fh.write(f"color{idx} = \"{ACCENTS[idx % len(ACCENTS)]}\"\n")
        fh.write("\n")
        for idx in range(tokens):
            color = ACCENTS[idx % len(ACCENTS)]
            fh.write('[[tokenColors]]\n')
            fh.write(f'name = "token_{idx}"\n')
            fh.write(f'scope = ["scope.{idx}"]\n')
            fh.write(f'settings = {{ foreground = "{color}" }}\n')


def ensure_binary(root: str, profile: str, existing: Optional[str], allow_build: bool) -> str:
    if existing:
        if not os.path.isfile(existing) or not os.access(existing, os.X_OK):
            raise RuntimeError(f"Binary at {existing} is not executable")
        return existing

    target_bin = os.path.join(root, "target", profile, "oxocarbon-themec")
    if os.path.isfile(target_bin) and os.access(target_bin, os.X_OK):
        return target_bin

    if not allow_build:
        raise RuntimeError("Binary missing and --no-build was requested")

    print("Building benchmark binary...", flush=True)
    subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            os.path.join(root, "Cargo.toml"),
            f"--{profile}",
        ],
        check=True,
    )
    if not os.path.isfile(target_bin):
        raise RuntimeError(f"Failed to build {target_bin}")
    return target_bin


def run_variant(binary: str, theme_path: str, iterations: int, warmups: int, neon: bool) -> list[float]:
    env = os.environ.copy()
    flag = "+neon" if neon else "-neon"
    env["RUSTFLAGS"] = f"-C target-feature={flag}"
    cmd = [binary, "--mono", theme_path]

    timings: list[float] = []
    drop = min(warmups, max(0, iterations - 1))

    for _ in range(iterations):
        start = time.perf_counter()
        subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env)
        timings.append(time.perf_counter() - start)

    return timings[drop:]


def summarize(label: str, data: list[float]) -> None:
    mean = statistics.mean(data)
    median = statistics.median(data)
    duration_min = min(data)
    duration_max = max(data)
    print(
        f"{label:>6}: mean={mean * 1000:.2f} ms "
        f"median={median * 1000:.2f} ms min={duration_min * 1000:.2f} ms "
        f"max={duration_max * 1000:.2f} ms runs={len(data)}"
    )


def main() -> int:
    args = parse_args()
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

    with tempfile.TemporaryDirectory(prefix="oxocarbon-neon.") as tmp:
        theme_path = os.path.join(tmp, "benchmark-theme.toml")
        synth_theme(theme_path, args.tokens, args.color_keys)
        size_mib = os.path.getsize(theme_path) / (1024 * 1024)
        print(
            f"Synthetic theme at {theme_path} tokens={args.tokens} colors={args.color_keys} "
            f"size={size_mib:.1f} MiB",
        )

        binary = ensure_binary(root, args.profile, args.bin, not args.no_build)

        results = {
            label: run_variant(binary, theme_path, args.iterations, args.warmups, neon)
            for label, neon in [("scalar", False), ("neon", True)]
        }

        for label, data in results.items():
            summarize(label, data)

        scalar_mean = statistics.mean(results["scalar"])
        neon_mean = statistics.mean(results["neon"])
        speedup = scalar_mean / neon_mean if neon_mean else float("inf")
        print(f"Speedup (scalar/neon): {speedup:.2f}x")

        if args.keep_temp:
            print(f"Temporary directory preserved at {tmp}")
            input("Press Enter to continue and clean up...")

    return 0


if __name__ == "__main__":
    sys.exit(main())
