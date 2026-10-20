# The update mechanism: ship-shape against what Wixen Mail already has

Research for phase 13.3, plans 13.3-23 and 13.3-24. Written 2026-10-20 on branch `13.3-plans`,
against `main` at `6fd465c6`. It is for Pratik, who decides, and for the planner who writes those
two plans.

Each claim is marked with how it is known:

- **read**: read in the source named, at the commit or version named;
- **measured**: a command was run on 2026-10-20, and the command and its output are given;
- **derived**: worked out from what was read, and never run;
- **not verified**: needs a build, a run or a screen reader, and has had none.

Nothing was built, installed or run. ship-shape and every crate it brings were read as text from
copies downloaded into this session's scratchpad, each one checked against the checksum crates.io
publishes for it. Line numbers in Wixen Mail's files are at `6fd465c6`. Line numbers in ship-shape
are in its published 0.4.1 source.

## The answer

**Do not adopt ship-shape 0.4.1. Keep the update code Wixen Mail has, fix five gaps in it during
phase 13.3, and change no package.**

ship-shape is a real, published crate: version 0.4.1 on crates.io, published 2026-10-09 by
trypsynth (Quin Gillespie), MIT, twelve versions, none yanked, and the downloaded file's SHA-256
matched the one crates.io gives (read). It is carefully made. It checks a signature before it keeps
the file, lets the person cancel the download, deletes a half-finished file, runs one check at a
time, and has tests for the quoting of the script it runs. Its author also wrote `live-region` and
other screen reader crates for wxDragon.

It does not fit Wixen Mail, for three reasons. The first decides it.

1. **It cannot offer the builds Wixen Mail publishes.** It reads `1.0.0-alpha.1`, `1.0.0-alpha.2`
   and `1.0.0` as the same version, and its normal channel asks GitHub a question whose answer
   never includes a prerelease. Every alpha, beta and release candidate Wixen Mail publishes is a
   GitHub prerelease. A tester on alpha.1 would never be offered alpha.2, or the final 1.0.0.
2. **It moves the toolkit every window is built on.** It needs wxDragon 0.9.22 or later, and Wixen
   Mail pins 0.9.17. That move breaks the build in one file, quietly changes when the message
   editor saves a draft on its own, and brings a newer wxWidgets whose native controls only an
   NVDA sitting can judge. Doing it now puts unheard changes into the build the sittings measure.
3. **Its windows and words would replace ones built for a screen reader.** Its dialogs set no
   accessible names, put focus in a read-only text box, ask Yes or No without saying what Yes does,
   close the program without saying so, show English-only technical errors, and never show their
   own security warning.

What phase 13.3 should do instead is fix five gaps in Wixen Mail's own update code (see "Known
gaps"). None needs a package. The two that matter most: once any newer release is published, a
copy that chose a kind of version downloads about 15.7 MB at every start, refuses it, and opens a
dialog; and the handover to a signed installer does not quit the way Quit does when "keep running
in the tray" is on.

The signing question stays where decision 6 of 2026-09-06 put it: Azure Artifact Signing, which
the project needs anyway (SHIP-01 names Smart App Control on Windows 11 as a reason). A second
signing key, minisign, is worth adding only if that account will not exist before anyone but
Pratik needs to update from inside the program.

## What Wixen Mail does today

Two modules do the work: `src/service/update_check.rs` (1,399 lines) finds out whether a newer
version exists, and `src/service/update_download.rs` (2,160 lines) fetches it, checks it, and
starts it. Neither changed between `0b781473`, where phase 13.2 closed, and `6fd465c6`:

```text
$ git diff --stat 0b781473 6fd465c6 -- src/service/update_check.rs \
    src/service/update_download.rs src/common/version.rs \
    src/presentation/wx_app.rs src/application/closing.rs
(no output, exit 0)
```

### Finding a newer version

- The Help menu has Check for Updates (`wx_app.rs:8200-8205`). A check also runs once at start,
  but only if a kind of version was chosen in Settings, General, "New versions" (read).
- The setting has three values: do not look (the default), released versions, and released
  versions and test versions (`src/common/version.rs:275-287`, read).
- Released versions asks GitHub's `releases/latest`, which never answers with a prerelease. Test
  versions asks for the list of releases and orders it itself (`update_check.rs:326-331`, read).
- The comparison reads Wixen Mail's own version strings correctly: `1.0.0-alpha.1` is older than
  `1.0.0-alpha.2`, which is older than `1.0.0`. Everything after `+` is ignored, so two tester
  builds of one version compare equal (`version.rs:89-95`, `:215-229`, read). That is what SHIP-02
  asks for, and it means tester builds are installed by hand, never through the check.
- A 404 on the released channel reads as "nothing published yet", and a rate limit has its own
  sentence (`update_check.rs:413-461`, read).

### Fetching and checking it

- With an offer in hand, the installer is downloaded without asking again. The consent was given
  at the setting, whose description says so (decision 16 of 2026-09-06), or by pressing Check for
  Updates (read).
- It saves into `%LOCALAPPDATA%\wixen-mail\updates`, follows at most five redirects and only over
  HTTPS, and stops past 300 MiB (`update_download.rs:95`, `:109`, `:1082-1168`, read).
- The check is Authenticode: Windows must trust the signature, the signer's name must be "Pratik
  Patel", and the certificate must carry this project's Artifact Signing profile among its usages
  (`update_download.rs:590-981`, read).
- That profile identifier is empty, so every installer is refused, a genuinely signed one included
  (`update_download.rs:76`, read). It stays empty until Pratik's signing account exists (ledgers
  329 and 870).

### Asking and handing over

- One question, only about a file that passed the check: "Wixen Mail {v} is ready to install. It
  was checked and it is signed by Pratik Patel. If you install it now, Wixen Mail will close and
  the installer will open in its place." Enter answers No (`update_download.rs:226-235`,
  `wx_app.rs:24433-24440`, read).
- Yes: a high-priority announcement says Wixen Mail is closing, the installer is started with no
  arguments (so the normal Inno wizard opens), and then the window is closed
  (`update_download.rs:289-303`, `wx_app.rs:24464-24473`, read).
- The installer starts first on purpose, so that a failure to start can still be said. The cost is
  ledger 340: the installer can reach its "Wixen Mail is running" check before Wixen Mail has
  finished closing (read).

### What is spoken and shown

Every outcome goes to the status bar and is announced at high priority on the "command" topic. A
refused or failed download then opens a dialog offering the releases page, where Enter answers No
(`wx_app.rs:24383-24417`, read). A check at start says nothing unless it ends in a download (read).
There is no progress display and no way to cancel the download (read).

### What the alpha's testers meet

No release has ever been published (`update_download.rs:23`, read) and no tag exists here
(measured, below), so every check today answers "nothing published yet".
`docs/ALPHA_TESTING.md:1009` tells testers that updating itself will not work yet, and is meant
not to. Testers install each build by hand, and that stays true for the sittings whatever is
decided here.

```text
$ git tag | wc -l
0
$ git rev-list --count origin/main..main
332
$ ls -l dist/            (the newest of three lines)
-rwxr-xr-x 1 prati 197609 15725012 Oct 20 06:02 Wixen-Mail-Setup-1.0.0-alpha.1+1403.g6fd465c6.exe
```

### Known gaps

The first five need no package and are what this research recommends for 13.3.

| # | Gap | What happens | How it is known |
|---|-----|--------------|-----------------|
| 1 | A refused download repeats at every start | Once a newer release is published, a copy that chose a kind of version downloads the installer at every start (about 15.7 MB today), refuses it because nothing can pass the signer check while the profile identifier is empty, and opens the releases-page dialog. The at-start path sends `AnUpdateDidNotHappen` for a refusal (`wx_app.rs:11663-11668`) and that arm always opens the dialog (`:24401-24408`). | read |
| 2 | The handover does not quit the way Quit does | `frame.close(true)` (`wx_app.rs:24472`) does not set `really_quitting`, which only Quit sets (`:5334`). With "keep running in the tray" on and the icon showing, the close handler takes its hide branch (`:6444-6460`, `closing.rs:50-62`). That branch never runs the Quit path's work: removing the tray icon, emptying the Trash where set, signing out of sessions, and `frame.destroy()`. It also says "Wixen Mail is still running, in the notification area" at the moment the person was told it is closing. wxDragon 0.9.17's event shim passes an unconsumed close event on to wxWidgets' own handler, and a forced close cannot be vetoed, so the window is probably destroyed anyway; whether the process then exits or the tray icon keeps it alive, holding the `WixenMail-Running` mutex the installer checks, was not run. It cannot happen until an installer passes the check, so nobody can meet it before signing exists. No ledger row names it. | read; end state not verified |
| 3 | Two checks can run at once | Nothing stops a check by hand while the check at start runs, or two presses. Both download to the same path, and `File::create` truncates it (`update_download.rs:1143`). One task can be writing over the file the other has just checked and is about to run. | read; the overlap is derived |
| 4 | No timeouts | The check's client is `reqwest::Client::new()` (`update_check.rs:552`) and the download's builder sets only a redirect rule (`update_download.rs:1125-1128`). A connection that stalls holds the task for good, and a check by hand that said "Downloading it now" says nothing further. | read |
| 5 | Words a person reads or hears are wrong | The settings description, which NVDA reads on the control, says "about 12 MB" (`update_check.rs:57-60`); the installer is 15.7 MB. `docs/privacy.md:34` says the updates folder is "emptied when the update is installed"; it is emptied at the next start (`main.rs:403-412`, read). `docs/BETA_RELEASE.md:68` names the installer with the tag (it carries the version without `v`) and `:85-88` says no Check for Updates item has ever existed. Four doc comments still describe the code before plan 07-09 landed: `update_check.rs:20-25`, `config.rs:610-614`, `ui_types.rs:545-561`, `outward.rs:568-571`. | read |
| 6 | No progress and no cancel | A by-hand check says "Downloading it now" and then nothing until it ends. | read |
| 7 | The mutex race | Ledger 340, open. | read |
| 8 | Nothing ties the signed file to the version named | The check proves who signed the file, not which version it is. An older installer signed by this project, published under a newer tag, would pass. ship-shape has the same gap. | derived |
| 9 | Revocation is not checked | On purpose (`update_download.rs:675-684`). | read |
| 10 | Nothing has run against a real release | No tag, the release workflow has never run (ledgers 326 and 891), nothing is signed (329). | measured, read |

Gaps 6 to 10 are listed so the table is the whole set this reading found, not as 13.3 work. Gap 6
is the one most worth doing later, because a screen reader user hears nothing for the length of a
download.

## What ship-shape does

### Its flow on Windows

1. `ui::run_update_check` starts a check on a plain thread. Its default ending is
   `process::exit(0)` (`ui.rs:125-131`, read).
2. The stable channel asks `https://api.github.com/repos/{repo}/releases/latest` (`github.rs:29`,
   read). The dev channel asks for a release tagged `latest` whose notes list commits as `- <hash>`
   lines (`check.rs:71-96`, read).
3. It compares versions with `parse_semver`, which keeps three numbers and drops everything after
   a hyphen in each, then answers "up to date" when `current >= latest` (`version.rs:1-12`,
   `check.rs:64`, read). Its own test says so:
   `assert_eq!(parse_semver("1.2.3-beta.1"), Some((1, 2, 3)));` (`version.rs:22`).
4. With something newer, it opens its own dialog: "A new version of %s is available. Here's what's
   new:", the release notes in a read-only box, and Yes and No buttons. Focus goes to the notes
   (`ui.rs:76-115`, read).
5. Yes downloads `{app_name}_setup.exe` and `{app_name}_setup.exe.minisig` into `%TEMP%`, with a
   native progress window and a cancel button, and checks the minisign signature before renaming
   the file into place (`github.rs:68-69`, `download.rs:52-103`, read).
6. It then starts a hidden `powershell.exe -ExecutionPolicy Bypass` script that waits for the
   program to exit, runs the setup file with `/silent`, waits for it, and starts the program again
   (`platform/windows.rs:45-81`, read). The program exits as soon as the script starts.

### How it fits what Wixen Mail needs

| What Wixen Mail needs | What ship-shape 0.4.1 does | Fit |
|---|---|---|
| Offer alpha.2 to a tester on alpha.1, and 1.0.0 to everybody on a prerelease | Drops the prerelease, so all three are the same version. The stable channel cannot see a prerelease at all (GitHub's docs, read 2026-10-20: "the most recent non-prerelease, non-draft release"). | No |
| A channel for test versions | The dev channel needs a rolling release tagged `latest`, rebuilt on every push. Publishing on every push is what guardrail 7 forbids. | No |
| Versions with a build counter, `1.0.0-alpha.1+1403.g6fd465c6` | Parses to (1,0,0). A release build's `1.0.0+1403.g...` does not parse and every check says "Invalid version". Passing the version without its build part avoids this. | Workable |
| Wixen Mail's asset names | Must be exactly `{app_name}_setup.exe` with no version in the name. Wixen Mail publishes `Wixen-Mail-Setup-{version}.exe` (`release.yml:114-117`, `.iss:128`). | Needs renames |
| A signer check | minisign: a public key compiled in, a `.minisig` beside each asset. It accepts legacy signatures and never reads the trusted comment, so an older signed installer verifies under a newer tag. | Different key |
| The Inno installer | `/silent` is right for Inno. The script waits for the program to exit, which removes ledger 340's race. A `[Code]` message box in `PrepareToInstall` (`.iss:608-657`) uses plain `MsgBox`, which `/SUPPRESSMSGBOXES` does not cover, so it can still stop a silent install (derived from Inno's help). | Partly |
| A failure after the program closes is said | Nothing is said. The script has no error handling and starts the program again whatever happened, so a failed install reopens the old version without a word. | No |
| Accessible names and plain words | No accessible name set anywhere on the updater path (searched for `.set_name(`, `accessib`, `announce`, `live.region` in its source and wx-utils': no hits outside a shortcut dialog). Yes and No answer a question nobody asked. Errors read "HTTP error: 404". The "Security Error" text is reachable only from check errors, and a signature failure is a download error, so a tampered file shows "Update failed: Verification error: ..." titled "Error" (`ui.rs:157`, `:198`, `:246-273`). | No |
| Release notes a screen reader can move through | Flattened to plain text: headings, links and tables lost (`ui/markdown.rs:30-73`). | No |
| Translation | gettext catalogues through patois; error texts never pass through translation. Wixen Mail's plan is a Fluent catalogue in version 2 (`src/vendor/paperback/mod.rs:20-21`). | No |
| Threads | Its own threads, results back through `call_after`. Wixen Mail uses tokio and a channel the UI timer drains. | Workable |
| Logging | Logs nothing itself; ureq logs host names at debug level, full paths at trace. Whether those records reach Wixen Mail's tracing log was not checked. | Workable |
| Network | ureq with rustls, ring and webpki-roots: Mozilla's root list compiled in, not the Windows certificate store, and only proxy environment variables, not the Windows proxy. | Weaker |
| Timeouts, cancel, one check at a time | All three, which Wixen Mail lacks. | Better |

Two of its ideas are worth taking without the crate: timeouts and a one-check-at-a-time guard,
which gaps 3 and 4 above ask for. A third, exiting before the installer runs, removes ledger 340's
race at the price of being unable to report a failure. That trade was weighed and refused in
`update_download.rs:273-288`, and nothing here changes it.

## What adopting it would cost

### Moving wxDragon from 0.9.17 to 0.9.22 or 0.9.23

The exact pin dates from `795e2229` (2026-07-26): "the pin stays exact because the UI depends on
its widget behaviour". Ledger 53 says moving it is its own piece of work. ship-shape and wx-utils
both need `^0.9.22` and ship-shape has no feature that drops wxDragon (read). Cargo keeps one
version per compatible range, and 0.9.17 and 0.9.22 are one range (derived from Cargo's rule). So
adopting ship-shape, even for its download function alone, moves wxDragon.

Six releases sit between the pin and 0.9.23: 222 commits over 278 files, and the bundled wxWidgets
moves from 3.3.2 to 3.3.3 (0.9.18 to 0.9.22) or 3.3.4 (0.9.23) (read by the dependency audit from
upstream's releases and the build scripts).

**What breaks the build.** 0.9.18 turned the accessibility status constants into Rust types.
`src/presentation/accessibility/names.rs` uses the old `ffi::wxd_AccStatus_*` constants at 11
sites, and its `use wxdragon::ffi;` lines at 54 and 544 then go unused, which Clippy refuses.

```text
$ git show HEAD:src/presentation/accessibility/names.rs | grep -c 'wxd_AccStatus'
11
```

**What changes and still compiles.** 0.9.19 made a timer's `on_tick` bind to that timer's own id.
In 0.9.17 it binds every timer event on the owner (`timer.rs:75`, `handler.bind_internal`); in
0.9.22 it binds by id (`timer.rs:65`, `bind_with_id_internal`) (read, both). The message editor's
automatic save has no handler of its own "because a timer event here reaches every handler bound
on the dialog" (`wx_compose.rs:2794`). After the move, that save happens only when another timer
on the dialog fires. Seven `.on_tick(` calls in `src` and twelve in `tests` would need reading
(counted with the Grep tool over the working tree, 2026-10-20). And `wx_compose.rs:2444` passes
`navigate`'s argument backwards because upstream did; that fix is unreleased on upstream's main,
so it is safe at 0.9.23 and reverses Tab and Shift+Tab out of the message body at the next release.

**What it fixes.** ListCtrl losing the last character of every cell (ledgers 28 and 159), the
TreeCtrl data leak (ledger 41), wxWidgets 3.3.4's fix for WebView2 completions reaching a destroyed
control (ledger 466), a leak in every accessible-name helper, heap corruption when a handler binds
during dispatch, and guards against unwinding across the C++ boundary. wxdragon-macros 0.9.22 also
moves quick-xml to 0.42, which clears RUSTSEC-2026-0194 and 0195; `scripts/audit.sh` refuses a
run while an accepted id is no longer reported, so both lines leave `.cargo/audit.toml:54-55` in
the same commit (read).

**The risk.** The wxWidgets change reaches every native control a screen reader reads. Only an
NVDA sitting over the windows can say whether anything moved. Those fixes are real reasons to move
wxDragon, but they are reasons to move it as its own piece of work, not as a side effect of an
updater.

### The crates it adds

Adding ship-shape 0.4.1 with wxDragon at 0.9.22 adds 13 crates compiled on Windows and 4 more that
only appear in `Cargo.lock` (resolved by hand from the crates.io index by the dependency audit and
cross-checked against ship-shape's own lock, so treat it as a floor). New: ship-shape, homeport,
patois, patois-macros, wx-utils, ureq, ureq-proto, utf8-zero, webpki-roots, minisign-verify,
inventory, sys-locale, and a second `dirs` (7.0.0 beside 6.0.0). Five of them (ship-shape,
homeport, patois, patois-macros, wx-utils) come from one publisher, were first published in 2026,
and had between 1,984 and 5,847 downloads each on 2026-10-20. ship-shape had three breaking
releases in six weeks (0.2.0 on 2026-08-24, 0.3.0 on 09-30, 0.4.0 on 10-03). patois and
patois-macros ship no licence text and name no repository.

```text
$ git show HEAD:Cargo.lock | grep -c '^name = "webpki-roots"$'
0
$ git show HEAD:Cargo.lock | grep -c '^name = "ureq"$'
0
$ git show HEAD:Cargo.lock | grep -c '^name = "ring"$'
1
```

### A second HTTP client and a second set of trusted roots

The researchers disagreed on whether this is "a second TLS stack". It is not a second TLS library:
ureq uses rustls 0.23 and ring, both already in the lock. It is a second HTTP client, ureq beside
reqwest, and a second set of trusted roots: webpki-roots compiles Mozilla's list in, while
reqwest 0.13.4 depends on `rustls-platform-verifier`, which reads the Windows certificate store
(read in the lock; which verifier runs was not run). An update check through ureq would fail behind
an organisation's TLS-inspecting proxy whose root lives only in Windows (derived).

Two more costs follow from it. ureq's requests would not pass through Wixen Mail's `Outward` gate,
and `A_CRATE_THAT_CAN_REACH_A_SERVER` in `src/service/outward.rs` (near line 1580) fails until a
new networking crate is classified, in the same commit (read). And rustls in the lock already lists
both `aws-lc-rs` and `ring`; whether turning on ureq's `ring` feature changes which provider
Wixen Mail's own TLS code picks is not verified, and `cargo tree -e features -i rustls` on a
trial build would settle it.

### The compiler floor

ship-shape calls `Duration::from_mins`, stable since Rust 1.91.0. `Cargo.toml:15` declares 1.88
and CI builds on 1.98.1, so the build stays green while the declared floor becomes false (read).

### The release side

- A minisign key pair, made by Pratik with `minisign -G`, kept for good. A lost or replaced key
  leaves every installed copy refusing every update until someone installs a new version by hand.
- Two repository secrets, the key and its password, and a step in `release.yml` that installs
  minisign and signs each setup file, as Paperback's workflow does.
- Asset names changed to `{app_name}_setup.exe` and its `.minisig`, with the promise tests in
  `tests/installer.rs` and the six guard records on `release.yml` changed with them.
- For the dev channel, a rolling release on every push, which guardrail 7 rules out.

## The options

| Option | What it costs | Risk | Verdict |
|---|---|---|---|
| (a) Adopt ship-shape, with the wxDragon move first as its own plan | Three plans at least (the move, the integration, the release side), an NVDA sitting, and a change upstream or a fork before any tester could be offered a prerelease. 13 new crates, a second HTTP client, a second root list, a key to keep. | High, and placed just before the sittings | No |
| (b) Keep Wixen Mail's code and add minisign-verify alone | One crate with no dependencies of its own (`minisign-verify` 0.3.0 of 2026-09-25 or 0.2.5, MIT, by jedisct1, about 17.5 million downloads), a third check in `update_download.rs`, a key Pratik keeps, a signing step and two secrets. | Low in code; the release workflow that would sign has never run | Only on one condition, below |
| (c) Keep Wixen Mail's code, fix gaps 1 to 5 in 13.3, no package | Two plans of about three tasks each. | Low | **Recommended** |
| (d) Defer all update work until after the sittings | Nothing now. | Gaps 1 and 3 wait for the first published release and gap 2 for signing; gap 4 can happen on any check today, and gap 5 is heard today | Second best |

**Why (c) over (d).** Waiting adds no information: nothing the sittings will show changes how any
of the five should be fixed, and each one is small. Gap 5 is wrong today in a sentence NVDA reads
on the settings screen. Gap 2 cannot be met before signing exists, but it has no ledger row, so if
it is not fixed now it is the kind of thing found by the first person it strands.

**Why not (b) now.** minisign-verify is a good crate and option (b) has one real advantage: a test
can sign a fixture with a test key and run the whole check offline, which the Authenticode check
can never do until a real Wixen Mail signature exists. But the project has to sign with Authenticode
anyway, for Smart App Control and for the publisher's name (SHIP-01). Once that account exists, a
minisign key is a second secret that, if lost, strands every installed copy, protecting a path the
first one already protects. So (b) is right only if the signing account will not exist before
somebody other than Pratik needs to update from inside the program. That is his timing to say.

If (b) is chosen, require prehashed signatures (`allow_legacy` false) and read the trusted comment
to tie the signature to the version, which closes gap 8 for that route.

## What 13.3-23 and 13.3-24 would hold under (c)

For the planner. Every item has a failing test first; where the test reads source rather than
behaviour, say so in the plan.

**13.3-23: the check stops fetching what it cannot accept, says a refusal at start without a
dialog, runs one check at a time, and gives up on a stalled connection** (gaps 1, 3 and 4; spoken
or shown: yes).

- While `THE_CERTIFICATE_PROFILE_IT_SIGNS_WITH` is empty, nothing is downloaded. The person is told
  the newer version exists and offered the releases page, which is all the download can lead to
  today. The decision lives in a pure function beside `whether_a_signature_can_be_checked_here`
  (`update_download.rs:621-630`), so it is testable with the identifier passed in.
- A check at start that ends in a refusal or a failed download goes to the status bar and is said
  once. It opens no dialog; a check by hand still does. That is guardrail 5, bounded feedback.
- One check at a time. A check by hand while another runs is answered with a sentence; one at
  start is dropped. A pure guard testable without a socket.
- Timeouts on both clients. For the download, prefer a connect timeout and a read timeout to a
  whole-request timeout, which would cut a slow but working download. Check that
  `ClientBuilder::read_timeout` exists in reqwest 0.13.4 before writing it into the plan.

**13.3-24: the handover quits the way Quit does, and what the update code says is true** (gaps 2
and 5; spoken or shown: yes).

- The handover takes the Quit path, so the tray setting cannot turn it into a hide. The quit flag
  lives in the setup closure (`wx_app.rs:4291`), not in the update handler, so the plan says how
  the handler reaches it. Open a ledger row for the finding, numbered at commit time.
- The settings description stops naming 12 MB. A figure that only grows stays true as "more than
  15 MB"; the measured size goes in `docs/privacy.md` with its date. Correct `docs/privacy.md:34`
  and `:896`, `docs/BETA_RELEASE.md:68` and `:85-88`, the four doc comments, and the size comment
  at `update_download.rs:99-104`.
- `update_check.rs` reads `docs/privacy.md` in a test (near `:999`), and ledger 583 says that read
  reaches no scoped run, so run `cargo test --lib service::update_check::` by hand after editing it.

Files written by rule for both: `docs/changelog.md`, `guards/guards.toml`, `.planning/WINDOWS.md`.
Guard records to read by both readings include, by `file` or `tests_last_seen`, three on
`update_download.rs`, two on `update_check.rs`, three on `version.rs` and those on
`src/service/outward.rs` and `src/presentation/wx_app.rs`; count them with the parser command in
`CLAUDE.md`, not from this list. The requirement is SHIP-02. Neither plan closes a `[D]` clause,
and its box stays open until a signed release has been fetched and installed.

**If Pratik picks (b)**, 13.3-24 also carries the minisign check and opens with a blocking
checkpoint for his word on the package, and the release side goes in as 13.3-24.1, with making the
key as his checkpoint. `minisign-verify` joins `A_CRATE_THAT_CANNOT` in `outward.rs` in the same
commit as `Cargo.toml`. **If he picks (a)**, it does not belong in 13.3: it becomes its own phase
after the sittings, starting with the wxDragon move, and 13.3-23 and 13.3-24 do (c)'s work in the
meantime.

## What Pratik decides and does, in the order he meets it

1. **Should Wixen Mail use ship-shape?** Recommendation: no. It cannot offer a tester the next
   alpha or the final release, and it needs the toolkit moved first.
2. **Should 13.3 fix the update code's five gaps, with no package change?** Recommendation: yes,
   in 13.3-23 and 13.3-24.
3. **Will the Azure Artifact Signing account exist before anyone but you needs to update from
   inside Wixen Mail?** If yes, or if you do not know yet: no second key. If no: add minisign-verify,
   one small crate, and make a minisign key you keep for good. Recommendation: no second key now.
4. **When should wxDragon move?** Recommendation: as its own phase after the sittings, for its own
   fixes, with an NVDA sitting over the windows. Sooner only if a sitting meets one of the crashes
   it fixes, such as ledger 466's WebView2 crash.
5. **Do you want to tell ship-shape's author about the prerelease ordering?** Optional, and yours
   to do; nothing in this work posts anything. Recommendation: one issue is worth it if you want
   ship-shape to be an option later. It would need prerelease ordering and a channel that can see
   prereleases.

What only Pratik can do, whatever he picks: open the Artifact Signing account (ledger 329). Until
then no installer can pass the check, and no answer above changes that.

## What was not verified

- What NVDA or Narrator reads in ship-shape's dialogs, and what Enter does in its notes box.
- Whether PowerShell goes on past a failed `Start-Process` in ship-shape's script.
- Whether a silent per-user install over an all-users copy behaves, and whether an all-users
  upgrade raises an elevation prompt when started from a hidden script.
- Whether ureq's `ring` feature changes which provider Wixen Mail's TLS code picks.
- Whether Wixen Mail builds on wxDragon 0.9.22 or 0.9.23 beyond the `names.rs` break; only a build
  settles it.
- How gap 2 ends: whether the process exits or stays alive behind the tray icon.

## Sources

- ship-shape 0.4.1, wx-utils 0.4.4 and 0.4.2, homeport 0.1.0, patois 0.2.3, minisign-verify 0.2.5,
  ureq 3.4.2, wxDragon 0.9.22 and 0.9.23, and the crates.io and GitHub answers about them: under
  `third-party/` in the session scratchpad, downloaded 2026-10-20.
- Paperback's workflows and manifest, read for how ship-shape's author releases with it: the same
  folder, `paperback-workflows/`.
- Inno Setup's help on `/SILENT`, `/SUPPRESSMSGBOXES`, `AppMutex`, `CloseApplications` and
  `UsePreviousPrivileges`, read 2026-10-20 at jrsoftware.org.
- Wixen Mail at `6fd465c6`: the modules, documents and ledger rows named above, and SHIP-01 and
  SHIP-02 in `.planning/REQUIREMENTS.md`.
