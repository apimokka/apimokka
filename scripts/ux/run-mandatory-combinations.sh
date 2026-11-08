#!/usr/bin/env bash
# RFC MK-056 layer L2 — the four mandatory combinations named in decision
# 7, resolved 2026-11-04 to be L2 work in full (dev-team task 037).
#
#   1. Japanese at the smallest supported window (880x700)       — Guided,
#      the mode carrying the most text, so expansion is exercised where it
#      bites hardest (not confounded with density by also using Expert).
#   2. High-contrast theme, keyboard-only input                  — run
#      twice, HC Light and HC Dark, since decision 7 does not name one;
#      --show-cursor on both, since "focus against high-contrast borders"
#      is the thing this combination exists to check.
#   3. Guided mode at 200% scale                                 — 200% of
#      this host's own 1.2 baseline is 2.4; see §3 below for why the host
#      output's own mode must be switched first, and is restored after.
#   4. Expert mode at the smallest supported window (880x700)    — English,
#      so density is the only variable.
#
# Usage: bash scripts/ux/run-mandatory-combinations.sh <evidence-root-dir>
#          [--only 1,4] [--renderer software]
#   --only      run just these combination numbers (2 = both HC runs)
#   --renderer  passed to every run; software is for when wgpu cannot run
#               and is NOT comparable evidence for combination 2 or 3
#   e.g. bash scripts/ux/run-mandatory-combinations.sh \
#          .git-exclude/release-evidence/2026-11-07-task037-m6-l2-mandatory

set -uo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) || {
    printf '[ux] ERROR: cannot determine script directory\n' >&2
    exit 2
}
# shellcheck source=scripts/ux/lib.sh
source "$script_dir/lib.sh"

if (( $# < 1 )); then
    printf 'Usage: %s <evidence-root-dir> [--only 1,4] [--renderer software]\n' "${0##*/}" >&2
    exit 2
fi
evidence_root=$1
shift
only=1,2,3,4
renderer_args=()
while (( $# > 0 )); do
    case "$1" in
        --only) only=$2; shift 2 ;;
        --renderer) renderer_args=(--renderer "$2"); shift 2 ;;
        *) ux_die "unknown argument: $1" ;;
    esac
done
wanted() { [[ ",$only," == *",$1,"* ]]; }
mkdir -p -- "$evidence_root"

run_one() {
    local combo_dir=$1
    shift
    mkdir -p -- "$combo_dir"
    bash "$script_dir/run-configuration.sh" --out-dir "$combo_dir" "${renderer_args[@]}" "$@" ||
        ux_die "run-configuration.sh failed for $combo_dir (see its own output above) — stopping rather than silently continuing to the next combination"
}

if wanted 1; then
    ux_log "=== combination 1: Japanese at 880x700, Guided ==="
    run_one "$evidence_root/combo1-japanese-880x700" \
        --name combo1 --width 880 --height 700 --mode guided --locale ja --theme light

fi

if wanted 2; then
    ux_log "=== combination 2: high contrast, keyboard-only — HC Light ==="
    run_one "$evidence_root/combo2-high-contrast-keyboard" \
        --name combo2-hclight --width 1280 --height 800 --mode guided --locale en \
        --theme hclight --show-cursor

    ux_log "=== combination 2: high contrast, keyboard-only — HC Dark ==="
    run_one "$evidence_root/combo2-high-contrast-keyboard" \
        --name combo2-hcdark --width 1280 --height 800 --mode guided --locale en \
        --theme hcdark --show-cursor

fi
# ── combination 3: Guided at 200% scale ─────────────────────────────────────
#
# 200% of this host's 1.2 baseline is nested output scale 2.4. An
# 880x700-logical nested window at scale 2.4 needs a 1760x1400-logical
# window on the *host* (nested physical = host logical × host scale; nested
# logical = nested physical ÷ nested scale) — taller than this host's
# current 1200-logical-px-tall mode, the exact trap the session pack's own
# §2.4 names. Its own fix: switch the host output to a taller mode first,
# run the segment, switch back — `niri msg output` changes are temporary
# and not written to config, so a crash mid-run still reverts on the next
# login, but this script restores it explicitly and verifies the restore
# rather than relying on that alone.
if wanted 3; then
    ux_log "=== combination 3: Guided at 200% scale (880x700 nested-logical) ==="

    host_output=$(niri msg --json outputs | jq -r 'keys[0]')
    [[ -n "$host_output" ]] || ux_die "could not determine the host's own output name"

    read -r orig_mode_w orig_mode_h orig_mode_rate orig_scale < <(
        niri msg --json outputs | jq -r --arg o "$host_output" '
            .[$o] | .modes[.current_mode] as $m |
            "\($m.width) \($m.height) \($m.refresh_rate / 1000) \(.logical.scale)"'
    )
    ux_log "host output $host_output: current mode ${orig_mode_w}x${orig_mode_h}@${orig_mode_rate}, scale $orig_scale"

    tall_mode=$(niri msg --json outputs | jq -r --arg o "$host_output" '
        .[$o].modes | map(select(.height >= 2000)) | sort_by(-.width) | .[0] |
        if . == null then empty else "\(.width)x\(.height)@\(.refresh_rate / 1000)" end')
    [[ -n "$tall_mode" ]] ||
        ux_die "host output $host_output has no mode tall enough (>=2000px) for the 200% segment — pick a window that fits the current mode instead and say so in the write-up"

    ux_log "switching host output $host_output to $tall_mode for the 200% segment"
    niri msg output "$host_output" mode "$tall_mode" ||
        ux_die "failed to switch host output mode to $tall_mode"
    sleep 0.5

    restore_host_mode() {
        ux_log "restoring host output $host_output to ${orig_mode_w}x${orig_mode_h}@${orig_mode_rate}, scale $orig_scale"
        niri msg output "$host_output" mode "${orig_mode_w}x${orig_mode_h}@${orig_mode_rate}"
        niri msg output "$host_output" scale "$orig_scale"
        sleep 0.5
        read -r now_w now_h now_rate now_scale < <(
            niri msg --json outputs | jq -r --arg o "$host_output" '
                .[$o] | .modes[.current_mode] as $m |
                "\($m.width) \($m.height) \($m.refresh_rate / 1000) \(.logical.scale)"')
        if [[ "$now_w" == "$orig_mode_w" && "$now_h" == "$orig_mode_h" && "$now_scale" == "$orig_scale" ]]; then
            ux_log "host output restored and verified: ${now_w}x${now_h}, scale $now_scale"
        else
            ux_log "WARNING: host output restore did not verify — now ${now_w}x${now_h}@${now_scale}, expected ${orig_mode_w}x${orig_mode_h}@${orig_scale}. Re-run manually: niri msg output $host_output mode ${orig_mode_w}x${orig_mode_h}@${orig_mode_rate} && niri msg output $host_output scale $orig_scale"
        fi
    }
    trap restore_host_mode EXIT

    # run-configuration.sh's own --width/--height are HOST-logical (passed
    # straight to ux_host_float_and_resize, which floats/resizes the HOST
    # container window) — not nested-logical. At nested-scale 2.4 against this
    # host's 1.2 baseline, an 880x700 NESTED-logical window needs a
    # 1760x1400 HOST-logical container (host-logical = nested-logical ×
    # nested-scale ÷ host-scale), exactly the session pack's own §2.4 worked
    # example. Passing 880x700 directly here was tried first and produced a
    # measured nested-logical of only 440x350 — a real harness bug, caught by
    # the RESULTS.md measurement itself disagreeing with the request, not
    # trusted from the command alone (task 033's own "measure the window
    # before believing the frame" lesson, applied to the rig this time).
    run_one "$evidence_root/combo3-guided-200pct-scale" \
        --name combo3 --width 1760 --height 1400 --scale 2.4 --mode guided --locale en --theme light

    restore_host_mode
    trap - EXIT
fi

if wanted 4; then
    ux_log "=== combination 4: Expert at 880x700, English ==="
    run_one "$evidence_root/combo4-expert-880x700" \
        --name combo4 --width 880 --height 700 --mode expert --locale en --theme light
fi

ux_log "requested combinations ($only) run; see $evidence_root/*/RESULTS.md-named files"
