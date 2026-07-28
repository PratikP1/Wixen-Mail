"""Draw the small subset of SVG this project's assets are written in.

There is no rasteriser on the development machine, and a full SVG engine as a
build dependency for a handful of small files is not a trade worth making. The
assets are each a few flat shapes, so this draws them directly.

The point is that the SVG stays the only description of a mark. The earlier
version of this idea kept the shapes in Python and the SVG beside it as a
readable copy, which is two sources of truth wearing one hat: editing either
one alone leaves them disagreeing and nothing says so.

Supported, because it is what the assets use and nothing more:

    rect        x, y, width, height, rx, fill
    polygon     points, fill
    polyline    points, stroke, stroke-width  (round caps and joins)
    path        d with M and L only, fill or stroke
    circle      cx, cy, r, fill
    g           clip-path="url(#id)" naming a clipPath holding one polygon,
                transform="translate(tx, ty) scale(s)"

Anything else raises rather than being ignored, so an asset that quietly stops
matching its source fails the build instead of shipping half drawn.

Stdlib only: zlib and struct are all a PNG needs, and an ICO is a header and a
directory in front of them.
"""

import math
import re
import struct
import xml.etree.ElementTree as ElementTree
import zlib

SVG_NS = "{http://www.w3.org/2000/svg}"

# Four samples per axis. Enough that a diagonal reads as a clean edge at
# sixteen pixels, which is the size that decides whether a mark looks made or
# found.
SUPERSAMPLE = 4


class UnsupportedSvg(Exception):
    """The file uses something this renderer does not draw."""


# ── Geometry ────────────────────────────────────────────────────────────────


def _in_rounded_rect(x, y, left, top, right, bottom, radius):
    if not (left <= x <= right and top <= y <= bottom):
        return False
    if radius <= 0:
        return True
    # The nearest corner centre. When the point is in the middle band on either
    # axis the clamp returns the point itself, which means it is not in a
    # corner and is therefore inside.
    cx = min(max(x, left + radius), right - radius)
    cy = min(max(y, top + radius), bottom - radius)
    if cx == x or cy == y:
        return True
    return math.hypot(x - cx, y - cy) <= radius


def _in_polygon(x, y, points):
    """Ray casting. Odd crossings to the left means inside."""
    inside = False
    count = len(points)
    for i in range(count):
        ax, ay = points[i]
        bx, by = points[(i + 1) % count]
        if (ay > y) != (by > y):
            crossing = ax + (y - ay) * (bx - ax) / (by - ay)
            if x < crossing:
                inside = not inside
    return inside


def _distance_to_segment(px, py, ax, ay, bx, by):
    dx, dy = bx - ax, by - ay
    length_squared = dx * dx + dy * dy
    if length_squared == 0:
        return math.hypot(px - ax, py - ay)
    t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / length_squared))
    return math.hypot(px - (ax + t * dx), py - (ay + t * dy))


def _near_polyline(x, y, points, width):
    """Within half a stroke of the line, which is what a round cap means."""
    half = width / 2.0
    return any(
        _distance_to_segment(x, y, *points[i], *points[i + 1]) <= half
        for i in range(len(points) - 1)
    )


# ── Reading the file ────────────────────────────────────────────────────────


def _colour(value):
    """`#RRGGBB` to a triple. `none` and a missing value both mean no paint."""
    if not value or value == "none":
        return None
    match = re.fullmatch(r"#([0-9A-Fa-f]{6})", value.strip())
    if not match:
        raise UnsupportedSvg(f"colour {value!r} is not #RRGGBB")
    digits = match.group(1)
    return tuple(int(digits[i : i + 2], 16) for i in (0, 2, 4))


def _points(value):
    numbers = [float(n) for n in re.findall(r"-?[\d.]+", value or "")]
    if len(numbers) < 4 or len(numbers) % 2:
        raise UnsupportedSvg(f"points {value!r} is not a list of pairs")
    return list(zip(numbers[::2], numbers[1::2]))


def _path_points(d):
    """M and L only. A curve would need a flattener and no asset has one."""
    commands = re.findall(r"([MLml])\s*(-?[\d.]+)[\s,]+(-?[\d.]+)", d or "")
    if not commands:
        raise UnsupportedSvg(f"path {d!r} has no M or L commands")
    consumed = sum(len(c[0]) + len(c[1]) + len(c[2]) for c in commands)
    if consumed < len(re.sub(r"[\s,]", "", d)):
        raise UnsupportedSvg(f"path {d!r} uses a command other than M and L")
    points = []
    x = y = 0.0
    for letter, sx, sy in commands:
        dx, dy = float(sx), float(sy)
        if letter.islower() and points:
            x, y = x + dx, y + dy
        else:
            x, y = dx, dy
        points.append((x, y))
    return points


#: Presentation attributes a group passes down to the shapes inside it.
INHERITED = ("fill", "stroke", "stroke-width")


def _attribute(element, inherited, name, default=None):
    """The element's own value, or the nearest enclosing group's."""
    value = element.get(name)
    return inherited.get(name, default) if value is None else value


def _float(element, name, default=0.0, inherited=None):
    value = _attribute(element, inherited or {}, name)
    return default if value is None else float(value)


#: No move and no scale. Groups without a transform get this.
IDENTITY = (0.0, 0.0, 1.0)


def _parse_transform(value):
    """`translate(tx, ty) scale(s)`, in that order, and nothing else.

    Enough to place one drawing inside another, which is all the assets ask
    for. A rotation or a skew would need a matrix, and no mark here has one.
    """
    if not value:
        return IDENTITY
    tx = ty = 0.0
    scale = 1.0
    seen = 0
    for name, arguments in re.findall(r"(\w+)\(([^)]*)\)", value):
        numbers = [float(n) for n in re.findall(r"-?[\d.]+", arguments)]
        seen += 1
        if name == "translate" and len(numbers) in (1, 2):
            tx = numbers[0]
            ty = numbers[1] if len(numbers) > 1 else 0.0
        elif name == "scale" and len(numbers) == 1:
            scale = numbers[0]
        else:
            raise UnsupportedSvg(f"transform {name}({arguments})")
    if not seen:
        raise UnsupportedSvg(f"transform {value!r}")
    return (tx, ty, scale)


def _combine(outer, inner):
    """Nest one transform inside another."""
    ox, oy, os = outer
    ix, iy, iscale = inner
    return (ox + os * ix, oy + os * iy, os * iscale)


def _bounds(points, margin=0.0):
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    return (min(xs) - margin, min(ys) - margin, max(xs) + margin, max(ys) + margin)


class _Shape:
    """One painted thing: a test for whether a point is on it, and a colour."""

    def __init__(self, hit, colour, bounds, clip=None, transform=IDENTITY):
        self.hit = hit
        self.colour = colour
        self.bounds = bounds
        self.clip = clip
        self.transform = transform

    def covers(self, x, y):
        # The transform says where the shape's own coordinates land, so testing
        # a point means undoing it. Stroke widths come out right for free:
        # they are measured in the space the test happens in.
        tx, ty, scale = self.transform
        x, y = (x - tx) / scale, (y - ty) / scale
        # Nearly every sample of nearly every shape is outside it, and this
        # rejects those for four comparisons rather than a walk of the edges.
        # Without it the wordmark spends its whole time asking nine thin
        # strokes about points nowhere near any of them.
        left, top, right, bottom = self.bounds
        if not (left <= x <= right and top <= y <= bottom):
            return False
        if self.clip is not None and not self.clip(x, y):
            return False
        return self.hit(x, y)


def _shape_from(element, clip, transform=IDENTITY, inherited=None):
    inherited = inherited or {}
    tag = element.tag.removeprefix(SVG_NS)
    fill = _colour(_attribute(element, inherited, "fill"))
    stroke = _colour(_attribute(element, inherited, "stroke"))

    if tag == "rect":
        left, top = _float(element, "x"), _float(element, "y")
        right = left + _float(element, "width")
        bottom = top + _float(element, "height")
        radius = _float(element, "rx")
        return _Shape(
            lambda x, y: _in_rounded_rect(x, y, left, top, right, bottom, radius),
            fill,
            (left, top, right, bottom),
            clip,
            transform,
        )

    if tag == "circle":
        cx, cy, r = (_float(element, n) for n in ("cx", "cy", "r"))
        return _Shape(
            lambda x, y: math.hypot(x - cx, y - cy) <= r,
            fill,
            (cx - r, cy - r, cx + r, cy + r),
            clip,
            transform,
        )

    if tag in ("polygon", "polyline", "path"):
        points = (
            _path_points(element.get("d"))
            if tag == "path"
            else _points(element.get("points"))
        )
        if stroke is not None:
            width = _float(element, "stroke-width", 1.0, inherited)
            return _Shape(
                lambda x, y: _near_polyline(x, y, points, width),
                stroke,
                _bounds(points, width / 2.0),
                clip,
                transform,
            )
        if fill is None:
            raise UnsupportedSvg(f"{tag} has neither a fill nor a stroke")
        return _Shape(
            lambda x, y: _in_polygon(x, y, points),
            fill,
            _bounds(points),
            clip,
            transform,
        )

    raise UnsupportedSvg(f"element {tag!r}")


class Drawing:
    """A parsed asset: a coordinate space and the shapes to paint in order."""

    def __init__(self, viewbox, shapes):
        self.viewbox = viewbox
        self.shapes = shapes

    @property
    def aspect(self):
        _, _, width, height = self.viewbox
        return width / height

    def colour_at(self, x, y):
        """Painter's algorithm: the last shape covering the point wins."""
        found = None
        for shape in self.shapes:
            if shape.covers(x, y):
                found = shape.colour
        return found


def load(path):
    root = ElementTree.parse(path).getroot()
    box = [float(n) for n in re.findall(r"-?[\d.]+", root.get("viewBox") or "")]
    if len(box) != 4:
        raise UnsupportedSvg("the root element needs a four number viewBox")

    clips = {}
    for element in root.iter(f"{SVG_NS}clipPath"):
        children = [c for c in element if c.tag == f"{SVG_NS}polygon"]
        if len(children) != 1:
            raise UnsupportedSvg("a clipPath must hold exactly one polygon")
        points = _points(children[0].get("points"))
        clips[element.get("id")] = lambda x, y, p=points: _in_polygon(x, y, p)

    shapes = []
    ignored = (f"{SVG_NS}title", f"{SVG_NS}desc", f"{SVG_NS}clipPath")

    def walk(parent, clip, transform, inherited):
        for element in parent:
            if element.tag in ignored or not isinstance(element.tag, str):
                continue
            if element.tag == f"{SVG_NS}g":
                reference = element.get("clip-path") or ""
                match = re.fullmatch(r"url\(#(.+)\)", reference)
                inner = clip
                if match:
                    if match.group(1) not in clips:
                        raise UnsupportedSvg(f"no clipPath {match.group(1)!r}")
                    inner = clips[match.group(1)]
                passed = dict(inherited)
                passed.update(
                    {
                        name: element.get(name)
                        for name in INHERITED
                        if element.get(name) is not None
                    }
                )
                walk(
                    element,
                    inner,
                    _combine(transform, _parse_transform(element.get("transform"))),
                    passed,
                )
                continue
            shapes.append(_shape_from(element, clip, transform, inherited))

    walk(root, None, IDENTITY, {})
    if not shapes:
        raise UnsupportedSvg("nothing to draw")
    return Drawing(tuple(box), shapes)


# ── Writing the file ────────────────────────────────────────────────────────


def render(drawing, width, height=None, background=None):
    """One RGBA image, supersampled and averaged, as raw PNG scanlines."""
    height = height or round(width / drawing.aspect)
    origin_x, origin_y, box_width, box_height = drawing.viewbox
    step_x = box_width / (width * SUPERSAMPLE)
    step_y = box_height / (height * SUPERSAMPLE)
    samples = SUPERSAMPLE * SUPERSAMPLE

    pixels = bytearray()
    for row in range(height):
        pixels.append(0)  # PNG filter: none
        for column in range(width):
            r = g = b = covered = 0
            for sy in range(SUPERSAMPLE):
                y = origin_y + (row * SUPERSAMPLE + sy + 0.5) * step_y
                for sx in range(SUPERSAMPLE):
                    x = origin_x + (column * SUPERSAMPLE + sx + 0.5) * step_x
                    found = drawing.colour_at(x, y)
                    if found is None:
                        found = background
                    if found is not None:
                        r += found[0]
                        g += found[1]
                        b += found[2]
                        covered += 1
            if covered == 0:
                pixels.extend((0, 0, 0, 0))
                continue
            # Averaged over covered samples only, so an edge blends towards
            # transparent rather than towards black.
            pixels.extend(
                (r // covered, g // covered, b // covered, covered * 255 // samples)
            )
    return pixels, width, height


def png(raw, width, height):
    def chunk(tag, payload):
        return (
            struct.pack(">I", len(payload))
            + tag
            + payload
            + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def ico(images):
    """An ICO from `(size, png bytes)` pairs, largest last.

    A header, one directory entry per image, then the images. Windows has
    accepted PNG-compressed entries since Vista, and 256 is written as 0
    because the field is one byte.
    """
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
    return out
