//! Reading the message cache, read only, while the application is using it.
//!
//! # Why read only is not just good manners
//!
//! This code runs inside `SearchProtocolHost.exe`, on the same SQLite file the
//! running application is writing to. Two processes on one SQLite database is
//! supported and ordinary; a second process that can also write is where
//! corruption comes from. So the connection is opened without write access at
//! all, and `PRAGMA query_only` is set on top of that, which is belt as well as
//! braces: the flag stops the file being opened for writing, and the pragma
//! stops a statement in this crate ever being one that tries.
//!
//! # What was worked out about the flags, and what is still a guess
//!
//! The application keeps the cache in write ahead logging mode. SQLite's own
//! documentation says a reader of a database in that mode needs write access to
//! the `-shm` file beside it, or to the folder holding it if that file does not
//! exist yet. Read only means read only for the *database*; the shared memory
//! file is how readers and writers find each other, and a reader still has to
//! be able to touch it. So:
//!
//! - `SQLITE_OPEN_READ_ONLY`, and deliberately not `SQLITE_OPEN_CREATE`. A
//!   wrong path has to fail rather than quietly make an empty database, which
//!   would tell the indexer the mailbox is empty and make it throw away
//!   everything it had already found.
//! - Not `immutable=1`. That is the one flag that lets a reader skip the shared
//!   memory file entirely, and it is a promise that the file is not changing.
//!   The application is running and writing, so the promise would be false, and
//!   what comes back would be stale or torn rather than merely old.
//! - Not `nolock=1`, for the same reason. It removes the locking that makes a
//!   concurrent reader safe.
//! - A short busy timeout rather than the application's five seconds. Nothing
//!   here is worth making the indexer wait for.
//!
//! What is not established here is which account the indexer's host process
//! runs the handler as, and therefore whether it can write that `-shm` file in
//! somebody's local application data folder. That decides whether any of this
//! works at all, and it cannot be answered without installing the handler and
//! watching a real indexer run. If opening fails on a live machine, this is the
//! first thing to look at.

use crate::record::Message;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

/// How long a query waits for the application to finish a write.
///
/// Much shorter than the application's own wait. The indexer is doing
/// background work on somebody's machine and would rather come back later than
/// hold a thread.
const BUSY_WAIT: Duration = Duration::from_millis(500);

/// The code reported when the failure was ours rather than SQLite's.
///
/// SQLite's own result codes are never negative, so this cannot be mistaken
/// for one.
const NOT_FROM_SQLITE: i32 = -1;

/// Where the application keeps the cache, below somebody's profile folder.
///
/// Spelled out rather than asked for, because the indexer's host process is
/// not running in the signed-in session and cannot ask Windows where that
/// person's local application data is. It has the profile folder from the
/// registry and works down from there.
///
/// This is the one place that has to agree with the application's own idea of
/// where its files live. It does not cover somebody who moved the data with
/// the application's data folder setting: that setting is read from a place
/// this process does not see, so a moved cache is simply not found, and the
/// handler reports nothing rather than reporting the wrong mailbox.
pub fn cache_path_in(profile: &Path) -> std::path::PathBuf {
    cache_path_under_local_data(&profile.join("AppData").join("Local"))
}

/// The same layout, starting from a local application data folder.
///
/// Split out because there are two ways to arrive at it and the folder names
/// below should only be written down once.
pub fn cache_path_under_local_data(local_data: &Path) -> std::path::PathBuf {
    local_data
        .join("wixen-mail")
        .join("cache")
        .join("message_cache.db")
}

/// Why the cache could not be read.
///
/// Carries SQLite's numeric result code and nothing else. Anything richer
/// would mean putting a folder name or a query into an error, and an error in
/// this crate can end up somewhere neither the project nor the person whose
/// mail it is has any control over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    CannotOpen(i32),
    CannotRead(i32),
}

/// Enough about a message to decide whether it needs indexing again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stub {
    pub uid: u32,
    /// When it was sent, in seconds since the start of 1970, when that can be
    /// read from what the server wrote.
    pub modified: Option<i64>,
}

/// A read only view of the message cache.
pub struct Store {
    /// Behind a lock because the class is registered as working on any thread,
    /// so nothing stops the indexer calling two of these at once. A SQLite
    /// connection is not shareable, and this is cheaper than reopening the
    /// database for every call.
    connection: Mutex<Connection>,
}

impl Store {
    /// Open the cache without any way of changing it.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|e| StoreError::CannotOpen(sqlite_code(&e)))?;

        connection
            .busy_timeout(BUSY_WAIT)
            .and_then(|()| connection.execute_batch("PRAGMA query_only = 1;"))
            .map_err(|e| StoreError::CannotOpen(sqlite_code(&e)))?;

        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Every account with a folder in the cache.
    pub fn accounts(&self) -> Result<Vec<String>, StoreError> {
        self.read(|connection| {
            connection
                .prepare("SELECT DISTINCT account_id FROM folders ORDER BY account_id")?
                .query_map([], |row| row.get::<_, String>(0))?
                .collect()
        })
    }

    /// Every folder in one account, named by the path the application uses.
    pub fn folders(&self, account: &str) -> Result<Vec<String>, StoreError> {
        self.read(|connection| {
            connection
                .prepare("SELECT path FROM folders WHERE account_id = ?1 ORDER BY path")?
                .query_map([account], |row| row.get::<_, String>(0))?
                .collect()
        })
    }

    /// Every message in one folder that has not been deleted.
    ///
    /// A message whose number will not fit in the URL scheme is left out. That
    /// cannot happen with a message from an IMAP server, where the number is
    /// already this size, and leaving it out is better than naming it with a
    /// number that would fetch a different message back.
    pub fn message_stubs(&self, account: &str, folder: &str) -> Result<Vec<Stub>, StoreError> {
        self.read(|connection| {
            let mut statement = connection.prepare(
                "SELECT m.uid, CAST(strftime('%s', m.date) AS INTEGER)
                 FROM messages m
                 JOIN folders f ON f.id = m.folder_id
                 WHERE f.account_id = ?1 AND f.path = ?2 AND IFNULL(m.deleted, 0) = 0
                 ORDER BY m.uid",
            )?;
            let rows = statement.query_map([account, folder], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
            })?;

            let mut stubs = Vec::new();
            for row in rows {
                let (uid, modified) = row?;
                if let Ok(uid) = u32::try_from(uid) {
                    stubs.push(Stub { uid, modified });
                }
            }
            Ok(stubs)
        })
    }

    /// One message, or nothing when it is gone.
    ///
    /// A message the indexer asks for and no longer finds is ordinary: it was
    /// deleted between the crawl and the fetch. That is an empty answer rather
    /// than a failure, because a failure tells the indexer the handler is
    /// broken instead of telling it the item has gone.
    ///
    /// Only the plain text part is treated as words. The stored HTML is not
    /// handed over, because without stripping it the index fills with tag and
    /// style names as if somebody had written them. A message that arrived as
    /// HTML only is therefore findable by its subject, sender and date, and not
    /// by anything in its body. That is a real gap, written down here rather
    /// than papered over.
    pub fn message(
        &self,
        account: &str,
        folder: &str,
        uid: u32,
    ) -> Result<Option<Message>, StoreError> {
        self.read(|connection| {
            connection
                .query_row(
                    "SELECT m.subject, m.from_addr, m.to_addr, IFNULL(m.cc, ''),
                            CAST(strftime('%s', m.date) AS INTEGER),
                            IFNULL(m.body_plain, '')
                     FROM messages m
                     JOIN folders f ON f.id = m.folder_id
                     WHERE f.account_id = ?1 AND f.path = ?2 AND m.uid = ?3
                       AND IFNULL(m.deleted, 0) = 0",
                    rusqlite::params![account, folder, i64::from(uid)],
                    |row| {
                        Ok(Message {
                            uid,
                            subject: row.get(0)?,
                            from: row.get(1)?,
                            to: row.get(2)?,
                            cc: row.get(3)?,
                            sent: row.get(4)?,
                            body: row.get(5)?,
                        })
                    },
                )
                .optional()
        })
    }

    /// Run one piece of reading with the connection held.
    fn read<T>(
        &self,
        work: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Result<T, StoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| StoreError::CannotRead(NOT_FROM_SQLITE))?;

        work(&connection).map_err(|e| StoreError::CannotRead(sqlite_code(&e)))
    }

    /// Whether this connection really cannot write, asked of SQLite itself.
    ///
    /// Only compiled for the tests. Nothing in the handler needs to ask, and
    /// the question is worth asking exactly once, in a test that would notice
    /// if the open flags were ever loosened.
    #[cfg(test)]
    fn write_is_refused(&self) -> bool {
        match self.connection.lock() {
            Ok(connection) => connection
                .execute_batch("CREATE TABLE a_table_the_handler_should_never_make (x)")
                .is_err(),
            Err(_) => true,
        }
    }
}

/// SQLite's own number for what went wrong.
fn sqlite_code(error: &rusqlite::Error) -> i32 {
    match error {
        rusqlite::Error::SqliteFailure(failure, _) => failure.extended_code,
        _ => NOT_FROM_SQLITE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::TempDir;

    /// A cache with the same tables the application makes.
    ///
    /// Only the columns that were in the first version are used anywhere in
    /// this module. The application adds columns over time and opens older
    /// databases as they are, so a handler that needed a recent column would
    /// fail on exactly the installations that have been running longest.
    struct Cache {
        _dir: TempDir,
        path: std::path::PathBuf,
        writer: Connection,
    }

    fn a_cache() -> Cache {
        let dir = TempDir::new().expect("a temporary directory");
        let path = dir.path().join("message_cache.db");
        let writer = Connection::open(&path).expect("a writable cache");
        writer
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 CREATE TABLE folders (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     account_id TEXT NOT NULL,
                     name TEXT NOT NULL,
                     path TEXT NOT NULL,
                     folder_type TEXT NOT NULL,
                     unread_count INTEGER DEFAULT 0,
                     total_count INTEGER DEFAULT 0,
                     UNIQUE(account_id, path)
                 );
                 CREATE TABLE messages (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     uid INTEGER NOT NULL,
                     folder_id INTEGER NOT NULL,
                     message_id TEXT NOT NULL,
                     subject TEXT NOT NULL,
                     from_addr TEXT NOT NULL,
                     to_addr TEXT NOT NULL,
                     cc TEXT,
                     date TEXT NOT NULL,
                     body_plain TEXT,
                     body_html TEXT,
                     read BOOLEAN DEFAULT 0,
                     starred BOOLEAN DEFAULT 0,
                     deleted BOOLEAN DEFAULT 0,
                     UNIQUE(folder_id, uid)
                 );",
            )
            .expect("the cache tables");
        Cache {
            _dir: dir,
            path,
            writer,
        }
    }

    impl Cache {
        fn folder(&self, account: &str, path: &str) -> i64 {
            self.writer
                .execute(
                    "INSERT INTO folders (account_id, name, path, folder_type)
                     VALUES (?1, ?2, ?2, 'inbox')",
                    rusqlite::params![account, path],
                )
                .expect("a folder");
            self.writer.last_insert_rowid()
        }

        fn message(&self, folder: i64, uid: u32, subject: &str, date: &str) {
            self.writer
                .execute(
                    "INSERT INTO messages
                         (uid, folder_id, message_id, subject, from_addr, to_addr, cc,
                          date, body_plain, body_html, deleted)
                     VALUES (?1, ?2, 'mid', ?3, 'a@example.com', 'b@example.com', '',
                             ?4, 'the body', NULL, 0)",
                    rusqlite::params![uid, folder, subject, date],
                )
                .expect("a message");
        }

        fn open(&self) -> Store {
            Store::open(&self.path).expect("the cache should open read only")
        }
    }

    #[test]
    fn test_the_cache_sits_where_the_application_puts_it_under_a_profile() {
        // The indexer runs outside the signed-in session, so it cannot ask the
        // usual way where somebody's local application data is. It works it
        // out from the profile folder instead, and this has to match what the
        // application itself uses or the handler reads nothing.
        assert_eq!(
            cache_path_in(Path::new(r"C:\Users\someone")),
            Path::new(r"C:\Users\someone\AppData\Local\wixen-mail\cache\message_cache.db")
        );
    }

    #[test]
    fn test_both_ways_of_finding_the_cache_arrive_at_the_same_file() {
        // One route starts from a profile folder read out of the registry and
        // the other from a local application data folder. They are used in
        // different situations and they have to agree, because a difference
        // would only show up as one of the two silently finding nothing.
        assert_eq!(
            cache_path_in(Path::new(r"C:\Users\someone")),
            cache_path_under_local_data(Path::new(r"C:\Users\someone\AppData\Local"))
        );
    }

    const A_REAL_DATE: &str = "2023-11-14T22:13:20+00:00";
    const A_REAL_DATE_IN_SECONDS: i64 = 1_700_000_000;

    #[test]
    fn test_the_cache_is_opened_read_only_so_the_indexer_can_never_change_it() {
        // This runs inside a Microsoft process, on a database the application
        // is using at the same time. A handler that could write could corrupt
        // somebody's whole mailbox, and nothing here has any reason to.
        let cache = a_cache();
        let store = cache.open();

        assert!(store.write_is_refused(), "the connection accepted a write");
    }

    #[test]
    fn test_a_cache_that_is_not_there_is_refused_rather_than_created() {
        // Without this, a wrong path makes an empty database, the indexer is
        // told the mailbox is empty, and everything looks like it is working.
        let dir = TempDir::new().expect("a temporary directory");
        let missing = dir.path().join("message_cache.db");

        assert!(matches!(
            Store::open(&missing),
            Err(StoreError::CannotOpen(_))
        ));
        assert!(!missing.exists(), "opening created a database");
    }

    #[test]
    fn test_reading_works_while_the_application_still_has_the_cache_open() {
        // The whole point. The application is running, holding this database
        // in write ahead logging mode, and the indexer has to be able to read
        // it anyway without either one blocking the other.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache.message(inbox, 7, "Quarterly report", A_REAL_DATE);

        let store = cache.open();
        assert_eq!(
            store.accounts().expect("accounts"),
            vec!["work".to_string()]
        );

        // And the writer is still usable afterwards, so the read took nothing
        // the application needed.
        cache.message(inbox, 8, "Another one", A_REAL_DATE);
    }

    #[test]
    fn test_each_account_is_named_once_however_many_folders_it_has() {
        // The indexer walks the list it is given. A repeated account would be
        // crawled twice over, and every message in it indexed twice.
        let cache = a_cache();
        cache.folder("work", "INBOX");
        cache.folder("work", "Sent");
        cache.folder("home", "INBOX");

        let mut accounts = cache.open().accounts().expect("accounts");
        accounts.sort();

        assert_eq!(accounts, vec!["home".to_string(), "work".to_string()]);
    }

    #[test]
    fn test_folders_are_listed_for_the_account_asked_for_and_no_other() {
        // Two accounts on one machine is ordinary, and mixing them would put
        // work mail under a personal account in every search result.
        let cache = a_cache();
        cache.folder("work", "INBOX");
        cache.folder("work", "Sent");
        cache.folder("home", "Personal");

        let mut folders = cache.open().folders("work").expect("folders");
        folders.sort();

        assert_eq!(folders, vec!["INBOX".to_string(), "Sent".to_string()]);
    }

    #[test]
    fn test_a_folder_name_containing_a_quotation_mark_is_matched_and_not_run() {
        // Folder names come from a mail server, so they are untrusted text
        // going into a query. Bound parameters are what keeps that text from
        // being read as part of the statement.
        let cache = a_cache();
        let awkward = "It's \"quoted\"; DROP TABLE messages;--";
        let folder = cache.folder("work", awkward);
        cache.message(folder, 7, "Quarterly report", A_REAL_DATE);

        let store = cache.open();
        assert_eq!(store.folders("work").expect("folders"), vec![awkward]);
        assert_eq!(
            store.message_stubs("work", awkward).expect("stubs").len(),
            1
        );
    }

    #[test]
    fn test_each_message_is_listed_with_the_moment_it_was_sent() {
        // The moment is what lets the indexer skip a message it already has.
        // It is stored as the text a mail server wrote, so this also proves
        // that text really does turn into a number.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache.message(inbox, 7, "Quarterly report", A_REAL_DATE);

        assert_eq!(
            cache.open().message_stubs("work", "INBOX").expect("stubs"),
            vec![Stub {
                uid: 7,
                modified: Some(A_REAL_DATE_IN_SECONDS),
            }]
        );
    }

    #[test]
    fn test_a_date_the_server_wrote_oddly_leaves_the_message_findable_without_one() {
        // Mail servers write all sorts of things in a Date header. Losing the
        // message because its date is strange would be a far worse trade than
        // indexing it without one.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache.message(inbox, 7, "Quarterly report", "sometime last Tuesday");

        let store = cache.open();
        assert_eq!(
            store.message_stubs("work", "INBOX").expect("stubs"),
            vec![Stub {
                uid: 7,
                modified: None,
            }]
        );
        let message = store
            .message("work", "INBOX", 7)
            .expect("a readable message")
            .expect("the message should still be there");
        assert_eq!(message.sent, None);
        assert_eq!(message.subject, "Quarterly report");
    }

    #[test]
    fn test_a_message_is_read_back_with_everything_the_indexer_is_told() {
        // Each of these is a field somebody searches by. A column read into
        // the wrong place shows up as a message that is findable by the wrong
        // person's address, which is worse than not being findable at all.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache.message(inbox, 7, "Quarterly report", A_REAL_DATE);

        let message = cache
            .open()
            .message("work", "INBOX", 7)
            .expect("a readable message")
            .expect("the message should be there");

        assert_eq!(
            message,
            crate::record::Message {
                uid: 7,
                subject: "Quarterly report".to_string(),
                from: "a@example.com".to_string(),
                to: "b@example.com".to_string(),
                cc: String::new(),
                sent: Some(A_REAL_DATE_IN_SECONDS),
                body: "the body".to_string(),
            }
        );
    }

    #[test]
    fn test_a_message_that_is_not_there_is_an_empty_answer_and_not_a_failure() {
        // The indexer keeps urls it saw earlier and comes back for them. A
        // message deleted since then is normal, and reporting it as a failure
        // makes the indexer treat the handler as broken rather than treating
        // the item as gone.
        let cache = a_cache();
        cache.folder("work", "INBOX");
        let store = cache.open();

        assert_eq!(store.message("work", "INBOX", 999), Ok(None));
        assert_eq!(store.message("work", "Nowhere", 7), Ok(None));
        assert_eq!(store.message("nobody", "INBOX", 7), Ok(None));
    }

    #[test]
    fn test_a_message_marked_deleted_is_not_offered_to_the_indexer() {
        // A message somebody has deleted should stop being findable. Leaving
        // it in the index means Windows keeps showing mail the person believes
        // they have thrown away.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache.message(inbox, 7, "Quarterly report", A_REAL_DATE);
        cache
            .writer
            .execute("UPDATE messages SET deleted = 1 WHERE uid = 7", [])
            .expect("marking it deleted");

        let store = cache.open();
        assert_eq!(store.message_stubs("work", "INBOX").expect("stubs"), vec![]);
        assert_eq!(store.message("work", "INBOX", 7), Ok(None));
    }

    #[test]
    fn test_a_message_with_only_a_web_page_body_is_indexed_without_one() {
        // The cache keeps the plain text and the html separately, and only the
        // plain text is words. Handing over html would put tag names and style
        // rules into the index as if somebody had written them, so a message
        // with no plain part is indexed by its subject and sender alone. That
        // is a real gap and it is a smaller one than filling the index with
        // markup.
        let cache = a_cache();
        let inbox = cache.folder("work", "INBOX");
        cache
            .writer
            .execute(
                "INSERT INTO messages
                     (uid, folder_id, message_id, subject, from_addr, to_addr, cc,
                      date, body_plain, body_html, deleted)
                 VALUES (7, ?1, 'mid', 'Quarterly report', 'a@example.com',
                         'b@example.com', '', ?2, NULL, '<p>hello</p>', 0)",
                rusqlite::params![inbox, A_REAL_DATE],
            )
            .expect("a message with only html");

        let message = cache
            .open()
            .message("work", "INBOX", 7)
            .expect("a readable message")
            .expect("the message should be there");

        assert_eq!(message.body, "");
        assert_eq!(message.subject, "Quarterly report");
    }

    #[test]
    fn test_a_cache_missing_the_tables_altogether_fails_rather_than_looking_empty() {
        // A file that is not our cache, or one from before these tables
        // existed. Reporting no accounts would tell the indexer the mailbox is
        // empty, and it would then remove everything it had already indexed.
        let dir = TempDir::new().expect("a temporary directory");
        let path = dir.path().join("message_cache.db");
        Connection::open(&path).expect("an empty database");

        let store = Store::open(&path).expect("an empty database still opens");
        assert!(matches!(store.accounts(), Err(StoreError::CannotRead(_))));
    }
}
