---
phase: 14-the-real-account-proofs
plan: 03
subsystem: pim-sync
status: complete
tags: [google, calendar, contacts, tasks, oauth, account-manager, help, "#22", route-b]
requires: [14-01, 14-02]
provides:
  - "service::oauth: GOOGLE_CALENDARS_CONTACTS_AND_TASKS, its provider entry, the_google_sign_in_for, a_sign_in_is_held, a_google_token_from; a_google_token_for now takes the Account"
  - "application::who_holds_the_calendars: a_google_token_with, WhyNothingWasAsked::TheSeparateSignInRanOut, WhatTheSeparateSignInDoes and its sentences, SIGNED_IN_FOR_CALENDARS, signing_in_for_calendars_failed, what_sign_in_again_says_for_a_password_account, with_those_signed_in_for_calendars"
  - "the Account Manager's Sign In for Calendars, Contacts and Tasks (Alt+T), and AccountManagerAction::Updated carrying signed_in_for_calendars"
  - "docs/PROVIDER_SETUP.md: Calendars, contacts and tasks for a Gmail account"
affects: [14-04, 14-07, 14-08, 14-09, 14-12]
tech-stack:
  added: []
  patterns: ["a second sign-in for one provider under a name of its own, so its credential store entry is its own", "the token for a module chosen by how the account's mail signs in"]
key-files:
  created: []
  modified:
    - src/service/oauth.rs
    - src/application/who_holds_the_calendars.rs
    - src/application/forget.rs
    - src/presentation/wx_account_manager.rs
    - src/presentation/wx_app.rs
    - src/presentation/managers.rs
    - tests/one_check_says_who_runs_the_mail.rs
    - guards/guards.toml
    - docs/PROVIDER_SETUP.md
    - docs/USER_GUIDE.md
    - docs/KEYBOARD_SHORTCUTS.md
    - docs/ALPHA_TESTING.md
    - docs/installing.md
    - docs/privacy.md
    - docs/changelog.md
    - oauth.toml.example
decisions:
  - "The separate sign-in is a provider of its own, google-calendars-contacts-tasks, so keyring_service files it apart and entries_for_account, which forgetting and uninstalling read, names it with no other change"
  - "Both Google sign-ins ask for a refresh token, decided by Google's sign-in page rather than by the name gmail"
  - "The sign-in a module uses is chosen by use_oauth alone (the_google_sign_in_for), for the syncs and the free time lookup alike"
  - "Nothing held for the separate sign-in is the app-password reason, which now names the button; held and refused is a reason of its own"
  - "The Account Manager's buttons sit in two rows, in tab order, because a ninth button with a long label pushed the one row past the window"
metrics:
  duration: "about 5 hours, 2026-10-05"
  completed: 2026-10-05
actuals:
  # chars/4 over added lines, git diff main..HEAD on the branch before the documents commit, plus the documents
  tokens: 23000
  tasks: 3
  commits: 5
---

# Phase 14 Plan 03: A separate Google sign-in for calendars, contacts and tasks Summary

A Gmail account whose mail keeps its app password signs in to Google
through the browser a second time, for its calendars, contacts and tasks
alone, from the Account Manager's Sign In for Calendars, Contacts and Tasks
(`Alt+T`); the three syncs and Find When Everyone Is Free ask Google with
that sign-in; and Help's Setting up a provider walks through making the
Google testing key and placing `oauth.toml` (route B and answer 1 of
2026-10-05).

## What works now

- `OAuthService::providers()` holds a third entry,
  `google-calendars-contacts-tasks`, with Google's sign-in and token
  addresses and the calendar, contacts and tasks permissions, never mail. Its
  token is filed under `wixen-mail-google-calendars-contacts-tasks`, so
  `entries_for_account`, which removing an account and uninstalling read,
  names it with nothing else changed. Both Google sign-ins ask for
  `access_type=offline` and `prompt=consent`, chosen now by Google's sign-in
  page rather than by the name `gmail`.
- `oauth::the_google_sign_in_for(use_oauth)` picks the mail sign-in for an
  account whose mail signs in through the browser (D-14) and the separate
  one otherwise; `oauth::a_google_token_from` is the one place a Google token
  is got. `who_holds_the_calendars::a_google_token_with` (the syncs, through
  `a_google_token`) and `oauth::a_google_token_for` (the free time lookup,
  which now reads the account from the store to choose) both go through it.
- With nothing held, an app-password account's reason now ends "Open the
  Account Manager with Ctrl+Shift+A, choose the account and press Sign In for
  Calendars, Contacts and Tasks." With the sign-in held and refused, a reason
  of its own, `separate_sign_in_ran_out` in the log, names the same button
  rather than Sign In Again.
- The button: no account chosen says "Choose an account first."; an account
  not at Google, and a Gmail account whose mail signs in through the browser,
  say why it is not needed (choice 3); no key says so and names Help's page;
  otherwise "Signing in to Google for the account's calendars, contacts and
  tasks. Finish in the browser.", then "Signed in to Google for this account's
  calendars, contacts and tasks. They are brought when you close the Account
  Manager." or the reason it failed, with the attention cue. One Info line
  names the account by its id and the outcome. The token goes only to the
  credential store.
- The account signed in is carried out of the Account Manager even when
  nothing else changed (`AccountManagerAction::Updated::signed_in_for_calendars`),
  and `handle_account_mgr` brings it once beside the accounts added in the
  visit (`with_those_signed_in_for_calendars`, D-13). What adding an account
  starts now asks whether the separate sign-in is held.
- Sign In Again on an app-password Gmail account names the new button.

Reachability, read by grep after the green: the button's `on_click`
(`wx_account_manager.rs:478`) calls `sign_in_for_calendars_selected`, which
calls `sign_in_for_calendars`, which builds `AuthManager::new(...,
GOOGLE_CALENDARS_CONTACTS_AND_TASKS, ...)` and calls `sign_in.authorize()`
(`:810` to `:826`); `handle_account_mgr` calls
`with_those_signed_in_for_calendars(` (`wx_app.rs:22167`) and
`bring_what_a_new_account_holds(` (`:22236`); the three spawns call
`who_holds_the_calendars::a_google_token(` (`:30967`, `:31128`, `:31362`),
which reaches `oauth::a_google_token_from(` (`who_holds_the_calendars.rs:555`);
the free time lookup calls `a_google_token_for(whose)` (`managers.rs:1232`),
which reaches the same (`oauth.rs:1110`).

The Account Manager's labels after the green, `grep -n '\.with_label('
src/presentation/wx_account_manager.rs` (first eleven lines): `&Add
Account...`, `&Edit...`, `&Look People Up at Work...`, `&Other Addresses to
Send From...`, `&Delete`, `Set Acti&ve`, `Set as Defa&ult`, `&Sign In
Again`, `Sign In for Calendars, Contacts and &Tasks`, `&Close`, and the
header `Configured Email Accounts:` with no letter: one label per letter.

Counts on 2026-10-05, after the green: `application::who_holds_the_calendars::`
41 passed, `application::forget::` 27, `service::oauth::` 59,
`presentation::wx_account_manager::` 14, `presentation::wx_app::` 199,
`one_check_says_who_runs_the_mail` 2,
`a_key_is_documented_where_the_surface_that_binds_it_is` 3, `wired` 77,
`account_manager_immediate_actions` 1, `item_form_free_busy` 7,
`the_log_carries_what_a_report_needs` 19, `house_style` 74, `docs_links` 6,
`the_words_that_say_nothing` 10, `every_number_carries_its_command_and_its_date`
29, `the_planning_files_agree_with_themselves` 17.

The tracer, red and then green:

```
red  (90aa9809): the separate sign-in was not used: nothing asked: SignsInWithAnAppPassword
green (b315dfc6): test ...test_an_app_password_gmail_account_holding_the_separate_sign_in_asks_google_with_it ... ok
```

The stand-in heard `GET /calendars/primary/events?...` with
`authorization: Bearer the-separate-token`, and the empty store answered
`NothingAsked(SignsInWithAnAppPassword)`, whose sentence names the button.

Nothing here has met a real Google account. No NVDA case presses the button
(ledger 820).

## Google's pages, read on the day

Read 2026-10-05 with `curl` into the scratchpad and stripped to text. Every
step on the Help page follows them.

| Page | Last updated | What it says, as used |
|---|---|---|
| developers.google.com/workspace/guides/create-project | 2026-09-03 | Menu, IAM & Admin, Create a Project; Project Name; Create |
| developers.google.com/workspace/guides/enable-apis | 2026-09-03 | Menu, APIs & Services, Library, **Google Workspace**; the API; Enable |
| developers.google.com/workspace/guides/configure-oauth-consent | 2026-09-03 | Menu, Google Auth platform, Branding; Get Started; App name, User support email, Next; Audience; Contact Information; "I agree to the Google API Services: User Data Policy", Continue, Create; Audience, Test users, Add users, Save; Data Access, Add or Remove Scopes, Save |
| developers.google.com/workspace/guides/create-credentials | 2026-09-03 | Menu, Google Auth platform, Clients; Create Client; Application type, Desktop app; Name; Create |
| support.google.com/cloud/answer/15549257 | no date on the page | client secrets "are only visible and downloadable from the Google Cloud Console at the time of their creation"; afterwards "only display the last four characters"; Add Secret |
| developers.google.com/identity/protocols/oauth2 | 2026-05-26 | Testing "is issued a refresh token expiring in 7 days" |
| developers.google.com/identity/protocols/oauth2/scopes | 2026-09-14 | calendar: "See, edit, share, and permanently delete all the calendars you can access using Google Calendar"; contacts: "See, edit, download, and permanently delete your contacts"; tasks: "Create, edit, organize, and delete all your tasks" |

**What moved since premise 6:** the enable page now reads "APIs & Services >
Library > Google Workspace", one level more than the premise's "APIs and
Services, Library"; the page says so in its step 1. The contacts and tasks
permissions' words, which premise 6 had not read, are quoted above. The
create-project page was not in premise 6 and was read for the first step.

The page for Pratik is `docs/PROVIDER_SETUP.md`, section "Calendars,
contacts and tasks for a Gmail account", which Help opens as Setting up a
provider; every `docs/*.md` ships in the installer.

## Commits

| Commit | What | Hook |
|---|---|---|
| `90aa9809` | test: failing cases for the separate Google sign-in | red, 197 s (a formatting refusal first, 27 s) |
| `b315dfc6` | feat: an app-password account asks Google with a sign-in of its own; three records new, one re-anchored, twenty-one re-measured | affected, 329 s (a refusal first, 344 s: house_style read an unstaged page naming `oauth2`) |
| `182e3a38` | test: failing cases for the Account Manager's button | red, 223 s |
| `fe71b237` | feat: the button, the account carried out and brought, Sign In Again's sentence; three records new, twenty-two re-measured, one corrected | affected, 423 s (a refusal first, 424 s: the untracked summary counted against STATE's completed_plans) |
| docs | the Help section, the pages it corrects, the changelog, ledgers 817 to 820, REAL-01's line and row, the four marks, this summary | see the report |

## Guard records

Six new; the arrived-since count 662 to 668, and 1,465 records by the TOML
reader.

| Record | Break | Red |
|---|---|---|
| the separate google sign-in never asks for mail | mail added to its list | the address case |
| the separate google sign-in keeps an entry of its own | its name is `gmail` | the address case, the entry case, the browser account's case, forget's case |
| an account on an app password takes the separate google sign-in | the choice answers `gmail` | the tracer and the choice case |
| the separate sign-in button signs an app-password gmail account in | every Gmail account told the mail sign-in covers it | the button's rows |
| removing an account erases its separate google sign-in | the provider taken off the list | forget's case, the address case, the entry case |
| an account signed in for calendars is carried out of the account manager | carried only when something else changed | the reading and its companion |

"A gmail account on an app password is not asked of google" was re-anchored on
the arm that gained the held flag and now reddens the empty-store case as
well. Re-measured, one call per commit: twenty-one after task 1, all agreeing;
twenty-two after task 2, one short ("keeps an entry of its own" also reddens
forget's new case), corrected by hand and measured again alone.

## Ledger

- 817 opened and fixed (`deviation`): no Gmail account on an app password
  could reach its calendars, contacts or tasks, and no page said how to make
  the key (#22).
- 818 opened and fixed (`deviation`): `docs/installing.md` said `oauth.toml`
  held the keys the build was made with.
- 819 opened (`todo`): `run_oauth_flow` writes the account's whole address
  to the log.
- 820 opened (`unrun-verify`): the button and its sentences heard, and the
  Help section followed by ear through Google's console, in sitting 1.
- Header: 698 open, 122 fixed, 820 in all.

## Deviations from Plan

1. **[Premise] Line numbers** held as premises 1 to 5 gave them, read at
   `a458811a`; the guard register held 1,459 records at the start, not
   1,437, since 14-01 and 14-02 added 22.
2. **[Shape] The separate sign-in joined `providers()` in task 1**, not task
   2, because the tracer's address and entry cases need it. So forget.rs's
   case in task 2's red passed on arrival and was not named; it is held by two
   records instead.
3. **[Shape] Nothing held is the app-password reason**, which now names the
   button, rather than a new "not yet made" reason; held and refused is the
   new `TheSeparateSignInRanOut`. One reason fewer for the same two cases.
4. **[Rule 3] `managers.rs` changed**, not in the plan's file list: the free
   time lookup now reads the account from the store so `a_google_token_for`
   can choose the sign-in by how mail signs in.
5. **[Rule 2] The Account Manager's buttons sit in two rows**, in tab order,
   because nine buttons with this label ran past the 820-pixel window. The
   second row is filled first in the source only because `tests/wired.rs`
   reads the first row written for Set Active, Set as Default and Sign In
   Again; the comment says so. Not checked by eye: the scan reads names, not
   clipping.
6. **[Shape] The new sign-in's log lines are two**, signed in and not signed
   in, each naming the account by its id.
7. **[Process] GSD's `state.advance-plan` and `state.record-session` were
   run and reverted**: they inserted blank lines inside paragraphs of
   `STATE.md`, removed `percent` and changed the phase totals. The file was
   restored with `git checkout` and the plan number set by hand in its
   frontmatter and body, with `completed_plans` 252.
8. **[Process] The stream editor ran twice in read-only commands** in the
   first hour, against the brief; no file was written by it. Recorded as an
   observation.
9. **[Docs] A page naming `oauth2` is refused** by
   `test_no_page_a_person_reads_names_the_machinery`, so two of Google's
   page addresses are named by title rather than by path.

## TDD Gate Compliance

Task 1 (tracer): `90aa9809` red, accepted by `red-commit.sh`, then
`b315dfc6`. Task 2: `182e3a38` red, accepted, then `fe71b237`. Task 3 is
documents. Every red ran against stubs that compile and answer wrongly.

## Threat Flags

None beyond the register. T-14-11: the token is written only by
`AuthManager::authorize` to the credential store under its own entry; the
account and the database never carry it; `entries_for_account` names it,
held by forget's case and a record. T-14-12: the permissions case and its
record. T-14-13: the key search over `docs`, `oauth.toml.example` and `src`
printed nothing and exited 1; no script sets a key. T-14-15: the run-out
reason names the button. T-14-SC: nothing installed.

## Self-Check: PASSED

`90aa9809`, `b315dfc6`, `182e3a38` and `fe71b237` are on the branch;
`docs/PROVIDER_SETUP.md` holds the section.
