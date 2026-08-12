#!/usr/bin/env bash
# RFC MK-056 layer L2 — shared helpers for scripted GUI verification.
#
# Driven with niri msg action (window placement, resize, screenshot) and,
# where the capability probe confirms it works, xdotool (keyboard/pointer
# input synthesis). This library targets niri specifically; it says so
# rather than pretending to be compositor-agnostic. Not sourced by
# scripts/check-*.sh and never wired into check-release-gates.sh — this
# requires a live display and cannot run in the canonical gate.
#
# Every function that touches the live session logs what it did to stderr,
# so a script built on this library produces an audit trail even when run
# non-interactively.

set -uo pipefail

ux_log() {
    printf '[ux] %s\n' "$1" >&2
}

ux_die() {
    printf '[ux] ERROR: %s\n' "$1" >&2
    exit 1
}

for _ux_required in niri jq; do
    command -v "$_ux_required" >/dev/null 2>&1 ||
        ux_die "required utility not found: $_ux_required"
done
unset _ux_required

# ux_launch_app <binary-path> -> prints PID to stdout
ux_launch_app() {
    local binary=$1
    [[ -x "$binary" ]] || ux_die "binary not found or not executable: $binary"
    "$binary" >/dev/null 2>&1 &
    local pid=$!
    ux_log "launched $binary (pid $pid)"
    printf '%s\n' "$pid"
}

# ux_find_window_id <pid> [timeout-seconds] -> prints niri window id to stdout
ux_find_window_id() {
    local pid=$1
    local timeout=${2:-10}
    local id=
    # Poll in half-second ticks, counted as an integer, rather than
    # accumulating a fractional elapsed-seconds value: bash's `(( ))` is
    # integer-only, and comparing it against a value like "0.5" is a
    # runtime arithmetic error, not a false condition — the loop would
    # silently give up after a single check instead of polling for the
    # full timeout.
    local max_ticks=$(( timeout * 2 ))
    local tick=0

    while (( tick < max_ticks )); do
        id=$(niri msg --json windows 2>/dev/null |
            jq -r --argjson pid "$pid" '[.[] | select(.pid == $pid)] | .[0].id // empty')
        if [[ -n "$id" ]]; then
            printf '%s\n' "$id"
            return 0
        fi
        sleep 0.5
        tick=$(( tick + 1 ))
    done
    return 1
}

# ux_window_title <niri-window-id> -> prints current title to stdout
ux_window_title() {
    local id=$1
    niri msg --json windows 2>/dev/null |
        jq -r --argjson id "$id" '.[] | select(.id == $id) | .title // empty'
}

# ux_process_alive <pid> -> exit status 0 if the process still exists
ux_process_alive() {
    kill -0 "$1" 2>/dev/null
}

# ux_float_and_resize <niri-window-id> <width> <height>
#
# Niri is a scrolling-tiled compositor: a window has an arbitrary pixel
# size only once floated. This moves the window to the floating layout
# first, then sets an absolute width and height. `set-window-width` /
# `set-window-height`'s <CHANGE> grammar was not exercised against a live
# window before this script was written (the author held off pending
# go-ahead to touch the live session) — verify a plain integer is accepted
# as an absolute pixel value on first real run, and adjust here if niri
# expects a different form (e.g. a `px` suffix).
ux_float_and_resize() {
    local id=$1
    local width=$2
    local height=$3

    ux_log "floating and resizing window $id to ${width}x${height}"
    niri msg action move-window-to-floating --id "$id" ||
        ux_die "move-window-to-floating failed for window $id"
    niri msg action set-window-width --id "$id" "$width" ||
        ux_die "set-window-width failed for window $id"
    niri msg action set-window-height --id "$id" "$height" ||
        ux_die "set-window-height failed for window $id"
}

# ux_screenshot <niri-window-id> <absolute-output-path>
ux_screenshot() {
    local id=$1
    local path=$2
    mkdir -p -- "$(dirname -- "$path")"
    ux_log "screenshot: window $id -> $path"
    niri msg action screenshot-window --id "$id" --path "$path" \
        --write-to-disk true --show-pointer false ||
        ux_die "screenshot-window failed for window $id"
    [[ -s "$path" ]] || ux_die "screenshot did not produce a file: $path"
}

# ux_kill_app <pid>
ux_kill_app() {
    local pid=$1
    if ux_process_alive "$pid"; then
        kill "$pid" 2>/dev/null
        for _ in 1 2 3 4 5 6 7 8 9 10; do
            ux_process_alive "$pid" || return 0
            sleep 0.3
        done
        ux_log "pid $pid did not exit after SIGTERM, sending SIGKILL"
        kill -9 "$pid" 2>/dev/null
    fi
}

# ux_probe_xdotool <niri-window-id> -> exit status 0 and prints an
# X11 window id to stdout if xdotool can see and target this window
# (meaning it is available via XWayland); exit status 1 with nothing on
# stdout if not (native-Wayland surfaces are not visible to X11 clients by
# design — this is expected, not a bug, and callers must degrade
# gracefully rather than fail).
ux_probe_xdotool() {
    local niri_id=$1
    command -v xdotool >/dev/null 2>&1 || return 1

    local title
    title=$(ux_window_title "$niri_id") || return 1
    [[ -n "$title" ]] || return 1

    local x11_id
    x11_id=$(xdotool search --name -- "$title" 2>/dev/null | head -1)
    [[ -n "$x11_id" ]] || return 1
    printf '%s\n' "$x11_id"
}
