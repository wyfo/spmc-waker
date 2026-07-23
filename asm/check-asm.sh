#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Default mode regenerates the reference asm files. Pass "--check" to instead
# diff the freshly generated asm against the committed references without
# touching them (e.g. in CI / pre-commit).
MODE="${1:-}"

# Exit statuses check_unit uses (in regenerate mode) to tell the parent what it
# did with a reference file. Kept in the 3-125 range so they never collide with
# 0 (unchanged/success), the conventional 1/2 errors, or the 126+ shell-reserved
# codes.
readonly RC_CREATED=10
readonly RC_UPDATED=11
readonly RC_REMOVED=12
readonly RC_BUILD=13   # the crate failed to compile for this cfg (hard failure)

# arch label : rust target triple : extra rustflags
# +lse on aarch64 emits inline cas/casal instead of the __aarch64_cas8_* outline-atomics calls
ARCHES=(
    "x86_64:x86_64-unknown-linux-gnu:"
    "aarch64:aarch64-unknown-linux-gnu:-C target-feature=+lse"
)

# cfg prefix : directory label
VARIANTS=(
    "--cfg=synchronized --cfg=cached --cfg=strict:S=Synchronized,CACHED=true,R=Strict"
    "--cfg=synchronized --cfg=strict:S=Synchronized,CACHED=false,R=Strict"
    "--cfg=sequential --cfg=cached --cfg=strict:S=Sequential,CACHED=true,R=Strict"
    "--cfg=sequential --cfg=strict:S=Sequential,CACHED=false,R=Strict"
    "--cfg=unsynchronized --cfg=cached --cfg=strict:S=Unsynchronized,CACHED=true,R=Strict"
    "--cfg=unsynchronized --cfg=strict:S=Unsynchronized,CACHED=false,R=Strict"
    "--cfg=synchronized --cfg=cached --cfg=unchecked:S=Synchronized,CACHED=true,R=Unchecked"
    "--cfg=synchronized --cfg=unchecked:S=Synchronized,CACHED=false,R=Unchecked"
    "--cfg=sequential --cfg=cached --cfg=unchecked:S=Sequential,CACHED=true,R=Unchecked"
    "--cfg=sequential --cfg=unchecked:S=Sequential,CACHED=false,R=Unchecked"
    "--cfg=unsynchronized --cfg=cached --cfg=unchecked:S=Unsynchronized,CACHED=true,R=Unchecked"
    "--cfg=unsynchronized --cfg=unchecked:S=Unsynchronized,CACHED=false,R=Unchecked"
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
    # cargo/the pipe may emit CRLF on Windows; normalize to LF so generated
    # references are platform-independent.
    actual="${actual//$'\r'/}"

    local asm_file="$arch/$dir/$name.s"

    # Empty output is ambiguous: the function was inlined away (e.g. a small
    # wake_impl_cold gets inlined despite #[cold]) and legitimately has no asm,
    # OR the crate failed to compile for this cfg (e.g. a rename broke asm/), OR
    # cargo asm flaked under load (the retry above absorbs most of those). Tell a
    # real build break apart from an inline/flake with a plain cargo check for
    # this cfg: if it fails, it's a compilation error and the whole run must fail.
    if [[ -z "$actual" ]]; then
        if ! CARGO_TARGET_DIR=$tdir RUSTFLAGS="$cfg $extra" \
                cargo check --lib --target "$target" > /dev/null 2>&1; then
            echo "BUILD FAILED: $label"
            return $RC_BUILD
        fi
        # Compiles cleanly, so there is genuinely no asm for this function.
        if [[ "$MODE" == "--check" ]]; then
            # A committed reference for a now-inlined function is stale drift.
            if [[ -f "$asm_file" ]]; then
                echo "FAIL: $label (stale ref; function produces no asm)"
                return 1
            fi
            return 0
        fi
        if [[ -f "$asm_file" ]]; then
            rm -f "$asm_file"
            echo "removed: $label"
            return $RC_REMOVED
        fi
        return 0
    fi

    if [[ "$MODE" != "--check" ]]; then
        mkdir -p "$(dirname "$asm_file")"
        # Only touch the file (and report) when the content actually differs,
        # so reruns that produce identical asm stay silent.
        if [[ ! -f "$asm_file" ]]; then
            printf '%s\n' "$actual" > "$asm_file"
            echo "created: $label"
            return $RC_CREATED
        fi
        if diff -q "$asm_file" <(printf '%s\n' "$actual") > /dev/null 2>&1; then
            return 0
        fi
        printf '%s\n' "$actual" > "$asm_file"
        echo "updated: $label"
        return $RC_UPDATED
    fi

    if [[ ! -f "$asm_file" ]]; then
        echo "MISSING ref: $asm_file (run without '--check' to generate)"
        return 1
    fi
    if diff -u "$asm_file" <(printf '%s\n' "$actual") > /dev/null 2>&1; then
        return 0
    else
        echo "FAIL: $label"
        return 1
    fi
}

# The 8 asm units generated per (arch, cfg) variant: 6 hot-path functions plus
# 2 cold helpers outlined from them, as <cargo-asm symbol>\x1f<reference name>.
FUNCS=(
    "asm_take_asm"$'\x1f'"take"
    "asm_wake_asm"$'\x1f'"wake"
    "asm_wake_cold_asm"$'\x1f'"wake_cold"
    "asm_register_asm"$'\x1f'"register"
    "asm_unregister_asm"$'\x1f'"unregister"
    "asm_poll_wait_until_asm"$'\x1f'"poll_wait_until"
    "wake_impl_cold"$'\x1f'"wake_impl_cold"
    "register_impl_cold"$'\x1f'"register_impl_cold"
)

# Flatten every (function, arch, variant) combination into a task list, fields
# \x1f-separated (dir/cfg contain spaces, '=' and ','). Function is the OUTER
# loop on purpose: consecutive tasks then hit *different* per-(arch, cfg) build
# dirs, so the worker pool below runs distinct cargo builds in parallel rather
# than piling onto one dir's build lock.
tasks=()
declare -A EXPECTED   # every .s path this run owns; the stale sweep removes the rest
for func in "${FUNCS[@]}"; do
    IFS=$'\x1f' read -r fn name <<< "$func"
    for arch_entry in "${ARCHES[@]}"; do
        IFS=: read -r arch target extra <<< "$arch_entry"
        for variant_entry in "${VARIANTS[@]}"; do
            IFS=: read -r cfg dir <<< "$variant_entry"
            tasks+=("$arch"$'\x1f'"$target"$'\x1f'"$extra"$'\x1f'"$dir"$'\x1f'"$cfg"$'\x1f'"$fn"$'\x1f'"$name")
            EXPECTED["$arch/$dir/$name.s"]=1
        done
    done
done

# Top-level arch directories to sweep for stale .s files (first field of ARCHES).
ARCH_DIRS=()
for arch_entry in "${ARCHES[@]}"; do
    IFS=: read -r arch _ <<< "$arch_entry"
    ARCH_DIRS+=("$arch")
done

TOTAL=${#tasks[@]}
DONE=0

# Cap on concurrently running cargo jobs; override via the environment, defaults
# to the CPU count. Forking all ~200 at once both thrashes the machine (dozens
# of simultaneous builds) and delays *every* completion to the very end, so the
# counter would sit at 0 then jump. A bounded pool interleaves spawn and reap, so
# the counter climbs from the first completion.
JOBS="${JOBS:-$(nproc 2>/dev/null || echo 4)}"

# Redraw an in-place progress counter on stderr as jobs are reaped. Goes to
# stderr with a bare '\r' so it stays on one line and doesn't pollute stdout
# (which carries the per-file action lines and the final summary). Only shown
# on a terminal; piped/redirected runs skip it to avoid junk in logs.
progress() {
    [[ -t 2 ]] || return 0
    printf '\r  [%d/%d]' "$DONE" "$TOTAL" >&2
}

# Commit the final progress frame with a newline instead of erasing it. On a
# fast (warm-cache) run the whole '\r' animation happens in well under a repaint
# interval; terminals that coalesce frames (e.g. JetBrains' JediTerm) would drop
# an erased line entirely, so we leave the last '[N/N]' on screen as a permanent
# row before the summary.
progress_end() {
    [[ -t 2 ]] || return 0
    printf '\n' >&2
}

CREATED=0 UPDATED=0 REMOVED=0 UNCHANGED=0 FAIL=0 STALE=0
running=0

# Remove (regenerate) or flag (--check) any committed .s file that no unit in
# this run owns — e.g. left behind after a function is deleted. EXPECTED holds
# every path we're responsible for; anything else under the arch dirs is stale.
sweep_stale() {
    local dirs=() d f
    for d in "${ARCH_DIRS[@]}"; do [[ -d "$d" ]] && dirs+=("$d"); done
    (( ${#dirs[@]} )) || return 0
    while IFS= read -r -d '' f; do
        [[ -n "${EXPECTED[$f]:-}" ]] && continue
        STALE=$((STALE + 1))
        if [[ "$MODE" == "--check" ]]; then
            echo "STALE ref: $f (regenerate to remove)"
        else
            rm -f "$f"
            echo "removed: $f"
        fi
    done < <(find "${dirs[@]}" -name '*.s' -print0 2>/dev/null)
}

# A compilation error is fatal and unambiguous: stop the whole run at the first
# one rather than letting the remaining jobs churn out empty (then removed)
# files. check_unit already printed the offending "BUILD FAILED: ..." line.
abort_build() {
    progress_end
    local p
    p=$(jobs -p)
    if [[ -n "$p" ]]; then kill $p 2>/dev/null || true; fi
    exit 1
}

# Reap one finished job in completion order and fold its result into the
# tallies. `wait -n` returns that job's exit status: in regenerate mode the
# RC_* codes (see top of file); in --check mode 0=ok, non-zero=failure.
reap() {
    local rc=0
    wait -n || rc=$?
    if (( rc == RC_BUILD )); then abort_build; fi
    if [[ "$MODE" == "--check" ]]; then
        if (( rc != 0 )); then FAIL=$((FAIL + 1)); fi
    else
        case $rc in
            $RC_CREATED) CREATED=$((CREATED + 1)) ;;
            $RC_UPDATED) UPDATED=$((UPDATED + 1)) ;;
            $RC_REMOVED) REMOVED=$((REMOVED + 1)) ;;
            *)           UNCHANGED=$((UNCHANGED + 1)) ;;
        esac
    fi
    running=$((running - 1))
    DONE=$((DONE + 1))
    progress
}

progress
for task in "${tasks[@]}"; do
    # Block until a slot frees up, reaping (and counting) finished jobs as we go.
    while (( running >= JOBS )); do reap; done
    IFS=$'\x1f' read -r arch target extra dir cfg fn name <<< "$task"
    check_unit "$arch" "$target" "$extra" "$dir" "$cfg" "$fn" "$name" &
    running=$((running + 1))
done
while (( running > 0 )); do reap; done
progress_end
sweep_stale

if [[ "$MODE" != "--check" ]]; then
    # A compile error would already have aborted the run via abort_build, so
    # reaching here means every unit succeeded.
    echo "$TOTAL function(s): $CREATED created, $UPDATED updated, $REMOVED removed, $UNCHANGED unchanged."
    if (( STALE > 0 )); then echo "$STALE stale file(s) removed."; fi
    exit 0
fi
PASS=$(( TOTAL - FAIL ))
if (( FAIL > 0 || STALE > 0 )); then
    echo "$FAIL function(s) failed, $PASS passed; $STALE stale file(s)."
    exit 1
fi
echo "All $PASS function(s) passed."
