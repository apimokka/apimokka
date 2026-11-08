#!/usr/bin/env bash
# RFC MK-056 layer L2 — run one configuration end to end: start a nested
# niri compositor sized and scaled for the configuration, launch apimokka
# inside it, drive it to the required audience mode / locale / theme,
# capture the application's primary surfaces, tear down, and print the
# measured geometry plus decision 7's recorded fields as KEY=value lines.
#
# Usage:
#   bash scripts/ux/run-configuration.sh \
#     --name <config-name> \
#     --width <host-logical-px> --height <host-logical-px> \
#     [--scale <nested-output-scale, default 1.2>] \
#
# --width/--height size the HOST container window (passed straight to
# ux_host_float_and_resize) — NOT the resulting nested-logical window
# size, which also depends on --scale against this host's own baseline
# scale (host-logical = nested-logical × nested-scale ÷ host-scale). When
# --scale matches the host's own baseline the two are equal, which is why
# this went unnoticed until the 200%-scale combination (--scale 2.4
# against a 1.2 host) needed the caller to do that conversion explicitly
# — see scripts/ux/run-mandatory-combinations.sh's own combination 3
# comment for the worked numbers.
#     [--mode guided|expert (default guided)] \
#     [--locale en|ja (default en)] \
#     [--theme light|dark|hclight|hcdark (default light)] \
#     [--show-cursor (Tab once before each deliverable capture, so the
#        region-cursor ring is visible in it — combination 2's own need)] \
#     [--renderer wgpu|software (default wgpu; software forces iced's
#        tiny-skia, slows every key and capture to match it, and exists
#        only for when wgpu cannot run — the measured renderer is recorded
#        in RESULTS.md and must match this request, else the run fails)] \
#     [--rule N (0-based rule to open in the editor, default 0; 2 is the
#        rule with headers and body conditions)] \
#     [--walk-conditions (Expert, needs --rule 2: Tab the region cursor to the\n#        header value and a body value so the application's own scroll-snap\n#        brings them into view at any window height; 07/08 frames)] \
#     --out-dir <dir>
#
# Captures, under --out-dir:
#   <name>-00-launch.png    the mode picker, rest state
#   <name>-01-routes.png    the Routes workbench, workspace open
#   <name>-02-settings.png  Settings
#   <name>-04-editor.png    the rule editor (WHEN and RESPOND side by side),
#                           first rule of the first rule set selected
#   <name>-05-testrule.png  the Test Rule dialog over the editor
#   <name>-03-drawer.png    the save-diff drawer (the one dialog-equivalent
#                           surface every configuration can reach the same
#                           way — the mock's own default workspace is
#                           already dirty)
# plus a RESULTS.md carrying decision 7's nine fields and the measured
# geometry (never the geometry the commands requested), and a
# <name>-MANIFEST.md (manifest.sh): every kept frame's MD5 and ground pixel.
# The run FAILS if two kept frames are byte-identical or a frame's ground
# contradicts the requested theme.

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

name=
width=
height=
scale=1.2
mode=guided
locale=en
theme=light
show_cursor=no
renderer=wgpu
rule=0
walk_conditions=no
out_dir=

while (( $# > 0 )); do
    case "$1" in
        --name) name=$2; shift 2 ;;
        --width) width=$2; shift 2 ;;
        --height) height=$2; shift 2 ;;
        --scale) scale=$2; shift 2 ;;
        --mode) mode=$2; shift 2 ;;
        --locale) locale=$2; shift 2 ;;
        --theme) theme=$2; shift 2 ;;
        --show-cursor) show_cursor=yes; shift ;;
        --renderer) renderer=$2; shift 2 ;;
        --rule) rule=$2; shift 2 ;;
        --walk-conditions) walk_conditions=yes; shift ;;
        --out-dir) out_dir=$2; shift 2 ;;
        *) ux_die "unknown argument: $1" ;;
    esac
done

[[ -n "$name" && -n "$width" && -n "$height" && -n "$out_dir" ]] ||
    ux_die "--name, --width, --height, and --out-dir are required"

mkdir -p -- "$out_dir"
out_dir=$(cd -- "$out_dir" && pwd -P) || ux_die "cannot resolve --out-dir to an absolute path"
mkdir -p -- "$out_dir/_checkpoints"

app_env=()
case "$renderer" in
    wgpu) ;;
    software)
        app_env=(ICED_BACKEND=tiny-skia)
        export UX_PAUSE_SCALE=3 UX_CAPTURE_SETTLE=2.5
        ;;
    *) ux_die "--renderer must be wgpu or software, got: $renderer" ;;
esac

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka --locked"

build_sha=$(cd -- "$repository_root" && git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)
tree_dirty=$(cd -- "$repository_root" && git status --short 2>/dev/null)
[[ -n "$tree_dirty" ]] && build_sha="${build_sha}+dirty"

nested_cfg="$out_dir/_nested-niri.kdl"
cat >"$nested_cfg" <<EOF
output "winit" {
    scale $scale
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

nested_log="$out_dir/_nested-niri.log"
read -r nested_pid nested_display nested_sock < <(ux_start_nested_compositor "$nested_cfg" "$nested_log")

cleanup() {
    ux_kill_app "${app_pid:-}" 2>/dev/null || true
    ux_stop_nested_compositor "$nested_pid" 2>/dev/null || true
}
trap cleanup EXIT

host_win_id=$(ux_host_window_id_for_pid "$nested_pid" 10) ||
    ux_die "host niri never reported a window for nested compositor pid $nested_pid"
ux_host_float_and_resize "$host_win_id" "$width" "$height"
sleep 0.3

app_pid=$(ux_launch_app "$binary" -u DISPLAY "WAYLAND_DISPLAY=$nested_display" "${app_env[@]}")
app_win_id=$(ux_nested_window_id "$nested_sock" 10) ||
    ux_die "nested compositor never reported the app's window within 10s"
ux_assert_focus "$nested_sock" "$app_pid"

renderer_measured=$(ux_renderer_evidence "$app_pid")
case "$renderer:$renderer_measured" in
    wgpu:wgpu*|software:tiny-skia*) ;;
    *) ux_die "requested renderer '$renderer' but measured '$renderer_measured' — evidence would not be comparable" ;;
esac

prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-00-launch.png")

ux_app_select_mode "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$mode"
ux_app_open_first_workspace "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash

if [[ "$locale" != en ]]; then
    ux_app_set_locale "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$locale"
fi
if [[ "$theme" != light ]]; then
    ux_app_set_theme "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$theme" "$locale"
fi

maybe_show_cursor() {
    if [[ "$show_cursor" == yes ]]; then
        # D-29 is fixed: a Tab under an HC theme must now change the frame,
        # so the strict (retry, then fail) check applies to every theme —
        # `soft` stays available in app-nav.sh for a regression hunt only.
        ux_app_show_region_cursor "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash no
    fi
    return 0
}

# Each deliverable capture below follows a navigation step that already
# verified its own landing via a checkpoint capture (app-nav.sh's own
# ux_capture_step calls) — there is no *new* action between "state
# confirmed" and "screenshot it for the record", so a plain ux_capture is
# the honest call here, not a second expect-change comparison against a
# frame that correctly has not changed since the checkpoint right before
# it.
maybe_show_cursor
prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-01-routes.png")

# The rule editor — the application's densest surface, which round 1 never
# rendered below 1280 (Routes was captured in its empty state). It must be
# opened before anything else moves the region cursor on the workspace
# screen: five Tabs from a fresh workspace reach the sidebar, or four when
# --show-cursor's own Tab above already left the cursor on the top bar.
editor_tabs=5
[[ "$show_cursor" == yes ]] && editor_tabs=4
ux_app_select_first_rule "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$editor_tabs" "$rule"
prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-04-editor.png")
extra_frames=()

# Combination 2's own question — focus indication against high-contrast
# borders — needs the cursor, the selected state and an unselected row in ONE
# frame. 04 shows selected + cursored (Return leaves the cursor on the row it
# selected); Up moves the cursor to the neighbouring rule, so 06 shows
# selected (not cursored), cursored (not selected) and a further unselected
# row together. Down puts the cursor back so the Tab counts below stay valid.
if [[ "$show_cursor" == yes ]]; then
    ux_app_key "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash Up "cursor-up"
    prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-06-cursor-on-unselected.png")
    extra_frames+=("$out_dir/${name}-06-cursor-on-unselected.png")
    ux_app_key "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash Down "cursor-down"
fi

# Conditions below the fold: walk the region cursor into the WHEN column and
# let the application's own scroll-snap bring the field into view (task 037
# §9.1 item 1b) — the rig has no scroll of its own and needs none. Expert,
# rule 3: 11 Tabs from the sidebar reach the header VALUE (TestRule, Duplicate,
# MoveUp, MoveDown, Delete, UrlPath, UrlPathOp, Method, HeaderName, HeaderOp,
# HeaderValue), 6 more reach the first body VALUE (HeaderRemove, AddHeader,
# BodyPath, BodyPathAssistant, BodyOp, BodyValue).
if [[ "$walk_conditions" == yes ]]; then
    ux_app_tab_times "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash 11 "to-header-value"
    prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-07-header-value.png")
    ux_app_tab_times "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash 6 "to-body-value"
    prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-08-body-value.png")
    extra_frames+=("$out_dir/${name}-07-header-value.png" "$out_dir/${name}-08-body-value.png")
fi

ux_app_open_test_rule "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$locale"
# A dialog raised over a palette that has just closed takes the software
# renderer several repaints to settle (stale-damage ghosts of the palette
# otherwise survive in the frame); wait it out rather than capture the
# transition. A no-op on wgpu (UX_PAUSE_SCALE is 1: a 1.5 s wait).
ux_pause 1.5
prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-05-testrule.png")
ux_app_close_overlay "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash

ux_app_open_settings "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$locale"
maybe_show_cursor
prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-02-settings.png")

ux_app_open_routes "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$locale"
ux_app_open_save_diff_drawer "$nested_display" "$nested_sock" "$app_win_id" "$out_dir" prev_hash "$locale"
maybe_show_cursor
prev_hash=$(ux_capture "$nested_sock" "$app_win_id" "$out_dir/${name}-03-drawer.png")

# Frame integrity: names, MD5s and measured ground, failing the run on a
# duplicated frame or a theme contradicted by its own pixels.
bash "$script_dir/manifest.sh" --theme "$theme" --renderer "$renderer" \
    --out "$out_dir/${name}-MANIFEST.md" \
    "$out_dir/${name}-00-launch.png@overlay" \
    "$out_dir/${name}-01-routes.png" \
    "$out_dir/${name}-02-settings.png" \
    "$out_dir/${name}-03-drawer.png@overlay" \
    "$out_dir/${name}-04-editor.png" \
    "$out_dir/${name}-05-testrule.png@overlay" \
    "${extra_frames[@]}" ||
    ux_die "frame manifest failed for $name — evidence in $out_dir is not trustworthy"

read -r logical_w logical_h logical_scale physical_w physical_h < <(ux_measure_nested_output "$nested_sock")
png_physical=$(ux_png_physical_size "$out_dir/${name}-03-drawer.png")

cat >"$out_dir/${name}-RESULTS.md" <<EOF
# Configuration \`$name\` — measured

| Field | Value |
|---|---|
| Commit SHA | \`$build_sha\` |
| Platform | Linux, niri $(niri --version 2>&1 | head -1 | awk '{print $2}') |
| Theme | $theme |
| Locale | $locale |
| Audience mode | $mode |
| Window size (requested, host-logical container) | ${width}x${height} |
| Window size (measured, nested-logical) | ${logical_w}x${logical_h} @ scale ${logical_scale} |
| Window size (measured, physical) | ${physical_w}x${physical_h} |
| PNG physical size (cross-check, drawer frame) | $png_physical |
| Input method | keyboard (wtype on the nested compositor's socket) |
| Renderer (requested / measured) | $renderer / $renderer_measured |
| Entry state | Returning (opened the mock's default workspace) |

Screenshots: \`${name}-00-launch.png\`, \`${name}-01-routes.png\`,
\`${name}-02-settings.png\`, \`${name}-03-drawer.png\`,
\`${name}-04-editor.png\` (rule editor, first rule selected),
\`${name}-05-testrule.png\` (Test Rule dialog). Frame integrity:
\`${name}-MANIFEST.md\`.
EOF

ux_log "configuration $name complete: $out_dir/${name}-RESULTS.md"
cat "$out_dir/${name}-RESULTS.md" >&2

printf 'CONFIG=%s\n' "$name"
printf 'BUILD_SHA=%s\n' "$build_sha"
printf 'MEASURED_LOGICAL=%sx%s@%s\n' "$logical_w" "$logical_h" "$logical_scale"
printf 'MEASURED_PHYSICAL=%sx%s\n' "$physical_w" "$physical_h"
printf 'PNG_PHYSICAL=%s\n' "$png_physical"
printf 'RENDERER=%s\n' "$renderer_measured"
printf 'OUT_DIR=%s\n' "$out_dir"
