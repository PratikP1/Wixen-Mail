# Wixen Mail search handler

This crate builds two things:

- `wixen_mail_search.dll`, a Windows Search protocol handler and filter. It teaches
  the Windows indexer a URL scheme of its own, `wixen-mail://`, and hands back the
  text and properties for one message at a time.
- `wixen-mail-search-setup.exe`, a command line tool that registers the handler,
  tells the indexer to look, reports what is set up, and undoes all of it.

The point of the pair is that somebody can try this end to end and watch what
happens, instead of reading test results and guessing.

## Read this before you turn it on

**Everything the indexer takes lands in the Windows Search index, and that index
is not encrypted.** It is a database under `ProgramData` that any software on this
computer can read, and it keeps its own copy of the subjects and the message text.
Turning this on means somebody's mail is readable outside Wixen Mail, by anything
on the machine, until the index is rebuilt.

Wixen Mail's own message cache is not encrypted either, so this does not remove a
protection that was there. It does widen who can read it. That is a decision for
the person whose mail it is, which is why nothing turns it on by itself.

Turning it off later stops new mail going in. It does not take out what is already
there. Only rebuilding the index does that: Indexing Options, Advanced, Rebuild,
and it takes hours.

## What has been watched working, and what has not

Watched working on a real machine, from an ordinary prompt:

| Step | Result |
|---|---|
| Read the `SystemIndex` catalog and list its 49 scope rules | Works |
| Add the crawl scope rule and the search root | Works, and did not need administrator rights |
| Ask whether a message URL is now in scope | Answers yes |
| Watch the catalog move into a full crawl right afterwards | Happens within seconds |
| Remove the rule and the root again | Works, catalog back to its 49 original rules |
| Load `wixen_mail_search.dll` and call `DllRegisterServer` | The library loads and the function runs |

Not watched working, because it needs an administrator prompt and nobody has run
one yet:

- Writing the registry entries. `DllRegisterServer` returns `0x80004005` from an
  ordinary prompt, which is what a refused `HKEY_LOCAL_MACHINE` write looks like.
- **The indexer asking this handler for anything at all.** Nothing has ever seen
  that happen. The scope rule has only ever been added on a machine where the
  classes were not registered, so the indexer had nothing to load.
- Any mail being found. See [Did it find anything](#did-it-find-anything).

Treat the handler itself as unproven until somebody completes the walkthrough
below and sees a row come back.

## Which rights this needs

Two halves, and they turned out to need different things.

| Half | What it does | Rights needed |
|---|---|---|
| Registering the classes | Writes under `HKEY_LOCAL_MACHINE\SOFTWARE\Classes` and `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows Search\ProtocolHandlers` | Administrator |
| The crawl scope rule | `AddRoot`, `AddDefaultScopeRule` and `SaveAll` on the `SystemIndex` catalog | None, on the machine this was tried on |

The second row was a surprise. The expectation was that changing what a system
service crawls would need administrator rights, and on Windows 11 Pro 26200 it did
not: a standard prompt added the rule, the catalog started crawling, and a standard
prompt removed it again.

Do not build a settings toggle on that finding yet. It was measured once, on one
machine, with no Group Policy in the way. Group Policy can lock crawl scope
changes down, and a machine joined to a domain may well behave differently. What
this does mean is that a settings toggle for the scope half is worth investigating
rather than ruled out, and that the registration half is install-time work either
way.

## Build it

```
cd search-handler
cargo build --release
```

That produces `target/release/wixen_mail_search.dll` and
`target/release/wixen-mail-search-setup.exe`. Keep them in the same folder. The
tool looks for the library beside itself unless you pass `--library`.

Checks, which is what CI would run if CI covered this crate:

```
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

`search-handler` is a standalone crate, not a member of the Wixen Mail workspace,
and no workflow under `.github/workflows` mentions it. Nothing builds or tests it
unless somebody runs the three commands above by hand.

## Try it end to end

### 1. See what is set up now

From any prompt:

```
wixen-mail-search-setup.exe status
```

This changes nothing. On a machine where nothing is set up you get:

```
Account: S-1-5-21-2351894039-911648811-593791538-1001
URL prefix: wixen-mail://{S-1-5-21-2351894039-911648811-593791538-1001}/localhost/
Running with administrator rights: no

Registry
  Library Windows would load: nothing registered
  Handler registered for the wixen-mail scheme: no

Windows Search
  Crawl scope rule: not there
  Search root: not registered
  The indexer says a message URL here is in scope: no
  Rules in this catalog belonging to something else: 49 (not listed, because they name real folders on this computer)
  The indexer has not been told to look here, so it will never ask about any mail.

The indexer
  Doing: working through changes it was told about
  The indexer is working on something else right now.
  Items in the whole index, everything on this computer: 588666
```

Check the `Account:` line. That identifier is whose mail the rule will name. If you
elevated by typing a different account's password, this is that other account and
the rule would be wrong. Pass `--user` with the right identifier if so. To find it:

```powershell
[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
```

### 2. Make sure there is a message cache to read

The handler reads
`%LOCALAPPDATA%\wixen-mail\cache\message_cache.db`, read only, while Wixen Mail is
running. If that file is not there, the indexer will find nothing and nothing will
say why. Run Wixen Mail and let it sync at least one folder first.

Moving the data folder with Wixen Mail's own setting is not handled. The handler
runs outside your session and cannot read that setting, so a moved cache is simply
not found.

### 3. Install

Open an administrator prompt. Start menu, type `cmd`, then Ctrl+Shift+Enter.

```
wixen-mail-search-setup.exe install
```

Expect:

```
Registering C:\Program Files\Wixen Mail\wixen_mail_search.dll
  The classes and the wixen-mail scheme are registered.
Adding the crawl scope rule
  Done.
```

If the first step fails with `0x80004005`, the prompt is not an administrator one.
The tool stops there and does not add the scope rule, so a failed install leaves
nothing behind.

### 4. Watch

```
wixen-mail-search-setup.exe status
```

You want all three of these:

```
  Crawl scope rule: present, puts this location in the index, set by the application
  Search root: registered
  The indexer says a message URL here is in scope: yes
```

The indexer decides for itself when to start. It can be minutes, and longer on
battery or while the machine is busy. `Doing: doing a full crawl` in the last block
means it has started on something.

To push it:

```
wixen-mail-search-setup.exe reindex
```

That affects this handler's URLs only. It does not rebuild the whole index.

## The installer checkbox

The Wixen Mail installer offers this as a task on the Select Tasks page, unticked,
worded so the checkbox itself says the index is not encrypted. Ticking it and
pressing Next asks once more before setup goes ahead, and answering No unticks the
box and installs everything else.

The checkbox only appears when you install for everybody, because the registry
entries go under `HKEY_LOCAL_MACHINE` and an install for one user cannot write
there. Both files land in the program folder either way, so you can turn this on
by hand later from an administrator prompt.

Uninstalling runs `remove-scope` as the person uninstalling and then
`unregister-classes` elevated, whether or not the box was ever ticked. Both do
nothing and report success when there is nothing to undo. If either fails,
uninstall says so and tells you what is left.

This path has been compiled and read. It has not been run: installing needs an
elevation prompt nobody has answered yet.

## Did it find anything

The setup tool cannot answer this. There is no supported way to ask the catalog how
many items came from one place, and the item count it prints covers everything on
the computer.

Ask the index directly instead. In PowerShell:

```powershell
$query = "SELECT TOP 20 System.ItemUrl FROM SystemIndex WHERE System.ItemUrl LIKE 'wixen-mail:%'"
$connection = New-Object System.Data.OleDb.OleDbConnection "Provider=Search.CollatorDSO;Extended Properties='Application=Windows'"
$connection.Open()
$command = New-Object System.Data.OleDb.OleDbCommand $query, $connection
$reader = $command.ExecuteReader()
while ($reader.Read()) { $reader.GetString(0) }
$connection.Close()
```

**Prove the query works before you believe a zero.** Run it once with `'file:%'` in
place of `'wixen-mail:%'`. That should print file paths. If it prints nothing, the
query is broken and tells you nothing about the handler. Both were checked on
PowerShell 7.6.5: `file:%` returned rows, `wixen-mail:%` returned none.

This query prints real URLs, which carry account names and folder names. Do not
paste the output into a bug report without reading it first.

To search the message text rather than list URLs:

```powershell
$query = "SELECT TOP 20 System.ItemUrl, System.Subject FROM SystemIndex WHERE System.ItemUrl LIKE 'wixen-mail:%' AND CONTAINS('invoice')"
```

The Indexing Options window is the other place to look. Run `control srchadmin.dll`.
It shows the item count and, under Modify, the locations being indexed.

## Turn it off

From an administrator prompt:

```
wixen-mail-search-setup.exe uninstall
```

That takes the crawl scope rule and the search root out first, then removes the
registry entries. It keeps going past a failure in the first step, because stopping
half way leaves a machine that is registered and out of scope, which nothing
reports and nothing fixes.

To stop new mail going in without unregistering anything:

```
wixen-mail-search-setup.exe remove-scope
```

Neither one takes out what the index already holds. Only Indexing Options,
Advanced, Rebuild does that.

## Commands

| Command | What it does | Rights |
|---|---|---|
| `status` | Says what is set up and what the indexer is doing | None |
| `install` | Registers the classes, then adds the crawl scope rule | Administrator |
| `uninstall` | Removes the rule and the root, then unregisters the classes | Administrator |
| `add-scope` | Only the half that tells the indexer to look | None, so far |
| `remove-scope` | Only the half that tells it to stop | None, so far |
| `register-classes` | Only the half that writes the registry entries | Administrator |
| `unregister-classes` | Only the half that takes them out | Administrator |
| `reindex` | Asks the indexer to visit this handler's URLs again | None, so far |
| `help` | Prints the same list, with the warning above it | None |

`install` is `register-classes` then `add-scope`. `uninstall` is `remove-scope`
then `unregister-classes`. The halves exist separately because the installer has
to run them as different accounts: the registry entries need the elevated
installer, and the crawl scope rule has to name the person who started setup
rather than whichever administrator they typed at the elevation prompt.

Options: `--user <SID>` names whose mail this is about, and `--library <path>`
says where `wixen_mail_search.dll` is.

Exit codes: 0 it worked, 1 it failed, 2 the command line was wrong.

## When it goes wrong

**`0x80004005` while registering.** The prompt is not an administrator one, or
something is protecting `HKEY_LOCAL_MACHINE`. The library reports every registry
failure the same way, so this code does not prove which.

**`0x80070005` while changing the crawl scope.** Access denied. Try an
administrator prompt. If that also fails, Group Policy is locking crawl scope
changes.

**`0x80070057` from `reindex`.** No search root is registered for that URL, so
there is nothing to reindex. Run `install` or `add-scope` first.

**"could not reach the Windows Search service".** The service is not running:

```powershell
Get-Service WSearch
Start-Service WSearch
```

**Everything reports as set up and no mail is ever found.** This is the state the
whole handler is written to avoid and it is still the most likely outcome, because
nobody has got past it yet. Things to check, in order:

1. `%LOCALAPPDATA%\wixen-mail\cache\message_cache.db` exists and has messages in it.
2. The `Account:` line in `status` matches the person whose mail that is.
3. The indexer's host process can read that file. It runs outside your session, as
   the system account, and this has never been confirmed either way. If it cannot,
   nothing anywhere reports it.
4. The indexer accepts a hyphen in a scheme name. `wixen-mail` has one. RFC 3986
   allows it, and Microsoft's own advice is `companyName.scheme`, which also has a
   character outside the letters, but this has not been tested against the real
   indexer. If it turns out to be the problem, `SCHEME` in `src/url.rs` is the one
   place to change.

## Limits

- **Mail only.** The URL shape is mail shaped, so contacts, calendar, tasks, notes
  and reminders are not in it.
- **One person at a time.** A crawl scope rule names one account's mail. Two people
  sharing a computer need two rules, added one at a time with `--user`. Nothing
  walks the machine's accounts and sets them all up.
- **HTML-only messages give no body text.** A message that arrived as HTML with no
  plain alternative contributes its subject, sender and date. Handing raw markup to
  the indexer would fill the index with tag names.
- **No per-item security descriptor.** The handler does not give the index one, so
  the index applies its own rules about who may see a result.
- **No panic guard at the COM boundary.** The library has no `catch_unwind` around
  the methods Windows calls. Everything inside is written not to panic, and nothing
  proves it.
- **A moved data folder is not found.** See step 2 above.
- **Nothing here logs.** No subject, address or message text reaches a log file, an
  error message or a panic message. That is deliberate: this code runs inside a
  Microsoft process whose logs nobody here owns. It also means a failure inside the
  indexer leaves nothing to read.

## How the code is laid out

Everything that makes a decision is a plain Rust module with tests. The parts that
talk to Windows are as thin as they can be, because none of them can be tested from
here.

| Module | What it decides |
|---|---|
| `src/url.rs` | Turns a URL into a place in the store and back |
| `src/record.rs` | One message reduced to what the indexer is told about it |
| `src/chunks.rs` | The sequence a filter walks through |
| `src/store.rs` | Reads the message cache, read only |
| `src/registration.rs` | Which registry entries make Windows Search aware of this |
| `src/scope.rs` | Which crawl scope rule tells the indexer to look |
| `src/setup.rs` | Reads the setup tool's command line |
| `src/com/` | The plumbing. No decisions worth testing |
