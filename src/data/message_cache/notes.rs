//! Note and NoteFolder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, NoteEntry, NoteFolderEntry};

/// What a note's body is written in, as the `format` column spells it.
///
/// Every note this program has ever written says the same thing here, and
/// nothing reads it to decide anything. That is the answer rather than an
/// oversight waiting for a feature, and this is where it is written down so
/// the next person reading the schema does not have to work it out again.
///
/// A note's body is text. `application::long_text`'s module header states the
/// rule it is read by: what is stored is exactly what was typed, markdown is
/// legible as it stands, and anything that is not markdown is simply text with
/// no structure in it rather than an error. The reading looks for whatever
/// structure is there and finds none where there is none, so there is no fork
/// here for a column to pick.
///
/// Making the reader obey this column would be a loss rather than a feature.
/// Every row holds `plain`, the note editor labels its box "Body, in Markdown",
/// and `NoteItem::read_full` speaks the body's structure. A reader that took
/// the column at its word would stop reading headings and lists in every note
/// that already exists.
///
/// What the column can honestly be is the place a note that came from
/// somewhere else says what it is. [`NoteBody::Other`] is how it keeps that
/// open, following [`crate::data::message_cache::AddressBook`], whose own doc
/// comment gives the reason: a word this code does not recognise is still
/// somebody else's answer, and forgetting it rewrites their row on the next
/// save without anybody having asked for that.
///
/// The stored word stays `plain` because that is what every shipped row holds.
/// Changing it would sort the notes on somebody's disk into ones written
/// before a build and ones written after, for no reader's benefit, since no
/// reader consults it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteBody {
    /// Whatever was typed, read for whatever structure is in it. The one
    /// answer this program writes.
    AsTyped,
    /// A word some other build, or some other program, put there.
    Other(String),
}

impl NoteBody {
    /// The word the column holds.
    ///
    /// One place answers this, so a second writer cannot spell it differently.
    /// It used to be a literal at each write site, which is how a column comes
    /// to hold two words for one fact.
    pub fn as_stored(&self) -> &str {
        match self {
            NoteBody::AsTyped => "plain",
            NoteBody::Other(word) => word,
        }
    }

    /// What a stored word means.
    pub fn from_stored(value: &str) -> Self {
        match value {
            "plain" => NoteBody::AsTyped,
            other => NoteBody::Other(other.to_string()),
        }
    }
}

impl MessageCache {
    // ── Note Folders ────────────────────────────────────────────────────────

    /// Save (upsert) a note folder.
    pub fn save_note_folder(&self, nf: &NoteFolderEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO note_folders (id, account_id, name, display_order, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    display_order = excluded.display_order",
                rusqlite::params![
                    nf.id,
                    nf.account_id,
                    nf.name,
                    nf.display_order,
                    nf.created_at
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save note folder: {}", e)))?;
        Ok(())
    }

    /// Get all note folders for an account.
    pub fn get_note_folders_for_account(&self, account_id: &str) -> Result<Vec<NoteFolderEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, display_order, created_at
                 FROM note_folders WHERE account_id = ?1 ORDER BY display_order, name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare note folders query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(NoteFolderEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    display_order: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query note folders: {}", e)))?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(
                row.map_err(|e| Error::Other(format!("Failed to read note folder row: {}", e)))?,
            );
        }
        Ok(folders)
    }

    /// Delete a note folder and all its notes.
    pub fn delete_note_folder(&self, folder_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM notes WHERE folder_id = ?1",
                rusqlite::params![folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete notes in folder: {}", e)))?;
        self.conn
            .execute(
                "DELETE FROM note_folders WHERE id = ?1",
                rusqlite::params![folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete note folder: {}", e)))?;
        Ok(())
    }

    /// Ensure a default note folder exists.
    pub fn ensure_default_note_folder(&self, account_id: &str) -> Result<NoteFolderEntry> {
        let existing = self.get_note_folders_for_account(account_id)?;
        if let Some(first) = existing.into_iter().next() {
            return Ok(first);
        }
        let now = chrono::Utc::now().to_rfc3339();
        let nf = NoteFolderEntry {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            name: "General".to_string(),
            display_order: 0,
            created_at: now,
        };
        self.save_note_folder(&nf)?;
        Ok(nf)
    }

    // ── Notes ───────────────────────────────────────────────────────────────

    /// Save (upsert) a note.
    pub fn save_note(&self, n: &NoteEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO notes (
                    id, account_id, folder_id, title, body, format, pinned,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(id) DO UPDATE SET
                    folder_id = excluded.folder_id,
                    title = excluded.title,
                    body = excluded.body,
                    format = excluded.format,
                    pinned = excluded.pinned,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    n.id,
                    n.account_id,
                    n.folder_id,
                    n.title,
                    n.body,
                    n.format.as_stored(),
                    n.pinned,
                    n.created_at,
                    n.updated_at,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save note: {}", e)))?;
        Ok(())
    }

    /// Get all notes for a folder.
    pub fn get_notes_for_folder(&self, folder_id: &str) -> Result<Vec<NoteEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, folder_id, title, body, format, pinned,
                        created_at, updated_at
                 FROM notes WHERE folder_id = ?1
                 ORDER BY pinned DESC, updated_at DESC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare notes query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![folder_id], Self::map_note_row)
            .map_err(|e| Error::Other(format!("Failed to query notes: {}", e)))?;

        let mut notes = Vec::new();
        for row in rows {
            notes.push(row.map_err(|e| Error::Other(format!("Failed to read note row: {}", e)))?);
        }
        Ok(notes)
    }

    /// Get all notes for an account.
    pub fn get_all_notes_for_account(&self, account_id: &str) -> Result<Vec<NoteEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, folder_id, title, body, format, pinned,
                        created_at, updated_at
                 FROM notes WHERE account_id = ?1
                 ORDER BY pinned DESC, updated_at DESC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare notes query: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id], Self::map_note_row)
            .map_err(|e| Error::Other(format!("Failed to query notes: {}", e)))?;

        let mut notes = Vec::new();
        for row in rows {
            notes.push(row.map_err(|e| Error::Other(format!("Failed to read note row: {}", e)))?);
        }
        Ok(notes)
    }

    /// Load one note in full, including its body.
    ///
    /// The list carries only a short preview so it stays quick to arrow
    /// through, so the editor has to come back here for the real content.
    /// Returns `None` when the note no longer exists.
    pub fn get_note(&self, note_id: &str) -> Result<Option<NoteEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, folder_id, title, body, format, pinned,
                        created_at, updated_at
                 FROM notes WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare note query: {}", e)))?;

        let mut rows = stmt
            .query_map(rusqlite::params![note_id], Self::map_note_row)
            .map_err(|e| Error::Other(format!("Failed to query note: {}", e)))?;

        match rows.next() {
            Some(row) => {
                Ok(Some(row.map_err(|e| {
                    Error::Other(format!("Failed to read note row: {}", e))
                })?))
            }
            None => Ok(None),
        }
    }

    /// Delete a note.
    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM notes WHERE id = ?1",
                rusqlite::params![note_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete note: {}", e)))?;
        Ok(())
    }

    /// Toggle pin status of a note.
    pub fn toggle_note_pin(&self, note_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE notes SET pinned = NOT pinned, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, note_id],
            )
            .map_err(|e| Error::Other(format!("Failed to toggle note pin: {}", e)))?;
        Ok(())
    }

    /// Search notes by title or body.
    pub fn search_notes(&self, account_id: &str, query: &str) -> Result<Vec<NoteEntry>> {
        let pattern = super::like_pattern(query);
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, folder_id, title, body, format, pinned,
                        created_at, updated_at
                 FROM notes WHERE account_id = ?1 AND (title LIKE ?2 ESCAPE '!' OR body LIKE ?2 ESCAPE '!')
                 ORDER BY pinned DESC, updated_at DESC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare note search: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id, pattern], Self::map_note_row)
            .map_err(|e| Error::Other(format!("Failed to search notes: {}", e)))?;

        let mut notes = Vec::new();
        for row in rows {
            notes.push(row.map_err(|e| Error::Other(format!("Failed to read note row: {}", e)))?);
        }
        Ok(notes)
    }

    /// Map a rusqlite row to a NoteEntry.
    fn map_note_row(row: &rusqlite::Row) -> rusqlite::Result<NoteEntry> {
        Ok(NoteEntry {
            id: row.get(0)?,
            account_id: row.get(1)?,
            folder_id: row.get(2)?,
            title: row.get(3)?,
            body: row.get(4)?,
            format: match row.get::<_, Option<String>>(5)? {
                Some(word) => NoteBody::from_stored(&word),
                // The column carries a default and no NOT NULL, so a null is a
                // row written by something that did not name it. The schema's
                // own answer for that row is the default, and refusing to read
                // somebody's note over a word nothing consults is the worse of
                // the two.
                None => NoteBody::AsTyped,
            },
            pinned: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_note_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    #[test]
    fn test_note_folder_crud() {
        let cache = test_cache();
        let nf = cache.ensure_default_note_folder("acct-1").unwrap();
        assert_eq!(nf.name, "General");

        let folders = cache.get_note_folders_for_account("acct-1").unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn test_deleting_a_note_folder_takes_the_notes_in_it() {
        // Reporting success and removing nothing leaves the folder in the list
        // after somebody deleted it, and it comes back on the next start.
        // Leaving the notes behind is worse: they belong to a folder that is
        // gone, so nothing in the application can reach them again.
        let cache = test_cache();
        let kept = cache.ensure_default_note_folder("acct-1").unwrap();
        let going = NoteFolderEntry {
            id: "folder-going".to_string(),
            account_id: "acct-1".to_string(),
            name: "Old ideas".to_string(),
            display_order: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.save_note_folder(&going).unwrap();
        for (id, folder) in [("note-going", &going.id), ("note-kept", &kept.id)] {
            cache
                .save_note(&NoteEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(folder.clone()),
                    title: "A note".to_string(),
                    body: "Something worth keeping".to_string(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                })
                .unwrap();
        }

        cache.delete_note_folder(&going.id).unwrap();

        let folders = cache.get_note_folders_for_account("acct-1").unwrap();
        assert_eq!(
            folders.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
            [kept.id.as_str()],
            "the folder was reported deleted and is still there"
        );
        assert!(
            cache.get_notes_for_folder(&going.id).unwrap().is_empty(),
            "the notes outlived the folder holding them"
        );
        assert_eq!(
            cache.get_notes_for_folder(&kept.id).unwrap().len(),
            1,
            "deleting one folder took another folder's notes"
        );
    }

    #[test]
    fn test_note_crud() {
        let cache = test_cache();
        let nf = cache.ensure_default_note_folder("acct-1").unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        let n = NoteEntry {
            id: "note-1".to_string(),
            account_id: "acct-1".to_string(),
            folder_id: Some(nf.id.clone()),
            title: "Meeting Notes".to_string(),
            body: "Discussed roadmap for Q2.".to_string(),
            format: NoteBody::AsTyped,
            pinned: false,
            created_at: now.clone(),
            updated_at: now,
        };
        cache.save_note(&n).unwrap();

        let notes = cache.get_notes_for_folder(&nf.id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Meeting Notes");

        cache.toggle_note_pin("note-1").unwrap();
        let notes = cache.get_all_notes_for_account("acct-1").unwrap();
        assert!(notes[0].pinned);

        cache.delete_note("note-1").unwrap();
        let notes = cache.get_notes_for_folder(&nf.id).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn test_note_search() {
        let cache = test_cache();
        let nf = cache.ensure_default_note_folder("acct-1").unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        for (id, title, body) in [
            ("n1", "Project Alpha", "Timeline for alpha launch"),
            ("n2", "Project Beta", "Beta phase planning"),
            ("n3", "Grocery List", "Milk, eggs, bread"),
        ] {
            cache
                .save_note(&NoteEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(nf.id.clone()),
                    title: title.to_string(),
                    body: body.to_string(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                })
                .unwrap();
        }

        let results = cache.search_notes("acct-1", "Project").unwrap();
        assert_eq!(results.len(), 2);

        // Search by body content
        let results = cache.search_notes("acct-1", "bread").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Grocery List");
    }

    #[test]
    fn test_get_note_returns_the_full_body() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        let body = "line one
line two
"
        .repeat(40);
        cache
            .save_note(&NoteEntry {
                id: "n1".into(),
                account_id: "acct-1".into(),
                folder_id: Some(folder.id.clone()),
                title: "Long note".into(),
                body: body.clone(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".into(),
                updated_at: "2026-01-01".into(),
            })
            .unwrap();

        let loaded = cache.get_note("n1").unwrap().expect("note should exist");
        // The list preview is truncated on purpose; the editor must not be.
        assert_eq!(loaded.body, body);
        assert_eq!(loaded.title, "Long note");
    }

    #[test]
    fn test_get_note_returns_none_when_missing() {
        let cache = test_cache();
        assert!(cache.get_note("does-not-exist").unwrap().is_none());
    }

    /// A body that went into storage comes back as the same bytes.
    ///
    /// The editor calls its box "Body, in Markdown" and the reader looks for
    /// whatever structure is in what was typed, so everything downstream rests
    /// on the text arriving unchanged. Nothing asserted that through storage
    /// until now: `long_text`'s own round-trip tests put text through a
    /// formatter and back, which says nothing about SQLite, about the upsert,
    /// or about a tidy-up somebody adds to `save_note` later.
    ///
    /// Each fixture is aimed at a particular wrong implementation rather than
    /// chosen for size. A body that is merely long tests nothing.
    #[test]
    fn test_a_notes_body_comes_back_the_bytes_it_went_in_as() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        let fixtures = [
            (
                "a trailing space, which a trim added while tidying would take",
                "Remember the space at the end ",
            ),
            (
                "Windows line endings, which a normalisation would rewrite",
                "one\r\ntwo\r\n",
            ),
            (
                "a character outside the basic multilingual plane, which a \
                 byte-length assumption would cut in half",
                "Dinner at seven 🍜 and a walk after",
            ),
            (
                "a backslash before a newline, which an escape pass would read \
                 as an instruction",
                "C:\\Users\\notes\\\nand the line after it",
            ),
        ];

        for (n, (aimed_at, body)) in fixtures.iter().enumerate() {
            let id = format!("note-{n}");
            cache
                .save_note(&NoteEntry {
                    id: id.clone(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(folder.id.clone()),
                    title: "A note".to_string(),
                    body: (*body).to_string(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: "2026-01-01".to_string(),
                    updated_at: "2026-01-01".to_string(),
                })
                .unwrap();

            let loaded = cache
                .get_note(&id)
                .unwrap()
                .expect("the note that was just saved");
            assert_eq!(
                loaded.body, *body,
                "a body with {aimed_at} did not come back as it went in"
            );
        }
    }

    #[test]
    fn test_a_note_with_nothing_in_it_comes_back_with_nothing_in_it() {
        // An empty body is the case a NOT NULL column with a default invites
        // somebody to fill in, and a note that quietly acquires words nobody
        // typed is worse than one that is empty.
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "empty".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Nothing yet".to_string(),
                body: String::new(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
            })
            .unwrap();

        let loaded = cache.get_note("empty").unwrap().expect("the empty note");
        assert_eq!(loaded.body, "", "an empty note came back holding something");
    }

    /// The one answer this program writes is one word in one place.
    ///
    /// It can only assert a constant, and that is the honest shape of it: the
    /// column has held the same word on every note ever written here, so there
    /// is no information in it to test. What the assertion is really for is a
    /// rename. Change the word and every note already on somebody's disk stops
    /// being recognised by the build that reads it next, which is a schema
    /// change wearing the clothes of a tidy-up.
    #[test]
    fn test_what_this_program_says_a_notes_body_is_written_in_is_one_word() {
        assert_eq!(
            NoteBody::AsTyped.as_stored(),
            "plain",
            "the word shipped rows hold was changed, which orphans every one of them"
        );
        assert_eq!(NoteBody::from_stored("plain"), NoteBody::AsTyped);
        assert_eq!(
            NoteBody::Other("text/html".to_string()).as_stored(),
            "text/html",
            "a word from somewhere else was not written back as it arrived"
        );
    }

    /// A note whose stored form says nothing at all is still read.
    ///
    /// The column is `format TEXT DEFAULT 'plain'` and carries no NOT NULL, so
    /// a row that arrived without one holds a null rather than a word. Every
    /// note this program has ever written has a word in it, which means the
    /// only way to meet a null is a row from somewhere else: an older build, a
    /// later one, or a sync that names a subset of the columns.
    ///
    /// Refusing the note is the wrong answer to that. It is somebody's note,
    /// and what is missing from the row is a word nothing consults.
    #[test]
    fn test_a_note_whose_stored_form_says_nothing_is_still_read() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "From somewhere else".to_string(),
                body: "Worth keeping".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
            })
            .unwrap();
        cache
            .conn
            .execute(
                "UPDATE notes SET format = NULL WHERE id = ?1",
                rusqlite::params!["n1"],
            )
            .unwrap();

        let loaded = cache
            .get_note("n1")
            .expect("a note was refused because a word nothing reads was missing from its row")
            .expect("the note that was just saved");
        assert_eq!(
            loaded.format,
            NoteBody::AsTyped,
            "a row with nothing in the column did not read as what the schema defaults to"
        );
    }

    /// A word this build did not write survives being read and written back.
    ///
    /// `AddressBook::Other` is this project's worked example and its doc
    /// comment says why: a word this code does not recognise is still somebody
    /// else's answer, and forgetting it rewrites their row on the next save
    /// without anybody asking for that.
    #[test]
    fn test_a_stored_form_this_build_did_not_write_survives_being_written_back() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "From a later build".to_string(),
                body: "Worth keeping".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
            })
            .unwrap();
        cache
            .conn
            .execute(
                "UPDATE notes SET format = 'text/html' WHERE id = ?1",
                rusqlite::params!["n1"],
            )
            .unwrap();

        let read_back = cache.get_note("n1").unwrap().expect("the note");
        cache.save_note(&read_back).unwrap();

        let still_there: String = cache
            .conn
            .query_row(
                "SELECT format FROM notes WHERE id = ?1",
                rusqlite::params!["n1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            still_there, "text/html",
            "a word this build does not recognise was lost by reading the note and saving it again"
        );
    }

    /// What was stored is read for whatever structure is in it, both ways.
    ///
    /// This is the promise the note editor's own accessible description makes,
    /// asserted from the far side of storage rather than from a formatter's
    /// own tests: a note with headings and lists in it is spoken as headings
    /// and lists, and a note with none is spoken as it was written with
    /// nothing added.
    #[test]
    fn test_a_stored_body_is_read_for_whatever_structure_is_in_it() {
        use crate::application::long_text;

        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        for (id, body) in [
            ("structured", "# Shopping\n\n- milk\n- eggs\n"),
            ("flat", "Ring the plumber back on Tuesday."),
        ] {
            cache
                .save_note(&NoteEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(folder.id.clone()),
                    title: "A note".to_string(),
                    body: body.to_string(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: "2026-01-01".to_string(),
                    updated_at: "2026-01-01".to_string(),
                })
                .unwrap();
        }

        let structured = cache.get_note("structured").unwrap().expect("the note");
        assert_eq!(
            long_text::spoken(&structured.body),
            "heading level 1, Shopping\nbullet, milk\nbullet, eggs",
            "the structure somebody typed did not survive storage"
        );

        let flat = cache.get_note("flat").unwrap().expect("the note");
        assert_eq!(
            long_text::spoken(&flat.body),
            "Ring the plumber back on Tuesday.",
            "a note with no structure in it was given some"
        );
    }

    #[test]
    fn test_saving_a_note_twice_updates_rather_than_duplicates() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        let mut note = NoteEntry {
            id: "n1".into(),
            account_id: "acct-1".into(),
            folder_id: Some(folder.id.clone()),
            title: "Draft".into(),
            body: "first".into(),
            format: NoteBody::AsTyped,
            pinned: false,
            created_at: "2026-01-01".into(),
            updated_at: "2026-01-01".into(),
        };
        cache.save_note(&note).unwrap();
        note.body = "edited".into();
        note.title = "Final".into();
        cache.save_note(&note).unwrap();

        assert_eq!(cache.get_all_notes_for_account("acct-1").unwrap().len(), 1);
        let loaded = cache.get_note("n1").unwrap().unwrap();
        assert_eq!(loaded.body, "edited");
        assert_eq!(loaded.title, "Final");
    }
}
