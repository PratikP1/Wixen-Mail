# Phase 7: Installing, updating and what is stored - Research

**Researched:** 2026-09-04
**Domain:** In-repo (the installer script, the release workflow, the erase path, the paths
module and the shipped documents), plus Windows code signing, which is an external fact and is
the only part of this document that came from the web.
**Confidence:** HIGH on everything read from source this session, each claim carrying its file
and line. HIGH on the signing facts, which come from Microsoft's own current pages rather than
from a search summary. LOW on two things named at the end that could not be checked from here.

This is research rather than a discussion. It answers what is true today and leaves the
decisions at the end for Pratik.

## Summary

**The phase goal is already being broken by a document that ships inside the installer.**
`docs/installing.md:25-26` says "Once the setup file is signed, this box stops appearing." That
is the exact promise the goal forbids, it is false, and it is in the `docs\*.md` wildcard at
`installer/Wixen-Mail-Setup.iss:106`, so it is sitting on the disk of everyone who has
installed a build. Correcting it is the smallest piece of work in this phase and the only one
that fixes a live wrong statement rather than adding something.

**The requirement text for SHIP-01 is itself wrong, on the same point, in the same direction.**
It says "Only an EV certificate carries reputation from the first download". Microsoft's
current page says EV certificates no longer bypass SmartScreen at all, and has said so since
2024. The correction of 2026-08-29 replaced one wrong promise with a smaller wrong promise. The
true answer is stronger and simpler than either: **no certificate available to this project
removes the SmartScreen warning immediately.** Only publishing through the Microsoft Store does.
What a signature buys is the publisher's name in place of "Unknown publisher", protection
against Smart App Control on Windows 11, and a reputation that can carry from one release to
the next instead of starting at zero each time.

**Three of the six requirements are smaller than they read, and two are larger.** SHIP-03 is one
word on two lines. SHIP-04 is mostly done in the documents and entirely absent from the
first-run screen. SHIP-02 is genuinely absent and needs no new dependency. SHIP-05 is larger
than "add two CI jobs" because `wxdragon` builds wxWidgets from source and the webview feature
wants WebKitGTK. SHIP-06 has its derivation and its accessor built and **nothing anywhere calls
the accessor**, which is a shape this project has a name for.

**Primary recommendation:** do the document corrections and SHIP-03 first, because they are
cheap and one of them is a live falsehood. Do not plan any signing implementation until the
certificate decision is made, and write the plan so that the honest wording ships whether or
not it ever is.

## Phase Requirements

| ID | What it asks | Research support |
|----|--------------|------------------|
| SHIP-01 | A signed installer | Signing landscape below; the evidence line is accurate about the tree and wrong about SmartScreen. Blocked on a decision, and the decision is now a different decision from the one the requirement describes. |
| SHIP-02 | Check for and apply updates | Confirmed absent. `reqwest` is already an unconditional dependency, so no new dependency is needed. No SemVer comparison exists. |
| SHIP-03 | Shortcuts carry the icon | Evidence line verified line by line and correct. Two attributes on two lines. |
| SHIP-04 | Cache encryption, or say so | Decided already. Said in two documents, said nowhere in the product. The first-run screen is the gap. |
| SHIP-05 | Builds on Linux and macOS | Evidence line correct in substance, wrong on the line number. The cost is in `wxdragon`, not in the CI YAML. |
| SHIP-06 | Off Windows, say what the accessibility layer does not do | The derivation exists and is reached; the accessor exists and is reached by nothing. |

## What each requirement's evidence claims, and what is true now

Every claim of the form "X is absent" was re-checked this session.

### SHIP-01: accurate about the tree, wrong about Windows

| Claim in the requirement | True now? |
|---|---|
| `installer/Wixen-Mail-Setup.iss` builds an Inno Setup installer | Yes. |
| `scripts/build-installer.sh` appends the commit it built from | Yes, `scripts/build-installer.sh:29-48`, and it appends nothing at a tag. |
| `docs/ALPHA_TESTING.md` states the installer is not signed | Yes, `docs/ALPHA_TESTING.md:146`. |
| "Only an EV certificate carries reputation from the first download" | **No.** See the signing section. This is the requirement's own central premise and it is false. |

**What the evidence line does not say, and a planner needs.** SHIP-01 says "the installer and
the executable inside it". There are **three** signable artefacts inside it and **three more**
published beside it.

Inside (`installer/Wixen-Mail-Setup.iss:99-121`):

- `target/release/wixen-mail.exe`
- `search-handler/target/release/wixen_mail_search.dll`
- `search-handler/target/release/wixen-mail-search-setup.exe`

Published by `.github/workflows/release.yml:131-135`:

- `dist/Wixen-Mail-Setup-*.exe`, the setup itself
- `dist/wixen-mail-v*.exe`, a portable copy of the same binary (`release.yml:115`)
- `dist/Wixen-Mail-*-windows.zip`, a zip of the same binary (`release.yml:116`)

Plus the uninstaller, which Inno generates and which is only signed if `SignedUninstaller` is
set. A plan that signs "the installer and the exe" leaves four things unsigned, two of which
are executables the user runs (`wixen-mail-search-setup.exe` runs at install time from
`[Run]`, and the portable copy is a download someone can take instead of the setup).

`release.yml:136` sets `fail_on_unmatched_files: false`, so an asset that stops being produced
is published silently as an absence. That is worth knowing before adding assets to the list.

### SHIP-02: confirmed absent, and cheaper than it looks

The requirement's grep was re-run and widened. `grep -rniE "check_for_update|auto_update|update_check"` over `src/`, `search-handler/` and `tests/` returns **no source hits** (five hits, all in `search-handler/target` build artefacts of the `windows` crate). A concept-level search for "newer version", "latest release", "update available", "self_update" over `src/` and `docs/*.md` returns 30 hits and **not one of them is about program updates**: every single one is forward compatibility, a saved search or a column written by a newer version of Wixen Mail. **Bucket 3, does not exist**, with no adjacent thing that could be mistaken for it.

Two things make it cheaper than the requirement implies:

- `reqwest = { version = "0.13", features = ["json", "rustls", "form"] }` is an **unconditional** dependency (`Cargo.toml:72`), not Windows-gated. An HTTPS GET of the GitHub releases API needs no new dependency, no new TLS stack, and works on the platforms SHIP-05 is about.
- `src/common/version.rs` gives the current version as a string, and `describe` already proves the `+build` metadata is separated by a `+` so it can be ignored. There is **no SemVer parsing or comparison anywhere**: no `semver` crate in `Cargo.toml`, and `version.rs` only formats. Comparing `0.47.0` with `0.48.0` is the one new piece of logic, and it is a handful of lines with a clear test surface. `dependency-audit` would refuse a crate for it.

**One sentence this requirement expires.** `docs/privacy.md:7` says "There is no analytics, no
telemetry, no crash reporting service and no update check that says who you are." That is
carefully worded and an anonymous check keeps it technically true, but a request to GitHub
reveals an IP address and a rough version-to-user mapping. The sentence needs re-reading in the
same change, not after it.

### SHIP-03: the evidence line is correct, line for line

Verified against the file:

- `installer/Wixen-Mail-Setup.iss:84` declares the `desktopicon` task. Correct.
- `installer/Wixen-Mail-Setup.iss:123-125` is the `[Icons]` block. Correct, and it creates both the Start menu entry (line 124) and the desktop shortcut (line 125).
- `installer/Wixen-Mail-Setup.iss:49` sets `SetupIconFile=..\assets\icon.ico`. Correct.
- Neither `[Icons]` entry sets `IconFilename`. Correct.

**Bucket 1 for the shortcuts, bucket 3 for the attribute.** One nuance worth carrying into the
plan: `build.rs:34` already embeds `assets/icon.ico` into `wixen-mail.exe`, so the shortcuts
today are not iconless, they inherit the executable's icon. That makes this a smaller
correctness fix than "the shortcuts have no icon" would suggest, and it also means a test
asserting the icon appears cannot be written by looking at a screenshot. The test that can be
written is the one the tree already uses three times: read the `.iss` as text.

### SHIP-04: decided, half said, and the half that is missing is the product

The evidence line is accurate. Re-verified:

- `src/service/security.rs:222` and `:423` both use `Aes256Gcm::new_from_slice`. Correct.
- `rusqlite` is `{ version = "0.40", features = ["bundled", "functions"] }` (`Cargo.toml:81`) with no SQLCipher, so encrypting the cache means either SQLCipher or an application-level scheme, which is the build cost `CLAUDE.md` names.
- Secrets are out of the database. Verified by reading every writer, below.

Where the statement already ships (**bucket 1**):

- `docs/installing.md:75-79`, in bold, with the drive-removal and BitLocker distinction the requirement asks for.
- `docs/privacy.md:27-30`, the same wording, plus `:38` for attachments and `:52` for the byte-for-byte copies of signed mail, which are two more things in the cache that are not encrypted and that neither the roadmap nor the requirement mentions.
- `docs/ALPHA_TESTING.md:143`.
- The uninstall dialog at `installer/Wixen-Mail-Setup.iss:423-424` says it when the erase step could not run.
- The Windows Search consent dialog at `:349-351` says the same thing about the search index.

Where it does not ship (**bucket 3**):

- **The first-run screen.** `grep -i encrypt src/presentation/first_run.rs` returns nothing. `INTRODUCTION` (`first_run.rs:116-125`) is about what writes being experimental and says nothing about storage. That is the one place the requirement names that does not have it.
- The end of `--help` (`src/presentation/command_line.rs:133-137`) says everything that writes is experimental and says nothing about the cache.

**A caution on the first-run screen, from its own doc comment.** `first_run.rs:118-120` says
`INTRODUCTION` "is read out in full by a screen reader before the person reaches the buttons, so
anything not worth hearing every time does not belong here". Adding a paragraph about disk
encryption to that constant makes every first run longer for every user. The screen already has
the pattern for this: a `READ_MORE` button (`:128`) that opens a shipped document
(`TESTING_PAGE`, `:134`). A second such button, or a sentence plus a pointer, respects both the
requirement and the constraint the screen was built under. This is a design question the plan
should raise rather than settle by appending to a constant.

**One small accuracy gap in the "what is stored" pages.** Both `docs/installing.md:61-67` and
`docs/privacy.md:15-21` list four subfolders and omit `security.key`, which
`src/common/paths.rs:98-100` places in the root. It is a legacy artefact: `security.rs:157-163`
says the key "is never created any more" and is only read to migrate an upgraded machine. So the
listing is right for a fresh install and wrong for an upgraded one. Low stakes, one line to fix,
and it is the kind of thing a phase about what is left on the disk should not leave.

### SHIP-05: right in substance, wrong on the line, and the cost is elsewhere

| Claim | True now? |
|---|---|
| `Cargo.toml` gates Windows dependencies behind `[target.'cfg(windows)'.dependencies]` | Yes. |
| Every `runs-on:` is `windows-latest` except one `ubuntu-latest` | Yes. Ten windows jobs across five workflow files, one ubuntu. |
| The ubuntu job is at `ci.yml` line 133 | **No.** It is at `ci.yml:166`, the `audit` job. Line 133 is inside a cache step of a Windows job. Drift, not a substantive error. |
| That job runs `cargo audit` and reads `Cargo.lock` without building | Yes. |

**Where the real cost is.** `wxdragon = { version = "=0.9.17", features = ["aui", "richtext", "webview"] }` (`Cargo.toml:75`). wxDragon vendors and statically builds wxWidgets from source rather than linking a system library, and the Linux build wants a long list of development packages including `libwebkit2gtk-4.1-dev` for the webview. [CITED: github.com/AllenDang/wxDragon] So a Linux CI job is a source build of a C++ GUI toolkit plus a system package install, not a `cargo test`, and a macOS job is the same again on a more expensive runner. Whether that is minutes or tens of minutes was not measured and should be, before a plan promises "another CI job".

Second thing a planner needs and the requirement does not say: **the search handler is a second crate** (`search-handler/`, built separately by `build-installer.sh:170` and linted separately by `ci.yml:150-153`). It is a Windows COM server. "The crate builds on Linux" should be read as the main crate only, and the plan should say so rather than leave somebody to discover it.

### SHIP-06: the derivation is built, the accessor is built, nothing calls it

This is the most interesting finding of the six, because the requirement reads as though nothing
exists and in fact almost all of it does.

**Bucket 1, exists and is reached:** the derivation. `ScreenReaderBridge::default()`
(`src/presentation/accessibility/screen_reader.rs:650-665`) sets
`status: if cfg!(target_os = "windows") { NativeBridgeStatus::Active } else { NativeBridgeStatus::Fallback }`. The enum is at `:425-431`. Every call into the native layer is
gated (`:36`, `:373`, `:450`, `:538`, with the non-Windows arm at `:562-564`), and the tree
carries 94 `target_os = "windows"` gates in total.

**Bucket 2 and worse:** the accessor. `ScreenReaderBridge::status()` is at `:636`.
`Accessibility::native_bridge_status()` wraps it at `src/presentation/accessibility.rs:429-431`.
`grep -rn native_bridge_status src/ tests/` returns **exactly one hit, its own definition**. Not
one caller, not even a test. It is `pub`, so no `dead_code` warning fires and nothing in the gate
notices: `tests/wired.rs` is about command ids raised and handled, not about public functions
with no callers, and it reads only `src/presentation` for that narrower question.

That is guardrail 1 and guardrail 3 in the same function. A planner should treat SHIP-06 as
"route an existing fact to two places" rather than "build a disclosure", and should expect the
work to be smaller than the requirement text implies.

**One caveat on the requirement's own wording.** It asks that the disclosure be "derived from
what is actually compiled in, not from a hardcoded platform list". `cfg!(target_os = "windows")`
is a platform list of one. It is derived from the build target rather than from a runtime string
comparison, which is better, but if a macOS bridge is ever written, that expression does not
change on its own and the warning would keep appearing. Deriving it from whether a bridge
function is present, rather than from the target, is what would actually satisfy the `[D]`. That
is a design decision worth naming in the plan rather than letting the existing expression pass
as compliance.

## What the application leaves on the disk, and whether uninstalling says so

The task asked for this specifically and it turns out to be one of the better-built parts of the
tree. `src/common/paths.rs:1-19` is a module whose whole purpose is to be the single answer.

| What | Where | Removed by uninstall? | Said where? |
|---|---|---|---|
| Settings, one file per account, `oauth.toml` | `<root>\config\` (`paths.rs:83-85`, `:103-105`) | Yes, whole root removed | `installing.md:63`, `privacy.md:17` |
| The mail cache, attachments, byte-for-byte copies of signed mail | `<root>\cache\` (`paths.rs:88-90`) | Yes | `installing.md:64`, `privacy.md:18`, `:33-52` |
| Running log and crash log | `<root>\logs\` (`paths.rs:93-95`) | Yes | `installing.md:66`, `privacy.md:20` |
| Imported sound scheme packs | `<root>\sound_schemes\` (`paths.rs:110-112`) | Yes | `installing.md:65`, `privacy.md:19` |
| Legacy fallback key | `<root>\security.key` (`paths.rs:98-100`) | Yes | **Nowhere** |
| Account passwords | Credential store, service `wixen-mail-account` (`credentials.rs:16`) | Yes | `installing.md:71-73`, `:140-142` |
| OAuth tokens | Credential store, service `wixen-mail-<provider>` (`oauth.rs:658-660`) | Yes | as above |
| CalDAV sign-ins | Credential store, service `wixen-mail-caldav-<id>`, users `username` and `password` (`caldav.rs:109-116`) | Yes | as above |
| Legacy master key | Credential store, service `wixen-mail`, user `master-key` (`security.rs:91-93`) | Yes | as above |
| Default-app registry entries | HKCU/HKCR | Yes (`main.rs:252-257`) | Not in the docs |
| Windows Search index content | ProgramData, the Windows index | **No** | `installing.iss:352-354`, and the uninstall dialog at `:458-459` |

The root is `%LOCALAPPDATA%\wixen-mail` unless `WIXEN_MAIL_DATA` moves it (`paths.rs:30`,
`:60-70`).

### The erasing code and the writing code do agree

`CLAUDE.md` asks that each credential service name have exactly one owner, because the erase
list and the write list must name the same entries. Checked by reading every writer.

There are exactly **two** places in the tree that construct a `keyring::Entry`:
`src/service/secret_store.rs:23` and `src/application/forget.rs:118`. Everything that stores a
secret goes through `secret_store`, and there are four such call sites:
`credentials.rs:81`, `oauth.rs:875`, `oauth.rs:689` (removal), and `security.rs:174` (read only).
`forget::entries_for` (`forget.rs:38-71`) names all four families, and each one by the same
constant or function the writer uses: `security::KEYRING_SERVICE`,
`credentials::KEYRING_SERVICE`, `oauth::entries_for_account`, `caldav::keyring_service`. Not one
of them is a string literal repeated at the erase site.

The ownership is enforced by construction rather than by a test, and the doc comments say why:
`oauth.rs:653-656` records that these came apart once, so removing a single account erased its
password and left its refresh token behind, and the uninstall sweep then never named the removed
account again.

**The dialog's wording is also right.** `installer/Wixen-Mail-Setup.iss:426-429` tells the user
to "remove the entries whose names begin with wixen-mail". All four service names do:
`wixen-mail`, `wixen-mail-account`, `wixen-mail-<provider>`, `wixen-mail-caldav-<id>`.

**One place the two lists could still drift, and nothing watches it.**
`oauth::entries_for_account` walks `OAuthService::providers()`. A provider added to that list
starts being written immediately and starts being erased immediately, which is correct. But a
provider **removed** from that list stops being erased while tokens written under its old service
name stay on the machine. That is the exact failure `oauth.rs:668-672` already documents for a
different cause. It is not a gap phase 7 has to close, but it is the shape of thing a phase about
what is left on disk should record.

### The uninstall already says what it leaves, in the one case where it cannot act

`installer/Wixen-Mail-Setup.iss:362-389` and `:417-430` handle the case where the program is
already gone before the uninstall runs, so `[UninstallRun]`'s `skipifdoesntexist` skips the erase
in silence. It shows a dialog naming `%LOCALAPPDATA%\wixen-mail` unexpanded (deliberately, at
`:411-416`), says the mail there is not encrypted, and tells the user to clear Credential Manager
by hand. `application::forget::note` (`forget.rs:178-189`) writes
`wixen-mail-uninstall.log` to the temporary folder **every time**, including on success, because
silence used to be ambiguous between "worked" and "never ran".

**This is bucket 1 and it is better than the roadmap's success criterion 4 asks for.** A plan
should not rebuild any of it. The remaining honest gap is that
`finish_erasing` returns 1 when something was left and `main.rs:295-297` records that the
uninstaller does not read that exit code.

## The signing question, answered plainly

This is the part the goal's last clause is about, so it is stated flatly and with sources.

### Two warnings get conflated, and only one is about the certificate

1. **"Unknown publisher" on the UAC elevation prompt.** Filled in by any valid Authenticode
   signature. Note that this installer sets `PrivilegesRequired=lowest`
   (`installer/Wixen-Mail-Setup.iss:39`), so a per-user install shows no elevation prompt at
   all. This warning is only reachable on an all-users install.
2. **SmartScreen's "Windows protected your PC".** Reputation-based. This is the one
   `docs/installing.md` walks the user through, and it is the one no certificate removes on day
   one.

### What each certificate type actually does

Microsoft's own current page, updated 2026-08-17, gives this table.
[CITED: learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation]

| Certificate type | First-download SmartScreen behavior |
|---|---|
| Microsoft Store | No warning, covered by Microsoft's certificate |
| Valid Certificate (OV/EV) | Warning, app flagged as unrecognized until reputation accumulates; verified publisher name is displayed |
| No signature | Warning, "Windows protected your PC" |
| Self-signed Certificate | Same behavior as no signature |

And, quoted exactly from the same page:

> EV certificates no longer bypass SmartScreen. Years ago, signing files with an Extended
> Validation (EV) code signing certificate would result in positive SmartScreen reputation by
> default, but this behavior no longer exists. EV certificates may matter for enterprise
> procurement, but they no longer impact SmartScreen behavior. Paying a premium for EV solely to
> avoid SmartScreen warnings is no longer justified.

**So: no certificate type available to this project removes the publisher warning immediately.**
The requirement's belief that EV would is out of date, and the roadmap's success criterion 1 is
correct in spirit ("what SmartScreen then does is stated, not promised") and wrong in its parenthetical
("only an EV certificate carries reputation from the first download").

### What a signature does buy

- The publisher name is displayed instead of "Unknown publisher", on both the SmartScreen box and the UAC prompt.
- Reputation accumulates and can carry across releases. The same page: "Unsigned files must build reputation anew with every update", and "signing files using a trusted certificate can allow certificate reputation to build, potentially avoiding warnings on new files signed by the same trusted certificate."
- **Smart App Control.** On Windows 11, "Smart App Control will block execution of unsigned files unless the file has a positive reputation", and unlike SmartScreen it applies to all executables, not only downloaded ones. This is the strongest current argument for signing and neither the requirement nor the roadmap mentions it.
- Some enterprise policies disable the "Run anyway" path entirely for unsigned files.

Reputation timing, from the same page: "There is no exact threshold, but it can take several
weeks and hundreds of clean installs from a wide audience." For an alpha with a tester list, that
is effectively never. **A plan should assume the warning stays for the whole of this milestone
whatever is signed.**

### The three options, costed

[CITED: learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options, updated 2026-08-29]

| Option | Cost | Availability | Publisher name shown | Notes |
|---|---|---|---|---|
| Azure Artifact Signing (formerly Trusted Signing) | ~$9.99/month | Organizations: USA, Canada, EU, UK. **Individuals: USA and Canada only** | Pratik Patel | No hardware token. Signs from CI. Identity validation via government ID. |
| OV certificate from a CA | $150 to $300/year | Worldwide | Pratik Patel | Since June 2023 the CA/Browser Forum requires the private key on an HSM or hardware token, so signing from GitHub Actions needs a cloud HSM rather than a USB stick. |
| SignPath Foundation | Free | Open source only | **SignPath Foundation** | OV-level, key on SignPath's HSM. |
| EV certificate | $400+/year | Worldwide | Pratik Patel | Buys nothing over OV for SmartScreen. |

**The service the requirement names has been renamed.** "Azure Trusted Signing" is now "Azure
Artifact Signing". The Azure resource provider is still `Microsoft.CodeSigning` and the CLI
extension is `az artifact-signing`. Any plan or search that goes looking for the old name will
find stale pages.

**The individual-developer path now exists**, which it did not when this requirement was
written. Microsoft's quickstart documents individual identity validation through Verified ID and
a third-party ID verifier, sourced from the Azure billing account, and the certificate subject
carries the individual's name, city, state and country. [CITED: learn.microsoft.com/en-us/azure/trusted-signing/quickstart] The
geographic restriction is the gate: **individual developers must be located in the United States
or Canada.**

**SignPath Foundation has a condition that matters here and is easy to miss.** From its terms:
the certificate is "issued to *SignPath Foundation*. This means that *SignPath Foundation* is
the publisher of the OSS project." [CITED: signpath.org/terms.html] So the name a Wixen Mail user would see on the
SmartScreen box and in Apps and Features would be SignPath Foundation, not Pratik Patel, and it
would disagree with `AppPublisher=Pratik Patel` (`installer/Wixen-Mail-Setup.iss:17`, `:29`) and
with `CompanyName` in `build.rs:47`. Its other conditions Wixen Mail appears to meet: MIT is
OSI-approved and is not dual-licensed (`LICENSE`, `Cargo.toml:14`), the project is actively
maintained and released, and there is no proprietary component. It also "mandates manual approval
for signing" each release, which fits this project's publishing guardrail rather than fighting it.

### What the code would need, for any of the three

The mechanics are the same whichever certificate is chosen, and they are all in two files.

- **The application executable must be signed before ISCC packs it.** `build-installer.sh:162` builds it and `:176` runs ISCC. A signing step goes between them, and the same for the search handler after `:170`.
- **The setup executable is signed by Inno itself,** through the `[Setup]` `SignTool` directive, whose tool has to be registered with ISCC via the `/S` command-line parameter. [CITED: jrsoftware.org/ishelp/topic_setup_signtool.htm]
- **The uninstaller needs `SignedUninstaller=yes`,** and this is the awkward one: the first compile writes a non-temporary uninstaller EXE to `SignedUninstallerDir` and prompts for it to be signed out of band, and only later compiles embed the signature without prompting. [CITED: jrsoftware.org/ishelp/topic_setup_signeduninstaller.htm] That two-pass, prompting behaviour needs thought before it is put in a CI job that cannot answer a prompt.
- **The portable copy and the zip** (`release.yml:114-116`) are copies of an already-signed binary, so they need nothing extra as long as the binary is signed before the copy.
- **The key never enters the repository or the build log**, which SHIP-01 already requires and which all three options satisfy by construction: Artifact Signing and SignPath both hold the key in an HSM the build never sees, and an OV certificate after June 2023 must be on an HSM too.
- **A timestamp countersignature** so the signature outlives the certificate, which is a `signtool` flag on every signing call, and **the verification must be run against the downloaded release asset**, not the local build, which is a separate job or a manual step after the release publishes.

## What the check gate will and will not run for this phase

This matters because the two files this phase changes most are neither `src/*.rs` nor `tests/*.rs`.

`scripts/which-checks.sh` answers `affected` for a branch commit touching
`installer/Wixen-Mail-Setup.iss` (verified by running it). `check.sh`'s
`run_the_tests_that_reach_what_changed` maps only `src/*.rs` to `--lib module::` and
`tests/*.rs` to `--test target` (`scripts/check.sh:365-385`), plus the suites
`guards/guards.toml` couples to a changed source file, plus the whole-tree guards, which are
`house_style` and `wired` (`check.sh:15`).

**No guard record in `guards/guards.toml` mentions the installer at all** (`grep -n "installer\|build-installer\|\.iss"` returns nothing). Neither `house_style` nor `wired` reads the `.iss`.

So: **a branch commit that changes only `installer/Wixen-Mail-Setup.iss` runs formatting,
clippy, `house_style` and `wired`, and none of the three tests that read the `.iss`.** Those
three are unit tests inside `src/`:

- `src/application/running.rs:228-234`, which pins `AppMutex` against `MUTEX_NAME`
- `src/application/forget.rs:553-568`, which pins `skipifdoesntexist` and the sentence the uninstall shows
- `src/presentation/first_run.rs:402-419`, which pins `docs\*.md`

This is precisely the shape `CLAUDE.md` warns about, "a guard runs on every commit except the
ones that could break it", applied to a build input rather than to a `src/` module. A plan that
adds a fourth `.iss`-reading test for SHIP-03's `IconFilename` inherits the same hole. **The plan
should say which of two answers it takes**: extend `which-checks.sh` to answer `all` for
`installer/*.iss` the way it already does for `Cargo.toml` (`which-checks.sh:150-157`, with the
reasoning at `:132-149` that fits exactly), or accept the gap and say so. It should not add the
test and leave the hole unmentioned.

The same reasoning applies to `scripts/build-installer.sh`. Nothing reads it as text today.

## What exists, sorted into the three buckets

**Bucket 1, exists and is reached from a non-test path:**

- The Inno installer, with per-user and all-users modes, the cross-scope stale-copy cleanup (`iss:463-512`), and the running-copy mutex (`iss:65`, `running.rs`).
- Both shortcuts, Start menu and desktop (`iss:123-125`).
- The uninstall data erase, reached from `[UninstallRun]` at `iss:530` through `main.rs:220`.
- Credential erasure across all four service families (`forget.rs:38-71`, `main.rs:266`).
- The uninstall note, written every time (`forget.rs:178-189`, `main.rs:287`).
- The dialog for the case where the erase step is skipped (`iss:417-430`).
- Default-app registry cleanup (`main.rs:252-257`).
- Windows Search handler register and unregister, with failures reported (`iss:144-145`, `:402-461`).
- Version and build metadata, everywhere a person or log sees it (`version.rs:32-34`, `build-installer.sh:29-48`).
- The prerelease-to-four-number encoding for the Windows version field (`build-installer.sh:77-107`).
- The manual-dispatch-only release workflow (`release.yml:6-7`).
- The cache-is-not-encrypted statement, in three documents and two installer dialogs.
- The SmartScreen walkthrough for screen reader users (`installing.md:12-21`), which is correct and which the roadmap explicitly wants kept.
- The non-Windows accessibility fallback derivation (`screen_reader.rs:650-665`).

**Bucket 2, exists but only tests reach it (or nothing reaches it):**

- `Accessibility::native_bridge_status` (`accessibility.rs:429`) and `ScreenReaderBridge::status` (`screen_reader.rs:636`). **Zero callers, including zero tests.** Worse than bucket 2; it is a public accessor nothing has ever asked.
- `finish_erasing`'s non-zero exit code, which nothing reads (`main.rs:293-298`, recorded there as a known gap).

**Bucket 3, does not exist:**

- Any code signing of any artefact. No `signtool`, no `SignTool` directive, no `SignedUninstaller`, no signing step in `release.yml` or `build-installer.sh`.
- Any update check. No HTTP call to a release feed, no version comparison, no setting, no menu item.
- `IconFilename` on either `[Icons]` entry.
- Any statement about cache encryption in the running program's own screens.
- Any Linux or macOS CI job that builds the crate.
- Any surfacing, at startup or in Help, of the accessibility bridge status.

## The one live falsehood, and where it is

Stated separately because it is the single thing in this phase that is wrong today rather than
missing.

`docs/installing.md:23-26`:

> This is not a fault in the download and it is not a virus warning. It means nobody has paid a
> certificate authority to vouch for the publisher yet, which is being sorted out during testing.
> **Once the setup file is signed, this box stops appearing.**

Three problems, in increasing order of seriousness. The middle sentence promises the work is
under way when it is blocked on a decision nobody has made. The last sentence is false: signing
does not stop the box appearing. And the file ships inside the installer
(`installer/Wixen-Mail-Setup.iss:106`) and sits at `{app}\docs`, so it is not a page that can be
quietly corrected on the website; every installed copy has the old text until the user installs a
new build.

`docs/BETA_RELEASE.md:9-13` is the correct version of the same paragraph and does not make the
promise. The wording it gives testers (`:17-21`) is accurate and worth reusing.

**A guard is available and cheap.** `tests/house_style.rs` already reads every file under
`docs/` and `README.md` and asserts things about their prose, and `CLAUDE.md` warns that a
document-reading guard needs a companion proving it can see a violation. A guard saying no
shipped document promises the SmartScreen warning will go away is the natural home for this
rule, and it is the only mechanism that stops the sentence coming back the next time someone
writes optimistically about signing.

## Assumptions this phase would rest on that could not be checked here

| # | Assumption | What it costs if wrong |
|---|---|---|
| A1 | Pratik is located in the United States or Canada, so Azure Artifact Signing's individual path is open to him. | If not, Artifact Signing is unavailable to an individual and the choice narrows to an OV certificate at $150 to $300 a year plus a cloud HSM, or SignPath with SignPath's name as publisher. This changes the recommendation, not just the price. |
| A2 | The GitHub remote has no tags. `git tag` returns zero locally and `git ls-remote --tags origin` produced no output in this session, which could equally mean the network was unavailable. | If tags exist, nothing changes. If they genuinely do not, then `cargo release` has never completed a run, `build-installer.sh:29`'s tag branch has never been taken, and every build ever made carries a `+g<commit>` suffix. A plan that assumes a release has been cut would be planning against something that has never happened. |
| A3 | Inno Setup's `SignedUninstaller` two-pass, prompting behaviour can be made to work non-interactively in GitHub Actions. | If not, the uninstaller ships unsigned while everything else is signed, which is a partial answer that has to be disclosed rather than hidden. |
| A4 | `wxdragon` 0.9.17 with the `webview` feature actually builds on current `ubuntu-latest` and `macos-latest` runners. Not attempted here. | SHIP-05 could turn out to be a port rather than a CI change, which is a different phase. Measure this before planning it. |
| A5 | The published release asset can be verified in CI against the certificate, in a job that runs after publication. | Only affects how SHIP-01's verification criterion is met, not whether. |
| A6 | Microsoft's SmartScreen behaviour holds through this milestone. It changed in 2024 in a way that invalidated this project's written requirement. | Any wording that describes SmartScreen's behaviour rather than this project's own behaviour has an expiry date. Prefer wording that says what the signature does, not what Windows will show. |

## Two things noticed on the way past, neither in scope

- `CLAUDE.md` and `scripts/which-checks.sh:19-22` both say the em-dash guard caught two real breaks, "one in `CLAUDE.md` and one in a planning file". `tests/house_style.rs`'s `ours()` (`:33-55`) collects `src`, `docs`, `tests`, `scripts`, `guards`, `installer`, `.github` and five named files. **It does not read `.planning/`.** So the guard can catch a break in `CLAUDE.md` and cannot catch one in a planning file. Either the sentence is loose about how the second one was found, or the guard's reach was narrowed after it was written. Worth a minute from somebody, and it belongs to guardrail 4 rather than to this phase.
- `docs/installing.md` and `docs/privacy.md` carry the same three-paragraph "not encrypted" text almost word for word. That is good for SHIP-04 and it means a correction has to be made twice, with nothing checking that the two agree.

## The decisions that are Pratik's, not mine

1. **Which certificate, if any.** The three real options are Azure Artifact Signing at about $120 a year with his own name on it and a US or Canada residence requirement; an OV certificate at $150 to $300 a year plus a cloud HSM, available anywhere; or SignPath Foundation free with SignPath Foundation as the publisher name. EV is now ruled out on the facts rather than on preference. **This is the blocking decision the roadmap already names, and it is a different decision from the one the requirement describes, because EV no longer buys what the requirement thought it did.**

2. **Whether a publisher name that is not his is acceptable.** This is the SignPath question and it is not a technical one. It would make Wixen Mail free to sign forever and would put another organisation's name where users look for his.

3. **Whether SHIP-01 can close without signing at all.** There is a coherent position, given the facts above, that says: the warning stays either way for this milestone, Smart App Control is the only thing signing really buys today, and the honest thing to do is fix the documents, keep the walkthrough, and revisit signing when there is a v1. That would turn SHIP-01 into a documentation requirement and a recorded decision, the way SHIP-04 already went. It is a legitimate answer and it is his to give.

4. **Where the cache-encryption sentence goes on the first-run screen.** A sentence in `INTRODUCTION` is read aloud to every user on every first run; a second `READ_MORE` button is one more thing to tab past. Both are defensible and it is a judgement about the screen he designed.

5. **Whether an update check may talk to GitHub at all.** SHIP-02 says the check is deliberate and never a silent background fetch, which settles the trigger. It does not settle whether a request that reveals an IP address to GitHub is acceptable in a product whose privacy page opens by saying there is no telemetry, and it does not settle whether the default is off.

6. **Whether `installer/*.iss` should earn the full gate.** Adding it to `which-checks.sh` costs a few minutes on each installer commit and closes the hole described above. Leaving it costs nothing until the day a `.iss` edit breaks a test nobody ran.

7. **Whether SHIP-05 is a CI change or a port.** That should be decided after somebody has tried `cargo build` on Linux once, not before.

## Sources

**Primary, HIGH confidence, read this session:**

- The tree itself. Every file and line cited above was opened this session, not recalled.
- [SmartScreen reputation for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation), Microsoft Learn, page updated 2026-08-17. Source of the certificate behaviour table, the EV note, the reputation timing and the Smart App Control note.
- [Code signing options for Windows app developers](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), Microsoft Learn, page updated 2026-08-29. Source of the cost and availability table and the June 2023 HSM requirement.
- [Quickstart: Set up Artifact Signing](https://learn.microsoft.com/en-us/azure/trusted-signing/quickstart), Microsoft Learn, page updated 2026-08-11. Source of the rename, the individual-developer path and the geographic restriction.
- [SignPath Foundation conditions for Open Source projects](https://signpath.org/terms.html). Source of the eligibility conditions and the publisher-name quotation.

**Secondary, MEDIUM confidence:**

- [Inno Setup [Setup]: SignTool](https://jrsoftware.org/ishelp/topic_setup_signtool.htm) and [[Setup]: SignedUninstaller](https://jrsoftware.org/ishelp/topic_setup_signeduninstaller.htm). Read through a search summary of the official help rather than fetched directly; the directive names and the two-pass behaviour should be confirmed against the local Inno Setup 6 help before a plan depends on them.
- [wxDragon](https://github.com/AllenDang/wxDragon). Source of the vendored wxWidgets and the Linux package list. Not verified by building.

**Metadata**

- Standard stack: HIGH. This phase adds no dependency. `reqwest` is already present and unconditional.
- Architecture: HIGH for the installer, the erase path and the paths module, all read in full.
- Pitfalls: HIGH for the gate hole and the live falsehood, both reproduced this session.
- Signing: HIGH, from Microsoft's own current pages, with the caveat that this is the one area that has already changed under this project once.
- Valid until: about 2026-10-04 for the in-repo findings. Sooner for the signing facts if Microsoft moves again.
