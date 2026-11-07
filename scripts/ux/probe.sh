#!/usr/bin/env bash
# RFC MK-056 layer L2 — rig smoke test.
#
# Confirms the nested-niri-plus-wtype recipe works end to end on this
# host, before trusting it for a real run: starts a small nested
# compositor, launches apimokka inside it, sends one Tab, captures two
# frames and asserts they differ (proof the keystroke reached the app and
# the screenshot discipline landed), then tears everything down.
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
# shellcheck source=scripts/ux/app-nav.sh
source "$script_dir/app-nav.sh"

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka --locked"

work_dir=$(mktemp -d) || ux_die "mktemp -d failed"
cleanup() {
    ux_kill_app "${app_pid:-}" 2>/dev/null || true
    ux_stop_nested_compositor "${nested_pid:-}" 2>/dev/null || true
    rm -rf -- "$work_dir"
}
trap cleanup EXIT

cfg="$work_dir/nested-niri.kdl"
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

ux_log "compositor: $(niri --version 2>&1 | head -1)"
read -r nested_pid nested_display nested_sock < <(ux_start_nested_compositor "$cfg" "$work_dir/niri.log")

host_win_id=$(ux_host_window_id_for_pid "$nested_pid" 10) ||
    ux_die "host niri never reported a window for the nested compositor"
ux_host_float_and_resize "$host_win_id" 1280 720
sleep 0.3

app_pid=$(ux_launch_app "$binary" -u DISPLAY "WAYLAND_DISPLAY=$nested_display")
app_win_id=$(ux_nested_window_id "$nested_sock" 10) ||
    ux_die "nested compositor never reported the app's window within 10s"
ux_assert_focus "$nested_sock" "$app_pid"
ux_log "app pid=$app_pid, nested window id=$app_win_id, renderer: $(ux_renderer_evidence "$app_pid")"

hash_before=$(ux_capture "$nested_sock" "$app_win_id" "$work_dir/before.png")
ux_wtype_key "$nested_display" Tab
sleep 0.3
hash_after=$(ux_capture "$nested_sock" "$app_win_id" "$work_dir/after.png")

if [[ "$hash_before" == "$hash_after" ]]; then
    ux_log "RESULT: FAIL — a Tab keystroke produced no visible change; see $work_dir before cleanup runs"
    printf 'PROBE_RESULT=fail\n'
    exit 1
fi

ux_log "RESULT: pass — nested rig, wtype, and the capture-with-proof discipline all work on this host"
printf 'PROBE_RESULT=pass\n'
printf 'NESTED_GEOMETRY=%s\n' "$(ux_measure_nested_output "$nested_sock")"
