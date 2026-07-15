#!/usr/bin/env bash

set -u

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) ||
    {
        printf 'RFC checker self-test: cannot determine script directory\n' >&2
        exit 2
    }
checker="$script_dir/check-rfcs.sh"
repository_root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) ||
    {
        printf 'RFC checker self-test: cannot determine repository root\n' >&2
        exit 2
    }

for utility in cat cp dirname mkdir mktemp rm sed touch; do
    command -v "$utility" >/dev/null 2>&1 || {
        printf 'RFC checker self-test: required utility not found: %s\n' "$utility" >&2
        exit 2
    }
done

[[ -x "$checker" ]] || {
    printf 'RFC checker self-test: checker is not executable: %s\n' "$checker" >&2
    exit 2
}

tmp_parent=${TMPDIR:-"$repository_root/target/tmp"}
mkdir -p -- "$tmp_parent" || exit 2
sentinel="$tmp_parent/check-rfcs-self-test-sentinel.$$"
[[ ! -e "$sentinel" ]] || {
    printf 'RFC checker self-test: sentinel already exists\n' >&2
    exit 2
}
touch -- "$sentinel" || exit 2
temp_dir=$(mktemp -d "$tmp_parent/check-rfcs-self-test.XXXXXX") || exit 2

cleanup_temp() {
    if [[ -z "${temp_dir:-}" ||
          "$temp_dir" != "$tmp_parent"/check-rfcs-self-test.* ||
          ! -d "$temp_dir" ]]; then
        printf 'RFC checker self-test: refusing unsafe temporary cleanup: %s\n' "${temp_dir:-<empty>}" >&2
        return 2
    fi
    rm -rf -- "$temp_dir" || {
        printf 'RFC checker self-test: temporary cleanup failed: %s\n' "$temp_dir" >&2
        return 2
    }
}

cleanup_all() {
    local status=0
    cleanup_temp || status=$?
    rm -f -- "$sentinel"
    return "$status"
}
trap cleanup_all EXIT

failures=0
checks=0

fail() {
    printf 'FAIL %s\n' "$1" >&2
    (( failures += 1 ))
}

expect_result() {
    local label=$1
    local expected_status=$2
    local expected_text=$3
    shift 3
    local output status

    set +e
    output=$("$@" 2>&1)
    status=$?
    set -e
    (( checks += 1 ))

    if (( status != expected_status )); then
        fail "$label: expected exit $expected_status, got $status"
        printf '%s\n' "$output" >&2
        return
    fi
    if [[ "$output" != *"$expected_text"* ]]; then
        fail "$label: expected output containing '$expected_text'"
        printf '%s\n' "$output" >&2
    fi
}

expect_exact() {
    local label=$1
    local expected_status=$2
    local expected_output=$3
    shift 3
    local output status

    set +e
    output=$("$@" 2>&1)
    status=$?
    set -e
    (( checks += 1 ))

    if (( status != expected_status )); then
        fail "$label: expected exit $expected_status, got $status"
        printf '%s\n' "$output" >&2
        return
    fi
    if [[ "$output" != "$expected_output" ]]; then
        fail "$label: complete output differed"
        printf 'EXPECTED:\n%s\nOBSERVED:\n%s\n' "$expected_output" "$output" >&2
    fi
}

run_checker_from_root() {
    local fixture_root=$1
    (
        cd -- "$fixture_root" || exit 2
        "$checker" "$fixture_root"
    )
}

fixture="$temp_dir/valid"
mkdir -p -- "$fixture/rfcs/proposed" "$fixture/rfcs/done" "$fixture/rfcs/archive" "$fixture/docs"

cat > "$fixture/rfcs/proposed/MK-001-proposal.md" <<'EOF'
# RFC MK-001 — Proposal

**Status.** Proposed

## Summary

An open proposal.
EOF

cat > "$fixture/rfcs/done/MK-002-implemented.md" <<'EOF'
# RFC MK-002 — Implemented

**Status.** Implemented (v1.0.0)

## Summary

Implemented.
EOF

cat > "$fixture/rfcs/done/MK-003-implemented.md" <<'EOF'
# RFC MK-003 — Another implementation

**Status.** Implemented (version unverified)

## Summary

Implemented with an unverified historical version.
EOF

cat > "$fixture/rfcs/done/MK-007-unreleased.md" <<'EOF'
# RFC MK-007 — Unreleased implementation

**Status.** Implemented (Unreleased)

## Summary

Implemented but not assigned to a release.
EOF

cat > "$fixture/rfcs/archive/MK-004-single.md" <<'EOF'
# RFC MK-004 — Single supersession

**Status.** Superseded by RFC MK-002 — replaced by the implemented design

## Summary

Superseded.
EOF

cat > "$fixture/rfcs/archive/MK-005-series.md" <<'EOF'
# RFC MK-005 — Series supersession

**Status.** Superseded by RFCs MK-002–MK-003 — replaced by the redesign series

## Summary

Superseded by a series.
EOF

cat > "$fixture/rfcs/archive/MK-006-withdrawn.md" <<'EOF'
# RFC MK-006 — Withdrawn

**Status.** Withdrawn — no longer required

## Summary

Withdrawn.
EOF

cat > "$fixture/rfcs/README.md" <<'EOF'
# RFC index

## Proposed

| RFC | Title |
|---|---|
| [MK-001](./proposed/MK-001-proposal.md) | Proposal |

## Implemented

| RFC | Title |
|---|---|
| [MK-002](./done/MK-002-implemented.md) | Implemented |
| [MK-003](./done/MK-003-implemented.md) | Another implementation |
| [MK-007](./done/MK-007-unreleased.md) | Unreleased implementation |

## Archive

| RFC | Title |
|---|---|
| [MK-004](./archive/MK-004-single.md) | Single supersession |
| [MK-005](./archive/MK-005-series.md) | Series supersession |
| [MK-006](./archive/MK-006-withdrawn.md) | Withdrawn |
EOF

expect_result "valid fixture including supersession and Unreleased forms" 0 "RFC integrity: 0 error(s)" "$checker" "$fixture"

case_root="$temp_dir/missing-index"
cp -R -- "$fixture" "$case_root"
sed -i '/\[MK-001\]/d' "$case_root/rfcs/README.md"
expect_result "missing index entry" 1 "missing index entry for MK-001" "$checker" "$case_root"

case_root="$temp_dir/duplicate-id"
cp -R -- "$fixture" "$case_root"
cat > "$case_root/rfcs/archive/MK-001-duplicate.md" <<'EOF'
# RFC MK-001 — Duplicate

**Status.** Withdrawn — duplicate fixture

## Summary

Duplicate.
EOF
expect_result "duplicate identifier" 1 "duplicate RFC identifier MK-001" "$checker" "$case_root"

case_root="$temp_dir/status-mismatch"
cp -R -- "$fixture" "$case_root"
sed -i 's/\*\*Status\.\*\* Proposed/**Status.** Implemented (v1.0.0)/' "$case_root/rfcs/proposed/MK-001-proposal.md"
expect_result "folder and status mismatch" 1 "proposed RFC status must be 'Proposed'" "$checker" "$case_root"

case_root="$temp_dir/heading-mismatch"
cp -R -- "$fixture" "$case_root"
sed -i '1s/MK-001/MK-009/' "$case_root/rfcs/proposed/MK-001-proposal.md"
expect_result "heading mismatch" 1 "heading identifier MK-009 does not match MK-001" "$checker" "$case_root"

case_root="$temp_dir/malformed-supersession"
cp -R -- "$fixture" "$case_root"
sed -i 's/Superseded by RFC MK-002 — replaced by the implemented design/Superseded by RFC MK-02/' "$case_root/rfcs/archive/MK-004-single.md"
expect_result "malformed supersession" 1 "archive status must be" "$checker" "$case_root"

case_root="$temp_dir/reversed-series"
cp -R -- "$fixture" "$case_root"
sed -i 's/MK-002–MK-003/MK-003–MK-002/' "$case_root/rfcs/archive/MK-005-series.md"
expect_result "reversed supersession series" 1 "supersession range must be ascending" "$checker" "$case_root"

case_root="$temp_dir/empty-series"
cp -R -- "$fixture" "$case_root"
sed -i 's/MK-002–MK-003 — replaced by the redesign series/MK-002– — missing range end/' "$case_root/rfcs/archive/MK-005-series.md"
expect_result "empty supersession series endpoint" 1 "archive status must be" "$checker" "$case_root"

case_root="$temp_dir/empty-reason"
cp -R -- "$fixture" "$case_root"
sed -i 's/Withdrawn — no longer required/Withdrawn — /' "$case_root/rfcs/archive/MK-006-withdrawn.md"
expect_result "empty withdrawal reason" 1 "archive status must be" "$checker" "$case_root"

case_root="$temp_dir/missing-target"
cp -R -- "$fixture" "$case_root"
sed -i 's/RFC MK-002 — replaced/RFC MK-099 — replaced/' "$case_root/rfcs/archive/MK-004-single.md"
expect_result "missing supersession target" 1 "supersession target MK-099 does not exist" "$checker" "$case_root"

case_root="$temp_dir/wrong-section"
cp -R -- "$fixture" "$case_root"
sed -i '/\[MK-001\]/d; /\[MK-002\]/i\| [MK-001](./proposed/MK-001-proposal.md) | Proposal |' "$case_root/rfcs/README.md"
expect_result "wrong lifecycle section" 1 "MK-001 is in the wrong lifecycle section" "$checker" "$case_root"

case_root="$temp_dir/cross-section-duplicate"
cp -R -- "$fixture" "$case_root"
sed -i '/\[MK-006\]/a\| [MK-001](./proposed/MK-001-proposal.md) | Duplicate |' "$case_root/rfcs/README.md"
expect_result "cross-section duplicate" 1 "MK-001 appears 2 times" "$checker" "$case_root"

case_root="$temp_dir/broken-index-link"
cp -R -- "$fixture" "$case_root"
sed -i 's#./done/MK-002-implemented.md#./done/missing.md#' "$case_root/rfcs/README.md"
expect_result "incorrect index link" 1 "MK-002 link must be ./done/MK-002-implemented.md" "$checker" "$case_root"

case_root="$temp_dir/broken-markdown-link"
cp -R -- "$fixture" "$case_root"
printf '\n[missing](missing.md)\n' >> "$case_root/rfcs/proposed/MK-001-proposal.md"
expect_result "broken local Markdown link" 1 "unresolved local Markdown link: missing.md" "$checker" "$case_root"

case_root="$temp_dir/deterministic-multi-error"
cp -R -- "$fixture" "$case_root"
sed -i '/\[MK-001\]/d' "$case_root/rfcs/README.md"
sed -i '1s/MK-001/MK-009/' "$case_root/rfcs/proposed/MK-001-proposal.md"
printf '\n[missing](missing.md)\n' >> "$case_root/rfcs/proposed/MK-001-proposal.md"
expected_output='ERROR rfcs/README.md: missing index entry for MK-001
ERROR rfcs/proposed/MK-001-proposal.md: heading identifier MK-009 does not match MK-001
ERROR rfcs/proposed/MK-001-proposal.md: unresolved local Markdown link: missing.md
RFC integrity: 3 error(s)'
expect_exact "deterministic complete multi-error output" 1 "$expected_output" "$checker" "$case_root"

case_root="$temp_dir/empty-proposed"
cp -R -- "$fixture" "$case_root"
rm -f -- "$case_root/rfcs/proposed/MK-001-proposal.md"
sed -i '/\[MK-001\]/d; /## Proposed/a\No RFCs are currently proposed.' "$case_root/rfcs/README.md"
expect_result "valid empty Proposed state" 0 "RFC integrity: 0 error(s)" "$checker" "$case_root"

case_root="$temp_dir/stale-empty-proposed"
cp -R -- "$fixture" "$case_root"
sed -i '/## Proposed/a\No RFCs are currently proposed.' "$case_root/rfcs/README.md"
expect_result "stale Proposed empty state" 1 "empty-state sentence is present while proposed RFCs exist" "$checker" "$case_root"

case_root="$temp_dir/injection-paths"
cp -R -- "$fixture" "$case_root"
touch -- "$case_root/docs/space name.md" "$case_root/docs/-leading.md" "$case_root/docs/[glob]*.md" "$case_root/docs/semi;colon.md" "$case_root/docs/"'`tick`.md' "$case_root/docs/"'$(touch marker-was-executed).md'
cat >> "$case_root/rfcs/proposed/MK-001-proposal.md" <<'EOF'

[space](<../../docs/space name.md>)
[leading](<../../docs/-leading.md>)
[glob](<../../docs/[glob]*.md>)
[semicolon](<../../docs/semi;colon.md>)
[backticks](<../../docs/`tick`.md>)
[dollar](<../../docs/$(touch marker-was-executed).md>)

```markdown
[ignored fenced link](missing-inside-fence.md)
```

[checked after fence](<../../docs/space name.md>)
EOF
expect_result "injection-shaped paths and fenced link" 0 "RFC integrity: 0 error(s)" run_checker_from_root "$case_root"
[[ ! -e "$case_root/marker-was-executed" ]] ||
    fail "injection-shaped path executed command substitution"

case_root="$temp_dir/indented-fence"
cp -R -- "$fixture" "$case_root"
cat >> "$case_root/rfcs/proposed/MK-001-proposal.md" <<'EOF'

   ```markdown
[ignored in indented fence](missing-inside-indented-fence.md)
   ```
EOF
expect_result "indented triple-backtick fence" 0 "RFC integrity: 0 error(s)" "$checker" "$case_root"

case_root="$temp_dir/suffixed-handoff"
cp -R -- "$fixture" "$case_root"
mkdir -p -- "$case_root/rfcs/handoffs/MK-002-implementation-notes"
expect_result "suffixed handoff directory" 0 "RFC integrity: 0 error(s)" "$checker" "$case_root"

case_root="$temp_dir/post-fence-link"
cp -R -- "$fixture" "$case_root"
cat >> "$case_root/rfcs/proposed/MK-001-proposal.md" <<'EOF'

```markdown
[ignored](missing-inside-fence.md)
```

[must be checked](missing-after-fence.md)
EOF
expect_result "post-fence link is checked" 1 "unresolved local Markdown link: missing-after-fence.md" "$checker" "$case_root"

expect_result "invalid invocation is operational failure" 2 "Usage:" "$checker" "$fixture" extra
expect_result "missing root is operational failure" 2 "repository root is missing or unreadable" "$checker" "$temp_dir/does-not-exist"

owned_temp_dir=$temp_dir
temp_dir="$tmp_parent/not-owned-by-self-test"
expect_result "unsafe cleanup guard is operational failure" 2 "refusing unsafe temporary cleanup" cleanup_temp
temp_dir=$owned_temp_dir

cleanup_temp
temp_dir=
(( checks += 1 ))
if [[ ! -e "$sentinel" ]]; then
    fail "cleanup removed its sibling sentinel"
fi
rm -f -- "$sentinel"
trap - EXIT

if (( failures > 0 )); then
    printf 'RFC checker self-test: %d/%d check(s) failed\n' "$failures" "$checks" >&2
    exit 1
fi

printf 'RFC checker self-test: %d checks passed\n' "$checks"
