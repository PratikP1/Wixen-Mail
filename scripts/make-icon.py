"""Render assets/icon.svg to assets/icon.ico.

The application icon, embedded in the executable by build.rs and used by the
installer wizard. Run it when assets/icon.svg changes:

    python scripts/make-icon.py

This used to hold its own copy of the shapes and its own rasteriser, with the
SVG beside it as a readable copy of the same numbers. That is two sources of
truth and no way to tell when they disagree, so the drawing moved to
scripts/render_svg.py and the SVG became the only description of the icon.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import render_svg  # noqa: E402  (needs the path set above)

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "icon.svg"
OUT = ROOT / "assets" / "icon.ico"

#: Every size Windows asks for, from the taskbar to the extra-large view in
#: Explorer. Missing one makes Windows scale a neighbour and the result looks
#: like a thumbnail of an icon rather than an icon.
SIZES = [16, 24, 32, 48, 64, 128, 256]


def main():
    drawing = render_svg.load(SOURCE)
    images = []
    for size in SIZES:
        raw, width, height = render_svg.render(drawing, size)
        images.append((size, render_svg.png(raw, width, height)))

    data = render_svg.ico(images)
    OUT.write_bytes(data)
    print(f"{OUT.relative_to(ROOT)}: {len(images)} sizes, {len(data) / 1024:.1f} KB")


if __name__ == "__main__":
    main()
