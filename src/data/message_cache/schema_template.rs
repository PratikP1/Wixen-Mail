//! The database a test's cache starts from.
//!
//! [`MessageCache::new`] builds the schema on every open: about thirty
//! `CREATE TABLE IF NOT EXISTS` statements, twenty-eight indexes, and a
//! hundred and six calls to `ensure_column_exists`, each of which reads a
//! table's columns and most of which then add one. Every one of those is a
//! write transaction. A user pays for that once, when their mail folder is
//! made. The library suite paid for it 685 times, once per test that opens a
//! cache, and `data` was 11% of the tests and 39% of the time.
//!
//! So it is paid once per process here instead. The first test to ask for a
//! cache builds a database by running the real schema, the bytes of that file
//! are kept for as long as the process lives, and every later test cache
//! starts as a copy of them.
//!
//! # Why this is safe
//!
//! **The schema is still exercised.** It is run, in full, against a fresh
//! database, once. A statement that no longer works reddens the first test to
//! ask for a cache instead of all of them, which is the same finding arriving
//! once rather than 685 times.
//!
//! **It is not a second description of the schema.** The template is whatever
//! [`MessageCache::new`] produces, so there is nothing here to drift away from
//! the code. A schema written out a second time would be a worse defect than
//! the cost this removes.
//!
//! **Test builds only.** The whole module is behind `#[cfg(test)]`, so none of
//! it is compiled into the program somebody installs, and a user's database is
//! never produced by copying a file. Integration tests under `tests/` link the
//! library built without `cfg(test)`, so they take the ordinary path as well:
//! `tests/a_half_finished_task_move.rs` builds an older database by hand and
//! opens a cache over it to watch a migration run, and nothing here is in that
//! build to hide it.
//!
//! **A database that is already there is left alone.** That is the second
//! guard on the case above, for the tests inside `src/` that do the same
//! thing, and there are several. `lay_it_down_at` writes nothing when a file
//! is already at the path.

use super::MessageCache;
use std::cell::Cell;
use std::path::Path;
use std::sync::OnceLock;

thread_local! {
    /// True while this thread is opening a cache without the template.
    ///
    /// The tests need one side of their comparison built the way a user's
    /// first run builds it, and the build of the template itself must not ask
    /// for the answer it is in the middle of working out.
    static WITHOUT_IT: Cell<bool> = const { Cell::new(false) };

    /// How many caches this thread has started from the template.
    ///
    /// Per thread rather than per process, because the suite runs tests in
    /// parallel and a count everything shares could only be asserted about
    /// loosely. A loose assertion here would pass whether or not
    /// [`MessageCache::new`] still lays the template down, which is the one
    /// thing it exists to say. Each test has a thread to itself.
    static STARTED_FROM_IT: Cell<usize> = const { Cell::new(0) };
}

/// Put the template where a cache about to open will find it.
///
/// Does nothing at all in three cases, and each of them matters. When this
/// thread is building the template, because that open is the ordinary path by
/// definition. When a database is already at the path, because a test that
/// writes an older schema by hand and then opens a cache over it is testing a
/// migration, and laying the current schema on top of it would hide exactly
/// what it is about. And when there is no template, because building one can
/// fail and the ordinary path is the right thing to fall back to.
pub(super) fn lay_it_down_at(db_path: &Path) {
    if WITHOUT_IT.get() || db_path.exists() {
        return;
    }
    let Some(bytes) = the_database_every_test_cache_starts_from() else {
        return;
    };
    if std::fs::write(db_path, bytes).is_err() {
        // Half a file is not a database. Take it away again, so the open
        // behind this builds the schema the slow and ordinary way rather than
        // opening something damaged.
        let _ = std::fs::remove_file(db_path);
        return;
    }
    STARTED_FROM_IT.set(STARTED_FROM_IT.get() + 1);
}

/// The bytes of a database holding the current schema and nothing else.
///
/// `None` when one could not be built, and then every cache is built the
/// ordinary way: correct, and as slow as it was before. Quiet on purpose, so a
/// machine that cannot write a temporary file still runs the suite. The tests
/// below assert that it is `Some`, because a fallback nothing notices is how a
/// check stops checking.
fn the_database_every_test_cache_starts_from() -> Option<&'static [u8]> {
    static TEMPLATE: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    TEMPLATE.get_or_init(build_one).as_deref()
}

/// Build it, by opening a cache the way a user's first run opens one.
///
/// Running the real schema rather than writing a second description of it.
/// A second description is a second source of truth, and it would drift from
/// the first with nothing failing, which is a worse defect than the cost this
/// removes.
///
/// The directory goes away as this returns. What is kept is the bytes, so
/// there is no file in the temporary folder outliving the process that made
/// it, and a test cache is written rather than copied.
fn build_one() -> Option<Vec<u8>> {
    let home = tempfile::tempdir().ok()?;
    let db_path = home.path().join("message_cache.db");
    {
        let cache =
            without_the_template(|| MessageCache::new(home.path().to_path_buf(), None)).ok()?;
        // The schema is written through the write-ahead log, so nearly all of
        // it is in the sidecar file rather than in the database itself until
        // something moves it across. Reading the database before that would
        // read a file with almost nothing in it.
        cache
            .conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .ok()?;
    }
    std::fs::read(&db_path).ok()
}

/// Open a cache without the template, the way a user's first run opens one.
fn without_the_template<T>(open: impl FnOnce() -> T) -> T {
    let _put_it_back = PutItBack;
    WITHOUT_IT.set(true);
    open()
}

/// Puts the flag back however the open above ended, panicking included, which
/// is how a failing test ends.
struct PutItBack;

impl Drop for PutItBack {
    fn drop(&mut self) {
        WITHOUT_IT.set(false);
    }
}

/// How many caches this thread has started from the template.
fn caches_started_from_it_on_this_thread() -> usize {
    STARTED_FROM_IT.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What SQLite says the database is: every table, index and trigger it
    /// holds, and every column of every table with its type, whether it may be
    /// null, its default, and its place in the key.
    ///
    /// Asked of SQLite rather than compared against a schema written out here,
    /// so the comparison cannot agree with a description that is itself wrong.
    fn the_shape_of(cache: &MessageCache) -> Vec<String> {
        let mut written = Vec::new();
        let mut tables = Vec::new();

        {
            let mut objects = cache
                .conn
                .prepare(
                    "SELECT type, name, tbl_name, IFNULL(sql, '')
                     FROM sqlite_master ORDER BY type, name",
                )
                .expect("SQLite to say what it holds");
            let rows = objects
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .expect("the objects in the database");
            for row in rows {
                let (kind, name, table, sql) = row.expect("an object");
                if kind == "table" {
                    tables.push(name.clone());
                }
                written.push(format!("{kind} {name} on {table}: {sql}"));
            }
        }

        for table in tables {
            let mut columns = cache
                .conn
                .prepare(&format!("PRAGMA table_info(\"{table}\")"))
                .expect("SQLite to say what a table's columns are");
            let rows = columns
                .query_map([], |row| {
                    Ok(format!(
                        "{} {} null={} default={:?} key={}",
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)? == 0,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, i64>(5)?,
                    ))
                })
                .expect("the columns of a table");
            for row in rows {
                written.push(format!("{table}.{}", row.expect("a column")));
            }
        }

        written
    }

    /// How many rows each table holds.
    ///
    /// A template carrying data would give every test in the suite a database
    /// that is not empty, which is the kind of thing that shows up as one
    /// unrelated test failing for a reason nobody can find.
    fn what_is_in(cache: &MessageCache) -> Vec<String> {
        let mut tables = Vec::new();
        {
            let mut named = cache
                .conn
                .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .expect("SQLite to say what tables it holds");
            let rows = named
                .query_map([], |row| row.get::<_, String>(0))
                .expect("the tables");
            for row in rows {
                tables.push(row.expect("a table"));
            }
        }

        tables
            .into_iter()
            .map(|table| {
                let held =
                    cache
                        .conn
                        .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |row| {
                            row.get::<_, i64>(0)
                        });
                match held {
                    Ok(rows) => format!("{table}: {rows}"),
                    // Kept rather than unwrapped, so a table SQLite will not
                    // count is compared as the same refusal on both sides
                    // instead of failing one of them for an unrelated reason.
                    Err(refused) => format!("{table}: {refused}"),
                }
            })
            .collect()
    }

    /// Why every test here begins by asserting there is a template at all.
    ///
    /// Without one, both sides of every comparison below are a cache built by
    /// running the schema, so they agree, and the run says nothing about the
    /// mechanism. The fallback is deliberately quiet, so something has to be
    /// noisy about it.
    const NO_TEMPLATE: &str =
        "there is no schema template, so a test cache is not started from one";

    #[test]
    fn test_a_cache_from_the_template_has_the_schema_the_code_builds() {
        assert!(
            the_database_every_test_cache_starts_from().is_some(),
            "{NO_TEMPLATE}"
        );

        let started_from_it = tempfile::tempdir().expect("a directory");
        let templated = MessageCache::new(started_from_it.path().to_path_buf(), None)
            .expect("a cache started from the template");

        let built_by_the_schema = tempfile::tempdir().expect("a directory");
        let ordinary = without_the_template(|| {
            MessageCache::new(built_by_the_schema.path().to_path_buf(), None)
        })
        .expect("a cache built by running the schema");

        assert_eq!(
            the_shape_of(&templated),
            the_shape_of(&ordinary),
            "a test's database is not shaped like the one a user's first run \
             builds, so the suite is testing a schema nobody ships"
        );
    }

    #[test]
    fn test_a_cache_from_the_template_holds_what_a_new_one_holds() {
        assert!(
            the_database_every_test_cache_starts_from().is_some(),
            "{NO_TEMPLATE}"
        );

        let started_from_it = tempfile::tempdir().expect("a directory");
        let templated = MessageCache::new(started_from_it.path().to_path_buf(), None)
            .expect("a cache started from the template");

        let built_by_the_schema = tempfile::tempdir().expect("a directory");
        let ordinary = without_the_template(|| {
            MessageCache::new(built_by_the_schema.path().to_path_buf(), None)
        })
        .expect("a cache built by running the schema");

        assert_eq!(
            what_is_in(&templated),
            what_is_in(&ordinary),
            "a test's database does not start with what a new one starts with"
        );
    }

    #[test]
    fn test_the_template_is_laid_down_where_a_cache_will_look_for_it() {
        let template = the_database_every_test_cache_starts_from();
        assert!(template.is_some(), "{NO_TEMPLATE}");

        let dir = tempfile::tempdir().expect("a directory");
        let db_path = dir.path().join("message_cache.db");
        lay_it_down_at(&db_path);

        assert_eq!(
            std::fs::read(&db_path).ok().as_deref(),
            template,
            "the file a cache opens is not the template"
        );
    }

    #[test]
    fn test_a_database_that_is_already_there_is_left_alone() {
        assert!(
            the_database_every_test_cache_starts_from().is_some(),
            "{NO_TEMPLATE}, so nothing could have landed on the file below and \
             this check would pass without checking anything"
        );

        let dir = tempfile::tempdir().expect("a directory");
        let db_path = dir.path().join("message_cache.db");
        let older: &[u8] = b"a database an older build wrote";
        std::fs::write(&db_path, older).expect("a database that is already there");

        lay_it_down_at(&db_path);

        assert_eq!(
            std::fs::read(&db_path).expect("the file to still be there"),
            older,
            "the template landed on a database that was already there, which \
             is how a test about a migration stops being about one"
        );
    }

    #[test]
    fn test_an_ordinary_test_cache_starts_from_the_template() {
        let before = caches_started_from_it_on_this_thread();

        let dir = tempfile::tempdir().expect("a directory");
        let _cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");

        assert_eq!(
            caches_started_from_it_on_this_thread(),
            before + 1,
            "MessageCache::new did not start this cache from the template, so \
             every test in the suite is still building the whole schema"
        );
    }
}
