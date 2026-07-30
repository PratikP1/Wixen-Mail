//! Note manager: business logic for notes and note folders.
//!
//! Provides an in-memory note store with filtering by folder, pin status,
//! and search. Notes are plain-text or markdown.

use crate::common::Result;
use crate::data::message_cache::{MessageCache, NoteEntry, NoteFolderEntry};

/// In-memory note store with queries.
#[derive(Default)]
pub struct NoteManager {
    folders: Vec<NoteFolderEntry>,
    notes: Vec<NoteEntry>,
}

impl NoteManager {
    /// Load note folders and all notes from the cache for a given account.
    pub fn load_notes(&mut self, cache: &MessageCache, account_id: &str) -> Result<()> {
        self.folders = cache.get_note_folders_for_account(account_id)?;
        self.notes = cache.get_all_notes_for_account(account_id)?;
        Ok(())
    }

    /// All note folders.
    pub fn all_folders(&self) -> &[NoteFolderEntry] {
        &self.folders
    }

    /// All loaded notes.
    pub fn all_notes(&self) -> &[NoteEntry] {
        &self.notes
    }

    /// Notes in a specific folder.
    pub fn notes_for_folder(&self, folder_id: &str) -> Vec<&NoteEntry> {
        self.notes
            .iter()
            .filter(|n| n.folder_id.as_deref() == Some(folder_id))
            .collect()
    }

    /// Pinned notes.
    pub fn pinned_notes(&self) -> Vec<&NoteEntry> {
        self.notes.iter().filter(|n| n.pinned).collect()
    }

    /// Search notes by title or body content (case-insensitive).
    pub fn search_notes(&self, query: &str) -> Vec<&NoteEntry> {
        let q = query.to_lowercase();
        self.notes
            .iter()
            .filter(|n| n.title.to_lowercase().contains(&q) || n.body.to_lowercase().contains(&q))
            .collect()
    }

    /// Add a folder to the in-memory store.
    pub fn add_folder(&mut self, folder: NoteFolderEntry) {
        self.folders.push(folder);
    }

    /// Remove a folder and its notes from the in-memory store.
    pub fn remove_folder(&mut self, folder_id: &str) {
        self.folders.retain(|f| f.id != folder_id);
        self.notes
            .retain(|n| n.folder_id.as_deref() != Some(folder_id));
    }

    /// Add a note to the in-memory store.
    pub fn add_note(&mut self, note: NoteEntry) {
        self.notes.push(note);
        self.sort_notes();
    }

    /// Remove a note by ID.
    pub fn remove_note(&mut self, note_id: &str) {
        self.notes.retain(|n| n.id != note_id);
    }

    /// Toggle pin status of a note in-memory.
    pub fn toggle_pin(&mut self, note_id: &str) {
        if let Some(n) = self.notes.iter_mut().find(|n| n.id == note_id) {
            n.pinned = !n.pinned;
        }
    }

    /// Update a note's title and body in-memory.
    pub fn update_note(&mut self, note_id: &str, title: &str, body: &str) {
        if let Some(n) = self.notes.iter_mut().find(|n| n.id == note_id) {
            n.title = title.to_string();
            n.body = body.to_string();
            n.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Get a note by ID.
    pub fn get_note(&self, note_id: &str) -> Option<&NoteEntry> {
        self.notes.iter().find(|n| n.id == note_id)
    }

    /// Sort notes: pinned first, then by updated_at descending.
    fn sort_notes(&mut self) {
        self.notes.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_folder(id: &str, name: &str) -> NoteFolderEntry {
        NoteFolderEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            name: name.to_string(),
            display_order: 0,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn make_note(id: &str, title: &str, body: &str, folder: &str, pinned: bool) -> NoteEntry {
        NoteEntry {
            id: id.to_string(),
            account_id: "test".to_string(),
            folder_id: Some(folder.to_string()),
            title: title.to_string(),
            body: body.to_string(),
            format: "plain".to_string(),
            pinned,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_note_manager_filtering() {
        let mut mgr = NoteManager::default();
        mgr.add_folder(make_folder("f1", "Personal"));
        mgr.add_note(make_note(
            "n1",
            "Grocery list",
            "eggs, milk, bread",
            "f1",
            true,
        ));
        mgr.add_note(make_note(
            "n2",
            "Meeting notes",
            "Discussed roadmap",
            "f1",
            false,
        ));

        assert_eq!(mgr.all_notes().len(), 2);
        assert_eq!(mgr.notes_for_folder("f1").len(), 2);
        assert_eq!(mgr.pinned_notes().len(), 1);
        assert_eq!(mgr.pinned_notes()[0].title, "Grocery list");
    }

    #[test]
    fn test_search_notes() {
        let mut mgr = NoteManager::default();
        mgr.add_note(make_note(
            "n1",
            "Grocery list",
            "eggs, milk, bread",
            "f1",
            false,
        ));
        mgr.add_note(make_note(
            "n2",
            "Meeting notes",
            "Discussed roadmap",
            "f1",
            false,
        ));

        assert_eq!(mgr.search_notes("grocery").len(), 1);
        assert_eq!(mgr.search_notes("roadmap").len(), 1);
        assert_eq!(mgr.search_notes("nothing").len(), 0);
    }

    #[test]
    fn test_toggle_pin() {
        let mut mgr = NoteManager::default();
        mgr.add_note(make_note("n1", "Test", "body", "f1", false));

        assert!(!mgr.get_note("n1").unwrap().pinned);
        mgr.toggle_pin("n1");
        assert!(mgr.get_note("n1").unwrap().pinned);
    }

    #[test]
    fn test_remove_folder_cascades() {
        let mut mgr = NoteManager::default();
        mgr.add_folder(make_folder("f1", "Personal"));
        mgr.add_note(make_note("n1", "Note 1", "body", "f1", false));
        mgr.add_note(make_note("n2", "Note 2", "body", "f1", false));

        assert_eq!(mgr.all_notes().len(), 2);
        mgr.remove_folder("f1");
        assert_eq!(mgr.all_notes().len(), 0);
        assert_eq!(mgr.all_folders().len(), 0);
    }

    #[test]
    fn test_update_note_title_and_body() {
        let mut mgr = NoteManager::default();
        mgr.add_note(make_note("n1", "Original", "old body", "f1", false));

        mgr.update_note("n1", "Updated Title", "new body");
        let note = mgr.get_note("n1").unwrap();
        assert_eq!(note.title, "Updated Title");
        assert_eq!(note.body, "new body");
    }

    #[test]
    fn test_update_nonexistent_note_is_noop() {
        let mut mgr = NoteManager::default();
        mgr.update_note("ghost", "Title", "Body"); // should not panic
        assert_eq!(mgr.all_notes().len(), 0);
    }
}
