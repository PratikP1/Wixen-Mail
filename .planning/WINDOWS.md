---
schema_version: 1
open_count: 9
waived_count: 0
fixed_count: 0
total_count: 9
last_updated: 2026-08-31T05:13:51.507Z
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
  }
]
````
