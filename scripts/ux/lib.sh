#!/usr/bin/env bash
# RFC MK-056 layer L2 — shared helpers for scripted GUI verification.
#
# Task 037 brought this library onto the recipe that actually works, proven
# across tasks 035 and 036: a **nested niri compositor** gives the
# apimokka window its size and output scale *at launch*, inside its own
# config, never by floating/resizing the window under test after the fact
# (`.git-exclude/rules/project-instructions-gui-common.md`'s own rule — the
# thing floated/resized here is the nested compositor's own host-level
# container window, not the application). Keyboard input is `wtype` on the
# nested compositor's socket — the only input path that reaches a native
# Wayland surface at all; `xdotool` cannot (`zwp_virtual_keyboard_manager_v1`
# vs. an X11-only tool) and is not used anywhere in this directory. It
# remains this project's pointer-input route elsewhere, through the X11
# backend — see the GUI evidence rules file above — which none of
# `scripts/ux/`'s own combinations need, since MK-056 decision 7's four
# mandatory combinations are all keyboard-only L2 checks.
#
# Not sourced by scripts/check-*.sh and never wired into
# check-release-gates.sh — this requires a live display and cannot run in
# the canonical gate.
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

for _ux_required in niri jq wtype file sha256sum python3 md5sum; do
    command -v "$_ux_required" >/dev/null 2>&1 ||
        ux_die "required utility not found: $_ux_required"
done
unset _ux_required

# ux_pause <seconds>
#
# Every settle delay in this directory goes through here so a slow renderer
# can be given proportionally more time without editing each call site:
# UX_PAUSE_SCALE multiplies the delay (default 1). The software renderer
# (run-configuration.sh --renderer software) sets it, because a repaint that
# takes seconds makes a key sent 0.3 s after the last one race the redraw.
ux_pause() {
    sleep "$(awk -v d="$1" -v k="${UX_PAUSE_SCALE:-1}" 'BEGIN { print d * k }')"
}

# ── Process lifecycle ──────────────────────────────────────────────────────

# ux_launch_app <binary-path> [env assignments...] -> prints PID to stdout
#
# Extra arguments are passed through to `env` ahead of the binary, so a
# caller can set WAYLAND_DISPLAY (and unset DISPLAY) for the nested socket
# without the launched process ever touching the host's own display.
ux_launch_app() {
    local binary=$1
    shift
    [[ -x "$binary" ]] || ux_die "binary not found or not executable: $binary"
    env "$@" "$binary" >/dev/null 2>&1 &
    local pid=$!
    ux_log "launched $binary (pid $pid) env: $*"
    printf '%s\n' "$pid"
}

# ux_process_alive <pid> -> exit status 0 if the process still exists
ux_process_alive() {
    kill -0 "$1" 2>/dev/null
}

# ux_kill_app <pid>
#
# "One instance only" (GUI evidence rules): kill by exact pid, never by
# name-matching a process list, so a stale instance elsewhere on the host
# is never touched.
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

# ── The nested compositor ───────────────────────────────────────────────────

# ux_start_nested_compositor <config-path> <log-path>
#   -> prints "PID WAYLAND_DISPLAY NIRI_SOCKET" to stdout
#
# Starts a nested niri (winit backend) with the given config, which must
# set `default-column-width { proportion 1.0; }` and `animations { off }`
# so the app window fills the nested output from its very first frame with
# no post-hoc resize, and an explicit output `scale` (1.2 to match this
# host's own baseline, or a multiple of it for a scale combination). Waits
# for niri's own log lines announcing the Wayland socket and the IPC
# socket path, rather than a fixed sleep.
ux_start_nested_compositor() {
    local cfg=$1
    local log=$2
    rm -f "$log"
    mkdir -p -- "$(dirname -- "$log")"
    niri -c "$cfg" >"$log" 2>&1 &
    local pid=$!

    local wl_display= sock_path= tick=0
    while (( tick < 100 )); do
        if [[ -f "$log" ]]; then
            wl_display=$(grep -oE 'listening on Wayland socket: [^ ]+' "$log" | head -1 | awk '{print $NF}')
            sock_path=$(grep -oE 'IPC listening on: [^ ]+' "$log" | head -1 | awk '{print $NF}')
            [[ -n "$wl_display" && -n "$sock_path" ]] && break
        fi
        ux_process_alive "$pid" || ux_die "nested niri (pid $pid) exited before reporting its socket — see $log"
        sleep 0.1
        tick=$(( tick + 1 ))
    done
    [[ -n "$wl_display" && -n "$sock_path" ]] ||
        ux_die "nested niri did not report its socket within 10s — see $log"

    ux_log "nested compositor pid=$pid display=$wl_display socket=$sock_path config=$cfg"
    printf '%s %s %s\n' "$pid" "$wl_display" "$sock_path"
}

# ux_stop_nested_compositor <pid>
ux_stop_nested_compositor() {
    ux_kill_app "$1"
}

# ── Host-level sizing of the nested compositor's own container window ──────
#
# This floats and resizes the **nested compositor's own window** on the
# host — never the application window itself, which the nested compositor
# sizes internally from its own config. Doing this to a running
# application window is exactly the thing the GUI evidence rules forbid
# ("never by resizing or floating a running window: on a shared desktop
# that rearranges the layout"); the nested compositor's container is a
# disposable wrapper created and destroyed by this script, not a window
# anyone else's session could be using.

# ux_host_window_id_for_pid <pid> [timeout-seconds] -> prints niri window id
ux_host_window_id_for_pid() {
    local pid=$1
    local timeout=${2:-10}
    local id=
    local max_ticks=$(( timeout * 2 ))
    local tick=0
    while (( tick < max_ticks )); do
        id=$(niri msg --json windows 2>/dev/null |
            jq -r --argjson pid "$pid" '[.[] | select(.pid == $pid)] | .[0].id // empty')
        [[ -n "$id" ]] && { printf '%s\n' "$id"; return 0; }
        sleep 0.5
        tick=$(( tick + 1 ))
    done
    return 1
}

# ux_host_float_and_resize <host-window-id> <logical-width> <logical-height>
ux_host_float_and_resize() {
    local id=$1 width=$2 height=$3
    ux_log "host: floating and resizing window $id to ${width}x${height} (logical, host scale)"
    niri msg action move-window-to-floating --id "$id" ||
        ux_die "move-window-to-floating failed for host window $id"
    niri msg action set-window-width --id "$id" "$width" ||
        ux_die "set-window-width failed for host window $id"
    niri msg action set-window-height --id "$id" "$height" ||
        ux_die "set-window-height failed for host window $id"
}

# ── The app window, inside the nested compositor ────────────────────────────

# ux_nested_window_id <niri-socket> [timeout-seconds] -> prints the app's
# window id as the nested compositor sees it (there is exactly one window
# inside the nested compositor by construction: this rig launches nothing
# else into it).
ux_nested_window_id() {
    local sock=$1
    local timeout=${2:-10}
    local id=
    local max_ticks=$(( timeout * 2 ))
    local tick=0
    while (( tick < max_ticks )); do
        id=$(NIRI_SOCKET="$sock" niri msg --json windows 2>/dev/null | jq -r '.[0].id // empty')
        [[ -n "$id" ]] && { printf '%s\n' "$id"; return 0; }
        sleep 0.5
        tick=$(( tick + 1 ))
    done
    return 1
}

# ux_assert_focus <niri-socket> <expected-pid>
#
# GUI evidence rules: "immediately before every keystroke confirm that
# niri's focused window belongs to the instance under test" — `wtype`
# types into whatever holds focus, not into a window chosen by id. Inside
# this rig's own nested compositor there is only ever the one window this
# script launched, so this is a correctness assertion rather than a
# disambiguation between sessions, and it is cheap enough to run before
# every drive step.
ux_assert_focus() {
    local sock=$1 expected_pid=$2
    local focused_pid
    focused_pid=$(NIRI_SOCKET="$sock" niri msg -j focused-window 2>/dev/null | jq -r '.pid // empty')
    [[ "$focused_pid" == "$expected_pid" ]] ||
        ux_die "focus check failed: nested compositor's focused window pid is '${focused_pid:-<none>}', expected $expected_pid"
}

# ── Keyboard input ───────────────────────────────────────────────────────────

ux_wtype_key() {
    local display=$1 key=$2
    WAYLAND_DISPLAY="$display" wtype -k "$key" || ux_die "wtype -k $key failed"
}

ux_wtype_shift_key() {
    local display=$1 key=$2
    WAYLAND_DISPLAY="$display" wtype -M shift -k "$key" -m shift || ux_die "wtype shift+$key failed"
}

ux_wtype_ctrl_key() {
    local display=$1 key=$2
    WAYLAND_DISPLAY="$display" wtype -M ctrl -k "$key" -m ctrl || ux_die "wtype ctrl+$key failed"
}

ux_wtype_text() {
    local display=$1 text=$2
    WAYLAND_DISPLAY="$display" wtype -- "$text" || ux_die "wtype text '$text' failed"
}

# ── Capture, with proof that it landed ──────────────────────────────────────
#
# `niri msg action screenshot-window` returns *before* the PNG is written —
# the write happens on a later compositor frame. Three capture attempts
# were lost to this in task 035 before the fix: delete the target first
# (so a stale file from a previous capture cannot be misread as this
# frame's output), poll for the file to exist and for two consecutive size
# reads to agree, then treat it as landed. This is the whole fix; nothing
# upstream of it is trustworthy without it.

# ux_capture <niri-socket> <window-id> <target-path> -> prints the new
# file's sha256 to stdout.
ux_capture() {
    local sock=$1 win_id=$2 target=$3
    mkdir -p -- "$(dirname -- "$target")"
    rm -f "$target"
    # UX_CAPTURE_SETTLE: seconds for the application to finish repainting
    # before the screenshot is requested. 0 on wgpu; the software renderer
    # lags the input by whole frames and a capture taken early shows the
    # previous state (found live, task 039).
    [[ "${UX_CAPTURE_SETTLE:-0}" != 0 ]] && sleep "$UX_CAPTURE_SETTLE"
    NIRI_SOCKET="$sock" niri msg action screenshot-window --id "$win_id" \
        --path "$target" --write-to-disk true --show-pointer false >/dev/null ||
        ux_die "screenshot-window failed for window $win_id"

    local tick=0 size1 size2
    while (( tick < 50 )); do
        if [[ -s "$target" ]]; then
            size1=$(stat -c %s "$target")
            sleep 0.1
            size2=$(stat -c %s "$target" 2>/dev/null || echo -1)
            if [[ "$size1" == "$size2" ]]; then
                sha256sum "$target" | awk '{print $1}'
                return 0
            fi
        fi
        sleep 0.1
        tick=$(( tick + 1 ))
    done
    ux_die "capture never settled: $target"
}

# ux_capture_step <niri-socket> <window-id> <target-path> <prev-hash-var>
#   [expect-change: yes|no, default yes]
#
# Wraps ux_capture with the retry discipline the GUI evidence rules and
# task 037 both ask for: "a frame that did not change must read as a
# dropped key, never a frozen application" — retry the drive step, do not
# reinterpret the frame. <prev-hash-var> is a nameref: read as the hash to
# compare against, written with the new hash on return. Caller supplies a
# retry callback as the remaining arguments (a command to re-run the
# dropped input before recapturing); with none given, this only captures
# and reports, it does not retry. "Repeat limit" (GUI evidence rules): at
# most two retries (three attempts total) before this stops the run rather
# than looping a third time.
ux_capture_step() {
    local sock=$1 win_id=$2 target=$3 prev_hash_var=$4 expect_change=${5:-yes}
    shift 5 || shift $#
    local -n _ux_prev_hash="$prev_hash_var"
    local attempt=1 new_hash
    while :; do
        new_hash=$(ux_capture "$sock" "$win_id" "$target")
        if [[ "$expect_change" == no || "$new_hash" != "$_ux_prev_hash" ]]; then
            _ux_prev_hash=$new_hash
            return 0
        fi
        if (( attempt >= 3 )); then
            ux_die "capture $target identical to the previous frame after 3 attempts — stopping rather than repeating a 4th time (GUI evidence rules' repeat limit); treat as a dropped key, not a rendered state"
        fi
        # A late key and a lost key look identical in a single comparison, and
        # replaying a key that was merely late doubles it (task 037 round 2: a
        # Return replayed into a fresh dialog ran the test). So before calling
        # a key dropped, wait a longer interval and compare again: if the
        # frame moved on its own in the meantime the key was only late.
        ux_pause "${UX_STABLE_WAIT:-1.0}"
        local settled
        settled=$(ux_capture "$sock" "$win_id" "$target")
        if [[ "$settled" != "$_ux_prev_hash" ]]; then
            ux_log "capture $target changed after a ${UX_STABLE_WAIT:-1.0}s re-poll — the key was late, not lost; not replaying it"
            _ux_prev_hash=$settled
            return 0
        fi
        ux_log "capture $target identical to the previous frame, and still so after a ${UX_STABLE_WAIT:-1.0}s re-poll (attempt $attempt) — treating as a dropped key, retrying the drive step"
        if (( $# > 0 )); then
            "$@"
        fi
        sleep 0.3
        attempt=$(( attempt + 1 ))
    done
}

# ── Measured geometry ────────────────────────────────────────────────────────

# ux_measure_nested_output <niri-socket>
#   -> prints "LOGICAL_W LOGICAL_H SCALE PHYSICAL_W PHYSICAL_H"
# All five values are read from the compositor, never assumed from the
# resize commands issued — the nested output's own report of what it is
# actually driving.
ux_measure_nested_output() {
    local sock=$1
    # `niri msg --json outputs` returns an object keyed by output name, not
    # an array -- `to_entries[0].value` is the first (here: only) output's
    # own object, regardless of what its name happens to be.
    NIRI_SOCKET="$sock" niri msg --json outputs 2>/dev/null |
        jq -r 'to_entries[0].value |
            "\(.logical.width) \(.logical.height) \(.logical.scale) \(.modes[.current_mode].width) \(.modes[.current_mode].height)"'
}

# ux_png_physical_size <path> -> prints "WxH" as `file` itself reports it,
# an independent cross-check against ux_measure_nested_output rather than
# trusting one source for both halves of the same claim.
ux_png_physical_size() {
    local path=$1
    file -b "$path" | grep -oE '[0-9]+ x [0-9]+' | tr -d ' ' | tr 'x' 'x'
}

# ── Renderer identification ─────────────────────────────────────────────────

# ux_renderer_evidence <pid> -> prints which /dev/dri/render* nodes the
# process holds open (wgpu) or nothing (tiny-skia fallback — iced's
# software renderer, used on VMs, remote desktops or missing drivers).
# Task 037 §2: renderer has its own history of not being recorded, and a
# snora defect existed only on the software renderer, so this is not
# optional.
ux_renderer_evidence() {
    local pid=$1
    local nodes
    nodes=$(ls -la "/proc/$pid/fd" 2>/dev/null | grep -oE '/dev/dri/render[A-Za-z0-9]*' | sort -u)
    if [[ -n "$nodes" ]]; then
        printf 'wgpu (%s)\n' "$(printf '%s' "$nodes" | tr '\n' ' ' | sed 's/ $//')"
    else
        printf 'tiny-skia (no /dev/dri/render* fd open)\n'
    fi
}
