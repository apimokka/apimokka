#!/usr/bin/env bash
# RFC MK-056 layer L2 — Tab-order discovery.
#
# There is no documented focus order for any screen in this app (confirmed:
# the app implements no custom focus manager; Tab traversal is iced/winit's
# default widget order, which is not recorded anywhere). Before a keyboard
# sequence can be hardcoded into run-configuration.sh's assertion runs, the
# sequence has to be discovered by watching it happen once. This script
# sends one Tab at a time and screenshots after each, so the resulting
# images can be read (by a human, or by the implementer inspecting them) to
# determine which Tab index lands on which control.
#
# This script produces evidence for a human/implementer decision — it does
# not itself assert pass or fail. Requires the capability probe
# (scripts/ux/probe.sh) to have reported PROBE_RESULT=capable first.
#
# Usage: bash scripts/ux/discover-tab-order.sh <max-tabs> <output-dir>
#   e.g. bash scripts/ux/discover-tab-order.sh 15 .git-exclude/release-evidence/2026-08-04-mk056-l2/tab-discovery

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

if (( $# != 2 )); then
    printf 'Usage: %s <max-tabs> <output-dir>\n' "${0##*/}" >&2
    exit 2
fi
max_tabs=$1
output_dir=$2
mkdir -p -- "$output_dir"
# niri's screenshot-window --path requires an absolute path and silently
# no-ops (exit 0, no file, no error) on a relative one.
output_dir=$(cd -- "$output_dir" && pwd -P) || ux_die "cannot resolve output-dir to an absolute path"

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka"
command -v xdotool >/dev/null 2>&1 || ux_die "xdotool not found"

pid=$(ux_launch_app "$binary")
cleanup() { ux_kill_app "$pid"; }
trap cleanup EXIT

niri_id=$(ux_find_window_id "$pid" 10) || ux_die "niri never reported a window for pid $pid within 10s"
x11_id=$(ux_probe_xdotool "$niri_id") ||
    ux_die "xdotool cannot see this window — run scripts/ux/probe.sh first and confirm PROBE_RESULT=capable"

ux_screenshot "$niri_id" "$output_dir/00-initial.png"

for (( i = 1; i <= max_tabs; i++ )); do
    xdotool key --window "$x11_id" Tab || ux_die "xdotool key Tab failed at step $i"
    sleep 0.2
    ux_process_alive "$pid" || ux_die "process died after Tab press $i"
    printf -v step '%02d' "$i"
    ux_screenshot "$niri_id" "$output_dir/${step}-after-tab-$i.png"
done

ux_log "done — inspect $output_dir/*.png in order to find the Tab index for each control"
ux_log "of interest, then record it as a named constant in run-configuration.sh"
