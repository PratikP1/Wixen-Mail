# The Wixen brand

Wixen is a family of applications, not one application. Wixen Mail and Wixen Terminal exist,
Wixen Chat is being built, and more will follow. They share a stance rather than a codebase:
each one is a tool that a blind person can use as well as a sighted person can, built on the
belief that the application should declare what it means instead of leaving a screen reader
to work it out.

This page is what holds them together visually. It is also the answer to a question that gets
asked in bad faith often enough to be worth answering plainly: a product built for people who
cannot see it still has to look like somebody cared. Most of the people using these tools have
some sight. Many have colleagues looking over their shoulder. All of them deserve a product
that does not announce its own charity status the moment it appears on a screen.

## The mark

A fox's head, seen face on, with a band across its eyes.

The fox is for the obvious reason and one less obvious one. Foxes are the animal we reach for
when we mean resourceful rather than powerful, and every one of these tools is a small thing
that gets around a large obstacle. They also hunt by sound. A fox locates a mouse under snow
by ear alone and drops on it without ever having seen it, which is a better description of what
a competent screen reader user does with an unfamiliar interface than any metaphor about
darkness.

The blindfold is the risky part of the design and everything else is arranged to make it land
the right way up. A blindfolded animal can read as captured, punished, or pitiable. What stops
it here is the ears. They are more than a third of the height of the mark, they point forward,
and they are the first thing you see. An animal with its ears up and forward is an animal
paying attention. The band is not what has been done to it, it is what it does not need.

Nothing in the mark is a smile, a helping hand, a puzzle piece, or a stylised person. Those are
the visual language of being helped, and this is not a charity.

### The pieces

| File | What it is | Use it for |
|---|---|---|
| `assets/brand/wixen-fox.svg` | The mark alone, no background | Anywhere the name is already on the page |
| `assets/brand/wixen-badge.svg` | The mark knocked out of a rounded orange tile | Avatars, favicons, taskbar buttons, tiles |
| `assets/brand/wixen-wordmark.svg` | The word Wixen | Where the name has to appear without the animal |
| `assets/brand/wixen-lockup.svg` | Mark and word together | Page headers, title slides, first impressions |

Each has a matching PNG beside it, and `assets/brand/wixen.ico` carries the badge at every size
Windows asks for.

### Alt text

Copy these rather than writing your own. They are the descriptions the marks were designed
against, and an asset whose alt text has to explain more than this has stopped being the asset.

| Asset | Alt text |
|---|---|
| The mark | `The Wixen fox` |
| The badge | `Wixen` |
| The wordmark | `Wixen` |
| The lockup | `Wixen` |

The mark on its own gets a longer name because it appears where the word does not, so it has to
carry the identification by itself. Everywhere the word is visible in the image, the alt text is
just the name: describing the picture as well would make a screen reader read the same thing
twice. If a mark is decorative on a page that already says Wixen, give it an empty alt attribute
and let it disappear.

## Colour

Three colours. That is the whole palette.

| Name | Value | What it is |
|---|---|---|
| Coat | `#C2410C` | The fox, and the field on the badge |
| Ink | `#1C1917` | The band, the nose, and the wordmark on a light page |
| Paper | `#FBFAF9` | The fox on the badge, and the wordmark on a dark page |

They live in `src/presentation/theme.rs` as `theme::brand`, with the contrast floors written as
tests, so the mark cannot drift into something prettier and less legible without a build failing.

Measured, against every background a mark gets put on:

| Pair | Ratio | Floor |
|---|---|---|
| Coat on the light surface | 4.97:1 | 3:1 |
| Coat on the light second surface | 4.52:1 | 3:1 |
| Coat on the dark surface | 3.58:1 | 3:1 |
| Coat on the dark second surface | 3.20:1 | 3:1 |
| Coat on white | 5.18:1 | 3:1 |
| Coat on black | 4.06:1 | 3:1 |
| Band on the coat | 3.38:1 | 3:1 |
| Cream fox on the orange field | 4.97:1 | 3:1 |
| Band on the cream fox | 16.78:1 | 3:1 |
| Wordmark on a light page | 17.49:1 | 4.5:1 |
| Wordmark on a dark page | 20.14:1 | 4.5:1 |

The floor is 3:1 because WCAG 1.4.11 sets that for a graphic that carries meaning, and a logo
that identifies the application carries meaning by definition. The wordmark is held to 4.5:1
because it is a word, and words are text.

The tight one is the coat on a dark second surface at 3.20:1. That is what fixes the orange
where it is: any lighter and it fails on a white page, any darker and it fails on a dark one.
It is not a shade somebody liked, it is the shade that works in both places.

### The family rule

Each application in the family keeps its own icon and its own accent colour. What makes them
one family is the construction, not the picture:

**A coloured field, a cream figure, and the detail in ink or in the field colour.**

- **Wixen** is an orange field, a cream fox, an ink band.
- **Wixen Mail** is a violet field, a cream envelope, a violet W for the flap.
- **Wixen Chat** and anything after it pick their own accent and follow the same three layers.

Put them side by side and they are visibly the same object doing different jobs. No application
has to carry another application's picture to belong.

Two rules for a new member. The accent has to clear 3:1 against both white and black, which is
what makes the mark survive being put anywhere. And the figure has to be one closed silhouette,
because that is what still reads at sixteen pixels after everything inside it has turned to mud.

## Type

**Wixen does not ship a typeface.**

The wordmark is not set in a font. Every letter in WIXEN is made of straight lines, with no curve
in the word, so it is drawn as nine stroked polylines. It needs no font file, cannot be
substituted by whatever the reader has installed, and stays one shape at any size.

Everything that is not that word is set in the system font at the size the reader chose. On
Windows 11 that is Segoe UI Variable and before it Segoe UI, and if somebody has changed it, it
is whatever they changed it to. A mail client that overrides the font of a person who set it
deliberately has broken the one thing they asked for, and a person who set their system text to
150% did not do it by accident.

So the type scale is relative, never absolute. Multiply the system size:

| Step | Multiplier | Where |
|---|---|---|
| Caption | 0.9 | Column headers, counts, timestamps in a dense list |
| Body | 1.0 | Message text, list rows, labels, everything unmarked |
| Subheading | 1.15 | Group headings, the sender line above a message |
| Heading | 1.3 | Panel and dialog titles |
| Display | 1.6 | The about box, the welcome page, and nothing else |

The ratio between steps is small on purpose. This is a dense information application, and a scale
built for a marketing page puts a list header three times the size of the list.

**Never go below 0.9.** Below that the multiplier starts undoing the system setting rather than
sitting inside it, and 0.9 of a size somebody chose because they cannot read anything smaller is
already a compromise.

## Using the marks

- Give the mark clear space of at least the height of one ear on every side.
- Do not put the mark on a photograph, a gradient, or a busy background. The measured contrast
  is against a flat colour, and none of it holds against an image.
- Do not recolour the fox. There are three colours and they are all doing a job.
- Do not stretch it. Scale both axes together.
- Do not add a drop shadow, an outline, a bevel, or a glow.
- Do not put the lockup on a page where the word Wixen is already in the heading beside it. Use
  the fox on its own. Two statements of the same name next to each other is one too many.
- Do not rotate the mark or tilt the head. A tilted head is a confused animal.

## Regenerating

The SVGs are the source. Everything else is derived:

```bash
python scripts/make-brand.py
```

That reads `assets/brand/*.svg` and writes the PNGs and the ICO beside them. It uses
`scripts/render_svg.py`, which draws the small subset of SVG these files are written in, because
there is no rasteriser on the development machine and a full SVG engine as a build dependency for
a handful of small flat files is not a trade worth making.

The badge and the lockup each draw the fox again rather than referencing it, since SVG's own way
of sharing a shape between files does not survive being opened in the tools people actually open
these in. That duplication is checked rather than trusted: the script counts every shared shape in
every file and refuses to write anything if the counts stop matching. It has already caught one
real drift, which is the only argument for a check worth making.

The Wixen Mail application icon is separate, and built the same way:

```bash
python scripts/make-icon.py
```
