#!/usr/bin/env bash

# RFC MK-057: enumerates .rs files above the 500-line signal threshold and
# fails when a flagged file has no recorded boundary decision. It never
# fails on size alone — a flagged file with a decision is not an error.
#
# A boundary decision is recorded as a doc comment inside the flagged file
# itself (never in a separate document, which is the staleness failure this
# checker replaces):
#
#   //! Boundary decision: split — <what was extracted and where>.
#   //! Boundary decision: single-responsibility — <the one responsibility>.
#
# The inventory below is generated fresh on every run, never transcribed.

set -uo pipefail

LC_ALL=C
export LC_ALL

readonly threshold=500
readonly decision_pattern='^//! Boundary decision: (split|single-responsibility) — .+'

usage() {
    printf 'Usage: %s [repository-root]\n' "${0##*/}" >&2
}

die_operational() {
    printf 'Source size: operational error: %s\n' "$1" >&2
    exit 2
}

if (( $# > 1 )); then
    usage
    exit 2
fi

(( BASH_VERSINFO[0] >= 4 )) ||
    die_operational "Bash 4 or newer is required"

for utility in find grep sort wc; do
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
[[ -d "$root/crates" && -r "$root/crates" ]] ||
    die_operational "crates directory is missing or unreadable"

relative_path() {
    printf '%s' "${1#"$root"/}"
}

declare -a flagged=()
declare -a undecided=()

while IFS= read -r -d '' file; do
    lines=$(wc -l < "$file") ||
        die_operational "cannot count lines: $(relative_path "$file")"
    lines=${lines//[[:space:]]/}
    (( lines > threshold )) || continue

    path=$(relative_path "$file")
    flagged+=("$lines"$'\t'"$path")

    grep -Eq -- "$decision_pattern" "$file" ||
        undecided+=("$path")
done < <(find "$root/crates" -type f -name '*.rs' -print0 | sort -z)

if (( ${#flagged[@]} == 0 )); then
    printf 'Source size: no files above the %d-line signal threshold\n' "$threshold"
    exit 0
fi

printf 'Source size: %d file(s) above the %d-line signal threshold\n' \
    "${#flagged[@]}" "$threshold"
printf '%s\n' "${flagged[@]}" | sort -rn | while IFS=$'\t' read -r lines path; do
    printf '  %6d  %s\n' "$lines" "$path"
done

if (( ${#undecided[@]} > 0 )); then
    printf 'Source size: %d file(s) missing a recorded boundary decision:\n' \
        "${#undecided[@]}" >&2
    printf '  %s\n' "${undecided[@]}" | sort >&2
    exit 1
fi

printf 'Source size: every flagged file has a recorded boundary decision\n'
