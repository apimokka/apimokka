#!/usr/bin/env bash
# RFC MK-056 layer L2 — run one configuration.
#
# Launches apimokka, resizes it to the configuration's target size, and
# (when a key sequence is supplied and xdotool can see the window) drives
# it through that sequence, asserting the window title changes as expected
# after opening a workspace. Screenshots at every step. Produces a
# machine-readable result line and leaves screenshots for human review.
#
# Usage:
#   bash scripts/ux/run-configuration.sh \
#     --name <config-name> \
#     --width <px> --height <px> \
#     --out-dir <dir> \
#     [--keys-file <path>]        # one xdotool key name per line, sent in order
#     [--expect-title-contains <substring>]  # asserted after the key sequence
#     [--input-method pointer|keyboard]      # recorded only, not enforced
#
# A key sequence is *discovered*, not guessed — see
# scripts/ux/discover-tab-order.sh. Without --keys-file, this script still
# performs and records the machine-assertable launch/resize checks and
# captures a rest-state screenshot; it does not attempt to fabricate a
# keyboard-reachability result it cannot support.

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

name=
width=
height=
out_dir=
keys_file=
expect_title_contains=
input_method=unspecified

while (( $# > 0 )); do
    case "$1" in
        --name) name=$2; shift 2 ;;
        --width) width=$2; shift 2 ;;
        --height) height=$2; shift 2 ;;
        --out-dir) out_dir=$2; shift 2 ;;
        --keys-file) keys_file=$2; shift 2 ;;
        --expect-title-contains) expect_title_contains=$2; shift 2 ;;
        --input-method) input_method=$2; shift 2 ;;
        *) ux_die "unknown argument: $1" ;;
    esac
done

[[ -n "$name" && -n "$width" && -n "$height" && -n "$out_dir" ]] ||
    ux_die "--name, --width, --height, and --out-dir are required"

mkdir -p -- "$out_dir"
# niri's screenshot-window --path requires an absolute path and silently
# no-ops (exit 0, no file, no error) on a relative one -- resolve it here
# so ux_screenshot's documented "<absolute-output-path>" contract holds
# regardless of what the caller passed.
out_dir=$(cd -- "$out_dir" && pwd -P) || ux_die "cannot resolve --out-dir to an absolute path"

binary="$repository_root/target/debug/apimokka"
[[ -x "$binary" ]] || ux_die "binary not found — build it first: cargo build -p apimokka"

result_launch=fail
result_resize=fail
result_responsive=fail
result_keyboard=not_exercised

pid=$(ux_launch_app "$binary")
cleanup() { ux_kill_app "$pid"; }
trap cleanup EXIT

if niri_id=$(ux_find_window_id "$pid" 10); then
    result_launch=pass
else
    ux_log "configuration $name: window never appeared — treating as launch failure"
fi

if [[ "$result_launch" == pass ]]; then
    ux_screenshot "$niri_id" "$out_dir/${name}-00-launch.png"

    if ux_float_and_resize "$niri_id" "$width" "$height"; then
        sleep 0.3
        if ux_process_alive "$pid"; then
            result_resize=pass
            result_responsive=pass
        else
            ux_log "configuration $name: process died after resize"
        fi
    else
        ux_log "configuration $name: resize action failed"
    fi
    ux_screenshot "$niri_id" "$out_dir/${name}-01-resized.png"

    if [[ -n "$keys_file" ]]; then
        if [[ ! -r "$keys_file" ]]; then
            ux_log "configuration $name: --keys-file $keys_file not readable, skipping keyboard drive"
        elif x11_id=$(ux_probe_xdotool "$niri_id"); then
            step=1
            while IFS= read -r key || [[ -n "$key" ]]; do
                [[ -n "$key" ]] || continue
                xdotool key --window "$x11_id" -- "$key" ||
                    ux_die "configuration $name: xdotool key '$key' failed at step $step"
                sleep 0.15
                ux_process_alive "$pid" ||
                    ux_die "configuration $name: process died during key sequence at step $step ('$key')"
                step=$(( step + 1 ))
            done < "$keys_file"
            sleep 0.3
            ux_screenshot "$niri_id" "$out_dir/${name}-02-after-keys.png"

            if [[ -n "$expect_title_contains" ]]; then
                title=$(ux_window_title "$niri_id")
                if [[ "$title" == *"$expect_title_contains"* ]]; then
                    result_keyboard=pass
                else
                    result_keyboard=fail
                    ux_log "configuration $name: expected title to contain '$expect_title_contains', got '${title:-<empty>}'"
                fi
            else
                result_keyboard="exercised_no_assertion"
            fi
        else
            ux_log "configuration $name: xdotool cannot see this window — keyboard drive not exercised"
            result_keyboard=infeasible
        fi
    fi
fi

printf 'CONFIG=%s\n' "$name"
printf 'INPUT_METHOD=%s\n' "$input_method"
printf 'WIDTHxHEIGHT=%sx%s\n' "$width" "$height"
printf 'LAUNCH=%s\n' "$result_launch"
printf 'RESIZE=%s\n' "$result_resize"
printf 'RESPONSIVE=%s\n' "$result_responsive"
printf 'KEYBOARD_REACHABILITY=%s\n' "$result_keyboard"
printf 'SCREENSHOTS_DIR=%s\n' "$out_dir"
