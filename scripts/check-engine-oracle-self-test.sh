#!/usr/bin/env bash

set -u

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd -P) || {
    printf 'Engine oracle self-test: cannot determine script directory\n' >&2
    exit 2
}
checker="$script_dir/check-engine-oracle.sh"
repository_root=$(cd -- "$script_dir/.." 2>/dev/null && pwd -P) || {
    printf 'Engine oracle self-test: cannot determine repository root\n' >&2
    exit 2
}

for utility in chmod cp env mkdir mktemp rm sed; do
    command -v "$utility" >/dev/null 2>&1 || {
        printf 'Engine oracle self-test: required utility not found: %s\n' "$utility" >&2
        exit 2
    }
done

[[ -x "$checker" ]] || {
    printf 'Engine oracle self-test: checker is not executable: %s\n' "$checker" >&2
    exit 2
}

tmp_parent=${TMPDIR:-"$repository_root/target/tmp"}
mkdir -p -- "$tmp_parent" || exit 2
temp_dir=$(mktemp -d "$tmp_parent/check-engine-oracle-self-test.XXXXXX") || exit 2

cleanup() {
    if [[ -n "${temp_dir:-}" &&
          "$temp_dir" == "$tmp_parent"/check-engine-oracle-self-test.* &&
          -d "$temp_dir" ]]; then
        rm -rf -- "$temp_dir"
    else
        printf 'Engine oracle self-test: refusing unsafe cleanup: %s\n' \
            "${temp_dir:-<empty>}" >&2
        return 2
    fi
}
trap cleanup EXIT

failures=0
checks=0

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
        printf 'FAIL %s: expected exit %d, got %d\n%s\n' \
            "$label" "$expected_status" "$status" "$output" >&2
        (( failures += 1 ))
    elif [[ "$output" != *"$expected_text"* ]]; then
        printf "FAIL %s: expected output containing '%s'\n%s\n" \
            "$label" "$expected_text" "$output" >&2
        (( failures += 1 ))
    fi
}

make_fixture() {
    local destination=$1
    mkdir -p -- "$destination"
    cp -- "$repository_root/Cargo.toml" "$destination/Cargo.toml"
    cp -- "$repository_root/Cargo.lock" "$destination/Cargo.lock"
}

fake_cargo="$temp_dir/fake-cargo"
cp -- /dev/null "$fake_cargo"
chmod +x "$fake_cargo"

write_features() {
    local config_features=$1
    printf '%s\n' '#!/usr/bin/env bash' \
        'case "$*" in' \
        '  *apimock-config@6.0.0*)' \
        "    printf '%s\\n' 'apimock-config v6.0.0' $config_features" \
        '    ;;' \
        '  *) exit 2 ;;' \
        'esac' > "$fake_cargo"
}

valid="$temp_dir/valid"
make_fixture "$valid"
write_features "'apimock-config feature \"default\"'"
expect_result "valid oracle contract" 0 "contract verified" \
    env CARGO="$fake_cargo" "$checker" "$valid"

case_root="$temp_dir/version"
cp -R -- "$valid" "$case_root"
sed -i '/name = "apimock-config"/{n;s/6\.0\.0/6.1.0/;}' "$case_root/Cargo.lock"
expect_result "config version drift" 1 "version must be 6.0.0" \
    env CARGO="$fake_cargo" "$checker" "$case_root"

case_root="$temp_dir/source"
cp -R -- "$valid" "$case_root"
sed -i '/name = "apimock-config"/{n;n;s#registry+https://github.com/rust-lang/crates.io-index#git+https://example.invalid/apimock-config#;}' "$case_root/Cargo.lock"
expect_result "config source drift" 1 "apimock-config source must be" \
    env CARGO="$fake_cargo" "$checker" "$case_root"

case_root="$temp_dir/checksum"
cp -R -- "$valid" "$case_root"
sed -i '/name = "apimock-config"/{n;n;n;s/70d8972c/00000000/;}' "$case_root/Cargo.lock"
expect_result "config checksum drift" 1 "apimock-config checksum must be" \
    env CARGO="$fake_cargo" "$checker" "$case_root"

write_features "'apimock-config feature \"default\"' 'apimock-config feature \"experimental\"'"
expect_result "config feature drift" 1 "resolved features must be [default]" \
    env CARGO="$fake_cargo" "$checker" "$valid"

if (( failures > 0 )); then
    printf 'Engine oracle self-test: %d/%d checks failed\n' "$failures" "$checks" >&2
    exit 1
fi

printf 'Engine oracle self-test: %d checks passed\n' "$checks"
