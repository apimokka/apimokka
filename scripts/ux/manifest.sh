#!/usr/bin/env bash
# RFC MK-056 layer L2 — per-run frame manifest and integrity check.
#
# Three capture traps in three tasks (the asynchronous screenshot write, a
# geometry taken from the request instead of measured, stale files surviving
# in the output directory) share one shape: the rig proved a file EXISTS,
# never that it is THIS run's frame of THIS state. This writes, for every
# kept frame, its name, MD5 and measured ground pixel, and FAILS the run if
#   - two kept frames share an MD5 (one image filed under two names — at
#     most one of the names is true), or
#   - a frame's ground pixel does not match the theme the run requested
#     (a name asserting a theme its pixels contradict).
#
# Usage: bash scripts/ux/manifest.sh --theme <light|dark|hclight|hcdark> \
#          --renderer <wgpu|software> --out <manifest.md> frame.png[@overlay]...
#
# A frame suffixed @overlay is dimmed by a scrim (dialog, drawer) or is not a
# workspace screen (the mode picker has no top bar): it still counts toward
# the duplicate check, but its ground pixel is recorded and not compared.
#
# The ground is the top bar's own fill (scripts/ux/frame-probe.py ground).
# It is compared by CLASS, not by exact value, because the software renderer
# (tiny-skia) overdraws stale damage without clearing and darkens whole
# panels progressively — measured, task 039: HC Light's 222 reads 198-206 and
# Light's 239 reads 209-213 in later frames of the same run. The classes are
# still far apart from one another, which is what the check needs:
#   hcdark  R <= 12, neutral (R=G=B +-1)       dark   R <= 60, bluish (B-R >= 2)
#   hclight R >= 150, neutral                  light  R >= 150, bluish
# wgpu references, for orientation: hclight 222, hcdark 3, light 244/246/248.

set -uo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) || exit 2
theme= renderer= out=
frames=()
while (( $# > 0 )); do
    case "$1" in
        --theme) theme=$2; shift 2 ;;
        --renderer) renderer=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        *) frames+=("$1"); shift ;;
    esac
done
[[ -n "$theme" && -n "$renderer" && -n "$out" && ${#frames[@]} -gt 0 ]] || {
    printf 'usage: manifest.sh --theme T --renderer R --out FILE frame...\n' >&2
    exit 2
}

class_of() { # class_of "r g b" -> hcdark|dark|hclight|light|other
    local c=($1) r g b
    r=${c[0]} g=${c[1]} b=${c[2]}
    local neutral=no bluish=no
    (( r - b <= 1 && b - r <= 1 && r - g <= 1 && g - r <= 1 )) && neutral=yes
    (( b - r >= 2 )) && bluish=yes
    if (( r <= 12 )) && [[ $neutral == yes ]]; then echo hcdark
    elif (( r <= 60 )) && [[ $bluish == yes ]]; then echo dark
    elif (( r >= 150 )) && [[ $neutral == yes ]]; then echo hclight
    elif (( r >= 150 )) && [[ $bluish == yes ]]; then echo light
    else echo other
    fi
}
case "$theme" in light|dark|hclight|hcdark) ;; *) printf 'manifest.sh: unknown theme %s\n' "$theme" >&2; exit 2 ;; esac

status=0
declare -A seen
{
    printf '# Frame manifest — theme `%s`, renderer `%s`\n\n' "$theme" "$renderer"
    printf 'Expected ground class (top bar fill): %s\n\n' "$theme"
    printf '| Frame | MD5 | Ground pixel | Check |\n|---|---|---|---|\n'
} > "$out"

for spec in "${frames[@]}"; do
    path=${spec%@overlay}
    overlay=no; [[ "$spec" == *@overlay ]] && overlay=yes
    name=$(basename -- "$path")
    if [[ ! -s "$path" ]]; then
        printf '| %s | - | - | FAIL: missing or empty |\n' "$name" >> "$out"
        status=1
        continue
    fi
    md5=$(md5sum "$path" | awk '{print $1}')
    ground=$(python3 "$script_dir/frame-probe.py" ground "$path") || { status=1; ground="unreadable"; }
    verdict=ok
    if [[ -n "${seen[$md5]:-}" ]]; then
        verdict="FAIL: identical to ${seen[$md5]}"
        status=1
    else
        seen[$md5]=$name
    fi
    if [[ "$overlay" == yes ]]; then
        [[ "$verdict" == ok ]] && verdict="ok (overlay/non-workspace: ground not compared)"
    elif [[ "$(class_of "$ground")" != "$theme" ]]; then
        verdict="FAIL: ground $ground reads as $(class_of "$ground"), not $theme"
        status=1
    fi
    printf '| %s | `%s` | %s | %s |\n' "$name" "$md5" "$ground" "$verdict" >> "$out"
done

if (( status != 0 )); then
    printf '[ux] ERROR: frame manifest FAILED — see %s\n' "$out" >&2
    grep -F 'FAIL' "$out" >&2
fi
exit "$status"
