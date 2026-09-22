#!/usr/bin/env bash
# For each non-Relaxed memory ordering (and each fence) of the library and of its tests, weaken
# it one step and run the loom tests expecting a failure — a downgrade that no test catches may
# be unnecessary. The downgrades that were not caught are printed as the `LOOM_DOWNGRADE` value
# selecting them, so a failure can be reproduced with a single test run.
set -euo pipefail
cd "$(dirname "$0")"

# Must match the CI loom job *verbatim*: cargo fingerprints RUSTFLAGS as a raw string, so
# even `--cfg=loom` instead of `--cfg loom` would rebuild the whole dependency graph.
# `loom_downgrade.sh` records this value and exports it for the downgraded runs.
export RUSTFLAGS="--cfg loom -C debug_assertions"
# Nearly every downgrade is caught at a single preemption, which is by far the cheapest pass:
# an uncaught downgrade explores the whole schedule space, whose size explodes with the number
# of preemptions. The few needing two (e.g. the Acquire fence of the `take_impl` retry, reached
# only after a concurrent `wake` and a new registration) are retried at two preemptions, see
# `retry` below.
export LOOM_MAX_PREEMPTIONS=1

# `no_missed_wakeup` alone executes every ordering and catches every downgrade of them, so run
# only it: the rest of the suite (notably `basic_notification`, a broad, slow test imported
# verbatim from tokio) is redundant for this check and much slower.
test=(
    cargo test
    # The tracing and the ordering downgrading come from a fork of loom, patched in here rather
    # than depended upon, so `Cargo.toml` keeps depending on the published loom.
    --config 'patch.crates-io.loom.git="https://github.com/wyfo/loom.git"'
    --config 'patch.crates-io.loom.branch="trace-downgrade"'
    --release --test spmc_waker no_missed_wakeup
)

# libtest exits successfully when its filter matches no test, and no test means no ordering
# collected, hence nothing to downgrade: a rename would silently make this check vacuous.
[[ "$("${test[@]}" -- --list)" == *': test'* ]] ||
    { echo "no test matching the filter" >&2 && exit 1; }

LOOM_DOWNGRADE=collect "${test[@]}" > /dev/null 2>&1 ||
    { echo "ordering collection failed" >&2 && exit 1; }

# `loom_downgrade.sh` runs its command once per downgrade, expecting a failure when the
# downgrade is caught, so a run passing at a single preemption is retried at two before being
# reported as uncaught. `exit` without argument forwards the status of the failed run, and
# `retry` only fills `bash -c`'s `$0` so that the test command lands in `$@`.
./loom_downgrade.sh bash -c '"$@" || exit; LOOM_MAX_PREEMPTIONS=2 "$@"' retry "${test[@]}"
