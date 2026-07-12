#!/usr/bin/env python3
"""
For each non-Relaxed memory ordering (and each fence) in src/lib.rs, weaken it
one step and run loom tests expecting a failure — a downgrade that no test
catches may be unnecessary.

The ordering is weakened at *runtime*, not by editing the source: `src/loom.rs`
reads a `LOOM_DOWNGRADE="<line>:<from>:<to>"` (or `"<line>:fence"`) env var and,
via `#[track_caller]`, substitutes/skips the ordering at the matching call site.
So the test binary is built ONCE and every downgrade is just another run of it —
no per-downgrade recompilation.

Usage: python check_ordering.py
"""
import json
import os
import re
import subprocess
import sys

FILE = "src/lib.rs"
BASE_ENV = {**os.environ, "RUSTFLAGS": "--cfg=loom -C debug_assertions"}
# Build the loom integration test once; skip the doctests.
BUILD_CMD = [
    "cargo", "test", "--release", "--test", "spmc_waker",
    "--no-run", "--message-format=json",
]
# Try the cheap preemption bound first; escalate only if the downgrade still passes.
PREEMPTIONS = [1, 2]
# `basic_notification` is a broad, slow test imported verbatim from tokio's
# suite; skipping it is the bulk of the speed-up, and the other tests still catch
# every downgrade. A skip-list is deliberate over an allow-list of the catching
# tests: those are local and may be renamed (silently dropping coverage), whereas
# `basic_notification` has a stable name inherited from tokio. A downgrade only
# it caught would surface as "may be unnecessary".
SKIP_TESTS = ["basic_notification"]

DOWNGRADE = {
    "SeqCst": ["AcqRel", "Acquire", "Release"],
    "AcqRel": ["Acquire", "Release"],
    "Acquire": ["Relaxed"],
    "Release": ["Relaxed"],
}
ORDERING_RE = re.compile(r"\b(SeqCst|AcqRel|Acquire|Release)\b")
FENCE_RE = re.compile(r"^(\s*)fence\(")
# Opens an atomic call whose ordering `#[track_caller]` attributes to this line.
ATOMIC_CALL_RE = re.compile(
    r"\.(?:load|store|swap|fetch_add|compare_exchange_weak|compare_exchange)\s*\("
    r"|\bfence\s*\("
)
# A per-test result line, e.g. `test foo::bar ... FAILED`. Excludes the
# `test result: FAILED.` summary line (no ` ... FAILED`).
FAILED_LINE_RE = re.compile(r"^test (.+) \.\.\. FAILED\s*$")


def find_downgrades(lines):
    """Yield (line_idx, start, end, old, new) for each downgrade to test."""
    for i, line in enumerate(lines):
        stripped = line.lstrip()
        if stripped.startswith("//") or "!ORDERING" in line:
            continue
        if FENCE_RE.match(line):
            yield i, None, None, line, None
            continue
        for m in ORDERING_RE.finditer(line):
            old = m.group(1)
            for new in DOWNGRADE[old]:
                yield i, m.start(), m.end(), old, new


def call_line(lines, line_idx):
    """The 1-based source line `#[track_caller]` reports for the atomic call: the
    nearest line at or above the ordering that opens an atomic call. This lets
    the ordering sit on a later line than the call — alone (`    Acquire`) or in
    a match/if arm (`SyncMode::Sequential => SeqCst,`) — while inline calls
    resolve to their own line. (It assumes no *other* atomic call is nested in
    the same multiline argument list, which never happens here.)"""
    for j in range(line_idx, -1, -1):
        if ATOMIC_CALL_RE.search(lines[j]):
            return j + 1
    return line_idx + 1


def downgrade_arg(lines, downgrade):
    """The `LOOM_DOWNGRADE` value that selects this downgrade at runtime."""
    line_idx, start, end, old, new = downgrade
    line = call_line(lines, line_idx)
    if start is None:
        return f"{line}:fence"
    return f"{line}:{old}:{new}"


def describe(lines, downgrade):
    line_idx, start, end, old, new = downgrade
    where = call_line(lines, line_idx)
    at = "" if where == line_idx + 1 else f" call@{where}"
    if start is None:
        return f"line {line_idx + 1}: remove fence{at}"
    return f"line {line_idx + 1}: {old} → {new}{at}"


def build_binary():
    """Compile the loom test binary once and return its path."""
    proc = subprocess.run(
        BUILD_CMD, env=BASE_ENV, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )
    exe = None
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        target = msg.get("target", {})
        if (
            msg.get("reason") == "compiler-artifact"
            and msg.get("executable")
            and target.get("name") == "spmc_waker"
            and "test" in target.get("kind", [])
        ):
            exe = msg["executable"]
    if exe is None:
        sys.exit("failed to build the loom test binary:\n" + proc.stderr)
    return exe


def skip_args():
    args = []
    for name in SKIP_TESTS:
        args += ["--skip", name]
    return args


# A downgraded ordering does not always surface as a clean libtest `... FAILED`:
# a broken ordering could make loom explore a divergent execution that overflows
# the stack, aborting the process before any result is printed. So "caught" means
# "did not cleanly pass" — a named failing test, or `CRASHED` if the process died
# — and "unnecessary" is concluded only from a *positive* clean pass, never from
# the absence of a FAILED line.
CRASHED = "<crashed>"


def run_tests(binary, downgrade):
    """Run the suite with the downgrade applied, escalating the preemption bound.
    Return (test, preemptions) for the first test that catches it (`test` may be
    `CRASHED`), or None if it cleanly passes at every bound (the ordering may be
    unnecessary)."""
    for p in PREEMPTIONS:
        env = {**os.environ, "LOOM_MAX_PREEMPTIONS": str(p), "LOOM_DOWNGRADE": downgrade}
        proc = subprocess.Popen(
            [binary, *skip_args()], env=env,
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
        )
        clean = False
        for line in proc.stdout:
            m = FAILED_LINE_RE.match(line)
            if m:
                proc.kill()
                proc.wait()
                return m.group(1), p
            if line.startswith("test result: ok"):
                clean = True
        proc.wait()
        if not (proc.returncode == 0 and clean):
            return CRASHED, p
        # Cleanly passed; try a higher preemption bound before giving up.
    return None


def main():
    with open(FILE, newline="") as f:
        lines = f.read().splitlines(keepends=True)

    downgrades = list(find_downgrades(lines))
    args = [downgrade_arg(lines, dg) for dg in downgrades]
    dupes = {a for a in args if args.count(a) > 1}
    if dupes:
        sys.exit(f"ambiguous LOOM_DOWNGRADE value(s) (same call line, from, to): {dupes}")
    n_orderings = len({(li, s, e, o) for li, s, e, o, _ in downgrades})

    print("Building the loom test binary ... ", end="", flush=True)
    binary = build_binary()
    print("done")

    print(f"Found {n_orderings} orderings ({len(downgrades)} downgrades) to test\n")

    unnecessary = []
    for idx, (downgrade, arg) in enumerate(zip(downgrades, args), 1):
        print(f"[{idx}/{len(downgrades)}] {describe(lines, downgrade)} ... ", end="", flush=True)
        if result := run_tests(binary, arg):
            test, p = result
            print(f"FAIL ✓ ({test=}, {p=})")
        else:
            print("PASS ✗  <-- ordering may be unnecessary!")
            unnecessary.append(describe(lines, downgrade))

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
