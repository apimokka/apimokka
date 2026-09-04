#!/usr/bin/env bash

set -uo pipefail

LC_ALL=C
export LC_ALL

readonly expected_source='registry+https://github.com/rust-lang/crates.io-index'
readonly expected_config_version='6.0.0'
readonly expected_config_checksum='70d8972cf7f30193d279d20b67e08250d777c19d8f7ec52cbeff84b909798e67'

usage() {
    printf 'Usage: %s [repository-root]\n' "${0##*/}" >&2
}

die_operational() {
    printf 'Engine oracle: operational error: %s\n' "$1" >&2
    exit 2
}

if (( $# > 1 )); then
    usage
    exit 2
fi

(( BASH_VERSINFO[0] >= 4 )) ||
    die_operational "Bash 4 or newer is required"

for utility in awk dirname sort; do
    command -v "$utility" >/dev/null 2>&1 ||
        die_operational "required utility not found: $utility"
done

if (( $# == 1 )); then
    root=${1%/}
else
    script_dir=$(dirname -- "${BASH_SOURCE[0]}") ||
        die_operational "cannot determine script directory"
    root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) ||
        die_operational "cannot determine repository root"
fi

[[ -n "$root" && -d "$root" && -r "$root" ]] ||
    die_operational "repository root is missing or unreadable: $root"
[[ -r "$root/Cargo.toml" ]] ||
    die_operational "workspace manifest is missing or unreadable"
[[ -r "$root/Cargo.lock" ]] ||
    die_operational "Cargo.lock is missing or unreadable"

declare -a errors=()

add_error() {
    errors+=("$1")
}

report_errors() {
    local error
    for error in "${errors[@]}"; do
        printf 'Engine oracle: ERROR %s\n' "$error" >&2
    done
    printf 'Engine oracle: %d error(s)\n' "${#errors[@]}" >&2
    exit 1
}

package_records() {
    local package_name=$1
    awk -v wanted="$package_name" '
        function unquote(value) {
            sub(/^[^"]*"/, "", value)
            sub(/".*$/, "", value)
            return value
        }
        function emit() {
            if (name == wanted) {
                printf "%s\t%s\t%s\n", version, source, checksum
            }
        }
        /^\[\[package\]\]$/ {
            if (in_package) emit()
            in_package = 1
            name = version = source = checksum = ""
            next
        }
        in_package && /^name = / { name = unquote($0); next }
        in_package && /^version = / { version = unquote($0); next }
        in_package && /^source = / { source = unquote($0); next }
        in_package && /^checksum = / { checksum = unquote($0); next }
        END { if (in_package) emit() }
    ' "$root/Cargo.lock"
}

check_package() {
    local name=$1
    local expected_version=$2
    local expected_checksum=$3
    local -a records=()
    local version source checksum

    mapfile -t records < <(package_records "$name") ||
        die_operational "cannot parse Cargo.lock for $name"
    if (( ${#records[@]} != 1 )); then
        add_error "$name must have exactly one registry package entry (found ${#records[@]})"
        return
    fi
    IFS=$'\t' read -r version source checksum <<< "${records[0]}"
    [[ "$version" == "$expected_version" ]] ||
        add_error "$name version must be $expected_version (found ${version:-<missing>})"
    [[ "$source" == "$expected_source" ]] ||
        add_error "$name source must be $expected_source (found ${source:-<missing>})"
    [[ "$checksum" == "$expected_checksum" ]] ||
        add_error "$name checksum must be $expected_checksum (found ${checksum:-<missing>})"
}

check_package apimock-config "$expected_config_version" "$expected_config_checksum"

(( ${#errors[@]} == 0 )) || report_errors

cargo_command=${CARGO:-cargo}
command -v "$cargo_command" >/dev/null 2>&1 ||
    die_operational "Cargo command not found: $cargo_command"

resolved_features() {
    local package=$1
    local version=$2
    local output status

    output=$(cd -- "$root" && "$cargo_command" tree --locked -p apimokka-model \
        -e features -i "$package@$version" --prefix none 2>&1)
    status=$?
    (( status == 0 )) ||
        die_operational "cargo tree failed for $package@$version: $output"
    awk -v prefix="$package feature \"" '
        index($0, prefix) == 1 {
            feature = substr($0, length(prefix) + 1)
            sub(/".*$/, "", feature)
            print feature
        }
    ' <<< "$output" | sort -u
}

check_features() {
    local package=$1
    local version=$2
    local expected=$3
    local actual

    actual=$(resolved_features "$package" "$version") || exit $?
    [[ "$actual" == "$expected" ]] ||
        add_error "$package resolved features must be [${expected//$'\n'/, }] (found [${actual//$'\n'/, }])"
}

check_features apimock-config "$expected_config_version" 'default'

if (( ${#errors[@]} > 0 )); then
    report_errors
fi

printf 'Engine oracle: apimock-config %s contract verified\n' \
    "$expected_config_version"
