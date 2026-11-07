#!/usr/bin/env bash
# RFC MK-056 layer L2 — Tab-order discovery.
#
# No screen's focus order is documented anywhere in this codebase outside
# the region/cursor model's own source (`crates/app/src/regions.rs`). For
# a screen not already covered by scripts/ux/app-nav.sh, this sends one
# Tab at a time from first launch and captures after each, so the
# resulting images can be read to find which Tab index reaches a given
# control, before hand-writing a new navigation helper from what is seen.
#
# This script produces evidence for a human/implementer decision — it
# does not itself assert pass or fail.
#
# Usage: bash scripts/ux/discover-tab-order.sh <max-tabs> <output-dir>
#   e.g. bash scripts/ux/discover-tab-order.sh 15 \
#          .git-exclude/release-evidence/2026-11-07-tab-discovery

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
output_dir=$(cd -- "$output_dir" && pwd -P) || ux_die "cannot resolve output-dir to an absolute path"

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka --locked"

cfg="$output_dir/_nested-niri.kdl"
cat >"$cfg" <<'EOF'
output "winit" {
    scale 1.2
}
layout {
    gaps 0
    default-column-width { proportion 1.0; }
    border {
        off
    }
    focus-ring {
        off
    }
}
animations {
    off
}
prefer-no-csd
spawn-at-startup "true"
EOF

read -r nested_pid nested_display nested_sock < <(ux_start_nested_compositor "$cfg" "$output_dir/_nested-niri.log")
cleanup() {
    ux_kill_app "${app_pid:-}" 2>/dev/null || true
    ux_stop_nested_compositor "$nested_pid" 2>/dev/null || true
}
trap cleanup EXIT

host_win_id=$(ux_host_window_id_for_pid "$nested_pid" 10) ||
    ux_die "host niri never reported a window for the nested compositor"
ux_host_float_and_resize "$host_win_id" 1280 720
sleep 0.3

app_pid=$(ux_launch_app "$binary" -u DISPLAY "WAYLAND_DISPLAY=$nested_display")
app_win_id=$(ux_nested_window_id "$nested_sock" 10) ||
    ux_die "nested compositor never reported the app's window within 10s"
ux_assert_focus "$nested_sock" "$app_pid"

prev_hash=""
ux_capture_step "$nested_sock" "$app_win_id" "$output_dir/00-initial.png" prev_hash yes

for (( i = 1; i <= max_tabs; i++ )); do
    printf -v step '%02d' "$i"
    step_fn() { ux_wtype_key "$nested_display" Tab; sleep 0.2; }
    step_fn
    ux_capture_step "$nested_sock" "$app_win_id" "$output_dir/${step}-after-tab-$i.png" prev_hash no step_fn
done

ux_log "done — inspect $output_dir/*.png in order to find the Tab index for each control of interest"
