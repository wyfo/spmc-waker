#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Default mode regenerates the reference asm files. Pass "--check" to instead
# diff the freshly generated asm against the committed references without
# touching them (e.g. in CI / pre-commit).
MODE="${1:-}"

# arch label : rust target triple : extra rustflags
# +lse on aarch64 emits inline cas/casal instead of the __aarch64_cas8_* outline-atomics calls
ARCHES=(
    "x86_64:x86_64-unknown-linux-gnu:"
    "aarch64:aarch64-unknown-linux-gnu:-C target-feature=+lse"
)

# cfg prefix : directory label
VARIANTS=(
    "--cfg=synchronized --cfg=cached:SYNC=Synchronized,CACHED=true"
    "--cfg=synchronized:SYNC=Synchronized,CACHED=false"
    "--cfg=sequential --cfg=cached:SYNC=Sequential,CACHED=true"
    "--cfg=sequential:SYNC=Sequential,CACHED=false"
    "--cfg=unsynchronized --cfg=cached:SYNC=Unsynchronized,CACHED=true"
    "--cfg=unsynchronized:SYNC=Unsynchronized,CACHED=false"
)

# check_unit <arch> <target> <extra_flags> <dir> <cfg> <fn> <name>
#   dir  - directory under <arch>/ where the .s file lives
#   cfg  - the --cfg flag value, also used as the target dir key
#   fn   - the symbol passed to `cargo asm`
#   name - reference file base name within <arch>/<dir>/ (defaults to <fn>)
# Runs as a backgrounded job, exits non-zero on failure.
check_unit() {
    local arch="$1" target="$2" extra="$3" dir="$4" cfg="$5" fn="$6" name="${7:-$6}"
    local label="$arch/$dir/$name"

    # Each cfg gets its own target dir so parallel jobs don't contend on the
    # shared cargo build lock; incremental reruns stay fast.
    local tdir="$SCRIPT_DIR/target/$arch/$cfg"
    mkdir -p "$tdir"

    # cargo-asm intermittently flakes under concurrency (empty output on a
    # valid function). Retry up to 3 times; the build is cached after the
    # first attempt so retries are cheap.
    local actual=""
    for _ in 1 2 3; do
        actual=$(CARGO_TARGET_DIR=$tdir RUSTFLAGS="$cfg $extra" cargo asm --lib --target "$target" --simplify "$fn" 2>/dev/null) || true
        [[ -n "$actual" ]] && break
    done

    local asm_file="$arch/$dir/$name.s"

    if [[ "$MODE" != "--check" ]]; then
        mkdir -p "$(dirname "$asm_file")"
        printf '%s\n' "$actual" > "$asm_file"
        echo "updated: $label"
        return 0
    fi

    if [[ ! -f "$asm_file" ]]; then
        echo "MISSING ref: $asm_file (run without '--check' to generate)"
        return 1
    fi
    if diff -u "$asm_file" <(printf '%s\n' "$actual") > /dev/null 2>&1; then
        echo "ok: $label"
        return 0
    else
        echo "FAIL: $label"
        return 1
    fi
}

pids=()
for arch_entry in "${ARCHES[@]}"; do
    IFS=: read -r arch target extra <<< "$arch_entry"

    for variant_entry in "${VARIANTS[@]}"; do
        IFS=: read -r cfg dir <<< "$variant_entry"

        # ── hot-path functions ────────────────────────────────────────────

        for op in wake wake_cold try_register register unregister poll_wait_until has_waker_registered; do
            check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "asm_${op}_asm" "$op" &
            pids+=("$!")
        done

        # ── cold helpers outlined from hot paths ─────────────────────────

        # wake's outlined cold helpers
        check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "wake_registered_cold" &
        pids+=("$!")
        check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "wake_fallback" &
        pids+=("$!")

        check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "register_cold" &
        pids+=("$!")

        check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "register_fallback" &
        pids+=("$!")
    done


done

FAIL=0
for pid in "${pids[@]}"; do
    wait "$pid" || FAIL=$((FAIL + 1))
done
PASS=$(( ${#pids[@]} - FAIL ))

echo ""
if [[ "$MODE" != "--check" ]]; then
    echo "regenerated ${#pids[@]} function(s)."
    exit 0
fi
if [[ $FAIL -gt 0 ]]; then
    echo "$FAIL function(s) failed, $PASS passed."
    exit 1
fi
echo "All $PASS function(s) passed."
