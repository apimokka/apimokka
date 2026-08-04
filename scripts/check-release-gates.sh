#!/usr/bin/env bash

set -u

tool_error() {
    printf 'Release gates: tooling error: %s\n' "$1" >&2
    exit 2
}

if (( $# != 0 )); then
    printf 'Usage: %s\n' "${0##*/}" >&2
    exit 2
fi

script_path=${BASH_SOURCE[0]}
if [[ "$script_path" != /* ]]; then
    script_path="$PWD/$script_path"
fi
script_dir=${script_path%/*}
repository_root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) ||
    tool_error "cannot determine repository root"
[[ -d "$repository_root" && -r "$repository_root/Cargo.toml" ]] ||
    tool_error "repository root is missing or unreadable: $repository_root"

for utility in bash git rustc cargo mkdir; do
    command -v "$utility" >/dev/null 2>&1 ||
        tool_error "required command not found: $utility"
done

cd -- "$repository_root" 2>/dev/null ||
    tool_error "cannot enter repository root: $repository_root"

TMPDIR=${TMPDIR:-"$repository_root/target/tmp"}
export TMPDIR

print_command() {
    printf 'Release gates: run'
    printf ' %q' "$@"
    printf '\n'
}

run_probe() {
    local status
    print_command "$@"
    "$@"
    status=$?
    (( status == 0 )) ||
        tool_error "preflight command failed with status $status: $1"
}

run_gate() {
    local status
    print_command "$@"
    "$@"
    status=$?
    if (( status != 0 )); then
        printf 'Release gates: failed with status %d: %s\n' "$status" "$1" >&2
        return "$status"
    fi
}

run_probe bash --version
run_probe git --version
run_probe rustc --version
run_probe cargo --version
run_probe rustc +1.91 --version
run_probe cargo +1.91 --version
run_probe cargo fmt --version
run_probe cargo clippy --version
run_probe cargo audit --version
run_probe mkdir -p "$TMPDIR"

run_gate cargo fmt --all -- --check || exit $?
run_gate cargo test --workspace --locked || exit $?
run_gate cargo build --workspace --locked || exit $?
run_gate cargo doc --workspace --no-deps --locked || exit $?
run_gate cargo clippy --workspace --all-targets --all-features --locked -- -D warnings || exit $?
run_gate cargo +1.91 test --workspace --locked || exit $?
run_gate cargo +1.91 build --workspace --locked || exit $?
run_gate cargo audit || exit $?
run_gate bash scripts/check-matcher-oracle-self-test.sh || exit $?
run_gate bash scripts/check-matcher-oracle.sh || exit $?
run_gate bash scripts/check-engine-oracle-self-test.sh || exit $?
run_gate bash scripts/check-engine-oracle.sh || exit $?
run_gate bash scripts/check-rfcs-self-test.sh || exit $?
run_gate bash scripts/check-rfcs.sh || exit $?
run_gate bash scripts/check-source-size-self-test.sh || exit $?
run_gate bash scripts/check-source-size.sh || exit $?
run_gate git diff --check || exit $?

printf 'Release gates: all checks passed\n'
