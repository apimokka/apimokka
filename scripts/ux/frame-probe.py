#!/usr/bin/env python3
"""Pixel probe for scripts/ux (stdlib only; decodes 8-bit RGB/RGBA PNGs).

  frame-probe.py size   FILE          -> "W H"
  frame-probe.py pixel  FILE X Y      -> "R G B"
  frame-probe.py ground FILE          -> "R G B" at the top bar's own fill

The "ground" point is inside the top bar (`panel_style`), between its status
chips and its buttons, at 62% of the width and 4.5% of the height — present
on every workspace screen at every window size and scale this directory
drives, and a different colour in each of the four theme presets.
"""
import struct
import sys
import zlib


def load(path):
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        sys.exit(f"{path}: not a PNG")
    pos, idat, w, h, ct = 8, b"", 0, 0, 2
    while pos < len(data):
        (n,) = struct.unpack(">I", data[pos : pos + 4])
        kind, body = data[pos + 4 : pos + 8], data[pos + 8 : pos + 8 + n]
        pos += 12 + n
        if kind == b"IHDR":
            w, h, depth, ct = struct.unpack(">IIBB", body[:10])
            if depth != 8 or ct not in (2, 6) or body[12] != 0:
                sys.exit(f"{path}: only 8-bit non-interlaced RGB/RGBA supported")
        elif kind == b"IDAT":
            idat += body
    bpp = 3 if ct == 2 else 4
    raw, stride, prev, rows, p = zlib.decompress(idat), w * bpp, bytearray(w * bpp), [], 0
    for _ in range(h):
        f, line = raw[p], bytearray(raw[p + 1 : p + 1 + stride])
        p += 1 + stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        rows.append(line)
        prev = line
    return w, h, bpp, rows


def pixel(path, x, y):
    w, h, bpp, rows = load(path)
    if not (0 <= x < w and 0 <= y < h):
        sys.exit(f"{path}: ({x},{y}) outside {w}x{h}")
    return tuple(rows[y][x * bpp : x * bpp + 3])


def main():
    cmd, path = sys.argv[1], sys.argv[2]
    if cmd == "size":
        w, h, _, _ = load(path)
        print(w, h)
    elif cmd == "pixel":
        print(*pixel(path, int(sys.argv[3]), int(sys.argv[4])))
    elif cmd == "ground":
        w, h, _, _ = load(path)
        print(*pixel(path, int(w * 0.62), int(h * 0.045)))
    else:
        sys.exit(__doc__)


main()
