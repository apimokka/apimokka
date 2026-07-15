#!/usr/bin/env bash

set -uo pipefail

LC_ALL=C
export LC_ALL

usage() {
    printf 'Usage: %s [repository-root]\n' "${0##*/}" >&2
}

die_operational() {
    printf 'RFC integrity: operational error: %s\n' "$1" >&2
    exit 2
}

if (( $# > 1 )); then
    usage
    exit 2
fi

(( BASH_VERSINFO[0] >= 4 )) ||
    die_operational "Bash 4 or newer is required"

for utility in awk basename dirname find sort; do
    command -v "$utility" >/dev/null 2>&1 ||
        die_operational "required utility not found: $utility"
done

if (( $# == 1 )); then
    root=${1%/}
else
    script_dir=$(dirname -- "${BASH_SOURCE[0]}") ||
        die_operational "cannot determine script directory"
    root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) ||
        die_operational "cannot determine repository root"
fi

[[ -n "$root" && -d "$root" && -r "$root" ]] ||
    die_operational "repository root is missing or unreadable: $root"

for required in rfcs/README.md rfcs/proposed rfcs/done rfcs/archive; do
    [[ -e "$root/$required" && -r "$root/$required" ]] ||
        die_operational "required path is missing or unreadable: $required"
done

declare -a errors=()
declare -a rfc_files=()
declare -a pending_references=()
declare -A id_path=()
declare -A id_state=()
declare -A index_count=()
declare -A index_state=()
declare -A index_link=()

add_error() {
    errors+=("$1")
}

relative_path() {
    printf '%s' "${1#"$root"/}"
}

for state in proposed done archive; do
    while IFS= read -r -d '' file; do
        rfc_files+=("$file")
        path=$(relative_path "$file")
        base=$(basename -- "$file")

        if [[ "$base" =~ ^MK-([0-9]{3})-[a-z0-9]+(-[a-z0-9]+)*\.md$ ]]; then
            id="MK-${BASH_REMATCH[1]}"
        else
            add_error "ERROR $path: filename must match MK-NNN-lowercase-slug.md"
            continue
        fi

        if [[ -n "${id_path[$id]+present}" ]]; then
            add_error "ERROR $path: duplicate RFC identifier $id (also ${id_path[$id]})"
        else
            id_path[$id]="$path"
            id_state[$id]="$state"
        fi

        IFS= read -r heading < "$file" ||
            add_error "ERROR $path: cannot read RFC heading"
        if [[ ! "${heading:-}" =~ ^#\ RFC\ (MK-[0-9]{3})\ —\ .+ ]]; then
            add_error "ERROR $path: first line must be '# RFC MK-NNN — title'"
        elif [[ "${BASH_REMATCH[1]}" != "$id" ]]; then
            add_error "ERROR $path: heading identifier ${BASH_REMATCH[1]} does not match $id"
        fi

        metadata=$(awk '
            /^## / { exit }
            /^\*\*Status\.\*\*/ {
                count++
                value = $0
                sub(/^\*\*Status\.\*\*[[:space:]]*/, "", value)
            }
            END { printf "%d\t%s\n", count, value }
        ' "$file") || die_operational "cannot parse status metadata: $path"
        status_count=${metadata%%$'\t'*}
        status=${metadata#*$'\t'}

        if [[ "$status_count" != 1 ]]; then
            add_error "ERROR $path: opening metadata must contain exactly one Status field"
            continue
        fi

        case "$state" in
            proposed)
                [[ "$status" == "Proposed" ]] ||
                    add_error "ERROR $path: proposed RFC status must be 'Proposed'"
                ;;
            done)
                if [[ ! "$status" =~ ^Implemented(\ \(v[0-9]+\.[0-9]+\.[0-9]+((–|-)v?[0-9]+\.[0-9]+\.[0-9]+)?\)|\ \(version\ unverified\)|\ \(Unreleased\))?$ ]]; then
                    add_error "ERROR $path: done RFC status must be Implemented with an allowed release marker"
                fi
                ;;
            archive)
                if [[ "$status" =~ ^Withdrawn\ —\ .*[^[:space:]].*$ ]]; then
                    :
                elif [[ "$status" =~ ^Superseded\ by\ RFC\ (MK-[0-9]{3})(\ —\ .*[^[:space:]].*)?$ ]]; then
                    pending_references+=("$path|${BASH_REMATCH[1]}|single")
                elif [[ "$status" =~ ^Superseded\ by\ RFCs\ (MK-[0-9]{3})–(MK-[0-9]{3})(\ —\ .*[^[:space:]].*)?$ ]]; then
                    start_id=${BASH_REMATCH[1]}
                    end_id=${BASH_REMATCH[2]}
                    start_num=${start_id#MK-}
                    end_num=${end_id#MK-}
                    if (( 10#$start_num > 10#$end_num )); then
                        add_error "ERROR $path: supersession range must be ascending"
                    else
                        for (( number=10#$start_num; number<=10#$end_num; number++ )); do
                            printf -v reference 'MK-%03d' "$number"
                            pending_references+=("$path|$reference|series")
                        done
                    fi
                else
                    add_error "ERROR $path: archive status must be Withdrawn with a reason or a valid Superseded form"
                fi
                ;;
        esac
    done < <(find "$root/rfcs/$state" -maxdepth 1 -type f -name 'MK-*.md' -print0 | sort -z)
done

for item in "${pending_references[@]}"; do
    path=${item%%|*}
    remainder=${item#*|}
    reference=${remainder%%|*}
    source_id=
    source_base=$(basename -- "$path")
    if [[ "$source_base" =~ ^(MK-[0-9]{3})- ]]; then
        source_id=${BASH_REMATCH[1]}
    fi
    if [[ "$reference" == "$source_id" ]]; then
        add_error "ERROR $path: supersession reference must not include itself"
    elif [[ -z "${id_path[$reference]+present}" ]]; then
        add_error "ERROR $path: supersession target $reference does not exist"
    fi
done

index="$root/rfcs/README.md"
section=
index_entry_pattern='\[MK-([0-9]{3})\]\(([^)]*)\)'
while IFS= read -r line || [[ -n "$line" ]]; do
    case "$line" in
        "## Proposed"*) section=proposed ;;
        "## Implemented"*) section=done ;;
        "## Archive"*) section=archive ;;
        "## "*) section= ;;
    esac

    if [[ "$line" =~ $index_entry_pattern ]]; then
        id="MK-${BASH_REMATCH[1]}"
        link=${BASH_REMATCH[2]}
        index_count[$id]=$(( ${index_count[$id]:-0} + 1 ))
        if [[ -z "$section" ]]; then
            add_error "ERROR rfcs/README.md: $id entry is outside a lifecycle section"
        elif [[ -z "${index_state[$id]+present}" ]]; then
            index_state[$id]="$section"
            index_link[$id]="$link"
        fi
    fi
done < "$index"

for id in "${!id_path[@]}"; do
    path=${id_path[$id]}
    expected_state=${id_state[$id]}
    count=${index_count[$id]:-0}
    if (( count == 0 )); then
        add_error "ERROR rfcs/README.md: missing index entry for $id"
        continue
    elif (( count > 1 )); then
        add_error "ERROR rfcs/README.md: $id appears $count times in the index"
    fi

    if [[ "${index_state[$id]:-}" != "$expected_state" ]]; then
        add_error "ERROR rfcs/README.md: $id is in the wrong lifecycle section"
    fi
    expected_link="./$expected_state/$(basename -- "$path")"
    if [[ "${index_link[$id]:-}" != "$expected_link" ]]; then
        add_error "ERROR rfcs/README.md: $id link must be $expected_link"
    fi
done

for id in "${!index_count[@]}"; do
    [[ -n "${id_path[$id]+present}" ]] ||
        add_error "ERROR rfcs/README.md: index entry $id has no RFC file"
done

proposed_count=0
for id in "${!id_state[@]}"; do
    [[ "${id_state[$id]}" == proposed ]] && (( proposed_count += 1 ))
done
empty_sentence_count=$(awk '
    $0 == "No RFCs are currently proposed." { count++ }
    END { print count + 0 }
' "$index") || die_operational "cannot inspect Proposed empty state"
if (( proposed_count == 0 )); then
    [[ "$empty_sentence_count" == 1 ]] ||
        add_error "ERROR rfcs/README.md: empty Proposed section must contain exactly 'No RFCs are currently proposed.'"
elif (( empty_sentence_count != 0 )); then
    add_error "ERROR rfcs/README.md: Proposed empty-state sentence is present while proposed RFCs exist"
fi

markdown_link_pattern='^(.*)(!?\[[^][]*\]\(<([^>]*)>\)|!?\[[^][]*\]\(([^[:space:]()]*)\))'
fence_pattern='^[[:space:]]*```'
for file in "${rfc_files[@]}"; do
    path=$(relative_path "$file")
    directory=$(dirname -- "$file")
    fenced=false
    while IFS= read -r line || [[ -n "$line" ]]; do
        if [[ "$line" =~ $fence_pattern ]]; then
            [[ "$fenced" == true ]] && fenced=false || fenced=true
            continue
        fi
        [[ "$fenced" == true ]] && continue

        scan=$line
        while [[ "$scan" =~ $markdown_link_pattern ]]; do
            prefix=${BASH_REMATCH[1]}
            destination=${BASH_REMATCH[3]}
            [[ -n "${BASH_REMATCH[4]}" ]] && destination=${BASH_REMATCH[4]}
            scan=$prefix

            [[ -n "$destination" ]] || continue
            case "$destination" in
                \#*|/*) continue ;;
            esac
            [[ "$destination" =~ ^[A-Za-z][A-Za-z0-9+.-]*: ]] && continue
            target_path=${destination%%#*}
            [[ "$target_path" == *.md ]] || continue
            if [[ ! -e "$directory/$target_path" ]]; then
                add_error "ERROR $path: unresolved local Markdown link: $destination"
            fi
        done
    done < "$file"
done

if [[ -d "$root/rfcs/handoffs" ]]; then
    while IFS= read -r -d '' handoff; do
        name=$(basename -- "$handoff")
        path=$(relative_path "$handoff")
        if [[ "$name" =~ ^(MK-[0-9]{3})-.+$ ]]; then
            handoff_id=${BASH_REMATCH[1]}
        else
            add_error "ERROR $path: handoff directory must be named MK-NNN-*"
            continue
        fi
        [[ -n "${id_path[$handoff_id]+present}" ]] ||
            add_error "ERROR $path: handoff has no matching RFC"
        for lifecycle in proposed done archive; do
            [[ ! -e "$handoff/$lifecycle" ]] ||
                add_error "ERROR $path: handoff content must not duplicate RFC lifecycle directories"
        done
    done < <(find "$root/rfcs/handoffs" -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)
fi

if (( ${#errors[@]} > 0 )); then
    printf '%s\n' "${errors[@]}" | sort
    printf 'RFC integrity: %d error(s)\n' "${#errors[@]}"
    exit 1
fi

printf 'RFC integrity: 0 error(s)\n'
