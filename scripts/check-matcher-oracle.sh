#!/usr/bin/env bash

set -uo pipefail

LC_ALL=C
export LC_ALL

readonly expected_source='registry+https://github.com/rust-lang/crates.io-index'
readonly expected_routing_version='5.10.0'
readonly expected_routing_checksum='72118fbc81807a3a3e511ec638b3fc798b5eee035c8d287158ae487763003cf1'
readonly expected_http_version='1.4.2'
readonly expected_http_checksum='6970f50e31d6fc17d3fa27329444bfa74e196cf62e95052a3f6fee181dba6425'

usage() {
    printf 'Usage: %s [repository-root]\n' "${0##*/}" >&2
}

die_operational() {
    printf 'Matcher oracle: operational error: %s\n' "$1" >&2
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
        printf 'Matcher oracle: ERROR %s\n' "$error" >&2
    done
    printf 'Matcher oracle: %d error(s)\n' "${#errors[@]}" >&2
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

check_package apimock-routing "$expected_routing_version" "$expected_routing_checksum"
check_package http "$expected_http_version" "$expected_http_checksum"

(( ${#errors[@]} == 0 )) || report_errors

cargo_command=${CARGO:-cargo}
command -v "$cargo_command" >/dev/null 2>&1 ||
    die_operational "Cargo command not found: $cargo_command"

resolved_features() {
    local package=$1
    local version=$2
    local output status

    output=$(cd -- "$root" && "$cargo_command" tree --locked -p apimokka \
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

check_features apimock-routing "$expected_routing_version" 'default'
check_features http "$expected_http_version" $'default\nstd'

if (( ${#errors[@]} > 0 )); then
    report_errors
fi

printf 'Matcher oracle: apimock-routing %s and http %s contract verified\n' \
    "$expected_routing_version" "$expected_http_version"
