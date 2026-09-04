# Phase 4: Writing and reading a message in full - Research

**Researched:** 2026-09-04
**Domain:** In-repo. Three vendored crates read directly: `wxdragon 0.9.17` (and
`wxdragon-sys 0.9.17`'s C++ shim), `mail-parser 0.11.5`.
**Confidence:** HIGH on everything read from source this session; every claim carries the file
and line it came from. Nothing here came from a web search. No external package is added by this
phase.

This is research rather than a discussion. It was produced by reading the tree at commit
`d3c6c7d`, not by asking questions. It answers what is true today and leaves the decisions at the
end for Pratik.

No `04-CONTEXT.md` exists, so nothing constrains scope yet.

## Summary

**Five of the six requirements describe the code wrongly, and four of those describe as absent
something that ships and is reached from the running program.** This is worse than phase 3, where
three of six were stale. The composer already inserts inline images with compulsory alt text, sends
them as `multipart/related` with `cid:` parts, marks misspellings as they are typed through the
browser engine, sounds an earcon at the end of a wrong word, and walks the message a misspelling at
a time on F7. Two settings for spelling exist in the Settings screen and both default on.

The single most dangerous stale claim is not an absence claim. **WRITE-03 names a blocker that is
real, and prescribes the remedy this codebase deliberately refused.** It says spell-check-while-
typing "waits on a rich editor control" and notes that `wxdragon` is pinned with the `richtext`
feature enabled. `src/presentation/editor_document.rs:1-13` says the composer's body is a WebView
`contenteditable` rather than a `wxRichTextCtrl`, that the reason is accessibility, and that
`wxRichTextCtrl` is drawn by wxWidgets so it exposes no per-range accessibility attributes on any
platform: no misspelling can ever be marked. Acting on WRITE-03 as written would swap the control
chosen for this product's reason to exist for the one refused on those grounds, and it would look
like clearing a known blocker.

The real work of this phase is smaller than the requirements imply on the writing side and
different in kind on the reading side. **What is genuinely missing is mostly on the receiving
boundary: `src/service/mime.rs` drops facts a message arrived carrying, and three features
downstream are unreachable because of it.**

**Primary recommendation:** treat this phase as (1) one new capability, drag and paste onto the
composer; (2) widening `mime::parse` to stop dropping what arrives; (3) turning three existing but
unreached mechanisms into reached ones; and (4) two honest gates for the things that cannot close
here. Do not rebuild inline images, do not rebuild spell-as-you-type, and do not adopt
`wxRichTextCtrl`.

## Requirement-by-requirement evidence audit

The single most valuable thing this document does. For each requirement, whether its stated
evidence is still accurate, and what is true instead.

| Req | Evidence as written | Verdict | What is true now |
|-----|--------------------|---------|------------------|
| WRITE-01 | `grep -rn "DropTarget\|OnDropFiles\|drop_target" src/` returns nothing; attachment handling built in `attaching.rs` | **Accurate** | Confirmed both halves. |
| WRITE-02 | "no inline image insertion path exists" | **Wrong** | Built end to end and reached. `wx_compose.rs:2934` |
| WRITE-03 | "`spell_session.rs` checks on send only"; waits on a rich editor control; richtext feature enabled | **Wrong twice** | As-you-type marking ships; the prescribed remedy was refused. `editor_document.rs:1-13` |
| READ-01 | "`src/service/pdf.rs` is the only in-app reader" | **Accurate but misleading** | True as stated; the whole pipeline around it is built and generic. |
| READ-02 | "`security.rs` does detection only… no PGP key handling, encryption or decryption path exists" | **Accurate for PGP, wrong about what exists beside it** | PGP is genuinely absent. S/MIME goes further than "verification" and six computed fields are dropped. |
| READ-03 | "No external spam classifier integration exists" | **Accurate but the wrong question** | A spam verdict already exists, is stored, listed and shown. Only the filter vocabulary is missing. |

### WRITE-01 — drag and drop, or paste, a file — evidence ACCURATE

The grep the requirement quotes still returns nothing outside two unrelated test names
(`bodies.rs:1047`, `wx_app.rs:23408`). `src/application/attaching.rs` is 476 lines and is the
attachment model, as claimed.

Two things the requirement does not say that a planner needs.

**The attach path is single-file.** `attach_files` (`wx_compose.rs:1408`) builds its picker with
`FileDialogStyle::Open | FileDialogStyle::FileMustExist` (`:1414`) and calls `picker.get_path()`
(`:1425`), singular. `wxdragon` has `FileDialogStyle::Multiple` (`src/dialogs/file_dialog.rs:17`)
and `FileDialog::get_paths() -> Vec<String>` (`:82`). A drop hands over many files at once, so the
drop path and the picker path will disagree about arity unless the picker is widened in the same
change.

**The framework has what is needed, and the risk is where the drop lands, not whether it can be
caught.** `FileDropTarget::builder(window).with_on_drop_files(|Vec<String>, i32, i32| -> bool)`
exists (`wxdragon-0.9.17/src/dnd/droptarget.rs:184`, `:260`, `:310`) and installs a real
`wxFileDropTarget` on the window. Paste is available too: `Clipboard::get_data(&FileDataObject)`
(`src/clipboard.rs:115`) with `FileDataObject::get_files()` (`src/data_object.rs:198`), backed by a
real `wxFileDataObject` (`wxdragon-sys-0.9.17/cpp/src/dataobject.cpp:59`). `DataFormat::FILENAME`
is `4` (`data_object.rs:27`), which looks wrong against wxWidgets' own enum and is not: the C++
shim translates case `4` to `wxDF_FILENAME` (`cpp/src/clipboard.cpp:79-80`).

The unresolved part is that the composer's body is a WebView. The page does not intercept drops —
`insertFromDrop` appears only as an input type the *typing rules* decline to run for
(`editor_page_harness.rs:424`, `editor_document.rs:624-627`) — so what WebView2 does with a file
dropped on the body is untested here and unknown. See "What cannot be settled here".

### WRITE-02 — insert an image inline — evidence WRONG

The requirement says "no inline image insertion path exists". It is built, and reached from a
non-test path.

- `insert_picture` (`wx_compose.rs:2934`) opens a picture picker, reads the file, then asks
  "Describe the picture, for somebody who cannot see it:" in a `TextEntryDialog` (`:2972-2984`).
- `a_picture_to_send` (`src/application/pictures.rs:349`) refuses an empty description outright
  (`:352-358`) and returns `<img src="data:{kind};base64,{…}" alt="{escaped}">` (`:366-371`).
- Reached: `ID_INSERT_PICTURE` (`wx_compose.rs:46`) is a real menu item, "Insert &Picture..."
  (`:515-517`), dispatched at `:1230-1232` from the formatting menu the toolbar button raises.
- The sanitiser admits exactly this shape and nothing else beginning `data:`
  (`html_renderer.rs:131-150`): `data:` is allowed only on an `img` and only when
  `pictures::is_a_picture_we_carried` recognises it.
- **The send path converts it properly.** `smtp.rs:176-217`: the comment records that Gmail and
  Outlook both drop `data:` pictures out of a received message, so `pictures_out_of(html)` rewrites
  them into `multipart/related` with `Attachment::new_inline(content_id)` per picture (`:214-216`),
  and `what_the_plain_text_should_say` puts the descriptions into the plain half so it has no
  silent hole (`:187-190`).

Two gaps remain against success criterion 2.

1. **There is no decorative option.** The criterion says "requires alt text **or an explicit
   decorative mark**". `a_picture_to_send` has one path and it refuses an empty description
   (`pictures.rs:352`), so a genuinely decorative image cannot be inserted at all. That is a
   deliberate stance (`wx_compose.rs:2926-2931` argues for it), and it is not what the criterion
   asks for. This is a decision, not a defect. See "Decisions for Pratik".
2. **"Survives a draft save and reload" is not proven here.** The round trip is
   `body_from_editor` → `HtmlRenderer::sanitize_html` (`editor_document.rs:1536-1542`) → drafts
   table → `editor_document(body, …)` back into the page (`wx_compose.rs:1037`). The sanitiser
   admits the shape at both ends, so it very likely survives; I found no test asserting it does.
   That is a cheap red/green pair, not a build.

### WRITE-03 — spell check while typing — evidence WRONG IN BOTH CLAUSES

**"Checks on send only" is false.** Three separate as-you-type mechanisms ship:

- The page carries `spellcheck="{true|false}"` on the `contenteditable`
  (`editor_document.rs:111`, `:164`), driven by the stored setting at `wx_compose.rs:1032-1037`.
  The module header (`editor_document.rs:10-13`) states what this buys: UIA spelling errors from
  Chromium, `AXMarkedMisspelled` from WebKit, AT-SPI `invalid:spelling` from WebKitGTK, each
  announced by the screen reader itself.
- The page posts `{kind:'word'}` when a word is finished (`editor_document.rs:686`, `:693`) and
  the composer sounds `Event::MisspelledWord` if the speller disagrees with it
  (`wx_compose.rs:2031-2037`). The comment says deliberately not spoken, because the engine has
  already marked it.
- `check_spelling` (`wx_compose.rs:2448`) walks the message a misspelling at a time on F7 or the
  Spelling button, selecting each word in the editor before asking, announcing
  `Finding::spoken()` at `Priority::High` (`:2497`), with Ignore, Ignore All, Add to dictionary
  and Change All (`:2505-2557`).

Both settings exist and are wired: `check_spelling_before_send` and `check_spelling_as_you_type`
(`config.rs:249`, `:343`), both defaulting true (`:581`, `:594`), both checkboxes in Settings
(`wx_settings.rs:96`, `:101`, read at `:2290-2291`).

**"Waits on a rich editor control" prescribes the rejected remedy.** `editor_document.rs:1-13` is
the refusal, quoted in the Summary. Treat the requirement's second clause as void.

The criterion's flood clause is **already satisfied by design**: `typingInput` admits only
`insertText` and `insertReplacementText`, and `editor_document.rs:624-627` says in as many words
that paste is excluded because a pasted block is checked by F7 with the rest of the message.
There is a harness test for it (`editor_page_harness.rs:417-431`).

What is genuinely not built for criterion 3 is **"a keyboard command moves between them"** in the
sense of moving the caret to the next or previous misspelling and staying in the editor. What
exists is a modal walk: a dialog per word. Whether that satisfies the criterion is a judgement,
not a fact, and it is on the decisions list.

**Today's spellcheck change, read from current source rather than any description.**
`WhatThisMachineOffers` (`spellcheck/mod.rs:235-243`) has two arms: `TheseLanguages(Vec<(String,
String)>)` — asked and answered, possibly with an empty list — and `CouldNotAsk { reason: String }`.
`language_to_check_in(system, offers)` (`:269`) is pure: no system language at all gives `"en"`
(`:270-275`); `CouldNotAsk` keeps the machine's own language and logs the reason (`:282-292`);
`TheseLanguages` matches and falls back to `"en"` (`:293-297`). It is reached in production through
`default_language()` (`config.rs:465-474`), the serde default for `AppConfig.language`
(`config.rs:241`), which the composer reads for both the page language and the speller
(`wx_compose.rs:1030-1037`, `:1538`, `:2456-2459`). The doc comment at `:224-232` records why:
a failed platform question and a machine with no checkers were the same value, so a French user
got English on a first run where the call happened to fail.

`language_of_this_machine` (`:216`) is the older entry point and **has no production caller** —
its three occurrences outside its own definition are all in `#[cfg(test)]` (`:1162`, `:1176`,
`:1557`). Bucket 2. Worth knowing before anything calls it by mistake.

### READ-01 — preview an image or a text attachment — evidence ACCURATE, IMPLICATION WRONG

"`src/service/pdf.rs` is the only in-app reader" is true. What the requirement does not say is
that **everything around that reader is generic and built**, so this requirement is a producer
away from done rather than a subsystem away.

- `read_attachment` (`wx_app.rs:18054`) fetches the bytes on a worker, calls `pdf::read`, and posts
  `UIUpdate::AttachmentRead(Box<ReaderDocument>)` (`:18089-18091`), which opens as a tab of its own.
- `pdf_document` (`reader_text.rs:615`) is the only producer: it builds a `ReaderDocument` with a
  title, text, and `Landmark`s the reader navigates by. Nothing about that struct is PDF-shaped.
- The gate is four lines. `can_be_read_here` (`wx_reader.rs:193-203`) returns true for
  `application/pdf` or a `.pdf` name and nothing else. The refusal is already written and already
  names what to do instead (`:156-166`, `describe_for_refusal` at `:206`).
- The bytes are already cached: `attachment_content` is a digest-keyed content store
  (`message_cache/mod.rs:1421-1442`).

So READ-01 is: widen `can_be_read_here`, add a text producer and an image producer beside
`pdf_document`, and route them in `read_attachment`.

**The hard half is the one the criterion actually names, and it is blocked upstream.** Criterion 4
says the preview must announce "any description the sender supplied and say plainly when there is
none". For an *image*, that description is the sender's `Content-Description` header or the `alt`
on the `<img>` that references it. **Neither reaches the application.** `AttachmentInfo`
(`mime.rs:48-52`) carries `filename`, `mime_type` and `size` and nothing else, and `described`
(`mime.rs:111-117`) constructs it from three accessors. `mail_parser 0.11.5` exposes
`content_description()`, `content_disposition()`, `content_id()` and `content_language()` on
`MimeHeaders` (`mail-parser-0.11.5/src/lib.rs:495-505`), and `MimeHeaders` is already imported in
`mime.rs:16`. So this is a widening of one struct and one function, not a new capability — but it
must happen before the image preview can say anything true.

### READ-02 — full PGP encryption and decryption — ACCURATE FOR PGP, INCOMPLETE ABOUT ITS SURROUNDINGS

**PGP is genuinely absent.** The only occurrences in `src/` outside tests are four string checks:
`detect_pgp_signed` looks for `-----BEGIN PGP SIGNED MESSAGE-----` and `-----BEGIN PGP
SIGNATURE-----` (`security.rs:269-272`), `detect_pgp_encrypted` for `-----BEGIN PGP MESSAGE-----`
(`:274-276`). No key handling, no armor parsing, no crate. Confirmed.

Three things the requirement does not record, all of which change the shape of the work.

**1. Six of the eight fields of `MessageSecurityReport` are computed and thrown away.** The struct
(`security.rs:72-83`) carries `pgp_signed`, `pgp_encrypted`, `smime_signed`, `smime_encrypted`,
`signature_status`, `phishing_risk`, `phishing_score`, `phishing_indicators`. Its only production
consumer is `body_safety::from_body` (`body_safety.rs:45-66`), which calls
`analyze_message_security` at `:60` and reads exactly two fields at `:65`:
`report.phishing_risk` and `report.phishing_indicators`. `signature_status` and `smime_signed` have
no reader anywhere in `src/` outside `security.rs`. So **the application already computes "this
message is PGP-encrypted" on every message it reads, and tells nobody.** `from_body` is reached
from `pop_sync.rs:518` and `wx_app.rs:18525`, so this is a live path dropping a live fact.

**2. S/MIME goes considerably further than "verification", and one part of it is unreachable.**
`signed_mail.rs` is 6,738 lines and carries a DER reader, a certificate store behind
`CertificateStore` with a real Windows implementation (`:3032-3038`), revocation and issuer trust
(`IssuerTrust` `:2506`, `Withdrawal` `:2526`), and signature checking that **is** reached:
`checking_signatures::for_message` is called from `wx_app.rs:11181` and its result reaches the
reader's warning bar (`reader_text.rs:1023-1030`).

`EncryptedMessage` (`:3645`) reads the outside of a PKCS #7 `EnvelopedData` — who it is addressed
to and under which cipher — and `spoken()` (`:3706`) already writes the exact sentence criterion 5
asks for, including "This computer holds a certificate this message was encrypted to." **It has no
caller at all**: `grep -rn "EncryptedMessage" src/ tests/` matches only `signed_mail.rs` itself.
Bucket 2, and the least-work half of criterion 5 is already written.

**3. Nothing goes out signed or encrypted.** There is no signing path: the only `fn sign_*` in
`src/` is `oauth::sign_in_not_saved` (`oauth.rs:900`), unrelated. The intel file already records
this (`.planning/intel/context.md:149`).

**The failure criterion 5 names is real and reachable today.** An S/MIME enveloped message has no
`text/*` part, so `mime::parse`'s `first_of_kind` (`:143-159`) yields `None` for both bodies and the
message reads as empty. A PGP-encrypted message is different: its armored block is a text part, so
it renders as the armor rather than as nothing. Those two need different handling and the
requirement treats them as one.

### READ-03 — hook into an external spam classifier — ACCURATE, BUT THE GAP IS ONE LIST ENTRY

"No external spam classifier integration exists" is true. Everything around it is built.

`src/service/safety.rs` (662 lines) reads the verdict a filter already reached out of the headers:
`X-Spam-Flag`, `X-Spam-Status`, `X-Forefront-Antispam-Report`, `X-Microsoft-Antispam` and
`Authentication-Results` (`from_headers`, `:150-168`). Its module header (`:1-18`) states the
design and the reason: the most reliable free detection available is the detection that has
already happened, and asking an outside service means handing it links from private
correspondence.

It is reached from three non-test paths: IMAP (`protocols/imap.rs:1788`), POP
(`pop_sync.rs:515`), and message import (`message_cache/messages.rs:4245`). The verdict is merged
with the folder's own signal, worst winning (`mail_sync.rs:454-458`), stored as `safety` and
`safety_reasons` columns (`messages.rs:357-359`), shown as a message-list column
(`Safety::label`, `safety.rs:41-49`) and as the reader's warning bar (`reader_text.rs:282`,
`:377`).

**What criterion 6 asks for is one addition to one list.** `A_FIELD_A_RULE_MAY_NAME`
(`filters.rs:61-71`) holds eleven names — `subject`, `from`, `to`, `cc`, `date`, `message_id`,
`body_plain`, `body_html`, `read`, `starred`, `deleted` — and `safety` is not among them.
`FilterEngine::matches` (`:302`) has a match arm per name (`:320-332`) and an unknown field
returns `false` deliberately. `CachedMessage.safety` is right there on the struct the matcher is
handed (`messages.rs:357`). The engine is reached (`wx_app.rs:17004`, `:18856` build
`mail_sync::Filtering`; `mail_sync.rs:803` evaluates it on arrival).

The comment at `filters.rs:56-59` warns that the list and the match arms are held in agreement by
a test in both directions. Adding one name touches both plus the spoken-words table. This is the
shape observation 0022 in the log is about — a completeness guard turning its struct into a closed
vocabulary — and here that guard is doing its job.

Criterion 6's second clause, "shown with its source named", is partly done: `Verdict.summary()`
(`safety.rs:131`) and `safety_reasons` already carry sentences into the warning bar. Whether they
name the source per reason wants reading before it is planned.

## What a received message carries and this application drops

The reading half of the phase goal. All of these are read from `src/service/mime.rs`, which is the
single boundary (`parse` at `:124`, `described` at `:111`).

| Dropped | Where it would come from | What it costs | Bucket |
|---|---|---|---|
| `Content-Description` on a part | `mail_parser` `MimeHeaders::content_description` (`lib.rs:495`) | READ-01 criterion 4 cannot be honest about an image | 3 |
| `Content-Disposition` (inline vs attachment) | `MimeHeaders::content_disposition` (`lib.rs:497`) | `is_embedded_in_the_body` (`mime.rs:298`) infers it; a sender's explicit `attachment` on a `cid:` part is not honoured | 3 |
| `Content-ID` on an attachment part | `MimeHeaders::content_id` (`lib.rs:499`) — already used for pictures (`mime.rs:280`) but not stored on `AttachmentInfo` | An attachment cannot be tied back to the `<img>` that names it | 3 |
| `Content-Language` / sender's `<html lang>` | `MimeHeaders::content_language` (`lib.rs:505`) | Stated WCAG 3.1.1 gap. `html_renderer.rs:14-20` says so outright: "no message carries a Content-Language header through this application, and a sender's own `<html lang="de">` is dropped on the way in". The renderer falls back to the machine's language and writes no attribute when it cannot say (`:31-41`) — a deliberate, documented choice not to guess | 3 |
| `List-Unsubscribe` | `message.header(name)`, the same route `receipt_request` uses (`mime.rs:170-179`) | **`blocking::WhatIsAlreadyTrue.how_to_leave_the_list` (`blocking.rs:363-368`) exists, is read at `:403`, and its one production construction passes a hardcoded `None` (`wx_app.rs:24655`).** The mailing-list warning at `blocking.rs:518-528` can never fire | 2 |
| `Bcc`, `Sender` | `Message::bcc` (`mail-parser core/message.rs:124`), `Message::sender` (`:360`) | A message this user sent, re-read from Sent, loses its Bcc list | 3 |
| Second and later text bodies | `first_of_kind` (`mime.rs:143`) takes the first | A message with alternatives beyond the first pair loses them | 3 |
| Five of eight `MessageSecurityReport` fields | Computed at `security.rs:246-260`, consumer reads two (`body_safety.rs:65`) | READ-02 criterion 5, above | 2 |

**Not dropped, contrary to what a reader of the requirements would expect:** inline `cid:` images.
`pictures_carried` (`mime.rs:272-290`) pulls every part with a `Content-ID` of a raster kind under
2 MB and `pictures::carry_the_pictures` writes them into the body as `data:` at parse time
(`mime.rs:152-159`). `pictures.rs:1-26` records that the sanitiser used to do exactly the wrong
thing in both directions. The `alt` attribute survives the sanitiser (`html_renderer.rs:1314` is a
test asserting it), and `long_text.rs:232-236`, `:457-468` and `:641` render "image with no
description" rather than dropping an undescribed image silently.

## The three buckets, for everything phase 4 would touch

Kept strictly apart, because this project has been bitten by the difference.

### 1. Exists and is reached from a non-test path in the running program

| Thing | Entry point |
|---|---|
| Attach a file by picker, announce name and size, refuse over the limit, Delete to remove | `wx_compose.rs:1408` (button `:1477`), model `attaching.rs` |
| Draft attachments reload, and a path that no longer reads is announced rather than dropped | `wx_compose.rs:1382-1403` |
| Attachments read at Send, not at pick; sent as `multipart/mixed` | `mail_controller.rs:226`, `smtp.rs:225-240` |
| Insert inline picture with compulsory alt text | `wx_compose.rs:2934`, menu `:515-517`, dispatch `:1230` |
| Inline picture sent as `multipart/related` + `cid:` | `smtp.rs:182-217` |
| Received `cid:` pictures written into the body; remote ones blocked by default | `mime.rs:152-159`, `pictures.rs:349`+ |
| Spellcheck marking as you type, via the engine | `editor_document.rs:111`, `:164`; setting `wx_compose.rs:1032` |
| Earcon at the end of a misspelled word | `wx_compose.rs:2031-2037` |
| F7 misspelling walk with change/ignore/add | `wx_compose.rs:2448`, `spell_session.rs` |
| Spelling check before send | `wx_compose.rs:2360`, `:2273` |
| Spelling language decided from what the machine could be asked | `config.rs:465-474` → `spellcheck/mod.rs:269` |
| PDF attachment opens as a reader tab with landmarks | `wx_app.rs:18054`, `reader_text.rs:615`, gate `wx_reader.rs:193` |
| Attachment save with download-folder default | `wx_app.rs:18099` |
| Provider spam/phishing verdict read, stored, listed, shown | `safety.rs:150` ← `imap.rs:1788`, `pop_sync.rs:515` |
| Our own phishing analysis merged in, capped at Suspicious | `body_safety.rs:45`, `safety.rs:183` |
| Filter engine evaluated on arriving mail | `mail_sync.rs:803`, built at `wx_app.rs:17004`, `:18856` |
| S/MIME signature checked and shown in the warning bar | `wx_app.rs:11181`, `reader_text.rs:1023` |

### 2. Exists but only tests reach it

| Thing | Definition | Why it is unreached |
|---|---|---|
| `EncryptedMessage` + `spoken()` — the sentence criterion 5 wants | `signed_mail.rs:3645`, `:3706` | No caller anywhere in `src/` or `tests/` |
| The "you are on a mailing list, here is how to leave" warning | `blocking.rs:518-528`, read at `:403` | Its only production caller hardcodes `how_to_leave_the_list: None` (`wx_app.rs:24655`) because `List-Unsubscribe` is never parsed |
| `pgp_encrypted`, `pgp_signed`, `smime_signed`, `smime_encrypted`, `signature_status` | `security.rs:75-79` | Computed on every message; the sole consumer reads two other fields (`body_safety.rs:65`) |
| `language_of_this_machine` | `spellcheck/mod.rs:216` | Three callers, all `#[cfg(test)]` |

### 3. Does not exist

- Any drop target or drop handler anywhere in `src/`.
- Any clipboard read of a file list. `editing.rs` models Cut/Copy/Paste as commands
  (`editing.rs:37`) but there is no `wxdragon::Clipboard` use in `src/` at all.
- Multi-file attach.
- A decorative-image path (`pictures.rs:352` refuses an empty description).
- Any PGP: no key import, no armor parse, no decrypt, no crate.
- S/MIME or PGP signing or encrypting of outgoing mail.
- Any producer of a `ReaderDocument` other than `pdf_document` and the message/conversation ones.
- `safety` as a filter-rule field (`filters.rs:61-71`).
- `Content-Description`, `Content-Language`, `Content-Disposition`, `Content-ID` on
  `AttachmentInfo`; `Bcc`, `Sender`, `List-Unsubscribe` on `ParsedMessage`.
- A "move to next/previous misspelling" key that leaves the caret in the editor.

## Assumptions phase 4 would rest on that I could not verify

| # | Assumption | Cost if wrong |
|---|---|---|
| A1 | A `wxFileDropTarget` installed on the compose dialog receives files dropped over the WebView child. | The whole of WRITE-01's drop half. WebView2 handles drops in its own HWND; if it swallows them, the drop has to be caught in the page's JavaScript and posted over the existing `wixenEditor` channel, which is a different design with a different security boundary (the page would be handling a file path). Settle this with a throwaway build before planning tasks around it. |
| A2 | A picture inserted as `data:` survives draft save and reload with its alt intact. | Criterion 2's second half. The sanitiser admits the shape at both ends (`html_renderer.rs:131-150`), so this is likely; nothing asserts it. Cheap to prove, expensive to assume. |
| A3 | The modal F7 walk satisfies "a keyboard command moves between them". | If it does not, criterion 3 needs a new interaction in the page (caret movement between engine-marked ranges), which the DOM does not expose directly — the marks are the engine's, not the document's. That could be a large, uncertain piece of work sitting behind an innocuous-looking criterion. |
| A4 | Widening `AttachmentInfo` is additive for the cache. | `attachments` table columns are added with `ensure_column_exists` per the project rule; a new field that is only computed at parse time and never stored costs nothing, one that is stored needs a column. Which of the two is a design choice nobody has made. |
| A5 | `Content-Description` is actually present on real senders' image parts. | READ-01 criterion 4 would then say "the sender supplied no description" on nearly every image. That is the honest answer and it is also a thin feature. No account has ever been used with this program, so this is unmeasurable here. |
| A6 | `EncryptedMessage::read` parses a real S/MIME envelope from a real sender. | It is tested against constructed DER only. A parser that works on synthetic input and not on Outlook's output would make criterion 5's message wrong rather than absent. |

## What cannot be settled here

Nothing in this project has ever run against a real mail account. Naming these precisely rather
than glossing them, per guardrail 9.

1. **Whether an inline image sent as `multipart/related` renders at the other end.** The build is
   right by the standard and `smtp.rs:176-181` records why `data:` was abandoned, but no message
   from this program has ever reached a recipient.
2. **Whether real senders supply `Content-Description`, and how often.** A5.
3. **Whether provider spam headers appear as `safety.rs` expects.** Every parser in it is tested
   against hand-written header blocks. Gmail in particular tells an IMAP client almost nothing
   except the folder, which `mail_sync.rs:456-458` already accounts for.
4. **Whether a real S/MIME encrypted message parses.** A6.
5. **Whether the drop lands.** A1 — settleable locally with a build, but not by reading.
6. **Every accessibility criterion.** Whether NVDA announces the engine's spelling marks in a
   WebView2 `contenteditable` in this application; whether the earcon and the screen reader's own
   announcement collide; whether an image preview tab announces its description at the right
   moment. Expect these to close as `unrun-verify` ledger entries the way phases 2 and 3 did.
   `.claude/gsd-core` cannot judge them and neither can a test.

## Project constraints that bear on this phase

From `CLAUDE.md`, and each of these changes what a plan may contain.

- **Red/green TDD on every eligible task.** `.planning/config.json` has `workflow.tdd_mode: true`
  (verified). A red commit on a branch names its failures as `Fails-until-green:` lines and is
  measured against them by `scripts/red-commit.sh`. Red commits are refused on `main`.
- **`scripts/check.sh` is the gate**, four checks, clippy at `-D warnings`. Never pipe it.
- **Guard records.** `guards/guards.toml` holds 565 records. Of the files this phase touches,
  `src/application/pictures.rs` has 4 and **`attaching.rs`, `filters.rs`, `safety.rs`,
  `spell_session.rs` and `editor_document.rs` have none**. Any integration guard written in
  `tests/` for those needs a record, or the gate will not run it on the commits that could break
  it. Records perish: any change adding tests near a rule re-measures that rule's record, and the
  per-commit count check will name which.
- **Guard re-measurement is off the critical path** as of 2026-09-03 — one sweep once the phase is
  complete, not per merge. Run the scoped remedy when a commit prints it; that is not optional.
- **Schema changes are additive.** New attachment or message columns go in with
  `ensure_column_exists`; nothing that shipped is dropped or renamed.
- **Secrets never touch `message_cache.db`.** Decisive for READ-02: a PGP private key is a secret
  and goes to the OS credential store through `service::credentials` or a new named owner, never
  to the cache, never to a log. Each service name has exactly one owner.
- **Untrusted input stays untrusted, and sanitising is not an excuse to drop structure.** Every
  new preview producer is a new untrusted-input boundary. `pdf_document` sets
  `looks_unsafe: false` (`reader_text.rs:639`) on the grounds that nothing judged it; a text or
  image producer needs that decision made deliberately rather than copied.
- **No AI attribution in commits, branches, comments or documents.**
- **A user-visible change gets a `docs/changelog.md` entry under `[Unreleased]` in the same
  commit**, honest "Known limitations" included.
- **Anything experimental says so in the product**, not only in a report — `application::allowed`
  and `presentation::first_run` are how it is done here.
- **`docs/KEYBOARD_SHORTCUTS.md` is updated in the same commit as a shortcut**, and a test checks
  it both ways. WRITE-01's "keyboard equivalent at least as quick to reach" lands there.

## Environment availability

No new external dependency is needed by any part of this phase except PGP.

| Dependency | Required by | Available | Notes |
|---|---|---|---|
| `wxdragon 0.9.17` drag-and-drop and clipboard | WRITE-01 | Yes | `src/dnd/`, `src/clipboard.rs`, `src/data_object.rs`; already a direct dependency, no feature flag needed |
| `mail_parser 0.11.5` MIME header accessors | READ-01, the dropped-headers list | Yes | `content_description`, `content_disposition`, `content_id`, `content_language` all on `MimeHeaders`, already imported |
| `ammonia` with `data:` and `cid:` admitted | WRITE-02 | Yes | `html_renderer.rs:131-150` |
| An image decoder for preview | READ-01 | **Not present** | Nothing in `Cargo.toml` decodes PNG/JPEG. For a blind-first preview the description may matter more than the pixels, which may make a decoder unnecessary — that is a decision, below |
| An OpenPGP implementation | READ-02 | **Not present** | No candidate evaluated here. Adding one is a dependency-audit conversation of its own and is not something to slip into a plan |

**Package legitimacy audit:** not applicable. This phase as scoped adds no package. If PGP is
taken on, the audit runs then, on whatever crate is proposed, and the choice is Pratik's.

## Validation architecture

`workflow.nyquist_validation` is not set in `.planning/config.json`, so it is enabled by default.

| Property | Value |
|---|---|
| Framework | `cargo test` (`--all-targets`), `tokio-test` for async, `tempfile` for filesystem |
| Config | none; unit tests in `#[cfg(test)] mod tests` beside the code, cross-layer in `tests/` |
| Scoped run | `bash scripts/check.sh` (mode decided by `scripts/which-checks.sh`) |
| Full suite | `bash scripts/check.sh all` — run by whoever merges |
| Guard sweep | `scripts/guards.sh`, deferred to once per completed phase |
| Env | `WIXEN_TEST_THREADS` defaults to 4 (guard runs only); `WIXEN_NO_AUDIO` where a sound device opens and does not work — it is not a way to skip sound tests |

Requirement-to-test map, with where each would live:

| Req | Behaviour | Type | Where |
|---|---|---|---|
| WRITE-01 | A dropped path list becomes `Chosen` values, refusing a folder and an unreadable file by name | unit | `attaching.rs` — pure, no window |
| WRITE-01 | Every drop action has a keyboard equivalent named in `docs/KEYBOARD_SHORTCUTS.md` | guard | `tests/`, needs a `guards.toml` record |
| WRITE-02 | A described picture survives sanitise → store → reload with its `alt` | unit | `editor_document.rs` or `pictures.rs` |
| WRITE-03 | Whatever the decision on "move between misspellings" turns out to be | unit | `spell_session.rs` |
| READ-01 | `can_be_read_here` says yes to text and image and no to the rest | unit | `wx_reader.rs` (a table test already exists at `:911`) |
| READ-01 | A part's `Content-Description` reaches `AttachmentInfo`; a part without one says so | unit | `mime.rs` |
| READ-02 | An S/MIME enveloped message produces the "cannot be opened, here is why" sentence rather than an empty body | unit | `mime.rs` + `reader_text.rs` |
| READ-03 | `A_FIELD_A_RULE_MAY_NAME` and the match arms agree, with `safety` in both | guard | `filters.rs`, the existing both-directions test |

Wave 0 gaps: none for framework. The gap is **guard records** for `attaching.rs`, `filters.rs`,
`safety.rs`, `spell_session.rs` and `editor_document.rs`, written when the first integration guard
for each is.

## Security domain

| ASVS | Applies | Control here |
|---|---|---|
| V5 Input validation | Yes | Every attachment, every dropped path, every previewed file is a stranger's bytes. `attachment_name::safe_file_name` already exists (`attaching.rs:66`); a drop path from the OS goes through the same. |
| V6 Cryptography | Only if PGP is taken on | Never hand-roll. And the project's own rule outranks the generic one: keys go to the credential store, never to `message_cache.db`, never to a log. |
| V1 Architecture | Yes | A new preview producer is a new boundary. `looks_unsafe` on `ReaderDocument` is a claim, not a default. |

Threat patterns specific to this phase:

| Pattern | STRIDE | Mitigation present |
|---|---|---|
| `data:` URI carrying an SVG that runs script | Tampering / Elevation | `html_renderer.rs:145-150` admits `data:` only on `img` and only shapes this application wrote; `KINDS_WORTH_CARRYING` is raster-only and says why (`pictures.rs:52-57`) |
| Remote image as a tracking pixel | Information disclosure | Blocked by default, `Fetching::from_setting` (`pictures.rs:37-49`) |
| Alt text with a quote breaking out of the attribute | Tampering | `html_escape::encode_double_quoted_attribute` (`pictures.rs:370`), with a test at `:486` |
| A dropped path that is a directory, a device, or a name that walks | Tampering | `Chosen::at` refuses a directory (`attaching.rs:57-63`); `safe_file_name` handles the name. A drop path has not been through either yet |
| A previewed file that parses partially and renders half | Spoofing | `pdf::read` returns a note saying which of three kinds it got (`pdf.rs:10-12`); new producers need the same |

## Decisions for Pratik

These change what gets built and are not mine to settle.

1. **The decorative image.** Criterion 2 says alt text *or* an explicit decorative mark.
   `a_picture_to_send` refuses an empty description outright and `wx_compose.rs:2926-2931` argues
   that letting one through would be "the one place this application still made the problem it
   exists to solve". Add `alt=""` with `role="presentation"` behind a deliberate "this picture is
   decorative" answer, or amend the criterion to match the stance already taken? Both are
   defensible; they cannot both be true.

2. **What "a keyboard command moves between misspellings" means.** F7 walks them in a modal dialog
   per word today. A key that moves the caret to the next marked range and leaves you in the editor
   is a different feature, and the marks belong to the browser engine rather than to the document,
   so the page cannot enumerate them. If the criterion means the second thing, it needs its own
   feasibility spike before it is planned; if the walk satisfies it, criterion 3 is close to
   already met and should say so.

3. **READ-02's scope, given that PGP is a dependency decision and S/MIME is nearly there.**
   Three choices, and they are not the same size. (a) Take on an OpenPGP crate, key import, and
   decryption — a large piece of work whose last mile cannot be tested here. (b) Reach the S/MIME
   half: call `EncryptedMessage::read`, say `spoken()`, and stop a message reading as empty. Small,
   provable, and closes the half of criterion 5 that is about honesty rather than about
   decryption. (c) Both. My reading is that (b) is the part that discharges the guardrail and (a)
   is the part that discharges the requirement, and they can be separate plans.

4. **Whether the six dropped `MessageSecurityReport` fields are wired now or as part of READ-02.**
   "This message is PGP-encrypted" is computed on every message today and told to nobody. That is
   a small change with a real user-facing effect and it is independent of ever decrypting
   anything.

5. **Whether `List-Unsubscribe` is in scope.** It is not named by any of the six requirements. But
   a whole warning path exists and is unreachable because one header is dropped at the same
   boundary READ-01 needs widening anyway, and it is the cheapest thing in this document. Fold it
   into the `mime.rs` widening, or leave it as a deferred item?

6. **What an image preview is, for this product's audience.** Decoding pixels needs a new
   dependency and serves the partially sighted. Announcing the sender's description, the
   dimensions, the format and the size needs no dependency at all and serves the blind reader
   directly. The requirement says "previews"; the criterion says "announcing any description the
   sender supplied". Which of the two READ-01 means decides whether this phase adds a decoder.

7. **Whether the requirements document is corrected in place.** Five of six are stale in ways that
   would mislead the next reader, and this document is not where somebody looks. Phase 3 recorded
   its corrections in each plan's `<premise_corrections>` and left the requirements as written.
   Same again, or amend `REQUIREMENTS.md` this time?

---
*Research written 2026-09-04, from a read of the tree at commit `d3c6c7d`.*
