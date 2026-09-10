//! Note and NoteFolder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{
    DeletedNote, MessageCache, NoteEntry, NoteFolderEntry, TheDeletionSoFar,
};

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

/// The columns [`MessageCache::map_note_row`] reads, in the order it reads them.
///
/// One list rather than a copy in each query. Five queries read a note, and a
/// column added to the reader and missed in one of them is a note read with
/// another column's value in it. SQL is a string here, so nothing but this
/// would say so.
const NOTE_COLUMNS: &str = "id, account_id, folder_id, title, body, format, pinned, \
                            created_at, updated_at, pending, provider_note_id, \
                            provider_version";

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
                    created_at, updated_at, pending, provider_note_id,
                    provider_version
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(id) DO UPDATE SET
                    folder_id = excluded.folder_id,
                    title = excluded.title,
                    body = excluded.body,
                    format = excluded.format,
                    pinned = excluded.pinned,
                    updated_at = excluded.updated_at,
                    pending = excluded.pending,
                    provider_note_id = excluded.provider_note_id,
                    provider_version = excluded.provider_version",
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
                    n.pending,
                    n.known_as,
                    n.known_version,
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to save note: {}", e)))?;
        Ok(())
    }

    /// Get all notes for a folder.
    pub fn get_notes_for_folder(&self, folder_id: &str) -> Result<Vec<NoteEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT {NOTE_COLUMNS}
                     FROM notes WHERE folder_id = ?1
                     ORDER BY pinned DESC, updated_at DESC"
            ))
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
            .prepare_cached(&format!(
                "SELECT {NOTE_COLUMNS}
                     FROM notes WHERE account_id = ?1
                     ORDER BY pinned DESC, updated_at DESC"
            ))
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
            .prepare_cached(&format!("SELECT {NOTE_COLUMNS} FROM notes WHERE id = ?1"))
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
    ///
    /// Leaves a record of the deletion, because a deleted row cannot carry a
    /// "not yet sent" flag and a deletion the backend is never told about is a
    /// note that comes back on the next read. [`crate::application::deletions`]
    /// states the rule; this follows it rather than restating it.
    pub fn delete_note(&self, note_id: &str) -> Result<()> {
        // Read before the row goes, because afterwards there is nothing left
        // to say what the backend called it, and that is what a removal names.
        if let Ok(Some(note)) = self.get_note(note_id) {
            self.record_a_deleted_note(&note)?;
        }
        self.conn
            .execute(
                "DELETE FROM notes WHERE id = ?1",
                rusqlite::params![note_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete note: {}", e)))?;
        Ok(())
    }

    /// Write down that this note was deleted here.
    fn record_a_deleted_note(&self, note: &NoteEntry) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO deleted_notes
                    (id, account_id, folder_id, provider_note_id, deleted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    note.id,
                    note.account_id,
                    note.folder_id,
                    note.known_as,
                    chrono::Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to record a deleted note: {}", e)))?;
        Ok(())
    }

    /// Every note this computer deleted, whether the backend has been told or
    /// not.
    ///
    /// Both, because two questions are asked of this list. The push asks what
    /// it still has to send and reads [`DeletedNote::so_far`] to find it; the
    /// read asks what this computer deleted, which a record the backend has
    /// already taken answers just as much as one still owed. Those two used to
    /// be one question everywhere in this program and stopped being the same
    /// answer at the worst possible moment.
    pub fn deleted_notes(&self, account_id: &str) -> Result<Vec<DeletedNote>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, folder_id, provider_note_id, deleted_at, taken_at
                 FROM deleted_notes WHERE account_id = ?1 ORDER BY deleted_at",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare the deletions query: {}", e)))?;
        let rows = stmt
            .query_map(rusqlite::params![account_id], |row| {
                Ok(DeletedNote {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    folder_id: row.get(2)?,
                    known_as: row.get(3)?,
                    deleted_at: row.get(4)?,
                    so_far: TheDeletionSoFar::from_stored(row.get(5)?),
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query the deletions: {}", e)))?;
        let mut gone = Vec::new();
        for row in rows {
            gone.push(row.map_err(|e| Error::Other(format!("Failed to read a deletion: {}", e)))?);
        }
        Ok(gone)
    }

    /// The backend has taken this deletion.
    ///
    /// The record stays. It stops being work the push has and becomes the only
    /// thing standing between the note and a read that is still naming it:
    /// dropping it here is what let a thing somebody deleted come back in the
    /// very sync that deleted it, three times in this program before the rule
    /// was written down. `let_go_of_deletions_taken_before` releases it later.
    ///
    /// The moment comes from the caller, written by `deletions::written`, so
    /// that the stamp on a record and the cutoff it is compared against are
    /// written the same way.
    pub fn a_backend_took_the_deletion_of_a_note(
        &self,
        note_id: &str,
        taken_at: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE deleted_notes SET taken_at = ?2 WHERE id = ?1",
                rusqlite::params![note_id, taken_at],
            )
            .map_err(|e| Error::Other(format!("Failed to record a deletion as taken: {}", e)))?;
        Ok(())
    }

    /// Toggle pin status of a note.
    ///
    /// Leaves the note waiting to be sent, and that half is the one worth
    /// reading. This writes its own `UPDATE` and does not go through
    /// [`Self::save_note`], so an invariant kept only there is bypassed by it,
    /// silently and for ever. Pinning is a change somebody made and would
    /// expect their backend to be told about, and the shape of the bug is
    /// already in this project's changelog: a move changed a row, nothing
    /// marked it as waiting, nothing ever pushed it, and the status line said
    /// the change had been made.
    pub fn toggle_note_pin(&self, note_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE notes SET pinned = NOT pinned, updated_at = ?1, pending = 1
                 WHERE id = ?2",
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
            .prepare_cached(&format!(
                "SELECT {NOTE_COLUMNS}
                     FROM notes WHERE account_id = ?1
                       AND (title LIKE ?2 ESCAPE '!' OR body LIKE ?2 ESCAPE '!')
                     ORDER BY pinned DESC, updated_at DESC"
            ))
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
            pending: row.get(9)?,
            known_as: row.get(10)?,
            known_version: row.get(11)?,
        })
    }

    /// One note folder, or nothing.
    ///
    /// By its own identifier rather than by walking an account's folders,
    /// because the two callers that need it have a folder in hand and not an
    /// account: a filing says which folder it is going into, and the sync is
    /// told which folder it is syncing.
    pub fn get_note_folder(&self, folder_id: &str) -> Result<Option<NoteFolderEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, display_order, created_at
                 FROM note_folders WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare a note folder query: {}", e)))?;
        let mut rows = stmt
            .query_map(rusqlite::params![folder_id], |row| {
                Ok(NoteFolderEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    display_order: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query a note folder: {}", e)))?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| {
                Error::Other(format!("Failed to read a note folder row: {}", e))
            })?)),
            None => Ok(None),
        }
    }

    /// Every note changed here and not yet sent.
    ///
    /// The same question `pending_tasks` answers for a task, asked the same
    /// way, so a note the push has to offer is found by one column rather than
    /// by comparing what is here against what a backend last said.
    pub fn notes_waiting_to_be_sent(&self, account_id: &str) -> Result<Vec<NoteEntry>> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT {NOTE_COLUMNS}
                 FROM notes WHERE account_id = ?1 AND pending = 1
                 ORDER BY updated_at"
            ))
            .map_err(|e| {
                Error::Other(format!("Failed to prepare the waiting notes query: {}", e))
            })?;
        let rows = stmt
            .query_map(rusqlite::params![account_id], Self::map_note_row)
            .map_err(|e| Error::Other(format!("Failed to query the waiting notes: {}", e)))?;
        let mut notes = Vec::new();
        for row in rows {
            notes.push(row.map_err(|e| Error::Other(format!("Failed to read a note: {}", e)))?);
        }
        Ok(notes)
    }

    /// The backend has taken this note, and calls it this.
    ///
    /// Written as one statement rather than by reading the note, changing three
    /// fields and saving it again. The read-change-save shape would carry the
    /// title and body that were in hand when the push started back over
    /// whatever somebody typed while it was in flight, which is a change made
    /// and lost with nothing said.
    pub fn a_backend_took_the_note(
        &self,
        note_id: &str,
        named: &str,
        version: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE notes
                 SET pending = 0, provider_note_id = ?2, provider_version = ?3
                 WHERE id = ?1",
                rusqlite::params![note_id, named, version],
            )
            .map_err(|e| Error::Other(format!("Failed to record a note as sent: {}", e)))?;
        Ok(())
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
                    pending: false,
                    known_as: None,
                    known_version: None,
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
            pending: false,
            known_as: None,
            known_version: None,
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
                    pending: false,
                    known_as: None,
                    known_version: None,
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
                pending: false,
                known_as: None,
                known_version: None,
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
                    pending: false,
                    known_as: None,
                    known_version: None,
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
                pending: false,
                known_as: None,
                known_version: None,
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
                pending: false,
                known_as: None,
                known_version: None,
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
                pending: false,
                known_as: None,
                known_version: None,
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
                    pending: false,
                    known_as: None,
                    known_version: None,
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

    /// Whether a note is waiting to be sent survives storage.
    ///
    /// The column is what a push reads to find its work, so a flag that is
    /// written and not read back is a change that never leaves, said as a
    /// success.
    #[test]
    fn test_whether_a_note_is_waiting_to_be_sent_comes_back_as_it_went_in() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        let mut note = NoteEntry {
            id: "n1".to_string(),
            account_id: "acct-1".to_string(),
            folder_id: Some(folder.id.clone()),
            title: "Wiring colours".to_string(),
            body: "Brown is live".to_string(),
            format: NoteBody::AsTyped,
            pinned: false,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            pending: true,
            known_as: Some("there-1".to_string()),
            known_version: Some("v1".to_string()),
        };
        cache.save_note(&note).unwrap();

        let loaded = cache.get_note("n1").unwrap().expect("the note");
        assert!(
            loaded.pending,
            "a note waiting to be sent came back settled"
        );
        assert_eq!(loaded.known_as.as_deref(), Some("there-1"));
        assert_eq!(loaded.known_version.as_deref(), Some("v1"));

        note.pending = false;
        cache.save_note(&note).unwrap();
        let settled = cache.get_note("n1").unwrap().expect("the note");
        assert!(
            !settled.pending,
            "a note that stopped waiting was written back as still waiting"
        );
    }

    /// Only the notes with something to send are offered to a backend.
    #[test]
    fn test_the_notes_offered_to_a_backend_are_the_ones_with_something_to_send() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        for (id, waiting) in [("waiting", true), ("settled", false)] {
            cache
                .save_note(&NoteEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(folder.id.clone()),
                    title: id.to_string(),
                    body: String::new(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: "2026-01-01".to_string(),
                    updated_at: "2026-01-01".to_string(),
                    pending: waiting,
                    known_as: None,
                    known_version: None,
                })
                .unwrap();
        }

        let waiting = cache.notes_waiting_to_be_sent("acct-1").unwrap();

        assert_eq!(
            waiting.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
            ["waiting"],
            "the push was handed the wrong set of notes"
        );
    }

    /// A note the backend took stops waiting and remembers what it is called.
    #[test]
    fn test_a_note_a_backend_took_stops_waiting_and_keeps_what_it_is_called_there() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Wiring colours".to_string(),
                body: "Brown is live".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
                pending: true,
                known_as: None,
                known_version: None,
            })
            .unwrap();

        cache
            .a_backend_took_the_note("n1", "there-1", Some("v1"))
            .unwrap();

        let after = cache.get_note("n1").unwrap().expect("the note");
        assert!(!after.pending);
        assert_eq!(after.known_as.as_deref(), Some("there-1"));
        assert_eq!(after.known_version.as_deref(), Some("v1"));
    }

    /// Pinning a note marks it as waiting to be sent.
    ///
    /// [`MessageCache::toggle_note_pin`] writes its own `UPDATE` and does not
    /// go through [`MessageCache::save_note`], so an invariant kept only in
    /// `save_note` is bypassed by it. Pinning is a change somebody made and
    /// would expect to reach their backend, and the shape of the bug this
    /// avoids is already in the changelog: a move changed a row, nothing
    /// marked it, nothing ever pushed it, and the status line said the change
    /// had been made.
    #[test]
    fn test_pinning_a_note_leaves_it_waiting_to_be_sent() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Wiring colours".to_string(),
                body: "Brown is live".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
                pending: false,
                known_as: Some("there-1".to_string()),
                known_version: Some("v1".to_string()),
            })
            .unwrap();

        cache.toggle_note_pin("n1").unwrap();

        let after = cache.get_note("n1").unwrap().expect("the note");
        assert!(after.pinned);
        assert!(
            after.pending,
            "pinning changed the note and left nothing saying so, so the pin \
             reaches the backend on some later change or never"
        );
    }

    /// Deleting a note leaves a record naming what the backend called it.
    #[test]
    fn test_deleting_a_note_leaves_a_record_of_the_deletion() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Going".to_string(),
                body: "Gone".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
                pending: false,
                known_as: Some("there-1".to_string()),
                known_version: Some("v1".to_string()),
            })
            .unwrap();

        cache.delete_note("n1").unwrap();

        let gone = cache.deleted_notes("acct-1").unwrap();
        assert_eq!(gone.len(), 1, "{gone:?}");
        assert_eq!(gone[0].id, "n1");
        assert_eq!(
            gone[0].known_as.as_deref(),
            Some("there-1"),
            "the record does not name what the backend calls the note, so \
             nothing can ask for it to be removed"
        );
        assert!(
            gone[0].so_far.still_owed(),
            "the deletion reads as already taken, so the backend is never told"
        );
    }

    /// Deleting a folder leaves a record for every note it held.
    ///
    /// The folder delete removes them all in one statement, so a rule applied
    /// one note at a time is bypassed by it. Without this, deleting a folder
    /// full of synced notes brings every one of them back on the next read.
    #[test]
    fn test_deleting_a_note_folder_leaves_a_record_for_every_note_in_it() {
        let cache = test_cache();
        let going = NoteFolderEntry {
            id: "folder-going".to_string(),
            account_id: "acct-1".to_string(),
            name: "Old ideas".to_string(),
            display_order: 1,
            created_at: "2026-01-01".to_string(),
        };
        cache.save_note_folder(&going).unwrap();
        for (id, there) in [("n1", "there-1"), ("n2", "there-2")] {
            cache
                .save_note(&NoteEntry {
                    id: id.to_string(),
                    account_id: "acct-1".to_string(),
                    folder_id: Some(going.id.clone()),
                    title: id.to_string(),
                    body: String::new(),
                    format: NoteBody::AsTyped,
                    pinned: false,
                    created_at: "2026-01-01".to_string(),
                    updated_at: "2026-01-01".to_string(),
                    pending: false,
                    known_as: Some(there.to_string()),
                    known_version: None,
                })
                .unwrap();
        }

        cache.delete_note_folder(&going.id).unwrap();

        let mut gone: Vec<String> = cache
            .deleted_notes("acct-1")
            .unwrap()
            .into_iter()
            .filter_map(|record| record.known_as)
            .collect();
        gone.sort();
        assert_eq!(
            gone,
            ["there-1", "there-2"],
            "a folder deleted whole left some of its notes with nothing saying \
             they were deleted, so they come back on the next read"
        );
    }

    /// A deletion the backend has taken is remembered rather than dropped.
    #[test]
    fn test_a_deletion_has_two_lives_and_the_second_is_a_memory() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Going".to_string(),
                body: "Gone".to_string(),
                format: NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-01-01".to_string(),
                updated_at: "2026-01-01".to_string(),
                pending: false,
                known_as: Some("there-1".to_string()),
                known_version: None,
            })
            .unwrap();
        cache.delete_note("n1").unwrap();

        let now = chrono::Utc::now();
        cache
            .a_backend_took_the_deletion_of_a_note(
                "n1",
                &crate::application::deletions::written(now),
            )
            .unwrap();

        let taken = &cache.deleted_notes("acct-1").unwrap()[0];
        assert!(
            !taken.so_far.still_owed(),
            "the push still owes a deletion the backend has already taken"
        );

        // And the clock, and nothing else, is what lets it go.
        crate::application::deletions::let_go_of_what_was_remembered_long_enough(&cache, now)
            .unwrap();
        assert_eq!(
            cache.deleted_notes("acct-1").unwrap().len(),
            1,
            "a memory was let go of the moment the backend took it, so a read \
             still naming the note writes it straight back down"
        );

        let long_after = now
            + crate::application::deletions::HOW_LONG_A_DELETION_IS_REMEMBERED
            + chrono::Duration::days(1);
        crate::application::deletions::let_go_of_what_was_remembered_long_enough(
            &cache, long_after,
        )
        .unwrap();
        assert!(
            cache.deleted_notes("acct-1").unwrap().is_empty(),
            "nothing lets a memory go, so the table grows for ever"
        );
    }

    #[test]
    fn test_one_note_folder_can_be_found_by_its_own_name() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();

        let found = cache
            .get_note_folder(&folder.id)
            .unwrap()
            .expect("the folder that was just made");
        assert_eq!(found.account_id, "acct-1");
        assert!(cache.get_note_folder("not-a-folder").unwrap().is_none());
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
            pending: false,
            known_as: None,
            known_version: None,
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
