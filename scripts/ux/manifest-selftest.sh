#!/usr/bin/env bash
# Self-test for manifest.sh: it must PASS a clean set, and FAIL on (a) a
# deliberately duplicated frame, (b) a frame whose ground contradicts the
# requested theme, (c) a missing frame. Builds synthetic PNGs; needs no
# display. Run: bash scripts/ux/manifest-selftest.sh
set -uo pipefail
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT

# mkpng <file> <r> <g> <b> <marker-grey> — 100x100 RGB, flat colour, plus one
# marker pixel at (0,0) so two "same ground" frames can still differ.
mkpng() {
    python3 - "$1" "$2" "$3" "$4" "$5" <<'PY'
import sys, zlib, struct
path, r, g, b, m = sys.argv[1], *map(int, sys.argv[2:6])
w = h = 100
rows = b"".join(b"\x00" + bytes([r, g, b]) * w for _ in range(h))
rows = bytearray(rows); rows[1:4] = bytes([m, m, m])
def chunk(k, d): return struct.pack(">I", len(d)) + k + d + struct.pack(">I", zlib.crc32(k + d))
open(path, "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(bytes(rows))) + chunk(b"IEND", b""))
PY
}

fail=0
expect() { # expect <0|1> <label> <manifest args...>
    local want=$1 label=$2; shift 2
    bash "$script_dir/manifest.sh" "$@" >/dev/null 2>&1
    local got=$?
    if [[ "$got" == "$want" ]]; then
        printf 'ok    %s (exit %s)\n' "$label" "$got"
    else
        printf 'WRONG %s: wanted exit %s, got %s\n' "$label" "$want" "$got"
        fail=1
    fi
}

mkpng "$work/a.png" 222 222 222 10
mkpng "$work/b.png" 222 222 222 20
mkpng "$work/dup.png" 222 222 222 10          # byte-identical to a.png
mkpng "$work/dark.png" 3 3 3 30               # an hcdark ground
mkpng "$work/drift.png" 198 198 198 40        # hclight, software-darkened
mkpng "$work/light.png" 209 210 212 50        # a (darkened) light ground
common=(--renderer software --out "$work/m.md")

expect 0 "clean set passes"                    --theme hclight "${common[@]}" "$work/a.png" "$work/b.png"
expect 1 "duplicated frame fails"              --theme hclight "${common[@]}" "$work/a.png" "$work/dup.png"
expect 1 "wrong-theme frame fails"             --theme hclight "${common[@]}" "$work/a.png" "$work/dark.png"
expect 0 "overlay frame skips the ground check" --theme hclight "${common[@]}" "$work/a.png" "$work/dark.png@overlay"
expect 1 "overlay frame still counts for duplicates" --theme hclight "${common[@]}" "$work/a.png" "$work/dup.png@overlay"
expect 0 "software-darkened hclight still classes as hclight" --theme hclight "${common[@]}" "$work/drift.png"
expect 1 "a light frame is not hclight"        --theme hclight "${common[@]}" "$work/light.png"
expect 0 "a darkened light frame is light"     --theme light "${common[@]}" "$work/light.png"
expect 1 "missing frame fails"                 --theme hclight "${common[@]}" "$work/a.png" "$work/nope.png"
expect 0 "hcdark frame passes as hcdark"       --theme hcdark "${common[@]}" "$work/dark.png"

if (( fail )); then
    printf 'manifest self-test FAILED\n'; exit 1
fi
printf 'manifest self-test passed\n'
