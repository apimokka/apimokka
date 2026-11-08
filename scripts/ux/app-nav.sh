#!/usr/bin/env bash
# RFC MK-056 layer L2 — apimokka-specific keyboard navigation, built on
# scripts/ux/lib.sh's generic rig primitives.
#
# Every function here performs one state-changing action via wtype, then
# verifies it landed with a checkpoint capture (lib.sh's ux_capture_step,
# retrying the same action on a dropped key rather than reinterpreting the
# frame) before returning — "assert, then continue" (GUI evidence rules).
# Checkpoint frames go to <out-dir>/_checkpoints/, not the run's own
# deliverable evidence; callers capture deliverable frames separately,
# after the state they need is already confirmed live.
#
# Sequences below are not guessed: each was driven interactively and
# confirmed against the running application before being written here
# (tasks 035/036 and this task's own session).

set -uo pipefail

_ux_nav_checkpoint_dir() {
    printf '%s/_checkpoints\n' "$1"
}

# ux_app_select_mode <display> <sock> <winid> <out-dir> <prev-hash-var> <guided|expert>
#
# At the mode picker (first launch, nothing else focusable yet):
# `Region::order_for` returns a single-item list (just `Region::ModePicker`)
# until a mode is chosen, so a SECOND Tab does not move between the two
# cards — `focus_adjacent_region` re-enters that same one-item region,
# which resets the cursor back to its own default (index 0 = "Guided") on
# every press. Found live this session: a run requesting Expert
# consistently landed in Guided instead (confirmed by Settings' own
# "Guidance" pill, not by this function's own log line, which always
# echoed the requested target regardless of what actually got selected).
# Moving to "Expert" (index 1) needs ArrowDown, matching this app's own
# `mode_picker_arrow_keys_move_between_the_two_options` test — not a
# second Tab.
ux_app_select_mode() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 mode=$6
    local ckpt; ckpt="$(_ux_nav_checkpoint_dir "$out_dir")/mode-selected.png"

    local do_select
    do_select() {
        ux_wtype_key "$display" Tab
        ux_pause 0.2
        if [[ "$mode" == expert ]]; then
            ux_wtype_key "$display" Down
            ux_pause 0.2
        fi
        ux_wtype_key "$display" Return
        ux_pause 0.4
    }
    do_select
    ux_capture_step "$sock" "$winid" "$ckpt" "$prev_hash_var" yes do_select
    ux_log "mode selected: $mode"
}

# ux_app_open_first_workspace <display> <sock> <winid> <out-dir> <prev-hash-var>
#
# Welcome: one Tab reaches "Open workspace", Enter opens the Dashboard.
# Dashboard: nothing is pre-focused; first Tab reaches "Create new
# workspace", second Tab reaches the search field, third Tab reaches the
# first (pinned) workspace card, Enter opens it onto Routes. (Confirmed by
# single-step live debugging this session — the search field is NOT the
# first Dashboard Tab stop.)
ux_app_open_first_workspace() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5
    local ckpt; ckpt="$(_ux_nav_checkpoint_dir "$out_dir")/workspace-opened.png"

    local do_open
    do_open() {
        ux_wtype_key "$display" Tab
        ux_pause 0.3
        ux_wtype_key "$display" Return
        ux_pause 0.4
        ux_wtype_key "$display" Tab
        ux_pause 0.3
        ux_wtype_key "$display" Tab
        ux_pause 0.3
        ux_wtype_key "$display" Tab
        ux_pause 0.3
        ux_wtype_key "$display" Return
        ux_pause 0.5
    }
    do_open
    ux_capture_step "$sock" "$winid" "$ckpt" "$prev_hash_var" yes do_open
    ux_log "first workspace opened, landed on Routes"
}

# ux_app_palette_run <display> <sock> <winid> <out-dir> <prev-hash-var>
#   <search-text> <expect-change: yes|no>
#
# Ctrl+K opens the command palette, types a search string, Down selects
# the (only matching) row, Return runs it, Escape closes the palette if it
# is still open (most commands leave it open; Escape on an already-closed
# palette is a documented no-op for EscapePressed's own overlay gate,
# never a drive error).
#
# <skip-escape: yes|no, default no>: `Message::OpenSaveDiffDrawer` (and
# `OpenValidationDrawer`) close the palette AND raise a new overlay (the
# drawer) in the same update — EscapePressed then reads the DRAWER as the
# topmost overlay and closes *that* (`self.drawer = None`), silently
# undoing the very command just run (found live this session: the drawer
# command's own checkpoint capture came back identical to the frame
# before it, net-zero, not a dropped key). Callers for such commands must
# pass skip-escape=yes.
ux_app_palette_run() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5
    local search=$6 expect_change=${7:-yes} skip_escape=${8:-no}
    local dir; dir="$(_ux_nav_checkpoint_dir "$out_dir")"
    local tag; tag="$(printf '%s' "$search" | tr -c 'a-zA-Z0-9' '-')"

    # Each step is verified on its own and retried on its own. Replaying the
    # whole sequence on a stalled frame is not safe: Ctrl+K toggles the
    # palette, so a replay after a dropped Return would CLOSE it (found
    # live, task 037 round 2: a Test Rule run captured the palette still
    # open because Return was dropped — the old end-of-sequence check saw a
    # changed frame and passed). Every step here ends in a frame that must
    # differ from the one before it, and its retry is idempotent.
    local do_open do_down do_return do_escape
    do_open() { ux_wtype_ctrl_key "$display" k; ux_pause 0.3; }
    do_down() { ux_wtype_key "$display" Down; ux_pause 0.3; }
    do_return() { ux_wtype_key "$display" Return; ux_pause 0.5; }
    do_escape() { ux_wtype_key "$display" Escape; ux_pause 0.3; }

    do_open
    ux_capture_step "$sock" "$winid" "$dir/palette-$tag-1-open.png" "$prev_hash_var" yes do_open
    ux_wtype_text "$display" "$search"
    ux_pause 0.3
    ux_capture_step "$sock" "$winid" "$dir/palette-$tag-2-typed.png" "$prev_hash_var" yes
    do_down
    ux_capture_step "$sock" "$winid" "$dir/palette-$tag-3-selected.png" "$prev_hash_var" yes do_down
    do_return
    ux_capture_step "$sock" "$winid" "$dir/palette-$tag-4-ran.png" "$prev_hash_var" "$expect_change" do_return
    if [[ "$skip_escape" != yes ]]; then
        do_escape
        ux_capture_step "$sock" "$winid" "$dir/palette-$tag-5-closed.png" "$prev_hash_var" yes do_escape
    fi
    ux_log "palette command run: $search"
}

# _ux_palette_search <cmd> <locale>
#
# `palette_commands::filtered_indices` filters against the row's own
# *translated* label (`app.t(...)`), not a fixed key — found live this
# session: once locale is ja, an English search string like "go to
# settings" matches zero rows, Enter on an empty filtered list is a
# documented no-op (`palette_keyboard.rs`'s own test), and the resulting
# frame is identical to before, indistinguishable from a dropped key until
# traced this far. Every caller that may run after a locale switch must
# route through this lookup rather than hardcoding one language's text.
# Substrings were chosen checked against crates/i18n/src/ja.rs for
# collisions with other rows (plain "設定" also matches "設定リロード" /
# "Reload config" — "設定へ" does not).
_ux_palette_search() {
    local cmd=$1 locale=$2
    case "$locale" in
        ja)
            case "$cmd" in
                settings) echo "設定へ" ;;
                routes) echo "ルートへ" ;;
                save_diff) echo "保存差分" ;;
                toggle_theme) echo "テーマ" ;;
                test_rule) echo "ルールをテスト" ;;
                *) ux_die "_ux_palette_search: no ja text known for '$cmd'" ;;
            esac ;;
        en)
            case "$cmd" in
                settings) echo "go to settings" ;;
                routes) echo "go to routes" ;;
                save_diff) echo "save diff" ;;
                toggle_theme) echo "toggle theme" ;;
                test_rule) echo "test current rule" ;;
                *) ux_die "_ux_palette_search: no en text known for '$cmd'" ;;
            esac ;;
        *) ux_die "_ux_palette_search: unknown locale '$locale'" ;;
    esac
}

# Theme cycle order, matching ThemeChoice::toggle (app.rs): Light -> Dark
# -> High Contrast Light -> High Contrast Dark -> Light.
_ux_theme_index() {
    case "$1" in
        light) echo 0 ;;
        dark) echo 1 ;;
        hclight) echo 2 ;;
        hcdark) echo 3 ;;
        *) ux_die "unknown theme target: $1" ;;
    esac
}

# ux_app_set_theme <display> <sock> <winid> <out-dir> <prev-hash-var> <target> [locale=en]
#
# Assumes the session starts at Light (every fresh launch's own default)
# and has not been toggled since; toggles forward via the palette's own
# "Toggle theme" row the number of steps the cycle needs, each toggle
# individually verified rather than assuming N sends landed N changes.
ux_app_set_theme() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 target=$6 locale=${7:-en}
    local steps; steps=$(_ux_theme_index "$target")
    local search; search=$(_ux_palette_search toggle_theme "$locale")
    local i
    for (( i = 0; i < steps; i++ )); do
        ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "$search" yes
    done
    ux_log "theme set to: $target ($steps toggle(s) from Light)"
}

# ux_app_set_locale <display> <sock> <winid> <out-dir> <prev-hash-var> <en|ja>
#
# Locale has exactly two values (Message::ToggleLocale is a flip); assumes
# the session starts at En (every fresh launch's own default).
ux_app_set_locale() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 target=$6
    if [[ "$target" == ja ]]; then
        ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "change locale" yes
    fi
    ux_log "locale set to: $target"
}

# ux_app_open_settings <display> <sock> <winid> <out-dir> <prev-hash-var> [locale=en]
ux_app_open_settings() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 locale=${6:-en}
    local search; search=$(_ux_palette_search settings "$locale")
    ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "$search" yes
}

# ux_app_open_routes <display> <sock> <winid> <out-dir> <prev-hash-var> [locale=en]
ux_app_open_routes() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 locale=${6:-en}
    local search; search=$(_ux_palette_search routes "$locale")
    ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "$search" yes
}

# ux_app_open_save_diff_drawer <display> <sock> <winid> <out-dir> <prev-hash-var> [locale=en]
#
# The mock's own default workspace carries a dirty rule-set file, so the
# save-diff drawer always has content — the one dialog-equivalent surface
# this rig can reach the same way in every configuration.
#
# skip-escape=yes: `OpenSaveDiffDrawer` closes the palette itself and
# opens the drawer in the same update, so the generic trailing Escape
# would close the drawer right back (see ux_app_palette_run's own note).
ux_app_open_save_diff_drawer() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 locale=${6:-en}
    local search; search=$(_ux_palette_search save_diff "$locale")
    ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "$search" yes yes
}

# ux_app_show_region_cursor <display> <sock> <winid> <out-dir> <prev-hash-var> [soft=no]
#
# One Tab from wherever focus currently is, so the next capture shows a
# visible region-cursor ring — combination 2's own requirement ("the
# keyboard cursor visible on at least one control per screen it covers").
#
# soft=yes: found live this session — `theme::card_selected_style`'s own
# high-contrast branch (crates/app/src/theme.rs) sets only a low-alpha
# (10-18%) background wash and omits the solid 2px border the non-HC
# branch draws, so under an HC theme a Tab can genuinely move focus
# without producing any visible frame difference at all (confirmed by
# reading the focus ring move cleanly under Dark, then vanish on the next
# toggle into HC Light, with no script change between the two). That is
# the finding itself, not a dropped key — `expect_change=yes`'s retry-
# then-die would misreport a real rendering gap as rig flakiness. Callers
# for an HC theme should pass soft=yes: the Tab is still sent once, the
# frame is captured either way, and a same-hash result is logged as a
# finding rather than retried to death.
ux_app_show_region_cursor() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 soft=${6:-no}
    local ckpt; ckpt="$(_ux_nav_checkpoint_dir "$out_dir")/cursor-shown.png"
    local do_tab
    do_tab() { ux_wtype_key "$display" Tab; ux_pause 0.3; }
    if [[ "$soft" == yes ]]; then
        do_tab
        local new_hash; new_hash=$(ux_capture "$sock" "$winid" "$ckpt")
        if [[ "$new_hash" == "${!prev_hash_var}" ]]; then
            ux_log "NOTE: region-cursor capture identical to the previous frame after Tab under an HC theme — not treated as a dropped key (see theme::card_selected_style's HC branch, scripts/ux/app-nav.sh's own comment on this function)"
        fi
        printf -v "$prev_hash_var" '%s' "$new_hash"
    else
        do_tab
        ux_capture_step "$sock" "$winid" "$ckpt" "$prev_hash_var" yes do_tab
    fi
}

# ux_app_select_first_rule <display> <sock> <winid> <out-dir> <prev-hash-var> [tabs=5] [rule=0]
#   rule: 0-based index of the rule within the first rule set (one extra Down each)
#
# Opens the rule editor: from a freshly opened workspace (nothing on the
# workspace screen has taken a Tab yet — `Region::order_for` then starts at
# the top bar), five Tabs reach the sidebar, Right expands and selects the
# first rule set, Down moves the cursor to its first rule, Return selects it
# and the centre panel becomes the editor (WHEN and RESPOND side by side).
# Not retried: a dropped key mid-sequence would be doubled by a replay, so
# an unchanged frame stops the run instead (call it first, before anything
# else has moved the region cursor, and read the frame it leaves).
ux_app_select_first_rule() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 tabs=${6:-5} rule=${7:-0}
    local ckpt; ckpt="$(_ux_nav_checkpoint_dir "$out_dir")/rule-selected.png"
    local i k
    for (( i = 0; i < tabs; i++ )); do
        ux_wtype_key "$display" Tab
        ux_pause 0.4
    done
    for k in Right Down $(for (( i = 0; i < rule; i++ )); do printf 'Down '; done) Return; do
        ux_wtype_key "$display" "$k"
        ux_pause 0.4
    done
    ux_capture_step "$sock" "$winid" "$ckpt" "$prev_hash_var" yes
    ux_log "first rule selected, editor open"
}

# ux_app_open_test_rule <display> <sock> <winid> <out-dir> <prev-hash-var> [locale=en]
#
# The Test Rule dialog, via its palette row (needs a rule selected).
# `TestRuleOpen` closes the palette itself and raises the dialog, so the
# generic trailing Escape would dismiss it again (same trap as the drawer).
ux_app_open_test_rule() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5 locale=${6:-en}
    local search; search=$(_ux_palette_search test_rule "$locale")
    ux_app_palette_run "$display" "$sock" "$winid" "$out_dir" "$prev_hash_var" "$search" yes yes
}

# ux_app_close_overlay <display> <sock> <winid> <out-dir> <prev-hash-var>
#
# One Escape: dismisses the topmost overlay (Test Rule, drawer, ...).
ux_app_close_overlay() {
    local display=$1 sock=$2 winid=$3 out_dir=$4 prev_hash_var=$5
    local ckpt; ckpt="$(_ux_nav_checkpoint_dir "$out_dir")/overlay-closed.png"
    local do_close
    do_close() { ux_wtype_key "$display" Escape; ux_pause 0.4; }
    do_close
    ux_capture_step "$sock" "$winid" "$ckpt" "$prev_hash_var" yes do_close
}
