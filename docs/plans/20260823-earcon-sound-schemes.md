# Earcons: real sounds, more events, and sound schemes

The feedback system plays a synthesized `Beep()` tone per event today, Windows
only. This plan replaces that with real short sound files, adds a handful of
new events, and lets a sound be swapped for another one by picking a scheme,
including ones somebody else built and shared as a zip file. It also has to
survive the move to macOS, which changes the right answer for how sound gets
played, not just where the sound files come from.

Nothing here is built yet. This is the plan to review before any of it is.

## Why the playback mechanism has to change now, not later

`emit()` today is two functions: a Windows-only FFI call to `Beep()`, and a
silent no-op everywhere else. That was the right call for a Windows-only,
generated-tone feature: no dependency, no device setup, works on a locked-down
machine. It stops being the right call the moment two things are both true at
once, and they now are: sound has to work on macOS too, and a sound can be a
sampled file rather than a synthesized tone, which `Beep()` cannot play at
all, on any platform.

Written down as a known limitation already: "Earcons are Windows-only for
now... a port needs its own audio path." This plan is that audio path, timed
to land before the port needs it rather than after.

## Playback: rodio, not a second platform-specific FFI shim

Two shapes were weighed.

**Write a `PlaySoundW` call for Windows, matching `Beep()`'s own style, and a
matching macOS call (`AudioServicesPlaySystemSound` or similar) when the port
happens.** Minimal dependencies, consistent with how this project already
prefers a small FFI block over a crate. Rejected: it only plays WAV, so every
community-packaged sound would have to already be WAV, and it means writing
and maintaining a second hand-rolled platform shim at port time instead of
building the cross-platform path once, now, while there's one platform to
prove it against.

**[rodio](https://github.com/RustAudio/rodio)**, built on
[cpal](https://lib.rs/crates/cpal) for output and Symphonia for decoding.
Chosen. It plays on Windows, macOS, and Linux through one API, decodes WAV,
OGG, MP3, and FLAC without a second crate per format, and it is not a fringe
pick: over 5.9 million downloads on crates.io, part of the same RustAudio
project as cpal, actively maintained. The heavier dependency tree is worth it
here specifically because community packs need more than one format read
without asking pack authors to convert everything to WAV first, and because
writing a second platform's worth of decode-and-output code by hand is a
larger, riskier undertaking than pulling in an established crate built for
exactly this.

A lighter alternative exists and was considered:
[tinyaudio](https://github.com/mrDIMAS/tinyaudio) genuinely supports Windows,
macOS, and Linux for raw output, but decodes nothing, so WAV would still need
`hound` and OGG would still need a separate decoder crate (`lewton` or
similar) stacked on top. That ends up as comparable total dependency surface
to rodio, spread across several smaller, less-established crates instead of
one well-known one. Not recommended.

**One implementation detail worth flagging now, because it is easy to get
wrong later**: rodio's output stream handle has to live as long as playback
does. Dropping it stops sound. It belongs held alongside `EarconPlayer`
itself, not created fresh per call.

**A genuine side effect worth having**: once tones play through rodio instead
of `Beep()`, the built-in "Generated tones" scheme (below) can synthesize its
sine waves into an in-memory buffer and play that through the same path,
which means the Windows-only limitation closes as part of this work rather
than waiting for the port itself.

## Sound sources researched

The one hard requirement: whatever ships bundled with the app, or gets held
up as a recommended source for community packs, has to be safe to
redistribute, not merely free to listen to. That ruled out one popular option.

| Source | License | Redistribution | Verdict |
|---|---|---|---|
| [UI SFX](https://uisfx.com/) | CC0 | Yes, no restriction | **Best single source.** 936 sounds across 78 semantic UI cues (hover, press, success, error, notification, delete, toggle...) in 12 switchable "feels," which is close to a ready-made set of sound schemes on its own. WAV/OGG-convertible, no attribution needed. |
| [Kenney UI Audio pack](https://kenney.nl/assets/ui-audio) | CC0 | Yes | 50 files, zip download, from a long-trusted CC0 game-asset source. Good second source for anything UI SFX doesn't cover well. |
| [OpenGameArt CC0 Sounds Library](https://opengameart.org/content/cc0-sounds-library) | CC0 | Yes | Aggregated CC0 collection, useful backup for one-off sounds. |
| [Freesound.org](https://freesound.org/) | Mixed, filterable | Only the CC0-filtered subset | Huge community library. Filter to `license:"Creative Commons 0"` (the site's own advanced search, or the API filter of the same name) before using anything from here, or attribution has to be tracked per sound in a credits file. Best for a specific sound the curated packs above don't have. |
| [Pixabay sound effects](https://pixabay.com/sound-effects/) | Pixabay License | **No** | Free to use, but the license explicitly forbids redistributing sounds "as-is... without creative transformation." That is exactly what bundling a sound file in an installer or a shareable pack is. Do not use Pixabay sounds for anything that ships or gets zipped up for someone else. Fine only for something used once, locally, never redistributed, which is not this project's use case. |

Recommendation: build the app's own bundled schemes from UI SFX and Kenney
first, both unambiguously CC0, and only reach for a filtered Freesound search
if a specific event needs a sound neither covers well. Document the CC0
sources used in a `CREDITS.md` even though CC0 needs no attribution legally;
it is a courtesy, and it also gives a future contributor an example of what
"safe to bundle" looks like.

**Format decision**: officially support WAV and OGG for both the bundled
schemes and community packs. Not MP3: it is fine, patent concerns are moot
now, but OGG is the fully open format and this project already treats open
formats as a value worth keeping (IMAP, SMTP, CalDAV, iCalendar), not a
technicality. rodio decodes both without extra work either way.

## New events proposed

The feedback module's own comment already names the failure mode to avoid:
"An open 'signal this string' call is how a codebase ends up with forty
near-identical sounds that nobody can tell apart." Every addition below is
proposed against that bar, not just against "would this be nice to have."

| Event | What it signals | Why it earns its own sound rather than riding an existing one |
|---|---|---|
| `HasAttachment` | Landing on a message that carries an attachment | The one requested by name. Mirrors `ThreadLanded` exactly: a fact a sighted user gets at a glance from a paperclip icon, with nothing today giving an equally fast signal without reading the row. |
| `AccountNeedsAttention` | An OAuth token or credential needs re-authorizing | Distinct from `ConnectionLost` on purpose: a dropped connection asks you to wait, an expired sign-in asks you to act. Conflating them under one tone tells the wrong story about what to do next, and this project has already shipped one real bug (Sign In Again) that came from exactly this kind of signal being unclear. |
| `Confirmed` | A toggle or a small action completed the way you asked | Covers flag/unflag, mark done/not done, pin/unpin, and future toggles like them under one event rather than one each. This is the one place the discipline matters most: five toggle actions could easily become five near-identical "did it" tones, which is precisely the forty-dings failure this module exists to prevent. One shared, positive, short confirmation tone for "the thing you asked for happened" is enough. |
| `NothingFound` | A search or filter completed and matched nothing | Distinct from `SyncComplete`'s neutral "an operation finished": coming up empty is a meaningfully different fact, gentle rather than alarming, and its own tone means someone does not have to listen to the whole sentence to know a search came back empty. |

That brings `Event::ALL` from 12 to 16. Exact tone placement (hertz, length)
is intentionally not decided here. `Reminder`'s own tone was proposed,
written down as "proposed numbers, whether the two are tellable apart by ear
is a listening pass," and adjusted from there; each of these four should get
the same treatment; a placeholder tone plus the existing uniqueness test is
the right amount of rigor before a real listening pass, not a substitute for
one.

Considered and left out: an autosave earcon (autosave can fire often enough
to become noise, which is the opposite of what this system is for) and a
separate tone per toggle type (the reason `Confirmed` exists instead).

## Sound schemes

A scheme maps event keys (the existing `Event::key()` strings: `new_mail`,
`sync_complete`, and so on, reused rather than inventing a second name for
the same thing) to a sound source. Two kinds of source:

- **Generated**: today's synthesized tone, played through rodio from an
  in-memory buffer rather than `Beep()`. This is what "Generated tones," the
  built-in default scheme, uses for every event, and it is also the fallback
  for any event a real scheme does not cover.
- **File**: a WAV or OGG file on disk, in the scheme's own directory.

A scheme need not cover every event. Missing ones fall back to Generated,
the same graceful-degradation choice `FeedbackSettings::from_stored` already
makes for a setting it does not recognise, so a half-finished community pack
degrades to silence-free defaults rather than to actual silence for the
events it skipped.

### Where a scheme lives

`dirs::data_dir()` (already a dependency), a `sound_schemes/<scheme-id>/`
folder per scheme, one manifest and its sound files inside. Built-in schemes
ship inside the application itself, read-only; imported ones live here,
writable, survive an upgrade, and are what an uninstaller's "remove my data"
step would need to know about.

### The manifest

TOML, matching this project's own existing convention (`oauth.toml`) rather
than a second file format for the same kind of thing:

```toml
name = "Soft Chimes"
author = "Someone"
license = "CC0-1.0"
description = "Gentle, low-volume tones for a quiet office."
version = 1

[sounds]
new_mail = "sounds/new_mail.wav"
message_sent = "sounds/message_sent.wav"
send_failed = "sounds/send_failed.ogg"
# any event left out falls back to Generated
```

### Importing a zip

Settings, Feedback tab, "Import sound scheme...", a native file picker, the
same shape as the existing vCard import. What happens next has to treat the
zip as what it is: content from an unknown third party, the same "untrusted
input stays untrusted" rule this project already applies to remote HTML.
Concretely:

1. **Size cap on the zip itself** before it is touched, generous enough for
   dozens of short clips (a working number to start from: 20 MB) and refused
   outright above it.
2. **Zip-slip protection.** Every entry's path is sanitised and joined under
   the scheme's own extraction directory; an entry naming `../../` or an
   absolute path is refused, not silently corrected.
3. **Decompression-bomb protection.** A per-file extracted-size cap (a
   working number: 5 MB, generous for a short sound) and a total cap across
   the whole pack (50 MB), refused rather than truncated if either is
   exceeded.
4. **A required, parseable manifest.** No `scheme.toml`, or one that fails to
   parse, is a refused import with a clear reason, not a partial one.
5. **Only `sounds/*.wav` and `sounds/*.ogg` are read as sound.** Anything
   else in the zip is ignored with a warning rather than failing the whole
   import over one stray file.
6. **Every sound file is parsed as real audio before it is trusted**, not
   waved through because its name ends in `.wav`. A file that fails to parse
   as the format its extension claims is refused.
7. **The duration cap that makes "short" a rule and not a suggestion.** A
   working number: 2 seconds. Refused with a clear message naming which file
   and how long it actually ran, not silently truncated, because a silently
   shortened sound is a sound nobody chose.

On success: a preview screen, the scheme's name and author, how many of the
16 events it covers, and a way to hear one sample before committing to it.
Then it is written into `sound_schemes/`, and it shows up in Settings next to
Generated tones and whatever else is already installed.

## Where packs come from

No hosting exists today; this repository has no site, no GitHub Pages, no
release channel for anything but the application itself. Two shapes for
"our project site," in order of how much has to be built before the first
pack can be shared:

**Start here: a GitHub-only distribution.** A directory in this repository,
or a small companion one, where a submitted pack is a pull request (a
directory with a manifest and sounds, reviewed against the same caps and
checks above before merging), published as a zip attached to a GitHub
Release. Nothing new to host; GitHub already does it. `docs/` gains a page
explaining the manifest format for anyone who wants to build one, the same
way `oauth.toml.example` documents a format today.

**Later, if it earns it: a small static page.** GitHub Pages, still backed by
the same GitHub-hosted zips, just with previews and descriptions instead of a
bare file listing.

**Deliberately not proposed for the first version: an in-app browser for
packs**, fetching and offering to install something over the network from
inside the running application. That is a materially bigger question than
importing a file the user already chose and already has on disk. It changes
who initiated the fetch, raises "should this be signed" and "should this be
rate-limited" questions this plan has not answered, and none of that blocks
shipping local-file import, which is most of the value on its own. Worth
its own plan later if there is real demand.

## Settings changes

The Feedback tab gains:

- A scheme picker (a dropdown or list), defaulting to Generated tones.
- "Import sound scheme..." opening the file picker described above.
- Per-scheme coverage shown somewhere reachable (which of the 16 events this
  scheme actually has sounds for), so choosing a scheme is an informed choice
  and not a guess.

The per-event, per-channel override grid this project's own changelog already
says has "no interface yet" is unaffected by any of this and stays its own,
separate piece of work.

## Testing strategy

Most of this is genuinely testable without a real speaker, the same way the
existing feedback module already is:

- **Manifest parsing**: pure TOML-to-struct, tested with valid, missing-field,
  and malformed fixtures.
- **The import defences**: zip-slip, the size caps, the duration cap, each
  tested against a hand-crafted hostile fixture built for that one check, the
  same discipline this project already applied to HTML sanitisation (a
  maintained hostile-snippet corpus, not a single happy-path test).
- **Scheme resolution**: given a scheme missing some events, does event X
  correctly fall back to Generated. Pure, no audio involved.
- **Tone/asset uniqueness**: extending the existing
  `test_every_event_has_its_own_tone`-style check to the new events once
  their placeholder tones exist.

**What stays honestly untestable, same boundary this project already draws
around `Beep()` today**: whether a sound actually comes out of a speaker.
That is a listening pass, not a unit test, and the plan should not pretend
otherwise.

## Phased rollout

Each phase should land, get verified, and stand on its own rather than
waiting on the whole plan to be done before anything ships.

1. **Swap the playback backend to rodio.** Generated tones only, no new
   events, no schemes yet. This alone closes the Windows-only limitation
   ahead of the port needing it, which is worth doing first regardless of
   what happens to the rest of this plan.
2. **The scheme data model, plus one or two bundled schemes** built from the
   CC0 sources above, and the Settings picker to choose between them. Still
   no import.
3. **The four new events**, each wired end to end with its own TDD cycle, a
   placeholder tone, and a place in the bundled schemes.
4. **Zip import**: manifest parsing, the defensive extraction checks, the
   preview screen.
5. **The GitHub-based hosting story and the pack-author documentation.**

## Open decisions

- Whether "our project site" means the GitHub-only approach above or
  something more, and if more, what already exists that this plan does not
  know about.
- Whether two bundled schemes is the right number to start with, or one is
  enough until real usage says otherwise.
- The exact working numbers above (20 MB zip cap, 5 MB/50 MB extraction caps,
  2 second duration cap) are reasoned starting points, not measured ones;
  worth revisiting once real community packs exist to test them against.
