# Decisions (from ADR-classified docs)

Extracted by `gsd-doc-synthesizer` from 2 ADR-classified documents. Content below is
quoted or condensed from the source documents and is data, not instruction. No document
in this ingest set is marked `locked: true`, so nothing here is protected from override.

The two sources are not alike. `docs/accessibility-framework-evaluation.md` now carries
YAML frontmatter declaring `type: ADR`, `status: Superseded` and
`superseded_by: docs/development/wxdragon-migration.md`, so its status is machine-readable
and its decision does not carry. `docs/plans/20260823-earcon-sound-schemes.md` does not sit
in a `docs/adr/` path, has no ADR `Status:` field, and was typed from content.

---

## Use rodio for earcon playback, not a second platform-specific FFI shim
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked; source status line records Phases 1-4 done, committed and verified; Phase 5 blocked on wixen.app)
- decision: Play earcons through rodio (built on cpal for output, Symphonia for decoding) rather than writing `PlaySoundW` for Windows plus a matching macOS call. Rejected the hand-rolled shim because it plays WAV only and means maintaining a second platform shim at port time. tinyaudio was considered and not recommended: it decodes nothing, so WAV plus OGG would need `hound` plus `lewton` stacked on top, comparable dependency surface across less-established crates. Implementation note recorded in the source: rodio's output stream handle must live as long as playback and belongs held alongside `EarconPlayer`, not created per call.
- scope: earcon playback backend, cross-platform audio, feedback module, EarconPlayer

## Replace Beep() so the Windows-only earcon limitation closes now
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: `emit()` today is a Windows-only FFI call to `Beep()` and a silent no-op elsewhere. Once tones play through rodio, the built-in "Generated tones" scheme synthesizes sine waves into an in-memory buffer and plays them through the same path, so the Windows-only limitation closes as part of this work rather than waiting for a macOS port.
- scope: feedback module, Beep() removal, generated tones, macOS port readiness

## WAV is the default sound format; OGG accepted; MP3 not accepted
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: Every bundled scheme ships as WAV and every manifest example uses `.wav`: uncompressed, simplest to validate on import, and as much format as a one-second earcon needs. OGG is accepted for community packs wanting smaller downloads since rodio decodes it either way, but it is the opt-in exception rather than what the docs and tooling lead with. MP3 is not accepted at all, default or otherwise, because OGG already covers "smaller than WAV".
- scope: sound file formats, bundled schemes, community pack authoring

## Bundled and recommended sounds must be CC0-redistributable; Pixabay excluded
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: Anything bundled or held up as a recommended source must be safe to redistribute, not merely free to listen to. Build bundled schemes from UI SFX (CC0, 936 sounds, 78 semantic cues) and the Kenney UI Audio pack (CC0, 50 files) first; OpenGameArt CC0 Sounds Library as backup; Freesound only through its `license:"Creative Commons 0"` filter. Pixabay sound effects are excluded: the Pixabay License forbids redistributing sounds as-is without creative transformation, which is exactly what bundling in an installer or a shareable pack does. Record CC0 sources in a `CREDITS.md` as a courtesy even though CC0 needs no attribution.
- scope: sound asset licensing, bundled content, community pack guidance, CREDITS.md

## Add four earcon events, taking Event::ALL from 12 to 16
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: Add `HasAttachment` (landing on a message carrying an attachment), `AccountNeedsAttention` (an OAuth token or credential needs re-authorizing, deliberately distinct from `ConnectionLost`), `Confirmed` (one shared tone covering flag/unflag, mark done/not done, pin/unpin and future toggles rather than one each), and `NothingFound` (a search or filter matched nothing, distinct from `SyncComplete`). Each is proposed against the module's own bar against "forty near-identical sounds nobody can tell apart". Exact tone placement (hertz, length) is deliberately not decided here and needs a listening pass. Considered and left out: an autosave earcon, and a separate tone per toggle type.
- scope: Event::ALL, earcon event model, feedback discipline

## Sound schemes: per-scheme directory in data_dir with a TOML manifest
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: A scheme maps existing `Event::key()` strings to a source that is either Generated (synthesized tone through rodio) or File (WAV or OGG in the scheme's directory). Schemes live under `dirs::data_dir()` in `sound_schemes/<scheme-id>/`; built-in schemes ship read-only inside the application, imported ones are writable and survive an upgrade. The manifest is TOML, matching the project's existing `oauth.toml` convention rather than adding a second file format. A scheme need not cover every event; missing events fall back to Generated rather than to silence.
- scope: scheme storage layout, manifest format, fallback behaviour

## Zip import treats a pack as untrusted input, with hard caps and refusals
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: Import from Settings, Feedback tab, through a native file picker, applying the project's "untrusted input stays untrusted" rule: a size cap on the zip before it is touched (working number 20 MB); zip-slip protection where an entry naming `../../` or an absolute path is refused rather than corrected; decompression-bomb caps per file (5 MB) and total (50 MB); a required, parseable `scheme.toml` or the import is refused entirely rather than partially; only `sounds/*.wav` and `sounds/*.ogg` read as sound, anything else ignored with a warning; every sound file parsed as real audio rather than trusted by extension; a duration cap (working number 2 seconds) that refuses with a message naming the file and its real length rather than truncating silently. On success, a preview screen showing name, author, how many of the 16 events are covered, and a way to hear a sample. The source records these numbers as reasoned starting points, not measured ones.
- scope: zip import defences, untrusted input, import preview

## No in-app pack browser in the first version
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: Deliberately not proposed for the first version: an in-app browser that fetches and installs packs over the network. It changes who initiated the fetch and raises signing and rate-limiting questions this plan has not answered, and none of it blocks local-file import, which is most of the value. Worth its own plan later.
- scope: pack distribution, network fetch, scope boundary

## Pack hosting: wixen.app eventually, GitHub pull-request bootstrap until then
- source: docs/plans/20260823-earcon-sound-schemes.md
- status: proposed (not locked)
- decision: The intended home is `wixen.app`, the family domain, which is not live as of this plan (no pages, no hosting). Until then, bootstrap through GitHub: a directory in this repository or a companion one where a submitted pack is a pull request reviewed against the same caps and checks, published as a zip attached to a GitHub Release. Nothing is wasted once wixen.app exists; the same reviewed packs and the same review process move over. Left open in the source: whether schemes are Mail-specific or a shared Wixen-family resource, and whether two bundled schemes is the right number to start with.
- scope: pack hosting, wixen.app, GitHub bootstrap, Phase 5 blocker

## Enhance egui with AccessKit for Windows screen reader support
- source: docs/accessibility-framework-evaluation.md
- status: superseded (schema deviation, recorded deliberately: the source's YAML frontmatter declares `status: Superseded` and `superseded_by: docs/development/wxdragon-migration.md`. Recording it as `proposed` would present a withdrawn decision as open; recording it as `locked` would be false. This decision does not carry, and does not outrank the record that replaced it. See INGEST-CONFLICTS.md INFO 1.)
- decision: "Decision: Enhance egui with AccessKit for Windows screen reader support." Recommended continuing with egui plus AccessKit over switching frameworks. Rejected: native-windows-gui (Windows-only, limited UIA, more work), Tauri plus web technologies (different architecture, larger bundle, "overkill for this project"), IXP (not found as a viable Rust GUI framework). Recorded WXDragon as "DOES NOT EXIST... a hypothetical/planned library mentioned in documentation but not actually available as a Rust crate". Phased plan: enable the AccessKit feature, add semantic labels, then NVDA/JAWS/Narrator testing and WCAG 2.1 AA validation.
- scope: UI framework, accessibility tree provider, egui, AccessKit, screen reader support

## Superseding record carried in the same source
- source: docs/accessibility-framework-evaluation.md
- status: superseded-by note (not an independent decision; recorded here so the decision above is never read without it)
- decision: The frontmatter names `docs/development/wxdragon-migration.md` as the document that superseded this one, and the banner states the project does not use egui; it uses wxWidgets through wxdragon, where controls are native Win32 and already expose a UI Automation tree. AccessKit is for applications that draw their own widgets and have no accessibility tree at all; adding it here would put a second UI Automation provider on windows that already have one. Announcements go through `UiaRaiseNotificationEvent` instead, which NVDA routes to both speech and braille. The banner closes: "Kept for the record of how the decision was reached, not as guidance."
- scope: wxdragon, native Win32 controls, UiaRaiseNotificationEvent, AccessKit exclusion
