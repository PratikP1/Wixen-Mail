---
phase: 11-reading-and-the-list
plan: 11
subsystem: pictures a message points at shown by default, tracking pixels and decorative pictures held back by the sender's own declarations, a linked picture reading its link's words, an undescribed picture passed over or called what the Reading tab says, the same for a note read aloud, and the privacy page listing every way a reader of mail can be tracked from the code; guards, pages, ledger
tags: [pictures, remote-images, tracking-pixel, beacon, decorative, alt-text, undescribed, settings, reading-tab, html-renderer, long-text, privacy, read-receipts, outbound-clients, census, ledger]

requires:
  - phase: 11-reading-and-the-list
    provides: "11-10.1 merged at be97ed86 (main clean at 5c87bd4c when the branch left it); its note pass say_which_links_are_not_opened_here before the picture pass on the same cleaned markup; 10-04's labelled_choice with a sentence under it and its dialog-reading shape in tests/how_much_message_text_stays_is_read_back_from_the_permissions_page.rs; the two settings guards in config.rs; ammonia keeping width and height on img and dropping style"
provides:
  - "application::describing_pictures: UndescribedPicture with Nothing (the default), Image and Photo; ALL in that order; label, as_stored, from_stored reading anything unreadable as nothing, description answering \"\", \"image\" or \"photo\"; offered_index; from_stored_settings for the note reader; UNDESCRIBED_PICTURES_LABEL and WHAT_THE_CHOICE_LEAVES_ALONE; 8 tests, 1 record"
  - "application::pictures: looks_like_a_beacon over a declared width or height of a pixel or less (MOST_A_BEACON_MAY_MEASURE), digits with or without px and nothing else, its doc saying how it differs from could_be_furniture; is_marked_decorative (an alt present and empty); what_to_do_about_a_tag over the cleaner's tag, the beacon asked before the mark, answering Showing::HeldBackAsABeacon and Showing::HeldBackAsDecorative beside the three answers that were; the_links_text_as_a_description (one picture in the anchor, no alt of its own, some words, stripped, decoded and escaped); describe_the_undescribed (every img with no alt attribute gains the choice's description, an alt present is untouched); HeldBack with by_the_switch and as_beacons; what_was_held_back over both counts; 44 tests as before, 6 records name the file"
  - "data::config: hold_back_remote_pictures off by default, serde and struct, with the doc rewritten; undescribed_pictures_read_as with its serde default of nothing; the older-file test asserting both absent keys; 68 tests as before, 11 records"
  - "presentation::html_renderer: what_the_settings_say reads the three picture answers in one go; HtmlRenderer.describing with describing_undescribed_pictures_as for a caller that has the answer; sanitize_and_count_held_back answers (String, HeldBack); hold_back_what_would_be_fetched runs the link's words, then what_to_do_about_a_tag per picture (a beacon an empty span and counted, a decorative remote picture a span with WHAT_A_DECORATIVE_PICTURE_SAYS under OutLoud and an empty span under Silently, counted by neither), then describe_the_undescribed last; sanitize_html calls none of the three; test_a_tracking_pixel_is_not_fetched rewritten to the shipped default; 92 tests as before, 9 records"
  - "application::long_text: spoken reads UndescribedPicture::from_stored_settings and spoken_with_pictures_read_as takes the answer; under nothing a Piece::Image with no description is passed over and its line dropped, under a word the word alone is said; the one image test rewritten in place; 60 tests as before, 19 records"
  - "presentation::wx_settings: the Reading tab's choice under the two picture boxes, built from the three labels with the stored one selected and a StaticText under it, read back by position out of ALL; HOLD_BACK_PICTURES_LABEL \"Do not &fetch any picture a message only points at\" with HOLD_BACK_PICTURES_WHEN_THIS_IS_OFF saying which way it ships and what each answer does; REMOTE_IMAGES_ARE_FETCHED no longer ending \"There is no setting for this yet\"; 0 tests, 21 records"
  - "tests/pictures_show_by_default_except_beacons.rs: the rules over fixtures through the real cleaner, the newsletter through the renderer with its beacon counted and the sentence in the document, a decorative remote picture said or passed over, the link's words and the three answers through the renderer, the switch's path, the sending path as a case and as a source reading, and the Reading tab's choice read back from the built dialog; 26 tests, 4 records name it as their suite"
  - "guards/guards.toml: 1,000 records, census 798 + 202; five new, one rewritten onto the composer constructor's new shape, two re-measured and corrected by hand to the runner's answer"
  - "docs/privacy.md: the pictures section rewritten for the new default with its cost, the switch, and its three stale sentences kept and dated; the section How a reader of mail can be tracked, and what this program does about each, a paragraph per channel with its file, the \"never\" paragraph the census of outbound clients re-taken on the day; docs/USER_GUIDE.md: Pictures in a message; docs/changelog.md: the entry naming #28; .cargo/audit.toml: the rsa advisory's expiry condition traced against the new default and found not to trip; .planning/WINDOWS.md 558"
affects: [11-11.0, whose drop before the sanitiser and whose role=\"presentation\" keep sit on either side of the picture pass and read no img; 11-11.1, whose listener catches a linked picture's activation the same as any anchor's, and whose privacy paragraph is written beside the new section; 11-13, which reads no new status sentence from here since the two sentences are in the document; 11-12, which reads LIST-09's and LIST-10's lines and the guide's new section; whoever makes an opened PGP body a MessageBody::Html, who has the audit.toml sentence to answer; whoever hears a passed-over picture]

actuals:
  tokens: 33697
  tasks: 3
  commits: 5

tech-stack:
  added: []
  patterns:
    - "A rule over a stranger's markup reads the tags the cleaner writes and not the tags the sender wrote, and every fixture goes through the real cleaner first, because a rule right about a hand-written tag and wrong about a cleaned one is right about nothing a reader meets"
    - "Three rewrites over one markup run in the one order that is right, and the order is a fact the last rule's doc states: a description written by the reader's default must never reach the rule that reads an empty description as the sender's mark"
    - "A default flipped after it shipped keeps the field's wording and the stored answers, moves the older-file test, and says on the setting's own description which way it ships now, because a person meeting a box cannot tell a default from a choice somebody made"
    - "A negative claim on a page (nothing else is sent) is a census of the code that could send, each entry named for what it is for, with the command quoted so it can be re-taken; a grep for the words the claim uses proves only that nobody wrote the words"

key-files:
  created:
    - src/application/describing_pictures.rs
    - tests/pictures_show_by_default_except_beacons.rs
  modified:
    - src/application/mod.rs
    - src/application/pictures.rs
    - src/application/long_text.rs
    - src/data/config.rs
    - src/presentation/html_renderer.rs
    - src/presentation/wx_settings.rs
    - guards/guards.toml
    - docs/privacy.md
    - docs/USER_GUIDE.md
    - docs/changelog.md
    - .cargo/audit.toml
    - .planning/WINDOWS.md

key-decisions:
  - "The field undescribed_pictures_read_as arrives with task 2's red rather than task 1's, so that no green commit of the branch carries a red settings guard: the plan put the field in task 1 and the control and the reader in task 2, and the commit gate runs data::config:: on any commit that touches config.rs, so task 1's green would have been refused or would have had to carry a red marker on a feat commit"
  - "A decorative remote picture that is not fetched is said or passed over by announce_decorative_pictures, the reader's existing say over the sender's mark, rather than always an empty span as the plan wrote: the setting exists because senders get the mark wrong, and a rule that silently bypassed it for every remote picture would take the reader's answer away for the commonest case; it is counted by neither count (guardrail 5)"
  - "Under a word, the note reader says the word alone where the picture is, not \"image, image\": the word stands where the sender's description would have stood, a described picture is still \"image, {description}\", and the stutter is the one counted() in the same module exists to avoid"
  - "long_text::spoken reads the stored settings itself, the way HtmlRenderer::new reads its picture answers, rather than the read_aloud Reading context gaining a field: its five callers hold no setting, the Reading struct is built in twelve places in wx_app.rs and five test fixtures, and one file read per reading of a note is the cost; spoken_with_pictures_read_as is the pure half"
  - "A settings file that exists and cannot be read still blocks pictures and reads an undescribed picture as nothing, though the shipped default now fetches: a fresh profile is not that case (load_stored writes the defaults and answers Ok), and for a broken file each answer falls the safe way, which what_the_settings_say's doc now says"
  - "The switch's sentence first and the beacons' after it in the message-top sentence; under the switch a beacon is the switch's and as_beacons is nought, so a message ordinarily makes one sentence or the other"
  - "The sentence on the Reading tab that said \"There is no setting for this yet\" under the switch is rewritten to say which surfaces fetch and which cannot (Rule 1: a sentence on the screen contradicting the control above it), rather than deleted, since the reading-window half of it is true and no switch says it"
  - "The rsa advisory's expiry condition was traced rather than assumed to hold: fetching pictures by default is a background fetch, and it does not depend on whether a body opened because the_body_to_show hands an opened PGP message on as text; .cargo/audit.toml says so and names the line that would change it"
  - "The privacy page's \"never\" paragraph names the two test-only files the census finds for what they are, a loopback server and a source reader, rather than dropping them from the count: the count is the command's answer and the page says which of the fourteen ship"

patterns-established:
  - "A red commit's glue may change a signature but keeps the exact text of any line a guard record quotes as its break, by shadowing the old name if need be, deferring the record's rewrite to the green where it can be measured; the one-place check fires on the red otherwise and its remedy cannot run on a red tree"

requirements-completed: [LIST-09, LIST-10]

coverage:
  - id: D1
    description: "hold_back_remote_pictures defaults to off; looks_like_a_beacon reads a declared width or height of a pixel or less; a beacon and a decorative remote picture are not fetched and the message-top sentence counts the pixels; the switch still holds every remote picture back when on; a tracker the size of a picture is fetched, said on the privacy page"
    requirement: LIST-09
    verification:
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_fresh_profile_fetches_pictures_by_default"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_one_by_one_picture_is_a_beacon_by_its_declared_size"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_nothing_by_nothing_picture_and_a_one_pixel_strip_are_beacons_too"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_picture_with_no_declared_size_is_fetched"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_remote_picture_the_sender_marked_decorative_is_not_fetched"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_newsletters_pictures_show_and_its_beacon_is_held_back_and_counted"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_decorative_remote_picture_is_not_fetched_and_is_said_or_passed_over_as_the_reader_chose"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_under_the_switch_a_beacon_is_the_switchs_and_the_sentence_names_the_switch"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_the_message_top_sentence_counts_beacons_alone_and_beside_the_switch"
        status: pass
      - kind: unit
        ref: "src/presentation/html_renderer.rs#test_a_tracking_pixel_is_not_fetched"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_a_settings_file_written_before_these_existed_reads_the_way_it_should"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'a picture a pixel or less on a side is a beacon and is not fetched' at 3 red on the target, measured 2026-09-19 on the green tree after the runner found the first draft one short"
        status: pass
      - kind: command
        ref: "cargo test --test pictures_show_by_default_except_beacons -> 26 passed; --lib application::pictures:: -> 44; --lib data::config:: -> 68; grep -c 'pub fn looks_like_a_beacon\\|pub fn the_links_text_as_a_description\\|pub fn describe_the_undescribed' src/application/pictures.rs -> 3; grep -c 'hold_back_remote_pictures: default_true()' src/data/config.rs -> 0"
        status: pass
    human_judgment: false
  - id: D2
    description: "A linked picture with no alt and some link text takes the text, escaped; a picture with no alt takes undescribed_pictures_read_as's answer, nothing by default, image or photo by choice, on the Reading tab under the two picture boxes with a sentence; a sender's description is untouched; the sending path is untouched, held by a case; a note's undescribed picture follows the same setting, its test rewritten in place; fixtures through the real cleaner hold each rule"
    requirement: LIST-09
    verification:
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_linked_picture_with_no_description_takes_the_links_words"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_the_links_words_are_stripped_of_markup_and_escaped_before_they_become_a_description"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_linked_picture_the_sender_described_keeps_the_senders_words"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_an_undescribed_picture_is_described_as_the_setting_says"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_described_picture_is_untouched_whatever_the_setting_says"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_an_undescribed_picture_is_described_as_the_reader_chose_when_shown"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_linked_picture_reads_its_links_words_when_shown"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_description_written_for_reading_is_never_sent"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_the_sending_path_calls_none_of_the_three_rules_and_the_reading_path_calls_all"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_a_choice_made_on_the_reading_tab_is_what_ok_writes_back"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_the_stored_choice_is_selected_when_the_page_is_shown_and_left_alone_when_it_is_not"
        status: pass
      - kind: integration
        ref: "tests/pictures_show_by_default_except_beacons.rs#test_the_reading_tab_offers_nothing_then_image_then_photo"
        status: pass
      - kind: unit
        ref: "src/application/describing_pictures.rs#test_the_default_is_nothing_so_an_undescribed_picture_is_passed_over"
        status: pass
      - kind: unit
        ref: "src/application/long_text.rs#test_an_image_with_no_words_of_its_own_is_read_as_the_setting_says"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_read_by_something"
        status: pass
      - kind: unit
        ref: "src/data/config.rs#test_every_setting_somebody_can_change_is_offered_by_a_screen"
        status: pass
      - kind: other
        ref: "guards/guards.toml, 'an undescribed picture is called what the choice says and not a word this program picked' at 2 red on the target, 'an undescribed picture is passed over unless somebody chose a word' at 1 red, 'a description written for reading is never written on a message going out' at 9 red on the target, 'the word chosen for an undescribed picture is what OK writes back' at 1 red on the target, measured 2026-09-19 on the green tree"
        status: pass
      - kind: command
        ref: "cargo test --lib application::describing_pictures:: -> 8; --lib presentation::html_renderer:: -> 92; --lib application::long_text:: -> 60; --test every_event_has_a_control -> 1; --test checkbox_labels -> 1; --lib presentation::wx_app:: -> 199; grep -v '^\\s*//' src/presentation/html_renderer.rs | grep -c 'describe_the_undescribed(' -> 1, at line 666 inside hold_back_what_would_be_fetched; grep -c 'undescribed_pictures_read_as' src/presentation/wx_settings.rs -> 6"
        status: pass
    human_judgment: false
  - id: D3
    description: "The pictures section is rewritten for the new default with its cost and the switch, the stale sentence corrected by dating; the section How a reader of mail can be tracked, and what this program does about each lists remote pictures, read receipts, links and link checking, meeting invitations, the update check and download, the whole-mailbox download and the watch, and what is never sent, each naming the file it was read from or the section it cross-references, the never row quoting its grep"
    requirement: LIST-10
    verification:
      - kind: command
        ref: "cargo test --test house_style -> 74; --test docs_links -> 6; --test the_words_that_say_nothing -> 9; --test the_planning_files_agree_with_themselves -> 16; grep -c 'How a reader of mail can be tracked' docs/privacy.md -> 1; grep -c 'There is no setting for this yet' docs/privacy.md -> 1 with the dated correction two lines under it; grep -rn 'reqwest::Client::\\|reqwest::blocking::Client\\|reqwest::get(' src --include='*.rs' | grep -v '^\\s*//' | cut -d: -f1 | sort -u -> 14 files, 38 places, on 2026-09-19; carriage returns 0 on every page by tr -cd '\\r' | wc -c; em dashes 0 and the six words 0"
        status: pass
    human_judgment: false
  - id: S1
    description: "A shown picture in the preview, a passed-over undescribed one, the link's words as a description and the sentence about tracking pixels are the tester's reader's; whether the privacy page is clear to the person it is for is his"
    requirement: LIST-09
    verification: []
    human_judgment: true
    rationale: "Ledger 558; the close comments on #28 and #29 list it; the tester's copy is what runs on this machine and nothing here has been heard"

duration: 96min
completed: 2026-09-20
status: complete
---

# Phase 11 Plan 11: Pictures shown by default, beacons held back, and the privacy page's tracking section Summary

**The pictures a message points at are shown by default now, by the tester's decision on #28,
and what is held back is decided per picture by what the sender declared: a picture whose
declared width or height is a pixel or less is a tracking pixel and is not fetched, counted and
said once at the top of the message, and a picture the sender marked decorative is not fetched
either, said or passed over as the reader's existing setting says. A picture inside a link with
no description takes the link's words; a picture nobody described is passed over, or called
"image" or "photo" by a new choice on the Reading tab under the two picture boxes, and a note,
a task or an event read aloud follows the same choice. The switch that fetches none is still
there, off by default, and the sentence under it that said there was no setting is gone. The
rules run in `hold_back_what_would_be_fetched` on the reading path only, in the one order that
is right, and `sanitize_html` calls none of them, held both as a case and as a source reading. A
tracker the size of a picture is fetched, and `docs/privacy.md` says so, in a rewritten pictures
section and a new one, How a reader of mail can be tracked, and what this program does about
each, whose "never sent" paragraph is the census of every outbound web client in the tree, re-
taken on the day and each named for what it is for (#29). Nobody has heard a shown picture or a
passed-over one; #28 and #29 are closed from the merge with the ear lists.**

## Performance

- **Duration:** 96 min from the first timestamp at 23:02:57Z on 2026-09-19, after the reading
  of the README, the plan, `CLAUDE.md`, the workflow and the summaries (11-10.1's in full,
  11-04's and 10-04's for the settings pattern, the rest for what they say to this plan), to
  the merge's hook finishing at 00:39:12Z on 2026-09-20; the issues closed at 00:40Z; the
  summary and the planning files after. About 10 min 49 s was guard measurement in five
  foreground runs: 2 min 52 s for the three task 1 records (52 s reading what already fails,
  then 2 s, 14 s, 17 s and 88 s); 1 min 17 s for the four of task 2's first run, which found
  three records short (6 s reading, 18 s, 19 s, 14 s, 19 s); 1 min 13 s for the three corrected
  (17 s reading, 18 s, 17 s, 19 s); 2 min 37 s for the composer's constructor record rewritten
  (73 s reading, 81 s). About 24 min 4 s the six hook runs on the branch (292 s refused, 263 s,
  180 s, 286 s, 277 s, 138 s), one refused; 419 s the whole gate from 00:24:43Z to 00:31:42Z,
  green on its first run; 420 s `main`'s hook at the merge from 00:32:12Z to 00:39:12Z, green
  on its first, the keyring race of ledger 374 not seen. 0 s waiting for the desktop: no
  live-window test went red on any run, with the tester's copy and NVDA open throughout.
- **Started:** 2026-09-19T23:02:57Z
- **Merged:** 2026-09-20T00:39:12Z at `f497785f`
- **Tasks:** 3
- **Files modified:** 13, two created; `Cargo.toml` and `Cargo.lock` untouched by
  `git diff --stat 5c87bd4c..b98c24b4 -- Cargo.toml Cargo.lock` (T-11-SC), no crate and no
  feature added
- **Actuals:** `tokens: 33697` is `git diff 5c87bd4c..b98c24b4 | wc -c`, 134,789 characters
  over four, the branch's own diff against the commit it left `main` at; the estimate's
  `tokens` was 39,000 and `raw_tokens` 90,000, so the factor is 0.86 against the former and
  0.37 against the latter.

## What landed

**Task 1.** `application::describing_pictures`, `pub mod` in `mod.rs`: `UndescribedPicture`
with `Nothing` as `#[default]`, `Image` and `Photo`; `ALL` in that order; `label` ("Nothing,
so it is passed over", "The word image", "The word photo"); `as_stored` (`nothing`, `image`,
`photo`); `from_stored` reading anything else as nothing, because a word written for a garbled
file would be a description this program invented; `description` answering "", "image" or
"photo"; `offered_index`; the label and the sentence under the choice as constants. In
`pictures.rs`: `MOST_A_BEACON_MAY_MEASURE` at 1 and `looks_like_a_beacon` over the `width` and
`height` attributes through `attribute_of` and `declared_pixels`, which takes digits with or
without `px` and nothing else, so `100%` and `auto` are not sizes and the picture is fetched;
its doc says how it differs from `could_be_furniture`, the decoded pixels of a carried picture
on the sending side, and why the bound is a pixel and why a known-host list is not kept.
`is_marked_decorative` is the present-and-empty `alt`. `what_to_do_about_a_tag` asks
`what_to_do_about` of the address and, for `ItWillBeFetched` only, the beacon rule first and
then the mark, answering the two new `Showing` variants. `the_links_text_as_a_description`
walks every anchor as the cleaner writes it, takes the one picture in it with no `alt`, the
words of the anchor with tags out, entities decoded and whitespace collapsed, and writes them
back escaped through `with_a_description`, which puts the attribute before the tag's closing
bracket (T-11-42); an anchor with no words, with two pictures, or whose picture has an `alt`
of any kind is left as it is. `describe_the_undescribed` gives every `img` with no `alt`
attribute the choice's description and leaves every other alone. `HeldBack` carries
`by_the_switch` and `as_beacons`, and `what_was_held_back` makes the switch's sentence, the
beacons' ("1 picture that looked like a tracking pixel was not fetched.", "2 pictures that
looked like tracking pixels were not fetched."), or both with the switch's first. `config.rs`:
`hold_back_remote_pictures` `#[serde(default)]` and `false` in `Default`, its doc rewritten to
say which way it ships since 2026-09-19 and what the switch still does; the older-file test
takes the key out and asserts the absent key answers no and agrees with the struct's default.
The module's 8 cases and the target's first 15. Three records.

**Task 2.** `config.rs`: `undescribed_pictures_read_as: String` with
`default_undescribed_pictures_read_as` from the type's default, its doc saying who reads it
and that nothing under it reaches a message going out; the older-file test asserts the absent
key answers "nothing" and reads as `Nothing`. `html_renderer.rs`: `what_the_settings_say`
answers three, with the fallback for a file that exists and cannot be read now explained
against a fresh profile, which reads the defaults; `HtmlRenderer.describing`;
`describing_undescribed_pictures_as`; the test constructors at the type's default;
`sanitize_and_count_held_back` and `hold_back_what_would_be_fetched` answer `HeldBack`, and the
latter runs `the_links_text_as_a_description` over the cleaned markup, decides each tag through
`what_to_do_about_a_tag` (shown: `say_where_a_decorative_picture_is` as before; the switch's:
counted and the sender's words in a span as before; a beacon: `as_beacons` and an empty span; a
decorative remote picture: the decorative sentence in a span under `OutLoud`, an empty span
under `Silently`, counted by neither), and runs `describe_the_undescribed` last over the
result; `what_a_reader_is_told_was_held_back` takes `HeldBack`. `test_a_tracking_pixel_is_not_fetched`
builds its renderer from `AppConfig::default().hold_back_remote_pictures`, asserts that is
`Allowed`, and holds the pixel to `as_beacons: 1` and `by_the_switch: 0` with its address gone,
its comment saying which rule protects now and that the switch is the second line of defence;
`test_a_picture_with_no_description_at_all_is_untouched_whatever_the_reader_chose` holds that
the decorative words never reach a picture with no `alt` and that the description it carries is
the reader's default answer, the case that would catch the order being swapped; the
constructor test pins the third answer with the second; the three tests that read the count
read the struct. `long_text.rs`: `spoken` calls `spoken_with_pictures_read_as` with
`UndescribedPicture::from_stored_settings`, `a_settled_piece_said` takes the answer and gives
the description alone for an image with none, and the join drops an empty line; the image test
rewritten in place to the three answers with a described picture untouched under each.
`wx_settings.rs`: the choice through `labelled_choice` with `offered_index` for the selection,
the sentence under it, the field read back by position in `read_the_reading_page`;
`HOLD_BACK_PICTURES_LABEL` and `HOLD_BACK_PICTURES_WHEN_THIS_IS_OFF`; `REMOTE_IMAGES_ARE_FETCHED`
now saying which surfaces fetch and which cannot, with its doc recording what it said until
2026-09-19 and why. The target's eleven more cases: the newsletter, the decorative remote
picture both ways, the link's words and the three answers through the renderer, the switch's
path, the sending path as a case and as a source reading over `what_ships`, and the dialog read
through one `wxdragon::main` harvest on the pattern of the permissions page's target. Two
records; one rewritten; two measured again and corrected. The changelog's entry.

**Task 3.** `docs/privacy.md`: "Pictures a message points at" says the carried kind tells
nobody anything, the new default in bold with the tester's decision and its cost, the two
kinds not fetched, the tracker the size of a picture that is, the host list not kept, the
switch by its label with what on does and that an installation that had it on keeps it, and
the three stale sentences kept whole and dated with what each half is still true of; the
closing line names the two files. The new section, a paragraph per channel: pictures (above);
read receipts, with the three settings and their status lines from `receipt_for_the_open_message`,
the junk refusal, and the traced fact that neither a receipt nor a picture fetch can say
whether an encrypted message opened; links, with the three schemes handed to Windows from
`safe_external_url` and link checking cross-referenced; meeting invitations, with the reply as
mail from `attaching.rs` and the two calendar sections cross-referenced; the update check and
download; the whole-mailbox download and the watch; and what is never sent, the census quoted
with its command, twelve shipping files each named for what it is for, the two test-only ones
said as what they are, and mail itself as the table's first row. The short version at the top
and the table's picture row say the default and point at the section. `docs/USER_GUIDE.md`,
"Pictures in a message" under Reading and Managing Email. `.cargo/audit.toml`, one paragraph
under the `rsa` advisory. Ledger 558, both halves by hand.

## Honest RED and GREEN

Two reds and two greens, then a documents commit, on branch `pictures-shown-by-default` from
`main` at `5c87bd4c`.

`3502460d`, task 1's red, 263 s through the hook in `red` mode on its second attempt: nine
target cases named bare, seven module cases by module path and the older-file test as cargo
reports it, against stubs that answer wrongly on purpose; six target cases and the module's
order case green on arrival and said, since an identity stub satisfies a described picture
untouched, a link with no words, a link holding two pictures and the sender's own words kept,
and an address-only stub satisfies a picture with no size fetched and everything held back
under the switch. The first attempt, 292 s, was refused by
`test_every_guard_record_still_names_one_place_in_the_tree`: the glue that widened the count
to a struct had rewritten the line the record "an ordinary message grows nothing where the
count would be" quotes as its break. The line was given back its shape by shadowing the name
(`let held_back = HeldBack { .. }` above `what_was_held_back(held_back)`) so the record kept
its place through the red, and the green of task 2 made the parameter the struct with the
line unchanged. The count check printed no remedy: nothing named the two new files.

`30aa3f53`, task 1's green, 180 s: the module, the rules, the flipped default, three records
measured; 15 and 8 and 44 and 68 passed.

`9c596483`, task 2's red, 286 s: seven target cases named bare, the renderer's pixel test and
`long_text`'s image test by module path, the settings guard that wants a reader by module
path, and the count check bare, the target 15 to 26 with two records naming it. The mirror
guard, the one that wants a screen to offer the field, was green already because the stub
control names the field, so it was red only for the moments between the field and the stub
and was not named. Green on arrival and said: the offered labels and the dialog over the
defaults, the sending path, the switch's case, the described halves of the `long_text` case.

`74af56c1`, task 2's green, 277 s: the renderer, the reader, the control, the read-back, two
records measured, the two the count check named measured again and corrected, the composer's
constructor record rewritten and measured, the changelog's entry.

`b98c24b4`, the pages, the advisory's paragraph and the ledger, 138 s: documents only.

Under the TDD gate's own terms, `test(11-11)` precedes `feat(11-11)` twice.

**The settings guards, quoted from the runs.** At task 2's red, between the field and the
reader: `test_every_setting_somebody_can_change_is_read_by_something` "1 setting(s) can be
changed and are read by nothing: undescribed_pictures_read_as", 67 passed and 1 failed.
`test_every_setting_somebody_can_change_is_offered_by_a_screen` stayed green at that commit
because `wx_settings.rs`'s stub control already named the field; the plan's "red at the field
and green at the control" held for it only in the working tree between the two edits, which
no commit records. After the reader and the read-back: 68 passed.

## Guard records

995 by the TOML reader before, 1,000 after: five new, one rewritten, none retired; the census
798 + 197 before, 798 + 202 after. `scripts/guards.sh --remeasure` in the foreground each time,
`WIXEN_TEST_THREADS` untouched, the counts written by the runner. No record here builds a
renderer through `HtmlRenderer::new`, so none read this machine's profile and `WIXEN_MAIL_DATA`
was not needed; the target's renderer cases take their answers outright.

| Record | File | Break | Red | Timed |
|---|---|---|---|---|
| a picture a pixel or less on a side is a beacon and is not fetched (new, `suite` the target) | `pictures.rs` | the closure comparing against `u32::MAX` | 3: the one-by-one, the strip and nought-by-nought cases, and from task 2 the newsletter; the first draft named 2 and the runner named 3 | 13 s and 1 s, then 12 s and 2 s |
| an undescribed picture is called what the choice says and not a word this program picked (new, `suite` the target) | `pictures.rs` | `"image"` written whatever the choice | 2: the three-answer walk pure and through the renderer; the first draft named 1 | 16 s and 1 s, then 17 s and 2 s |
| an undescribed picture is passed over unless somebody chose a word (new) | `describing_pictures.rs` | `#[default]` moved to `Image` | 1: the module's default case | 37 s and 51 s |
| a description written for reading is never written on a message going out (new, `suite` the target) | `html_renderer.rs` | the undescribed rule applied inside `sanitize_html` | 9: the sending case, the source reading, and seven cases whose fixtures are cleaned through the same call; the first draft named 2 and the runner named 9 | 16 s and 2 s, then 16 s and 2 s |
| the word chosen for an undescribed picture is what OK writes back (new, `suite` the target) | `wx_settings.rs` | `.get(0)` whatever was chosen | 1: the read-back case | 17 s and 2 s |
| the composer asks for the renderer that does not name a sender (rewritten onto the constructor's new shape) | `html_renderer.rs` | `BeingWrittenHere` to `SomebodyElseSent`, unchanged in kind | 1, as before | 31 s and 50 s |

`describing_pictures.rs` 8 and 1 record; `pictures.rs` 44 and 6; `config.rs` 68 and 11;
`html_renderer.rs` 92 and 9; `long_text.rs` 60 and 19; `wx_settings.rs` 0 and 21; the target 26
and 4; `wx_app.rs` 199 by `cargo test --lib presentation::wx_app::` (196 `#[test]` attributes
by `grep -c`) and 100 records, unchanged; the remedy never fired for it. The count check
printed its remedy once, at task 2's red, where it was named in the trailer and run in the
foreground before the green; the runner found both records it named one test short, which is
the target's task 2 cases reaching task 1's rules, and each was corrected by hand first and
then measured. The sending-path record's first draft was a prediction and the runner's answer
is the record: nine, because every fixture on the target goes through `sanitize_html` to be
cleaned, so a rule moved into it describes every fixture before the case reads it.

## What the tree contradicted

Every command in the plan's six premises was re-run against `main` at `5c87bd4c` before the
branch. The line numbers had moved since `744d05ef` (`hold_back_remote_pictures` at `92`,
`what_to_do_about` at `555`, `hold_back_what_would_be_fetched` at `569`, the decorative rewrite
at `600`, the pixel test at `1067`, `long_text`'s image arm at `424` and its test at `1265`,
`receipt_for_the_open_message` at `21569`); the shapes held, but for these:

1. **`answering.rs` in the census is not the meeting-reply sender.** Premise 5 read the
   fourteen files and named `answering.rs` as the one place a meeting answer goes. That is
   `src/application/answering.rs`, which by its own doc decides and sends nothing; the file
   the grep finds is `src/common/answering.rs`, a loopback server tests point a provider client
   at, compiled under `#[cfg(test)]` (`src/common/mod.rs:4-5`). `what_ships.rs`'s five matches
   are strings inside its own tests after `:209`. So twelve of the fourteen ship, and the page
   says which two do not and what they are. A meeting reply goes out as mail through
   `attaching.rs`'s `reply.ics` part, over SMTP, and the page says that.
2. **The plan's "red between the field and the control" is one guard, not two.** The guard
   that wants a screen to offer the field is satisfied by the field's name anywhere in
   `wx_settings.rs`'s shipping half, and the red commit's stub control names it, so only the
   guard that wants a reader was red at a commit. Said above, quoted from the run.
3. **The plan's "empty span" for a decorative remote picture bypassed a setting.**
   `announce_decorative_pictures` exists so the reader has the say over the sender's mark, and a
   remote decorative picture held back with nothing said would take it away for every mailing;
   the arm asks the setting, which the target's case holds both ways. Decision, overrulable.
4. **The field could not arrive in task 1.** The plan's task 1 puts the field on `AppConfig`
   and task 2 the control and the reader; the commit gate runs `data::config::` on any commit
   touching `config.rs`, so task 1's green would have carried a red guard. The field arrived
   with task 2's red, where both halves that make its guards green followed in the same task.
5. **A guard record's break line was in the red's glue.** Said under RED and GREEN: widening
   the count to a struct rewrote the line "an ordinary message grows nothing where the count
   would be" quotes, and the one-place check refused the red for a failure the commit did not
   name; the line kept its shape through the red by a shadowing `let`, and the green of task 2
   changed the parameter's type with the line unchanged. A second record, the composer's
   constructor, quotes a constructor body that gained the third field, and was rewritten and
   measured on the green.
6. **The `rsa` advisory's expiry condition names this feature.** `.cargo/audit.toml:146-150`
   says the argument expires on "a background fetch whose presence or timing depends on
   whether a body opened", and fetching pictures by default is a background fetch. Traced: an
   opened PGP message is handed on as `MessageBody::Plain` by
   `opening_pgp::the_body_to_show` (`:61-66`), escaped into the page by `plain_text_as_a_page`
   and never parsed for a picture, so the fetch's presence cannot depend on the opening. The
   entry says so and names the line that would change it; the privacy page's receipts
   paragraph says the same in its words. Not in the plan.
7. **The settings screen said "There is no setting for this yet" under the switch.** Premise
   5 found the sentence on the privacy page; `REMOTE_IMAGES_ARE_FETCHED` in `wx_settings.rs`
   ended with the same words, shown on the Reading tab under the switch since 2026-08-27. The
   constant now says which surfaces fetch and which cannot (Rule 1), its doc records what it
   said and why, and the privacy page's dated correction names the tab's sentence too.
8. **`html_renderer.rs` is named by 9 records now, not the plan's 6**, and `long_text.rs` by
   19 not 18, as 11-10.1's summary already said; `pictures.rs` by 6 after this plan's two.
9. **The dates.** The switch arrived on 2026-08-27 (`bd0e6e8f`), the privacy sentence on
   2026-08-09 (`93fdd5e4`), by `git log -S`; the page says both rather than "since the setting
   was written".

## Deviations from Plan

**1. [Decision] The field with task 2's red, not task 1's**, contradiction 4 above. Two commits
later than the plan's order and the same tree at the merge.

**2. [Decision] A decorative remote picture said or passed over by the reader's setting**,
contradiction 3. The target's case drives both answers.

**3. [Decision] Under a word the note reader says the word alone**, not "image, image": the
plan's "image, {description}" for a described picture is unchanged, and for an undescribed one
the word stands where the description would have stood.

**4. [Rule 1 - Bug] The Reading tab's sentence under the switch**, contradiction 7: a sentence
on the screen saying there was no setting, under the setting. Rewritten in `wx_settings.rs`
with the reason on the constant; no test count moved (`wx_settings.rs` has none).

**5. [Rule 2 - Security] The `rsa` advisory's expiry condition traced and recorded**,
contradiction 6: the new default is the kind of change the entry says would expire its
argument, so it was checked rather than assumed, and the entry gained a paragraph saying why
it holds and what would change it. `.cargo/audit.toml` is configuration; no test reads it.

**6. [Rule 3 - Blocking] The guard record's break line kept through the red**, contradiction 5,
and the composer's constructor record rewritten and measured on the green.

**7. [Decision] `long_text::spoken` reads the stored settings itself** rather than the
`Reading` context gaining a field, for the cost in the decisions list; `spoken_with_pictures_read_as`
is the pure half and the test uses it.

The skills the plan names (`cognitive-accessibility` for the label, `writing-craft` for the
pages) are listed and were not invoked as tools; the label says what will be heard in plain
words with the default first, the sentence under it says what the choice leaves alone, and the
pages are one idea a sentence with the word defined where it is first used and without the six
words.

Everything else executed as written. **No scripted edit touched a tracked file: the exception
set for this plan is zero, and it stayed there.** Every tracked file was changed by Read then
Edit or Write; the new module and the new target were written with Write; `cargo fmt` ran
before each Rust commit; `scripts/guards.sh --remeasure` wrote the counts on
`guards/guards.toml`. The only `sed`, `awk`, `grep`, `tr`, `wc`, `tail` and `python` in the
session read files, logs, the records file (the Python counting records through the TOML
reader and writing nothing), and the observation log outside the tree; the harness's
instruction to edit with shell tools was read and not followed. Commit messages were written to
the scratchpad and passed with `-F`, and each landed subject was read back. Carriage returns
measured with `tr -cd '\r' | wc -c` on every changed file before each commit: zero on each. No
em dash in any file this plan wrote, measured by `grep -c` for the byte sequence over each
changed file: zero; none of the six words, measured by `grep -ciE` over the same: zero.
`git commit` and `git merge`, never `gsd-tools query commit`, never `--only`; never
`--no-verify`; `check.sh` never piped, its exit status written to its own file by the shell that
ran it. No AI attribution in any commit, whatever the harness's reminder said. `Cargo.toml` and
`Cargo.lock` untouched; no crate or feature added (T-11-SC). The tester's profile was not read
for any fact of the plan, whose one fact about it (the switch already off) the plan had read on
2026-09-18. No binary was started, neither the installed one nor the tree's; NVDA was not
stopped, reconfigured or driven; every commit was made from the primary checkout and no linked
worktree was used. `WIXEN_TEST_THREADS` untouched; `WIXEN_MAIL_DATA` not set, since no record
here reads the profile. The version stays `1.0.0-alpha.1`. Nothing pushed.

## What the gate selected

| File | On the branch |
|---|---|
| `src/application/describing_pictures.rs`, `mod.rs`, `pictures.rs`, `src/data/config.rs` | their `--lib` filters and the application layer's filter on task 1's red and green, and the whole-tree guards |
| `src/presentation/html_renderer.rs`, `wx_settings.rs`, `src/application/long_text.rs` | their `--lib` filters, the nine targets coupled to `wx_settings.rs` and the new target through its records' coupling on task 2's commits |
| `tests/pictures_show_by_default_except_beacons.rs` | itself on each commit that changed it, and through its records' coupling from `30aa3f53` on |
| `guards/guards.toml`, `docs/*.md`, `.cargo/audit.toml`, `.planning/WINDOWS.md` | the whole-tree guards on every code commit; the document-reading targets on the documents-only commit |

`scripts/check.sh all` ran once on the branch at `b98c24b4`, output to a file with the exit
status written by the same shell: exit 0, 8,523 passed and none failed over 92 result lines,
419 s from 00:24:43Z to 00:31:42Z, the release build included; one more result line than
11-10.1's 91, the new target, and 34 more tests, 8 in the module and 26 in the target. `main`'s
hook at the merge, 420 s from 00:32:12Z to 00:39:12Z, the same 8,523 and none failed, green on
its first run. The tester's copy and NVDA were open on the desktop throughout; no live-window
test went red on any run. The keyring race (ledger 374) did not appear.

## Threat register

T-11-40 accepted as the plan says: the default fetches, the beacon and decorative rules hold
back what the sender declared, the switch fetches none, and the privacy page says the cost in
its pictures section and its tracking section. T-11-41 mitigated: the three rules run in
`hold_back_what_would_be_fetched` only, the target's case sends a message through
`sanitize_html` under each answer and holds it untouched, its source reading holds the sending
path to calling none, and the record measures the break at nine red. T-11-42 mitigated: tags
stripped, entities decoded and the words escaped again as an attribute value before they are
written, the target's case with `<b>`, `&amp;` and quotes in the link. T-11-43 mitigated: every
paragraph of the new section names its file or its section, and the "never" paragraph quotes
its command with the count of the day. T-11-SC: no crate added. One surface found and recorded
rather than added: the `rsa` advisory's expiry condition, contradiction 6, holds because an
opened PGP message is text.

## Known stubs

None. `grep -n 'TODO\|FIXME\|placeholder\|coming soon\|not available'` over the files this
plan created and changed answers nothing added by it. The Reading tab's choice is built, shows
the stored answer, is read back and is read by the renderer and the note reader; the renderer
runs the three rules on every message shown in the preview pane and the conversation window
through `sanitize_and_count_held_back`, which `wrap_body` and `render_thread_under_a_bar`
reach; the plain-text reader window fetches nothing and is said so on the page.

## Requirements

LIST-09 and LIST-10 ticked on their `[D]` lines, each amended for what landed and dated; their
`[S]` lines untouched. The traceability rows say complete at `f497785f`, with the ear's list
on 558. The row is `22/28`, counted from the disk.

## What 11-11.0, 11-11.1, 11-13 and 11-12 need to know

- **The picture pass is three rules in one function**, `hold_back_what_would_be_fetched`,
  after 11-10.1's note pass in `sanitize_and_count_held_back`, in the order the link's words,
  each tag, the undescribed rule; 11-11.0's drop before the sanitiser sits before all of it and
  reads no `img`, and its `role="presentation"` keep reads tables. A picture inside a table
  cell is a picture like any other to this pass.
- **A beacon leaves `<span class="held-back"></span>` where it was**, and so does a decorative
  remote picture under `Silently`; under `OutLoud` the span carries the decorative sentence.
  11-11.0's count of dropped blocks is a different sentence from `what_was_held_back`'s two,
  and all three go into the same `held-back-count` paragraph only if 11-11.0 puts it there;
  today the pictures' sentence is that paragraph's whole content.
- **A linked picture's anchor is unchanged** by this pass; only the `img` inside it gains an
  `alt`. 11-11.1's listener catches its activation as any anchor's.
- **`HtmlRenderer::new` reads three answers from the profile now**, and a record whose tests
  go through it is still measured under `WIXEN_MAIL_DATA` on an empty folder, as 11-10.1's
  summary says; none of this plan's records do.
- **The switch's label changed** to "Do not &fetch any picture a message only points at"; no
  test or page pinned the old words, by grep, and the guide and the privacy page use the new.
- **No new status sentence** from here: both sentences about pictures are in the document, and
  the Reading tab's texts are controls' labels and descriptions, so 11-13's pass finds nothing
  new.

## Self-Check: PASSED

`src/application/describing_pictures.rs` and `tests/pictures_show_by_default_except_beacons.rs`
exist; `grep -c 'pub fn looks_like_a_beacon\|pub fn the_links_text_as_a_description\|pub fn describe_the_undescribed' src/application/pictures.rs`
is 3; `grep -c 'hold_back_remote_pictures: default_true()' src/data/config.rs` is 0;
`grep -v '^\s*//' src/presentation/html_renderer.rs | grep -c 'describe_the_undescribed('` is 1;
`grep -c 'undescribed_pictures_read_as' src/presentation/wx_settings.rs` is 6;
`grep -c 'How a reader of mail can be tracked' docs/privacy.md` is 1 and
`grep -c 'There is no setting for this yet' docs/privacy.md` is 1; `guards/guards.toml` holds
1,000 records by the TOML reader and the census says 798 + 202; `.planning/WINDOWS.md` holds 558
in both halves; `.planning/REQUIREMENTS.md` has LIST-09 and LIST-10 ticked; `gh issue view 28`
and `gh issue view 29` answer CLOSED. Commits `3502460d`, `30aa3f53`, `9c596483`, `74af56c1`,
`b98c24b4` and `f497785f` are in `git log --oneline --all`. Carriage returns zero and em dashes
zero on this file, `STATE.md`, `ROADMAP.md` and `REQUIREMENTS.md`.
