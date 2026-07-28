"""Render assets/icon.svg to assets/icon.ico.

There is no rasteriser on this machine and adding one as a build dependency
would put a full SVG engine into every clean build for one 100 KB file. The
icon is four shapes, so this draws them directly and keeps the SVG as the
readable source of truth beside it.

Stdlib only: zlib and struct are all a PNG needs, and an ICO is a header and a
directory in front of them.

Run it when assets/icon.svg changes:

    python scripts/make-icon.py
"""

import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "icon.ico"

# The same values as the SVG, on the same 256 unit grid, so the two cannot
# drift without somebody noticing they edited only one.
VIOLET = (0x5B, 0x21, 0xB6)
OFF_WHITE = (0xFB, 0xFA, 0xF9)
CANVAS = 256.0
FIELD_RADIUS = 56.0
BODY = (40.0, 72.0, 216.0, 192.0)
BODY_RADIUS = 14.0
FLAP = [(40, 86), (84, 140), (128, 96), (172, 140), (216, 86)]
FLAP_WIDTH = 16.0

# Four samples per axis. Enough that the flap's diagonals read as clean edges
# at 16 pixels, which is the size that decides whether an icon looks made or
# found.
SUPERSAMPLE = 4
SIZES = [16, 24, 32, 48, 64, 128, 256]


def rounded_rect_contains(x, y, box, radius):
    left, top, right, bottom = box
    if not (left <= x <= right and top <= y <= bottom):
        return False
    # Only the corners need the circle test.
    cx = min(max(x, left + radius), right - radius)
    cy = min(max(y, top + radius), bottom - radius)
    return math.hypot(x - cx, y - cy) <= radius or (
        left + radius <= x <= right - radius or top + radius <= y <= bottom - radius
    )


def distance_to_segment(px, py, ax, ay, bx, by):
    dx, dy = bx - ax, by - ay
    length_squared = dx * dx + dy * dy
    if length_squared == 0:
        return math.hypot(px - ax, py - ay)
    t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / length_squared))
    return math.hypot(px - (ax + t * dx), py - (ay + t * dy))


def on_flap(x, y):
    """Within half a stroke width of the W, which is what a round cap means."""
    half = FLAP_WIDTH / 2.0
    return any(
        distance_to_segment(x, y, *FLAP[i], *FLAP[i + 1]) <= half
        for i in range(len(FLAP) - 1)
    )


def colour_at(x, y):
    """The colour of one point on the 256 unit grid, or None for transparent."""
    if not rounded_rect_contains(x, y, (0.0, 0.0, CANVAS, CANVAS), FIELD_RADIUS):
        return None
    if on_flap(x, y):
        return VIOLET
    if rounded_rect_contains(x, y, BODY, BODY_RADIUS):
        return OFF_WHITE
    return VIOLET


def render(size):
    """One RGBA image, supersampled and averaged."""
    pixels = bytearray()
    step = CANVAS / (size * SUPERSAMPLE)
    for row in range(size):
        pixels.append(0)  # PNG filter: none
        for column in range(size):
            r = g = b = a = 0
            for sy in range(SUPERSAMPLE):
                for sx in range(SUPERSAMPLE):
                    x = (column * SUPERSAMPLE + sx + 0.5) * step
                    y = (row * SUPERSAMPLE + sy + 0.5) * step
                    found = colour_at(x, y)
                    if found is not None:
                        r += found[0]
                        g += found[1]
                        b += found[2]
                        a += 255
            samples = SUPERSAMPLE * SUPERSAMPLE
            if a == 0:
                pixels.extend((0, 0, 0, 0))
                continue
            covered = a // 255
            # Averaged over covered samples only, so an edge blends towards
            # transparent rather than towards black.
            pixels.extend((r // covered, g // covered, b // covered, a // samples))
    return bytes(pixels)


def png(size, raw):
    def chunk(tag, payload):
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def main():
    images = [(size, png(size, render(size))) for size in SIZES]

    # ICO: a header, one directory entry per image, then the images. Windows
    # has accepted PNG-compressed entries since Vista.
    out = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    for size, data in images:
        out += struct.pack(
            "<BBBBHHII",
            0 if size >= 256 else size,
            0 if size >= 256 else size,
            0,
            0,
            1,
            32,
            len(data),
            offset,
        )
        offset += len(data)
    for _, data in images:
        out += data

    OUT.write_bytes(out)
    print(f"{OUT.relative_to(ROOT)}: {len(images)} sizes, {len(out) / 1024:.1f} KB")


if __name__ == "__main__":
    main()
