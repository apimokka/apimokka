#!/usr/bin/env bash

set -u

script_path=${BASH_SOURCE[0]}
if [[ "$script_path" != /* ]]; then
    script_path="$PWD/$script_path"
fi
script_dir=${script_path%/*}
repository_root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) || {
    printf 'Release gate self-test: cannot determine repository root\n' >&2
    exit 2
}
gate="$script_dir/check-release-gates.sh"

for utility in cat chmod cmp ln mkdir mktemp rm; do
    command -v "$utility" >/dev/null 2>&1 || {
        printf 'Release gate self-test: required utility not found: %s\n' "$utility" >&2
        exit 2
    }
done
[[ -x /usr/bin/bash ]] || {
    printf 'Release gate self-test: /usr/bin/bash is unavailable\n' >&2
    exit 2
}
[[ -x "$gate" ]] || {
    printf 'Release gate self-test: gate is not executable: %s\n' "$gate" >&2
    exit 2
}

tmp_parent=${TMPDIR:-"$repository_root/target/tmp"}
mkdir -p -- "$tmp_parent" || exit 2
temp_dir=$(mktemp -d "$tmp_parent/check-release-gates-self-test.XXXXXX") || exit 2

cleanup() {
    if [[ -n "${temp_dir:-}" &&
          "$temp_dir" == "$tmp_parent"/check-release-gates-self-test.* &&
          -d "$temp_dir" ]]; then
        rm -rf -- "$temp_dir"
    else
        printf 'Release gate self-test: refusing unsafe cleanup: %s\n' \
            "${temp_dir:-<empty>}" >&2
        return 2
    fi
}
trap cleanup EXIT

stub_dir="$temp_dir/stubs"
outside_dir="$temp_dir/outside"
mkdir -p -- "$stub_dir" "$outside_dir"

stub="$stub_dir/stub-command"
cat > "$stub" <<'STUB'
#!/usr/bin/bash
set -u

command_name=${0##*/}
{
    printf '%s\0%d\0' "$command_name" "$#"
    if (( $# > 0 )); then
        printf '%s\0' "$@"
    fi
} >> "$INVOCATION_LOG"

if [[ -n "${EXPECTED_WORKDIR:-}" && "$PWD" != "$EXPECTED_WORKDIR" ]]; then
    exit 97
fi

signature=$command_name
for argument in "$@"; do
    signature+="|$argument"
done
if [[ -n "${STUB_FAIL_SIGNATURE:-}" && "$signature" == "$STUB_FAIL_SIGNATURE" ]]; then
    exit "${STUB_FAIL_STATUS:-1}"
fi
STUB
stub_status=$?
(( stub_status == 0 )) || exit 2
chmod +x -- "$stub" || exit 2
for command_name in bash git rustc cargo mkdir; do
    ln -s -- stub-command "$stub_dir/$command_name" || exit 2
done

record() {
    local destination=$1
    local command_name=$2
    shift 2
    {
        printf '%s\0%d\0' "$command_name" "$#"
        if (( $# > 0 )); then
            printf '%s\0' "$@"
        fi
    } >> "$destination"
}

write_expected() {
    local destination=$1
    local stop_after=${2:-all}
    local runtime_tmp=$3
    : > "$destination"

    record "$destination" bash --version
    [[ "$stop_after" == bash_version ]] && return
    record "$destination" git --version
    [[ "$stop_after" == git_version ]] && return
    record "$destination" rustc --version
    [[ "$stop_after" == rustc_version ]] && return
    record "$destination" cargo --version
    [[ "$stop_after" == cargo_version ]] && return
    record "$destination" rustc +1.91 --version
    [[ "$stop_after" == rustc_191_version ]] && return
    record "$destination" cargo +1.91 --version
    [[ "$stop_after" == cargo_191_version ]] && return
    record "$destination" cargo fmt --version
    [[ "$stop_after" == fmt_version ]] && return
    record "$destination" cargo clippy --version
    [[ "$stop_after" == clippy_version ]] && return
    record "$destination" cargo audit --version
    [[ "$stop_after" == audit_version ]] && return
    record "$destination" mkdir -p "$runtime_tmp"
    [[ "$stop_after" == mkdir ]] && return

    record "$destination" cargo fmt --all -- --check
    [[ "$stop_after" == fmt ]] && return
    record "$destination" cargo test --workspace --lib --bins --locked
    [[ "$stop_after" == test ]] && return
    record "$destination" cargo test --workspace --doc --locked
    [[ "$stop_after" == doctest ]] && return
    record "$destination" cargo build --workspace --locked
    [[ "$stop_after" == build ]] && return
    record "$destination" cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    [[ "$stop_after" == clippy ]] && return
    record "$destination" cargo +1.91 test --workspace --lib --bins --locked
    [[ "$stop_after" == test_191 ]] && return
    record "$destination" cargo +1.91 build --workspace --locked
    [[ "$stop_after" == build_191 ]] && return
    record "$destination" cargo audit
    [[ "$stop_after" == audit ]] && return
    record "$destination" bash scripts/check-matcher-oracle-self-test.sh
    [[ "$stop_after" == matcher_self_test ]] && return
    record "$destination" bash scripts/check-matcher-oracle.sh
    [[ "$stop_after" == matcher ]] && return
    record "$destination" bash scripts/check-rfcs-self-test.sh
    [[ "$stop_after" == rfc_self_test ]] && return
    record "$destination" bash scripts/check-rfcs.sh
    [[ "$stop_after" == rfc ]] && return
    record "$destination" git diff --check
}

failures=0
checks=0

fail() {
    printf 'FAIL %s\n' "$1" >&2
    (( failures += 1 ))
}

run_case() {
    local label=$1
    local expected_status=$2
    local stop_after=$3
    local fail_signature=${4:-}
    local fail_status=${5:-1}
    local path=${6:-$stub_dir}
    local actual="$temp_dir/$checks.actual"
    local expected="$temp_dir/$checks.expected"
    local output="$temp_dir/$checks.output"
    local runtime_tmp="$temp_dir/runtime-$checks"
    local status

    : > "$actual"
    write_expected "$expected" "$stop_after" "$runtime_tmp"
    set +e
    (
        cd -- "$outside_dir" || exit 98
        PATH="$path" \
        TMPDIR="$runtime_tmp" \
        INVOCATION_LOG="$actual" \
        EXPECTED_WORKDIR="$repository_root" \
        STUB_FAIL_SIGNATURE="$fail_signature" \
        STUB_FAIL_STATUS="$fail_status" \
            /usr/bin/bash "$gate"
    ) > "$output" 2>&1
    status=$?
    set -e
    (( checks += 1 ))

    if (( status != expected_status )); then
        fail "$label: expected exit $expected_status, got $status"
    fi
    if ! cmp -s -- "$expected" "$actual"; then
        fail "$label: ordered argv log differed"
    fi
}

run_case "successful full contract from outside repository" 0 all
run_case "first substantive failure" 17 fmt "cargo|fmt|--all|--|--check" 17
run_case "middle Clippy failure" 23 clippy \
    "cargo|clippy|--workspace|--all-targets|--all-features|--locked|--|-D|warnings" 23
run_case "final Git failure" 29 all "git|diff|--check" 29

run_case "missing Rust 1.91" 2 rustc_191_version "rustc|+1.91|--version" 44
run_case "missing rustfmt component" 2 fmt_version "cargo|fmt|--version" 45
run_case "missing Clippy component" 2 clippy_version "cargo|clippy|--version" 46
run_case "missing cargo-audit component" 2 audit_version "cargo|audit|--version" 47
run_case "TMPDIR preparation failure" 2 mkdir "mkdir|-p|$temp_dir/runtime-$checks" 48

missing_dir="$temp_dir/missing-command-stubs"
mkdir -p -- "$missing_dir"
for command_name in bash rustc cargo mkdir; do
    ln -s -- "$stub" "$missing_dir/$command_name" || exit 2
done
missing_actual="$temp_dir/missing.actual"
: > "$missing_actual"
set +e
(
    cd -- "$outside_dir" || exit 98
    PATH="$missing_dir" \
    TMPDIR="$temp_dir/missing-runtime" \
    INVOCATION_LOG="$missing_actual" \
    EXPECTED_WORKDIR="$repository_root" \
        /usr/bin/bash "$gate"
) > "$temp_dir/missing.output" 2>&1
missing_status=$?
set -e
(( checks += 1 ))
if (( missing_status != 2 )); then
    fail "missing external command: expected exit 2, got $missing_status"
fi
if [[ -s "$missing_actual" ]]; then
    fail "missing external command: substantive invocation occurred"
fi

unsupported_actual="$temp_dir/unsupported.actual"
: > "$unsupported_actual"
set +e
(
    cd -- "$outside_dir" || exit 98
    PATH="$stub_dir" \
    TMPDIR="$temp_dir/unsupported-runtime" \
    INVOCATION_LOG="$unsupported_actual" \
    EXPECTED_WORKDIR="$repository_root" \
        /usr/bin/bash "$gate" unexpected
) > "$temp_dir/unsupported.output" 2>&1
unsupported_status=$?
set -e
(( checks += 1 ))
if (( unsupported_status != 2 )); then
    fail "unsupported argument: expected exit 2, got $unsupported_status"
fi
if [[ -s "$unsupported_actual" ]]; then
    fail "unsupported argument: external invocation occurred"
fi

if (( failures != 0 )); then
    printf 'Release gate self-test: %d failure(s) across %d checks\n' \
        "$failures" "$checks" >&2
    exit 1
fi

printf 'Release gate self-test: %d checks passed\n' "$checks"
