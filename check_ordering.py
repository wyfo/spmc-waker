#!/usr/bin/env python3
"""
For each non-Relaxed memory ordering (and each fence) in src/lib.rs,
mutate it one step weaker and run loom tests expecting a failure.
Skips commented lines and lines containing NOOP_VTABLE.

Usage: python check_ordering.py
"""
import os
import re
import subprocess
import sys

FILE = "src/lib.rs"
TEST_CMD = ["cargo", "test", "--release"]
TEST_ENV = {**os.environ, "LOOM_MAX_PREEMPTIONS": "1", "RUSTFLAGS": "--cfg=loom"}

DOWNGRADE = {
    "SeqCst": ["AcqRel"],
    "AcqRel": ["Acquire", "Release"],
    "Acquire": ["Relaxed"],
    "Release": ["Relaxed"],
}
ORDERING_RE = re.compile(r"\b(SeqCst|AcqRel|Acquire|Release)\b")
FENCE_RE = re.compile(r"^(\s*)fence\(")


def find_mutations(lines):
    """Yield (line_idx, start, end, old, new) for each mutation to test."""
    for i, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or "NOOP_VTABLE" in line:
            continue
        if FENCE_RE.match(line):
            # Remove the fence statement by commenting it out
            yield (i, None, None, line, FENCE_RE.sub(r"\1// fence(", line, count=1))
            continue
        for m in ORDERING_RE.finditer(line):
            old = m.group(1)
            for new in DOWNGRADE[old]:
                yield (i, m.start(), m.end(), old, new)


def apply(lines, line_idx, start, end, new):
    mutant = lines.copy()
    if start is None:
        mutant[line_idx] = new
    else:
        line = lines[line_idx]
        mutant[line_idx] = line[:start] + new + line[end:]
    return mutant


def run_loom():
    proc = subprocess.Popen(
        TEST_CMD, env=TEST_ENV, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
    )
    for line in proc.stdout:
        if "FAILED" in line:
            proc.kill()
            proc.wait()
            return True
    proc.wait()
    return proc.returncode != 0


def main():
    with open(FILE) as f:
        original = f.read()
    lines = original.splitlines(keepends=True)

    mutations = list(find_mutations(lines))
    n_orderings = len({(li, s, e, o) for li, s, e, o, _ in mutations})
    print(f"Found {n_orderings} orderings ({len(mutations)} mutations) to test\n")

    unnecessary = []
    try:
        for idx, (line_idx, start, end, old, new) in enumerate(mutations, 1):
            if start is None:
                desc = f"line {line_idx + 1}: remove fence"
            else:
                desc = f"line {line_idx + 1}: {old} → {new}"

            print(f"[{idx}/{len(mutations)}] {desc} ... ", end="", flush=True)

            with open(FILE, "w") as f:
                f.writelines(apply(lines, line_idx, start, end, new))

            if run_loom():
                print("FAIL ✓")
            else:
                print("PASS ✗  <-- ordering may be unnecessary!")
                unnecessary.append(desc)
    finally:
        with open(FILE, "w") as f:
            f.write(original)

    print(f"\n{'=' * 50}")
    if unnecessary:
        print(f"WARNING: {len(unnecessary)} possibly unnecessary ordering(s):")
        for d in unnecessary:
            print(f"  {d}")
        sys.exit(1)
    else:
        print(f"All {n_orderings} orderings are necessary.")


if __name__ == "__main__":
    main()
