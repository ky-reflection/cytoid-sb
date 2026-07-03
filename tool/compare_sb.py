#!/usr/bin/env python3
"""Compare two storyboard inputs after cytoid-sb compile normalization."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def compile_to_json(input_path: Path) -> dict:
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        out = Path(tmp.name)
    subprocess.check_call(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "cytoid-sb-cli",
            "--",
            "compile",
            str(input_path),
            "-o",
            str(out),
        ],
        cwd=Path(__file__).resolve().parents[1],
    )
    return json.loads(out.read_text(encoding="utf-8"))


def json_equal(a, b, path: str = "$") -> list[str]:
    if isinstance(a, float) and isinstance(b, float):
        if a == b or abs(a - b) <= 1e-9 * max(1.0, abs(a), abs(b)):
            return []
        return [f"{path}: {a!r} != {b!r}"]
    if type(a) is not type(b):
        return [f"{path}: type {type(a).__name__} != {type(b).__name__}"]
    if isinstance(a, dict):
        diffs: list[str] = []
        keys_a = set(a)
        keys_b = set(b)
        for key in sorted(keys_a - keys_b):
            diffs.append(f"{path}.{key}: only in left")
        for key in sorted(keys_b - keys_a):
            diffs.append(f"{path}.{key}: only in right")
        for key in sorted(keys_a & keys_b):
            diffs.extend(json_equal(a[key], b[key], f"{path}.{key}"))
        return diffs
    if isinstance(a, list):
        if len(a) != len(b):
            return [f"{path}: len {len(a)} != {len(b)}"]
        diffs = []
        for i, (left, right) in enumerate(zip(a, b)):
            diffs.extend(json_equal(left, right, f"{path}[{i}]"))
        return diffs
    if a != b:
        return [f"{path}: {a!r} != {b!r}"]
    return []


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: compare_sb.py <reference.json|dir> <candidate.lua|json>", file=sys.stderr)
        return 2
    left = Path(sys.argv[1])
    right = Path(sys.argv[2])
    ref = compile_to_json(left)
    cand = compile_to_json(right)
    diffs = json_equal(ref, cand)
    if diffs:
        print(f"DIFF ({len(diffs)} issues), first 20:")
        for line in diffs[:20]:
            print(line)
        return 1
    print("ok: compiled outputs match")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
