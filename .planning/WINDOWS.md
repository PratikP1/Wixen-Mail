---
schema_version: 1
open_count: 16
waived_count: 0
fixed_count: 0
total_count: 16
last_updated: 2026-09-01T04:57:57.717Z
---

# Broken Windows Ledger

> Cross-phase defect register. With `workflow.windows_enforce` enabled, `/gsd-ship` blocks while `open_count > 0`.
> Waive with `gsd-tools windows waive <id> "<reason>"` (reason required).
> Mark fixed with `gsd-tools windows fixed <id>`.

| id | phase | kind | file | line | description | status | reason | recorded_at | resolved_at |
|----|-------|------|------|------|-------------|--------|--------|-------------|-------------|
| 1 | 01 | unrun-verify | src/presentation/folder_tree.rs |  | The Favourites group is not confirmed with a screen reader; FOLDER-03's last criterion is satisfied structurally only | open |  | 2026-08-30T19:15:54.071Z |  |
| 2 | 01 | deviation | src/presentation/wx_app.rs |  | The running tree builds one account at a time, so D-29's per-account Favourites branches are tested but not visible in the program yet | open |  | 2026-08-30T19:15:54.459Z |  |
| 3 | 01 | stub | src/presentation/message_rows.rs |  | conversation_cell_text is written and tested per column and has no non-test caller; 01-12 draws the collapsed conversation list | open |  | 2026-08-31T01:12:14.164Z |  |
| 4 | 01 | stub | src/presentation/message_columns.rs |  | Sort::conversation_order_by_clause is written and tested and has no non-test caller; 01-12 passes the user's chosen sort | open |  | 2026-08-31T01:12:21.927Z |  |
| 5 | 01 | deviation | src/application/conversations.rs |  | Hungarian's one-letter I: forward marker is read as a reply marker, because mail_parser's trim_trailing_fwd ignores a parenthesised word of one character | open |  | 2026-08-31T01:12:22.334Z |  |
| 6 | 01 | unrun-verify | src/presentation/wx_app.rs |  | Rethreading on arrival repaints one row and does not touch the selection; no screen reader has confirmed that a repainted row is silent to somebody not on it | open |  | 2026-08-31T05:13:47.847Z |  |
| 7 | 01 | stub | src/application/thread_identity.rs |  | A conversation root arriving after a message that names it is not merged: the link lives only in the other message's stored refs_header, which no index can search. Needs an identifier-to-conversation table | open |  | 2026-08-31T05:13:49.247Z |  |
| 8 | 01 | deviation | src/data/message_cache/messages.rs |  | messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column | open |  | 2026-08-31T05:13:50.314Z |  |
| 9 | 01 | deviation | .planning/phases/01-folders-and-conversations/01-13-PLAN.md |  | Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one | open |  | 2026-08-31T05:13:51.507Z |  |
| 10 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The coverage sentence before a saved search is announced as a low-priority status topic and has not been heard under a screen reader; it also coalesces with the Running this saved search line, which is by design and unverified by ear | open |  | 2026-09-01T02:14:56.334Z |  |
| 11 | 02 | unrun-verify | src/application/mail_sync.rs |  | The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names | open |  | 2026-09-01T03:56:05.356Z |  |
| 12 | 02 | unrun-verify | src/presentation/wx_app.rs |  | The offer button and its experimental sentence have not been heard under a screen reader: whether the button is announced with its full label after a saved search, and whether the message text topic is heard rather than coalesced away, is unverified by ear | open |  | 2026-09-01T03:56:05.812Z |  |
| 13 | 02 | stub | src/presentation/wx_app.rs |  | The offer only appears while a saved search that reads message text is run; a person who never uses saved searches is never offered the fetch, which is where D-2-08 puts it and is a narrower reach than a menu command would have | open |  | 2026-09-01T03:56:12.023Z |  |
| 14 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The filter rule dialog's three Choice controls have an accessible object attached, which a test can see, but the name each one carries cannot be read back from wxdragon. Whether NVDA says 'Match field', 'Match type' and 'Action' rather than an unnamed combo box is unverified by ear | open |  | 2026-09-01T04:57:49.641Z |  |
| 15 | 02 | unrun-verify | src/presentation/wx_managers.rs |  | The Pattern box is disabled for the four ways of matching that read no pattern. Whether a disabled edit box is skipped cleanly in the tab order, and whether changing the Match Type while focus is nearby moves focus or is announced, is unverified by ear | open |  | 2026-09-01T04:57:57.259Z |  |
| 16 | 02 | unrun-verify | src/application/filters.rs |  | The eleven field names and eleven ways of matching are now read aloud as words. Whether 'Read is yes', 'Flagged is yes' and 'matches a text pattern' are understood when heard rather than seen is unverified by ear | open |  | 2026-09-01T04:57:57.717Z |  |

````json
[
  {
    "id": 1,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src/presentation/folder_tree.rs",
    "line": null,
    "description": "The Favourites group is not confirmed with a screen reader; FOLDER-03's last criterion is satisfied structurally only",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T19:15:54.071Z",
    "resolved_at": null
  },
  {
    "id": 2,
    "kind": "deviation",
    "phase": "01",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The running tree builds one account at a time, so D-29's per-account Favourites branches are tested but not visible in the program yet",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-30T19:15:54.459Z",
    "resolved_at": null
  },
  {
    "id": 3,
    "kind": "stub",
    "phase": "01",
    "file": "src/presentation/message_rows.rs",
    "line": null,
    "description": "conversation_cell_text is written and tested per column and has no non-test caller; 01-12 draws the collapsed conversation list",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:14.164Z",
    "resolved_at": null
  },
  {
    "id": 4,
    "kind": "stub",
    "phase": "01",
    "file": "src/presentation/message_columns.rs",
    "line": null,
    "description": "Sort::conversation_order_by_clause is written and tested and has no non-test caller; 01-12 passes the user's chosen sort",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:21.927Z",
    "resolved_at": null
  },
  {
    "id": 5,
    "kind": "deviation",
    "phase": "01",
    "file": "src/application/conversations.rs",
    "line": null,
    "description": "Hungarian's one-letter I: forward marker is read as a reply marker, because mail_parser's trim_trailing_fwd ignores a parenthesised word of one character",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T01:12:22.334Z",
    "resolved_at": null
  },
  {
    "id": 6,
    "kind": "unrun-verify",
    "phase": "01",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "Rethreading on arrival repaints one row and does not touch the selection; no screen reader has confirmed that a repainted row is silent to somebody not on it",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:47.847Z",
    "resolved_at": null
  },
  {
    "id": 7,
    "kind": "stub",
    "phase": "01",
    "file": "src/application/thread_identity.rs",
    "line": null,
    "description": "A conversation root arriving after a message that names it is not merged: the link lives only in the other message's stored refs_header, which no index can search. Needs an identifier-to-conversation table",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:49.247Z",
    "resolved_at": null
  },
  {
    "id": 8,
    "kind": "deviation",
    "phase": "01",
    "file": "src/data/message_cache/messages.rs",
    "line": null,
    "description": "messages.message_id holds two formats (bare from mail_parser, angle-bracketed from draft_message::message_id_for) while thread_id holds one; the lookup asks for both rather than rewriting a shipped column",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:50.314Z",
    "resolved_at": null
  },
  {
    "id": 9,
    "kind": "deviation",
    "phase": "01",
    "file": ".planning/phases/01-folders-and-conversations/01-13-PLAN.md",
    "line": null,
    "description": "Task 1's order-independence criterion is unsatisfiable with the signature the same task mandates: the lookup cannot see messages that name the arriving one",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-08-31T05:13:51.507Z",
    "resolved_at": null
  },
  {
    "id": 10,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The coverage sentence before a saved search is announced as a low-priority status topic and has not been heard under a screen reader; it also coalesces with the Running this saved search line, which is by design and unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T02:14:56.334Z",
    "resolved_at": null
  },
  {
    "id": 11,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/mail_sync.rs",
    "line": null,
    "description": "The bulk body fetch has never run against a real IMAP server: whether a provider permits, throttles or drops a run of hundreds of BODY.PEEK fetches is untestable here and is the one risk the experimental sentence names",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:05.356Z",
    "resolved_at": null
  },
  {
    "id": 12,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The offer button and its experimental sentence have not been heard under a screen reader: whether the button is announced with its full label after a saved search, and whether the message text topic is heard rather than coalesced away, is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:05.812Z",
    "resolved_at": null
  },
  {
    "id": 13,
    "kind": "stub",
    "phase": "02",
    "file": "src/presentation/wx_app.rs",
    "line": null,
    "description": "The offer only appears while a saved search that reads message text is run; a person who never uses saved searches is never offered the fetch, which is where D-2-08 puts it and is a narrower reach than a menu command would have",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T03:56:12.023Z",
    "resolved_at": null
  },
  {
    "id": 14,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The filter rule dialog's three Choice controls have an accessible object attached, which a test can see, but the name each one carries cannot be read back from wxdragon. Whether NVDA says 'Match field', 'Match type' and 'Action' rather than an unnamed combo box is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:49.641Z",
    "resolved_at": null
  },
  {
    "id": 15,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/presentation/wx_managers.rs",
    "line": null,
    "description": "The Pattern box is disabled for the four ways of matching that read no pattern. Whether a disabled edit box is skipped cleanly in the tab order, and whether changing the Match Type while focus is nearby moves focus or is announced, is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:57.259Z",
    "resolved_at": null
  },
  {
    "id": 16,
    "kind": "unrun-verify",
    "phase": "02",
    "file": "src/application/filters.rs",
    "line": null,
    "description": "The eleven field names and eleven ways of matching are now read aloud as words. Whether 'Read is yes', 'Flagged is yes' and 'matches a text pattern' are understood when heard rather than seen is unverified by ear",
    "status": "open",
    "reason": "",
    "recorded_at": "2026-09-01T04:57:57.717Z",
    "resolved_at": null
  }
]
````
