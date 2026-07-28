"""Render the Wixen brand assets from their SVG sources.

The SVGs under assets/brand are what somebody edits. Everything this writes is
derived from them, so a change to a mark is a change to one file and a rerun of
this script:

    python scripts/make-brand.py

The lockup and the badge each draw the fox again rather than referencing it,
because SVG's own way of sharing a shape between files does not survive being
opened in the tools people actually open these in. That duplication is the one
real risk in the whole set, so it is checked rather than trusted: every shape
that appears in more than one file has to appear identically, and this refuses
to write anything if it does not.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import render_svg  # noqa: E402  (needs the path set above)

ROOT = Path(__file__).resolve().parent.parent
BRAND = ROOT / "assets" / "brand"

#: The shapes that appear in more than one file, and where they came from.
#:
#: Keyed by the file that owns the shape. Each value lists the shapes it owns
#: and the files that redraw them.
SHARED = {
    "wixen-fox.svg": {
        "shapes": [
            "40,14 96,86 160,86 216,14 226,110 204,152 128,222 52,152 30,110",
            "110,186 146,186 128,208",
            'x="20" y="110" width="216" height="40"',
        ],
        "redrawn_in": ["wixen-badge.svg", "wixen-lockup.svg"],
    },
    "wixen-wordmark.svg": {
        "shapes": [
            "0,0 18,100 40,34 62,100 80,0",
            "106,0 106,100",
            "142,0 200,100",
            "200,0 142,100",
            "228,0 228,100",
            "228,0 280,0",
            "228,50 272,50",
            "228,100 280,100",
            "304,100 304,0 362,100 362,0",
        ],
        "redrawn_in": ["wixen-lockup.svg"],
    },
}

#: What to write. Source file, output name, and the widths to write it at.
OUTPUTS = [
    ("wixen-fox.svg", "wixen-fox", [512]),
    ("wixen-badge.svg", "wixen-badge", [512]),
    ("wixen-lockup.svg", "wixen-lockup", [1100]),
    ("wixen-wordmark.svg", "wixen-wordmark", [800]),
]

#: The sizes Windows and the web ask an icon for.
ICON_SIZES = [16, 24, 32, 48, 64, 128, 256]


def check_shared_shapes():
    """Fail loudly if a shape drawn twice has stopped being the same shape.

    Counted rather than merely looked for. Most of these shapes appear twice in
    their own file, once as the drawn thing and once inside a clipPath, and a
    plain "is it in there" check passes when one of the two has been edited and
    the other has not. That is not a hypothetical: it is what the first version
    of this let through.
    """
    problems = []
    for owner, shared in SHARED.items():
        source = (BRAND / owner).read_text(encoding="utf-8")
        for shape in shared["shapes"]:
            expected = source.count(shape)
            if not expected:
                problems.append(f"{owner} no longer contains {shape!r}")
                continue
            for other in shared["redrawn_in"]:
                found = (BRAND / other).read_text(encoding="utf-8").count(shape)
                if found != expected:
                    problems.append(
                        f"{other} has {found} of {shape!r} where {owner} has "
                        f"{expected}"
                    )
    if problems:
        raise SystemExit(
            "The marks have drifted apart:\n  "
            + "\n  ".join(problems)
            + "\n\nCopy the shape from the file that owns it into the ones that "
            "redraw it."
        )


def write_png(drawing, path, width):
    raw, actual_width, height = render_svg.render(drawing, width)
    path.write_bytes(render_svg.png(raw, actual_width, height))
    return actual_width, height


def main():
    check_shared_shapes()

    for source, stem, widths in OUTPUTS:
        drawing = render_svg.load(BRAND / source)
        for width in widths:
            out = BRAND / f"{stem}.png" if len(widths) == 1 else BRAND / f"{stem}-{width}.png"
            size = write_png(drawing, out, width)
            print(f"{out.relative_to(ROOT)}: {size[0]}x{size[1]}")

    # One ICO from the badge, for a favicon and anywhere Windows wants an icon
    # for the project rather than for one of its applications.
    badge = render_svg.load(BRAND / "wixen-badge.svg")
    images = []
    for size in ICON_SIZES:
        raw, width, height = render_svg.render(badge, size)
        images.append((size, render_svg.png(raw, width, height)))
    out = BRAND / "wixen.ico"
    out.write_bytes(render_svg.ico(images))
    print(f"{out.relative_to(ROOT)}: {len(images)} sizes")


if __name__ == "__main__":
    main()
