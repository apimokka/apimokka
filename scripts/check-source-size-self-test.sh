#!/usr/bin/env bash

set -u

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) ||
    {
        printf 'Source size self-test: cannot determine script directory\n' >&2
        exit 2
    }
checker="$script_dir/check-source-size.sh"
repository_root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) ||
    {
        printf 'Source size self-test: cannot determine repository root\n' >&2
        exit 2
    }

for utility in mkdir mktemp rm seq touch; do
    command -v "$utility" >/dev/null 2>&1 || {
        printf 'Source size self-test: required utility not found: %s\n' "$utility" >&2
        exit 2
    }
done

[[ -x "$checker" ]] || {
    printf 'Source size self-test: checker is not executable: %s\n' "$checker" >&2
    exit 2
}

tmp_parent=${TMPDIR:-"$repository_root/target/tmp"}
mkdir -p -- "$tmp_parent" || exit 2
sentinel="$tmp_parent/check-source-size-self-test-sentinel.$$"
[[ ! -e "$sentinel" ]] || {
    printf 'Source size self-test: sentinel already exists\n' >&2
    exit 2
}
touch -- "$sentinel" || exit 2
temp_dir=$(mktemp -d "$tmp_parent/check-source-size-self-test.XXXXXX") || exit 2

cleanup_temp() {
    if [[ -z "${temp_dir:-}" ||
          "$temp_dir" != "$tmp_parent"/check-source-size-self-test.* ||
          ! -d "$temp_dir" ]]; then
        printf 'Source size self-test: refusing unsafe temporary cleanup: %s\n' "${temp_dir:-<empty>}" >&2
        return 2
    fi
    rm -rf -- "$temp_dir" || {
        printf 'Source size self-test: temporary cleanup failed: %s\n' "$temp_dir" >&2
        return 2
    }
}

cleanup_all() {
    local status=0
    cleanup_temp || status=$?
    rm -f -- "$sentinel"
    return "$status"
}
trap cleanup_all EXIT

failures=0
checks=0

fail() {
    printf 'FAIL %s\n' "$1" >&2
    (( failures += 1 ))
}

expect_result() {
    local label=$1
    local expected_status=$2
    local expected_text=$3
    shift 3
    local output status

    set +e
    output=$("$@" 2>&1)
    status=$?
    set -e
    (( checks += 1 ))

    if (( status != expected_status )); then
        fail "$label: expected exit $expected_status, got $status"
        printf '%s\n' "$output" >&2
        return
    fi
    if [[ "$output" != *"$expected_text"* ]]; then
        fail "$label: expected output containing '$expected_text'"
        printf '%s\n' "$output" >&2
    fi
}

expect_not_containing() {
    local label=$1
    local expected_status=$2
    local forbidden_text=$3
    shift 3
    local output status

    set +e
    output=$("$@" 2>&1)
    status=$?
    set -e
    (( checks += 1 ))

    if (( status != expected_status )); then
        fail "$label: expected exit $expected_status, got $status"
        printf '%s\n' "$output" >&2
        return
    fi
    if [[ "$output" == *"$forbidden_text"* ]]; then
        fail "$label: output must not contain '$forbidden_text'"
        printf '%s\n' "$output" >&2
    fi
}

# Fixture repository skeleton: minimal but satisfies the checker's own
# preflight (Cargo.toml present, crates/ directory present).
fixture_root="$temp_dir/repo"

reset_fixture() {
    rm -rf -- "$fixture_root"
    mkdir -p -- "$fixture_root/crates/app/src"
    printf '[workspace]\nmembers = ["crates/app"]\n' > "$fixture_root/Cargo.toml"
}

# Writes an .rs file with the given number of body lines, optionally
# prefixed with a boundary-decision doc comment.
write_source_file() {
    local path=$1
    local body_lines=$2
    local decision=${3:-}

    mkdir -p -- "$(dirname -- "$path")"
    : > "$path"
    if [[ -n "$decision" ]]; then
        printf '%s\n' "$decision" >> "$path"
    fi
    local i
    for (( i = 1; i <= body_lines; i++ )); do
        printf '// line %d\n' "$i" >> "$path"
    done
}

# --- Operational errors -----------------------------------------------

expect_result "usage: too many arguments" 2 "Usage:" \
    "$checker" "$fixture_root" extra

expect_result "operational: missing repository root" 2 "operational error" \
    "$checker" "$temp_dir/does-not-exist"

reset_fixture
rm -f -- "$fixture_root/Cargo.toml"
expect_result "operational: missing Cargo.toml" 2 "operational error" \
    "$checker" "$fixture_root"

reset_fixture
rm -rf -- "$fixture_root/crates"
expect_result "operational: missing crates directory" 2 "operational error" \
    "$checker" "$fixture_root"

# --- No files above the threshold --------------------------------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/small.rs" 10
expect_result "clean: no flagged files" 0 "no files above" \
    "$checker" "$fixture_root"

# --- Boundary: exactly at the threshold is not flagged ------------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/exactly_500.rs" 500
expect_result "boundary: exactly 500 lines is not flagged" 0 "no files above" \
    "$checker" "$fixture_root"

# --- Flagged file with a recorded split decision ------------------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/big_split.rs" 501 \
    '//! Boundary decision: split — extracted foo and bar into siblings.'
expect_result "decided: split marker passes" 0 "every flagged file has a recorded boundary decision" \
    "$checker" "$fixture_root"

# --- Flagged file with a recorded single-responsibility decision --------

reset_fixture
write_source_file "$fixture_root/crates/app/src/big_single.rs" 600 \
    '//! Boundary decision: single-responsibility — mirrors one contract tier.'
expect_result "decided: single-responsibility marker passes" 0 "every flagged file has a recorded boundary decision" \
    "$checker" "$fixture_root"

# --- Flagged file with no decision fails, and names the file ------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/undecided.rs" 600
expect_result "undecided: missing marker fails" 1 "crates/app/src/undecided.rs" \
    "$checker" "$fixture_root"

# --- A decision-shaped comment with an unrecognised keyword still fails -

reset_fixture
write_source_file "$fixture_root/crates/app/src/wrong_keyword.rs" 600 \
    '//! Boundary decision: deferred — will decide later.'
expect_result "undecided: unrecognised keyword still fails" 1 "wrong_keyword.rs" \
    "$checker" "$fixture_root"

# --- A decision with an empty justification still fails ------------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/empty_reason.rs" 600 \
    '//! Boundary decision: split —'
expect_result "undecided: empty justification still fails" 1 "empty_reason.rs" \
    "$checker" "$fixture_root"

# --- Mixed repo: one decided, one undecided — only the undecided one is
# named, and the inventory still prints both -----------------------------

reset_fixture
write_source_file "$fixture_root/crates/app/src/decided.rs" 600 \
    '//! Boundary decision: single-responsibility — one thing.'
write_source_file "$fixture_root/crates/app/src/missing_marker.rs" 700
expect_result "mixed: inventory lists both files" 1 "decided.rs" \
    "$checker" "$fixture_root"
set +e
mixed_output=$("$checker" "$fixture_root" 2>&1)
set -e
(( checks += 1 ))
missing_section=${mixed_output#*"missing a recorded boundary decision:"}
if [[ "$missing_section" == *"decided.rs"* ]]; then
    fail "mixed: decided file must not appear in the missing-decision section"
    printf '%s\n' "$mixed_output" >&2
fi
expect_result "mixed: undecided file is reported as missing" 1 "missing_marker.rs" \
    "$checker" "$fixture_root"

# --- The checker never fails on size alone: never mention "too large" or
# similar size-based failure language, even for the largest fixture -------

reset_fixture
write_source_file "$fixture_root/crates/app/src/huge_but_decided.rs" 5000 \
    '//! Boundary decision: single-responsibility — deliberately huge fixture.'
expect_not_containing "size alone is never the failure" 0 "too large" \
    "$checker" "$fixture_root"

# --- Real repository sanity: the checker runs cleanly against the actual
# repository root without an operational error (its verdict on the current
# tree is a separate, tracked decision, not asserted here) ----------------

set +e
"$checker" "$repository_root" >/dev/null 2>&1
real_status=$?
set -e
(( checks += 1 ))
if (( real_status != 0 && real_status != 1 )); then
    fail "real repository: expected exit 0 or 1 (never an operational error), got $real_status"
fi

printf 'Source size self-test: %d checks, %d failure(s)\n' "$checks" "$failures"
(( failures == 0 )) || exit 1
