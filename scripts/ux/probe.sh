#!/usr/bin/env bash
# RFC MK-056 layer L2 — capability probe.
#
# Answers the one question every other script in scripts/ux/ depends on:
# can this host session actually drive the apimokka window with keyboard
# input, or only place/resize/screenshot it? Run this first, once, before
# anything else in this directory. It launches apimokka, locates its niri
# window, checks whether xdotool can see it (native-Wayland surfaces are
# not visible to X11 clients by design — a "no" here is expected
# information, not a bug), and cleans up. No resize, no keyboard input is
# sent to the app itself.
#
# Usage: bash scripts/ux/probe.sh

set -uo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) || {
    printf '[ux] ERROR: cannot determine script directory\n' >&2
    exit 2
}
repository_root=$(cd -- "$script_dir/../.." 2>/dev/null && pwd -P) || {
    printf '[ux] ERROR: cannot determine repository root\n' >&2
    exit 2
}
# shellcheck source=scripts/ux/lib.sh
source "$script_dir/lib.sh"

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka"

ux_log "compositor: $(niri --version 2>&1 | head -1)"
ux_log "WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>} DISPLAY=${DISPLAY:-<unset>} XDG_SESSION_TYPE=${XDG_SESSION_TYPE:-<unset>}"

pid=$(ux_launch_app "$binary")
cleanup() { ux_kill_app "$pid"; }
trap cleanup EXIT

niri_id=$(ux_find_window_id "$pid" 10) || ux_die "niri never reported a window for pid $pid within 10s"
ux_log "niri window id: $niri_id"

title=$(ux_window_title "$niri_id")
ux_log "window title: ${title:-<empty>}"
[[ "$title" == "apimokka" ]] ||
    ux_log "WARNING: expected title 'apimokka' at first launch (no workspace open), got '${title:-<empty>}'"

if x11_id=$(ux_probe_xdotool "$niri_id"); then
    ux_log "xdotool CAN see this window (X11 id $x11_id) — likely via XWayland; keyboard-driven"
    ux_log "assertions should work. Verify with a real key send before trusting this fully:"
    ux_log "  xdotool key --window $x11_id Tab"
    result=capable
else
    ux_log "xdotool CANNOT see this window — it is a native-Wayland surface, invisible to"
    ux_log "X11 clients by the Wayland security model. This is expected, not a defect."
    ux_log "Keyboard-input-synthesis tooling for native Wayland (ydotool, wtype) is not"
    ux_log "installed on this host. Without one of those, keyboard-reachability assertions"
    ux_log "cannot be made by this script; only launch/resize/screenshot evidence can be"
    ux_log "produced, and the reachability requirement becomes a captured-for-human-review"
    ux_log "artefact rather than a script-decided pass/fail."
    result=incapable
fi

printf 'PROBE_RESULT=%s\n' "$result"
printf 'PROBE_NIRI_WINDOW_TITLE=%s\n' "${title:-}"
