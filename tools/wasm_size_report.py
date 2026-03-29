#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


BUILD_TARGET = "wasm32-unknown-unknown"
RUSTFLAGS = "-Zunstable-options -Cpanic=immediate-abort"
BUILD_STD_ARGS = ["-Z", "build-std=std,panic_abort,core,alloc"]


@dataclass
class BuildSpec:
    name: str
    package: str


@dataclass
class BuildResult:
    name: str
    package: str
    wasm_path: str
    size_bytes: int
    stripped_size_bytes: int
    twiggy_top: str


DEFAULT_SPECS = [
    BuildSpec(name="example_motion", package="leptos_fluid_motion_example"),
    BuildSpec(
        name="example_motion_controller",
        package="leptos_fluid_motion_controller_example",
    ),
]


def run(command: list[str], *, cwd: Path, env: dict[str, str]) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            "command failed: {}\nstdout:\n{}\nstderr:\n{}".format(
                " ".join(command), completed.stdout, completed.stderr
            )
        )
    return completed.stdout


def read_leb_u32(data: bytes, start: int) -> tuple[int, int]:
    result = 0
    shift = 0
    index = start
    while True:
        byte = data[index]
        index += 1
        result |= (byte & 0x7F) << shift
        if byte & 0x80 == 0:
            return result, index
        shift += 7


def strip_custom_sections(wasm_bytes: bytes) -> bytes:
    strip_names = {b"__wasm_bindgen_unstable", b"name", b"producers"}
    if len(wasm_bytes) < 8 or wasm_bytes[:4] != b"\0asm":
        raise ValueError("not a wasm module")

    output = bytearray(wasm_bytes[:8])
    index = 8
    while index < len(wasm_bytes):
        section_start = index
        section_id = wasm_bytes[index]
        index += 1
        size, index = read_leb_u32(wasm_bytes, index)
        payload_start = index
        payload_end = payload_start + size

        keep = True
        if section_id == 0:
            name_len, name_index = read_leb_u32(wasm_bytes, payload_start)
            name = wasm_bytes[name_index : name_index + name_len]
            keep = name not in strip_names

        if keep:
            output.extend(wasm_bytes[section_start:payload_end])
        index = payload_end

    return bytes(output)


def build_example(
    repo_root: Path, target_dir: Path, spec: BuildSpec, twiggy_top_n: int
) -> BuildResult:
    env = os.environ.copy()
    env["RUSTFLAGS"] = RUSTFLAGS
    env["CARGO_TARGET_DIR"] = str(target_dir)

    cargo_command = [
        "cargo",
        "build",
        "--release",
        "--target",
        BUILD_TARGET,
        *BUILD_STD_ARGS,
        "-p",
        spec.package,
    ]
    run(cargo_command, cwd=repo_root, env=env)

    wasm_path = target_dir / BUILD_TARGET / "release" / f"{spec.package}.wasm"
    twiggy_output = run(
        ["twiggy", "top", "-n", str(twiggy_top_n), str(wasm_path)],
        cwd=repo_root,
        env=env,
    )
    stripped_size_bytes = len(strip_custom_sections(wasm_path.read_bytes()))

    return BuildResult(
        name=spec.name,
        package=spec.package,
        wasm_path=str(wasm_path),
        size_bytes=wasm_path.stat().st_size,
        stripped_size_bytes=stripped_size_bytes,
        twiggy_top=twiggy_output,
    )


def capture(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    target_dir = Path(args.target_dir).resolve()
    target_dir.mkdir(parents=True, exist_ok=True)

    results = [
        build_example(repo_root, target_dir, spec, args.twiggy_top_n)
        for spec in DEFAULT_SPECS
    ]

    payload = {
        "repo_root": str(repo_root),
        "target_dir": str(target_dir),
        "builds": [asdict(result) for result in results],
    }
    Path(args.output).write_text(json.dumps(payload, indent=2) + "\n")
    return 0


def compare(args: argparse.Namespace) -> int:
    baseline = json.loads(Path(args.baseline).read_text())
    current = json.loads(Path(args.current).read_text())

    baseline_by_name = {entry["name"]: entry for entry in baseline["builds"]}
    current_by_name = {entry["name"]: entry for entry in current["builds"]}

    lines = [
        "| Build | Baseline bytes | Final bytes | Delta bytes | Delta % |",
        "|---|---:|---:|---:|---:|",
    ]

    for name in sorted(current_by_name):
        base = baseline_by_name[name]
        cur = current_by_name[name]
        delta = cur["size_bytes"] - base["size_bytes"]
        pct = (delta / base["size_bytes"]) * 100 if base["size_bytes"] else 0.0
        lines.append(
            f"| `{name}` | {base['size_bytes']} | {cur['size_bytes']} | {delta} | {pct:.2f}% |"
        )

    lines.append("")
    lines.append(
        "| Build | Baseline stripped | Final stripped | Delta bytes | Delta % |"
    )
    lines.append("|---|---:|---:|---:|---:|")

    for name in sorted(current_by_name):
        base = baseline_by_name[name]
        cur = current_by_name[name]
        base_stripped = base.get("stripped_size_bytes")
        if base_stripped is None:
            base_stripped = len(
                strip_custom_sections(Path(base["wasm_path"]).read_bytes())
            )
        cur_stripped = cur.get("stripped_size_bytes")
        if cur_stripped is None:
            cur_stripped = len(
                strip_custom_sections(Path(cur["wasm_path"]).read_bytes())
            )
        delta = cur_stripped - base_stripped
        pct = (delta / base_stripped) * 100 if base_stripped else 0.0
        lines.append(
            f"| `{name}` | {base_stripped} | {cur_stripped} | {delta} | {pct:.2f}% |"
        )

    lines.append("")
    lines.append(
        "`twiggy top` snapshots are stored in the JSON artifacts for deeper inspection."
    )
    output = "\n".join(lines) + "\n"

    if args.output:
        Path(args.output).write_text(output)
    else:
        sys.stdout.write(output)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Capture and compare release wasm size snapshots."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    capture_parser = subparsers.add_parser(
        "capture", help="Build release wasm artifacts and record sizes."
    )
    capture_parser.add_argument("--repo-root", required=True)
    capture_parser.add_argument("--target-dir", required=True)
    capture_parser.add_argument("--output", required=True)
    capture_parser.add_argument("--twiggy-top-n", type=int, default=20)
    capture_parser.set_defaults(func=capture)

    compare_parser = subparsers.add_parser(
        "compare", help="Compare two captured size snapshots."
    )
    compare_parser.add_argument("--baseline", required=True)
    compare_parser.add_argument("--current", required=True)
    compare_parser.add_argument("--output")
    compare_parser.set_defaults(func=compare)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
