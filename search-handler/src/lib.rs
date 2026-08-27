//! The library Windows Search loads to read Wixen Mail's own message store.
//!
//! # What this is
//!
//! Windows Search will not read an application's private database on its own.
//! It knows how to walk a file system and how to run a filter over about two
//! hundred file types, and a SQLite file is neither. The only supported way in
//! is to write a *protocol handler*, which teaches the indexer a URL scheme of
//! our own, and a *filter*, which hands back the text and the properties for
//! one of those URLs. Both are COM objects, and the indexer loads them into its
//! own process rather than calling out to ours, so they have to live in a DLL.
//! That is what this crate builds. The main application is a single executable
//! and can never serve this job itself.
//!
//! # Read this before turning it on
//!
//! **Everything handed to the indexer ends up in the Windows Search index, and
//! that index is not encrypted.** It is a database under `ProgramData` that any
//! software running on this machine can query, and it keeps its own copy of the
//! text. Turning this on means the subjects and the message text of somebody's
//! mail are readable outside Wixen Mail, by anything on the computer, until the
//! index is rebuilt. The mail Wixen Mail already caches is not encrypted either,
//! so this does not undo a protection that existed; it does widen who can read
//! it, and that is a decision for the person whose mail it is rather than a
//! default.
//!
//! Nothing here logs. Subjects, addresses and message text never reach a log
//! file, an error message or a panic message, because the project forbids it and
//! because this code runs inside a Microsoft process whose logs we do not own.
//!
//! # What is verified and what is not
//!
//! Verified against Microsoft's documentation and against this machine:
//!
//! - A protocol handler needs [`ISearchProtocol`], [`IUrlAccessor`] and
//!   [`IFilter`]. The numbered extensions of the first two are optional.
//! - The handler is registered by ProgID under
//!   `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows Search\ProtocolHandlers`.
//!   That key exists on this machine and holds `Mapi`, `WinRT`, `IEHistory` and
//!   `IERSS`. Note the path: Microsoft's own page also lists a `HKEY_CURRENT_USER`
//!   copy, and there is no `CurrentVersion` level in either.
//! - Writing under `HKEY_LOCAL_MACHINE` needs administrator rights, so switching
//!   this on is an install-time step and not something a settings checkbox can
//!   do on its own. See [`registration`] for what that means in practice.
//! - The class itself is registered under `Software\Classes\CLSID` with an
//!   `InprocServer32` value, which is the in-process COM server registration.
//!
//! Also watched happen on this machine, using [`the setup tool`](setup):
//!
//! - The crawl scope rule and the search root can be added to the `SystemIndex`
//!   catalog, read back, and removed again, leaving the catalog with exactly the
//!   rules it had before.
//! - Once added, `IncludedInCrawlScope` answers yes for a message URL under the
//!   prefix, and the catalog moves into a full crawl straight away.
//! - **Changing the crawl scope did not need administrator rights**, which was
//!   not what was expected. Registering the classes still does. See
//!   [`com::crawl_scope`] and the README for exactly what was run.
//! - The library loads and its `DllRegisterServer` runs when the setup tool
//!   calls it the way `regsvr32` would.
//!
//! Not verified, and not verifiable without installing this and running the real
//! indexer:
//!
//! - That Windows actually loads this DLL, calls these methods in this order,
//!   and accepts the chunks handed back. **Nothing has ever seen the indexer ask
//!   this handler for anything.** The scope rule has only been added on a
//!   machine where the classes were not registered, because registering them
//!   needs an administrator prompt.
//! - That the indexer's own URL parser accepts a hyphen in the scheme name.
//!   RFC 3986 allows one, and Microsoft recommends `companyName.scheme`, which
//!   also contains a character outside the letters. If a real run rejects
//!   `wixen-mail`, [`url::SCHEME`] is the single place to change.
//! - Which account the indexer's host process runs as for this handler, which
//!   decides whether it can read the database at all. See [`store`].
//! - That any mail is ever found. That is one query away once somebody has run
//!   the setup tool from an administrator prompt, and the README says how.
//!
//! Until somebody has installed this and watched the indexer pick an item up,
//! treat the whole thing as unproven. It compiles, its pure parts are tested,
//! and the half that tells the indexer to look has been watched working; that is
//! still a different claim from working.
//!
//! # Limits to know before counting on this
//!
//! It covers mail only, and the URL shape is mail-shaped, so contacts,
//! calendar, tasks, notes and reminders are not in it. A message that arrived as
//! HTML with no plain alternative contributes its subject, sender and date but
//! no body text, because handing raw markup to the indexer would fill the index
//! with tag names. There is no per-item security descriptor, so the index does
//! not carry one, and nothing here catches a panic on the way out to Windows.
//!
//! One more, which arrived with the crawl scope work. A scope rule names one
//! person's mail, so two people who both want this on one computer need two
//! rules. The setup tool adds one at a time and says whose. There is no code
//! anywhere that walks the machine's accounts and sets them all up.
//!
//! # How it is put together
//!
//! The COM surface is deliberately thin, because none of it can be unit tested
//! from here. Everything that makes a decision lives in a plain Rust module with
//! tests, and the COM objects only marshal:
//!
//! - [`url`] turns a URL into a place in the store and back. Pure.
//! - [`record`] is one message reduced to what the indexer is told about it,
//!   and the properties it maps to. Pure.
//! - [`chunks`] is the sequence an [`IFilter`] walks through. Pure.
//! - [`store`] reads the database, read only.
//! - [`registration`] works out which registry entries to write. The plan is
//!   pure and tested; only the writing touches the registry.
//! - [`scope`] works out which crawl scope rule tells the indexer to look, and
//!   what a report about it may say. Pure.
//! - [`setup`] reads the setup tool's command line. Pure.
//! - [`com`] is the plumbing, and holds no decisions worth testing.
//!
//! The setup tool itself is `wixen-mail-search-setup.exe`, built from this same
//! crate. `search-handler/README.md` says how to run it and what to look for.
//!
//! [`ISearchProtocol`]: https://learn.microsoft.com/en-us/windows/win32/api/searchapi/nn-searchapi-isearchprotocol
//! [`IUrlAccessor`]: https://learn.microsoft.com/en-us/windows/win32/api/searchapi/nn-searchapi-iurlaccessor
//! [`IFilter`]: https://learn.microsoft.com/en-us/windows/win32/api/filter/nn-filter-ifilter

pub mod chunks;
pub mod com;
pub mod record;
pub mod registration;
pub mod scope;
pub mod setup;
pub mod store;
pub mod url;
