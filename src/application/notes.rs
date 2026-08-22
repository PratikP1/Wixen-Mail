//! Note manager: business logic for notes and note folders.
//!
//! Provides an in-memory note store with filtering by folder, pin status,
//! and search. Notes are plain-text or markdown.
//!
//! # Nothing in the running program reaches this
//!
//! The notes screen reads the cache itself. `load_pim_module` in
//! `presentation::wx_app` calls `ensure_default_note_folder`,
//! `get_all_notes_for_account` and `get_note_folders_for_account`, and counts
//! the notes in each folder where it builds the rows. The screen's own file
//! builds widgets and holds no data at all.
//!
//! That is checked, not assumed: gating this module behind `cfg(test)` still
//! compiles the library and every binary. Anything below is exercised by its
//! own tests and by nothing else, so a test added here would say what the
//! program does only once somebody wires it up.
//!
//! Everything here already exists in SQL that runs. `sort_notes` is the
//! stored order, pinned first and then most recently changed, key for key.

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
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_note_manager_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    #[test]
    fn test_load_notes_reads_the_folders_and_notes_the_cache_holds() {
        let cache = test_cache();
        let folder = cache.ensure_default_note_folder("acct-1").unwrap();
        cache
            .save_note(&NoteEntry {
                id: "n1".to_string(),
                account_id: "acct-1".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "From the cache".to_string(),
                body: "Loaded rather than typed in".to_string(),
                format: "plain".to_string(),
                pinned: false,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            })
            .unwrap();

        let mut mgr = NoteManager::default();
        mgr.load_notes(&cache, "acct-1").unwrap();

        assert_eq!(mgr.all_folders().len(), 1);
        assert_eq!(mgr.all_folders()[0].id, folder.id);
        assert_eq!(mgr.all_notes().len(), 1);
        assert_eq!(mgr.all_notes()[0].title, "From the cache");
    }

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
    fn test_remove_note_takes_only_the_one_asked_for() {
        let mut mgr = NoteManager::default();
        mgr.add_note(make_note("n1", "Keep", "body", "f1", false));
        mgr.add_note(make_note("n2", "Gone", "body", "f1", false));

        mgr.remove_note("n2");

        let ids: Vec<&str> = mgr.all_notes().iter().map(|n| n.id.as_str()).collect();
        assert_eq!(
            ids,
            ["n1"],
            "removed the wrong note, or none, or all of them"
        );
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
    fn test_all_folders_reflects_what_was_added() {
        // The existing cascade test only ever checks the count after a
        // removal, which a folder store that silently drops every add would
        // also report as zero. This checks the content survives the round
        // trip, not just that a count changed.
        let mut mgr = NoteManager::default();
        mgr.add_folder(make_folder("f1", "Personal"));

        assert_eq!(mgr.all_folders().len(), 1);
        assert_eq!(mgr.all_folders()[0].id, "f1");
        assert_eq!(mgr.all_folders()[0].name, "Personal");
    }

    #[test]
    fn test_adding_a_note_sorts_pinned_first_then_most_recently_changed() {
        // Insertion order is deliberately the opposite of the expected order,
        // so a sort that never ran would leave this exact sequence in place
        // instead of producing it.
        let mut older_unpinned = make_note("n1", "Older unpinned", "body", "f1", false);
        older_unpinned.updated_at = "2026-01-01T00:00:00Z".to_string();
        let mut newer_unpinned = make_note("n2", "Newer unpinned", "body", "f1", false);
        newer_unpinned.updated_at = "2026-01-03T00:00:00Z".to_string();
        let mut pinned = make_note("n3", "Pinned", "body", "f1", true);
        pinned.updated_at = "2026-01-02T00:00:00Z".to_string();

        let mut mgr = NoteManager::default();
        mgr.add_note(older_unpinned);
        mgr.add_note(newer_unpinned);
        mgr.add_note(pinned);

        let ids: Vec<&str> = mgr.all_notes().iter().map(|n| n.id.as_str()).collect();
        assert_eq!(
            ids,
            ["n3", "n2", "n1"],
            "pinned should lead, then most recently changed first"
        );
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
