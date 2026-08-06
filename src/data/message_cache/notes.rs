//! Note and NoteFolder CRUD operations.

use crate::common::{Error, Result};
use crate::data::message_cache::{MessageCache, NoteEntry, NoteFolderEntry};

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
            .prepare(
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
                    n.format,
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
            .prepare(
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
            .prepare(
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
            .prepare(
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
            .prepare(
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
            format: row.get(5)?,
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
                    format: "plain".to_string(),
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
            format: "plain".to_string(),
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
                    format: "plain".to_string(),
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
                format: "plain".into(),
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
            format: "plain".into(),
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
