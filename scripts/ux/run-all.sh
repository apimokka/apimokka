#!/usr/bin/env bash
# RFC MK-056 layer L2 — run the five designed configurations and produce a
# results table.
#
# Coverage design (per review of the original four-row design — see
# .git-exclude/reviewed/2026-08-04-mk056-scripted-gui-verification-review.md
# §3.1): mandatory #1 (Japanese at smallest) and mandatory #4 (Expert at
# smallest) target two distinct failure modes that must not share a
# configuration. #1's risk is text expansion, which bites hardest where
# there is most text — Guided mode, not Expert (Expert trades prose for
# density). Combining them into one row would test expansion on the mode
# with the *least* text and leave the highest-risk expansion case unrun,
# and would make a clipping finding impossible to attribute to either
# cause without a follow-up. Five rows, not four:
#
#   A: Guided,  JA, smallest(<900w), Light,    pointer  -> mandatory #1
#   B: Expert,  EN, smallest(<900w), Light,    pointer  -> mandatory #4
#   C: Guided,  EN, 1280x800,        HC Light, keyboard -> mandatory #2
#   D: Guided,  EN, 1920x1080,       Dark,     pointer  -> mandatory #3, size only
#   E: Expert,  EN, 1024x720,        HC Dark,  pointer
#
# Mandatory #3 ("Guided mode at 200% text scale") is NOT exercised by
# changing the niri output scale — per the same review (§3.2), that would
# rescale every window on the operator's active desktop for the run's
# duration, and the sub-900px row (A) already exercises substantially the
# same layout regime by near-equivalence: with no scale-aware rendering
# anywhere in the app (confirmed), a 2.0x output scale at a normal window
# is close to 1.0x scale at half the logical space, which is what row A
# already is. Row D below is the "Guided, otherwise-normal" control row
# this leaves in place; the 200%-scale gap itself is recorded in the
# evidence write-up, not worked around here.
#
# Usage: bash scripts/ux/run-all.sh <evidence-dir>
#   e.g. bash scripts/ux/run-all.sh .git-exclude/release-evidence/2026-08-04-mk056-l2

set -uo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) || {
    printf '[ux] ERROR: cannot determine script directory\n' >&2
    exit 2
}
# shellcheck source=scripts/ux/lib.sh
source "$script_dir/lib.sh"

if (( $# != 1 )); then
    printf 'Usage: %s <evidence-dir>\n' "${0##*/}" >&2
    exit 2
fi
evidence_dir=$1
mkdir -p -- "$evidence_dir"
results_file="$evidence_dir/results.txt"
: > "$results_file"

run_one() {
    local name=$1 width=$2 height=$3 input_method=$4
    shift 4
    ux_log "=== configuration: $name (${width}x${height}, input=$input_method) ==="
    bash "$script_dir/run-configuration.sh" \
        --name "$name" --width "$width" --height "$height" \
        --out-dir "$evidence_dir" --input-method "$input_method" \
        "$@" | tee -a "$results_file"
    printf '\n' >> "$results_file"
}

# Locale/theme/audience-mode are runtime UI state this script cannot set
# before the app opens — see run-configuration.sh's --keys-file mechanism
# once each screen's sequence is discovered (scripts/ux/discover-tab-order.sh).
# Below runs the size/launch/resize checks now; keyboard driving is added
# once a keys file exists for a given row.

# Row A: mandatory #1 — Japanese at the smallest supported window.
run_one row-a-guided-ja-smallest 880 700 pointer

# Row B: mandatory #4 — Expert mode at the smallest supported window,
# English so density is the only variable (not confounded with expansion).
run_one row-b-expert-en-smallest 880 700 pointer

# Row C: mandatory #2 — high-contrast theme with keyboard-only input.
run_one row-c-guided-en-hc-light-keyboard 1280 800 keyboard

# Row D: Guided-mode control row at a normal size/theme. Mandatory #3
# ("200% text scale") is deliberately NOT exercised here or anywhere in
# this script — see the header comment above. Record that gap in the
# evidence write-up; do not attempt an output-scale workaround.
run_one row-d-guided-en-dark-1920 1920 1080 pointer

# Row E: completes window-size coverage (1024x720) and theme coverage
# (HC Dark).
run_one row-e-expert-en-hc-dark-1024 1024 720 pointer

ux_log "all configurations run; results in $results_file, screenshots in $evidence_dir"
ux_log "REMINDER: mandatory #3 (200% text scale) was not exercised — record this as a"
ux_log "gap, with the near-equivalence argument to row A, in the evidence write-up."
