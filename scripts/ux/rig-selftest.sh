#!/usr/bin/env bash
# Self-test for the safety properties of scripts/ux/lib.sh that need no
# display: the focus guard cannot be skipped, the repeat limit cannot be
# reset by deleting a directory, the source check notices a new file or
# environment access, and the nested window is never chosen without a pid.
# Run: bash scripts/ux/rig-selftest.sh
set -uo pipefail
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

fail=0
# expect <ok|fail> <label> <command...>  — runs in a subshell so a ux_die
# (which exits) is observed as a status, not the end of this script.
expect() {
    local want=$1 label=$2; shift 2
    ( "$@" ) >"$work/out" 2>&1
    local got=$?
    if [[ "$want" == ok && "$got" == 0 ]] || [[ "$want" == fail && "$got" != 0 ]]; then
        printf 'ok    %s\n' "$label"
    else
        printf 'WRONG %s: wanted %s, got exit %s\n%s\n' "$label" "$want" "$got" "$(cat "$work/out")"
        fail=1
    fi
}
says() { # says <substring> <label> — the last run's output must contain it
    if grep -qF -- "$1" "$work/out"; then printf 'ok    %s\n' "$2"; else printf 'WRONG %s: output lacked %q\n%s\n' "$2" "$1" "$(cat "$work/out")"; fail=1; fi
}

lib() { source "$script_dir/lib.sh" 2>/dev/null; "$@"; }

# 1. A key cannot be sent before an instance is bound (wtype types into
#    whatever holds focus).
expect fail "ux_wtype_key refuses with no instance bound" lib ux_wtype_key wayland-9 Tab
says "no instance bound" "…and says why"
expect fail "ux_wtype_text refuses with no instance bound" lib ux_wtype_text wayland-9 hello
expect fail "ux_wtype_ctrl_key refuses with no instance bound" lib ux_wtype_ctrl_key wayland-9 k

# 2. The nested window is never "the first window found".
expect fail "ux_nested_window_id refuses without a pid" lib ux_nested_window_id /nonexistent.sock 1

# 3. The repeat limit: 1st silent, 2nd a notice, 3rd refused, and approval
#    is recorded. The ledger is a file, so deleting an evidence directory
#    does not reset it.
ledger="$work/ledger"
expect ok "1st run of a capture set passes" lib ux_note_capture_set "$ledger" "dir|set"
expect ok "2nd run passes" lib ux_note_capture_set "$ledger" "dir|set"
says "SECOND run" "…with a notice to say so in the review request"
expect fail "3rd run is refused" lib ux_note_capture_set "$ledger" "dir|set"
says "ask the architect" "…naming the architect"
expect ok "a different capture set is unaffected" lib ux_note_capture_set "$ledger" "dir|other"
expect ok "3rd run passes when approved" env UX_RERUN_APPROVED="architect, test" bash -c "source '$script_dir/lib.sh' 2>/dev/null; ux_note_capture_set '$ledger' 'dir|set'"
says "approved: architect, test" "…and the approval is logged"

# 4. The source check passes a clean tree and fails on a new file read.
tree="$work/tree"
mkdir -p "$tree/crates/app/src/app" "$tree/crates/model/src" "$tree/crates/i18n/src"
echo 'fn ok() {}' > "$tree/crates/app/src/lib.rs"
expect ok "source check passes a tree with no file access" lib ux_assert_app_reads_no_files "$tree"
echo 'fn bad() { let _ = std::fs::read_to_string("x"); }' > "$tree/crates/model/src/bad.rs"
expect fail "source check fails when production code reads a file" lib ux_assert_app_reads_no_files "$tree"
echo 'fn bad() { let _ = std::env::var("HOME"); }' > "$tree/crates/model/src/bad.rs"
expect fail "…or reads the environment" lib ux_assert_app_reads_no_files "$tree"

# 5. Isolation: an application-owned path under the XDG dirs is an error;
#    third-party entries are only reported.
iso="$work/iso"
expect ok "isolation env creates four directories" lib ux_isolation_env "$iso"
mkdir -p "$iso/xdg/cache/mesa_shader_cache"
expect ok "a third-party cache entry is reported, not failed" lib ux_assert_isolation "$iso"
mkdir -p "$iso/xdg/config/apimokka"
expect fail "an application-owned path fails the run" lib ux_assert_isolation "$iso"

if (( fail )); then printf 'rig self-test FAILED\n'; exit 1; fi
printf 'rig self-test passed\n'
