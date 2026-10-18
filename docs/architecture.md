# Wixen Mail architecture

This page is for somebody about to change Wixen Mail. It says what each part of
the program does and names the module that does it, so you can go straight to
the code. Every component under "Component Details" names at least one module,
and `tests/docs_links.rs` checks on every commit that each module it names is
on disk. A part the page cannot point at in the tree is not on the page.

Module paths are written the way Rust writes them inside the crate:
`data::message_cache::searching` is `src/data/message_cache/searching.rs`.

## Design principles

1. **Accessibility first.** Every control is built for a screen reader and the
   keyboard before anything else, and the program declares what things are to
   the platform rather than leaving a screen reader to guess.
2. **Layers.** The window, the decisions, the network and the storage live in
   separate layers, and a decision is written where a test can reach it without
   a window or a server.
3. **Responsive.** The window never waits on the network or a large read.
4. **Secrets in one place.** Passwords and tokens go to the Windows credential
   store and nowhere else.
5. **Tested at every level.** Unit tests sit beside the code they cover; tests
   under `tests/` cross layers and read the documents.

## High-level architecture

```
┌──────────────────────────────────────────────────────────────┐
│  Presentation          src/presentation/                      │
│  the main window, dialogs, the folder tree and message list,  │
│  and what is spoken, shown and sounded                        │
└──────────────────────────────────────────────────────────────┘
                              ↕
┌──────────────────────────────────────────────────────────────┐
│  Application           src/application/                       │
│  checking and sending mail, rules, search, drafts, contacts,  │
│  calendars, tasks and notes                                   │
└──────────────────────────────────────────────────────────────┘
                              ↕
┌──────────────────────────────────────────────────────────────┐
│  Service               src/service/                           │
│  IMAP, SMTP and POP3, signing in, reading MIME, PGP and       │
│  S/MIME, calendar and contact servers, safety checks          │
└──────────────────────────────────────────────────────────────┘
                              ↕
┌──────────────────────────────────────────────────────────────┐
│  Data                  src/data/                              │
│  the SQLite database, accounts and settings                   │
└──────────────────────────────────────────────────────────────┘

   Common, src/common/: errors, paths, logging and shared types,
   used by every layer.
```

## Component Details

### Presentation layer

#### The main window

`presentation::wx_app` builds the main window with wxDragon, the Rust bindings
for wxWidgets: the menus, the folder tree, the message list and the reading
pane. It owns the program's state and its async runtime, starts every piece of
background work, and applies what that work sends back.

#### The folder tree and the message list

What the folder tree shows is worked out in `presentation::folder_tree`
without a control, and then drawn. The message list runs in virtual mode, and
`presentation::message_rows` turns a message into the cells the list asks for.

#### Reading and writing a message

A message opens into the window in `presentation::wx_reader`, one message or
one conversation per tab. HTML mail is sanitized with `ammonia` and rendered
by `presentation::html_renderer`, which keeps the headings and link text a
screen reader navigates by. Writing, replying and forwarding happen in
`presentation::wx_compose`.

#### Settings

`presentation::wx_settings` is the Settings dialog. Every setting a person has
is offered there, and a test in `data::config` fails when a new one is not.

#### What the program says, shows and sounds

`presentation::accessibility` is the one way the program tells somebody that
something happened. `presentation::accessibility::announcements` queues and
ranks what is to be said so a syncing mailbox does not flood the screen reader,
and `presentation::accessibility::screen_reader` hands it to Windows through
UI Automation notifications. `presentation::accessibility::feedback` sends the
same events as sounds and visible signals, and
`presentation::accessibility::names` gives a name to controls that carry no
visible label.

### Application layer

#### Checking and sending mail

`application::mail_controller` holds the live sessions with each account's
servers. `application::mail_sync` brings an IMAP mailbox down into the
database, and `application::pop_sync` does the same for a POP account into its
local Inbox. Mail waiting to go sits in the Outbox, stored by
`data::message_cache::outbox`, and is sent from there.

#### Drafts, replies and attachments

`application::draft_message` keeps a draft as a message in the Drafts folder,
and `application::autosave` decides how often it is saved while somebody
writes. `application::reply` works out who a reply goes to,
`application::attaching` holds the files chosen to go with a message, and
`application::sending_later` holds a message back and lets it be taken back.

#### Rules

`application::filters` matches messages against the rules somebody wrote and
carries out their actions as mail arrives.

#### Search and saved searches

The search box is answered from a full text index over the mail, kept level
with the messages table by `data::message_cache::searching`. A search kept
under a name is a rule that selects rather than acts, so
`application::saved_searches` asks its questions in the rule engine's words.

#### Conversations

`application::threading` groups messages into conversations from their
`References` and `In-Reply-To` headers, and `application::conversations` says
what a conversation is called.

#### Contacts, calendars, tasks and notes

Contacts and groups are stored by `data::message_cache::contacts`;
`application::contact_groups` says what writing to a group comes to, and
`application::contacts_sync` reads Google and Microsoft address books and
sends new contacts back. Calendars on CalDAV servers are synced by
`application::caldav_sync`, and Google and Microsoft calendars by
`application::calendar`. Tasks are synced with Google Tasks and Microsoft To Do
by `application::tasks_sync`, and notes by `application::notes_sync`.

#### Bringing mail in and writing it out

`application::importing_messages` brings messages in from a file and writes
them out to one, and `application::exporting_mail` reads a folder's mail out of
the database for the writer.

### Service layer

#### Mail protocols

`service::protocols::imap`, `service::protocols::smtp` and
`service::protocols::pop3` are the clients. They keep the transport thin, and
what they parse is tested without a live server.

#### Signing in

`service::oauth` signs in to Google and Microsoft with OAuth and PKCE through a
local redirect. `service::credentials` keeps account passwords, and
`service::secret_store` is the one way in and out of the Windows credential
store.

#### Reading a fetched message

`service::mime` turns a fetched message into the parts the application shows:
the text, the HTML and the attachments.

#### Signed and encrypted mail

`service::pgp` reads and sends OpenPGP mail, `service::signed_mail` reads
S/MIME, and `service::security` spots PGP and S/MIME markers in a message and
scores the signs of phishing.

#### Mail from strangers

`service::safebrowsing` checks links with Google Safe Browsing without sending
it the links, `service::safety` holds what is already known about whether a
message is safe, and `service::attachment_name` turns a stranger's file name
into one that is safe to write and marks the file as having come from the
internet.

#### Calendar and contact servers

`service::caldav` and `service::carddav` talk to CalDAV and CardDAV servers,
`service::google_api` and `service::microsoft_graph` to Google's and
Microsoft's, and `service::ical_subscription` reads calendar feeds.

### Data layer

#### Database (SQLite)

One database, `cache\message_cache.db`, opened through `data::message_cache`.
Schema changes are additive: tables with `CREATE TABLE IF NOT EXISTS`, columns
with `ensure_column_exists`, and no dropping or renaming of anything that has
shipped. The one exception taken so far was a table of OAuth tokens that
nothing read, which was dropped because leaving secrets nobody rotates in a
file people copy is worse than the rule it broke.

Each kind of thing kept there has its own module under `data::message_cache`:
messages, folders, accounts, contacts, calendars, tasks, notes, rules, tags,
saved searches, the Outbox and the full text index among them.

#### Accounts

An account is `data::account`'s `Account`: its name, address, servers, whether
it signs in with OAuth, and how often it is checked. It is stored by
`data::message_cache::accounts`, and its password is not part of the row.

#### Where files live

One root, owned by `common::paths`, so there is a single answer to "where is my
mail" for backups, for support, and for the uninstaller. `WIXEN_MAIL_DATA`
moves the root, which is what a memory stick install needs.

```
%LOCALAPPDATA%\wixen-mail\
├── config\                 settings, one file per account, oauth.toml
├── cache\                  message_cache.db and its SQLite sidecars
└── logs\                   the running log and crash.log
```

Nothing roams. An earlier layout put the encryption key in the roaming profile
while the database it unlocked stayed local, so the key crossed the network at
every sign-in and the mail it protected did not.

**No secrets are in the database.** Account passwords go to the Windows
credential store through `service::credentials`, OAuth tokens through
`service::oauth`, and CalDAV sign-ins through `service::caldav`. Each service
name has exactly one owner, because the code that erases them on uninstall has
to name the same entries as the code that wrote them.

**The cached mail is not encrypted**, and the documentation says so rather than
implying otherwise. Windows keeps other users out of the folder; anything
running as that user can read it, and so can anyone who takes the drive out of
an unencrypted machine. See [Installing and uninstalling](installing.md).

#### Settings storage

`data::config` reads and writes the settings files under `config\`.

### Common

#### Errors, logging and shared types

`common::error` defines the one error type every layer returns, and
`common::logging` writes the log. `common::types` holds the types more than one
layer uses, such as an email address and a message body.

## Technology stack

- **Language**: Rust, stable channel.
- **Window toolkit**: wxDragon, the Rust bindings for wxWidgets, on Windows.
- **Database**: SQLite through `rusqlite`, built in.
- **Async runtime**: `tokio`.

The rest of the dependencies are in `Cargo.toml`, which is the only list kept
up to date. Read from it on 2026-10-18, the ones that shape the program most
are `async-imap` for IMAP, `lettre` for SMTP, `mail-parser` for reading
messages, `ammonia` for sanitizing HTML, `keyring` for the credential store,
`pgp` for OpenPGP, `tracing` for logging and `windows` for the Windows calls
the toolkit does not make. Why OpenPGP is done with that crate is in
[the PGP implementation choice](development/pgp-implementation-choice.md).

## Data models

The types are in the code, and this page does not copy them, because a copy
goes out of date without anything noticing. An account is
`data::account::Account`, a stored message is
`data::message_cache::CachedMessage`, and a stored folder is
`data::message_cache::CachedFolder`.

## Threading model

### The window's thread

wxWidgets draws on one thread, and everything that touches a control runs
there. It never waits on the network or on a large read.

### Background work

`presentation::wx_app` owns one `tokio` runtime, and each piece of background
work is started there by a function named for it:

- `spawn_mail_sync` checks the accounts it is handed for new mail.
- `spawn_mail_watch` keeps an IMAP IDLE connection open per enabled IMAP
  account, so new mail is noticed without a check.
- `flush_outbox` sends what is waiting in the Outbox.
- `spawn_body_fetch` fetches the text of a message being opened.
- `spawn_calendar_sync`, `spawn_contacts_sync`, `spawn_tasks_sync` and
  `spawn_notes_sync` sync the other modules.

Work that reads or writes the database runs on the runtime's blocking threads,
because a SQLite connection cannot be shared between threads. There is no
separate search index worker: the index is updated as messages are written.

### Communication

Background work sends its results back to the window as `UIUpdate` values over
an `async_channel` channel, and the window applies them on its own thread.

## Error handling

### Strategy

- Errors flow through `common::Error` and `common::Result`. A foreign error is
  mapped where it enters, at the boundary of the library that raised it.
- No `unwrap` or `expect` outside tests and `build.rs`.
- An error a person sees says what happened, why, and what to do next, and is
  announced to the screen reader as well as shown.

### Logging

- Structured logging with `tracing`, set up by `common::logging`.
- The log file rotates daily.
- Nothing logs a token, a password, or a message body.

## Security considerations

### Credential storage

Everything goes to the Windows credential store, which is DPAPI-backed and
per-user. No credential is written to the database or to any file this
application owns, so a database copied for a backup or handed over for support
carries no credentials with it. The mail cached in that database is a separate
question, and data protection below gives the answer.

There is no master key any more. It encrypted exactly one thing, a password
that can be typed again, and the key itself was one more thing to lose.
Uninstalling clears the credential store entries by running the application
once with `--erase-all-data`, because an uninstaller cannot reach them. It runs
as the account the uninstaller runs as: the person for a per-user install, and
for an all-users install the account that approved the elevation prompt, so
other accounts keep their entries and their data folder, and the uninstaller
tells the person so (`installer/Wixen-Mail-Setup.iss`).

### Data protection

- The cached mail is not encrypted at rest. Encrypting it means encrypting the
  whole database, which is a decision with a build cost rather than something
  to imply in a feature list.
- Uninstalling removes the data folder and the credential store entries of the
  account it runs as, and writes what it could not remove to the temporary
  folder rather than claiming success. An all-users uninstall names that
  account and tells the person that other accounts keep theirs.
- Nothing logs a token, a password, or a message body.

## Testing strategy

### Unit tests

- Unit tests sit in a `tests` module beside the code they cover.
- High code coverage (target: 80%+)
  Met by the library on 2026-09-14: 83.34% line coverage by
  `cargo llvm-cov --lib --summary-only` at `55464a5e`, the row on
  `docs/development/measurements.md`, against 60.4% on 2026-07-26 by the same
  command. That is the library as one number. Everything outside
  `src/presentation/` read 95.53% and the wxWidgets window files 26.88%, on
  the same page from the same run, because those files build windows and a
  `--lib` run opens none; the targets under `tests/` that open one are outside
  that command, so the target is kept as it is and read against the library,
  with the windows' figure written beside it rather than folded in.

### Integration tests

- Tests under `tests/` cross layers against a real database in a temporary
  folder, and some read the documents to keep them true.
- Network code is tested against its parsing and error mapping, and some of it
  against small servers on the loopback address. Nothing is tested against a
  real provider's servers.

### Accessibility tests

- Each window is scanned over UI Automation with Axe.Windows and over MSAA with
  `scripts/msaa-names.ps1`, because a name missing on either channel is a name
  somebody does not hear.
- The cases under `nvda-tests` drive the program with NVDA running and check
  what it says.
- None of this replaces listening with a real screen reader.

### Measurements

Timings and counts, each with its date and the command that took it, are on
[the measurements page](development/measurements.md).

## Build and distribution

- `cargo build --release` builds the program.
- `scripts/build-installer.sh` builds the Windows installer with Inno Setup,
  from `installer/Wixen-Mail-Setup.iss`.
- `service::update_check` asks GitHub whether a newer version exists.

## Other platforms

wxWidgets gives native, accessible controls on Windows, macOS and Linux, but
two things this program relies on exist only on Windows: `wxAccessible`, which
is how a name set in the code reaches the accessibility tree, and
`UiaRaiseNotificationEvent`, which is how announcements are spoken and
brailled. Both compile and do nothing elsewhere, so a port needs its own bridge
for each. Code for one platform sits behind `#[cfg(target_os = "windows")]`
with a fallback that keeps the crate building.

## References

- [Rust Language](https://www.rust-lang.org/)
- [Windows UI Automation](https://docs.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32)
- [IMAP RFC 3501](https://tools.ietf.org/html/rfc3501)
- [SMTP RFC 5321](https://tools.ietf.org/html/rfc5321)
- [Email Message Format RFC 5322](https://tools.ietf.org/html/rfc5322)
