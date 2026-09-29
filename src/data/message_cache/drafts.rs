//! Draft persistence operations

use super::{CachedDraft, MessageCache};
use crate::application::protecting::Choice;
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, Row, params};

/// Every column a draft is read from, for both of the reads a draft has.
const DRAFT_COLUMNS: &str = "id, account_id, to_addr, cc, bcc, subject, body, created_at, \
     updated_at, in_reply_to, references_header, body_html, attachments, protection, \
     from_address, from_name";

/// One draft from a row selected with [`DRAFT_COLUMNS`].
///
/// Read by column name, so a column added to the list cannot move another,
/// and the listing and the opening of a draft cannot come to disagree.
fn a_draft(row: &Row<'_>) -> rusqlite::Result<CachedDraft> {
    Ok(CachedDraft {
        id: row.get("id")?,
        account_id: row.get("account_id")?,
        to_addr: row.get("to_addr")?,
        cc: row.get("cc")?,
        bcc: row.get("bcc")?,
        subject: row.get("subject")?,
        body: row.get("body")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        in_reply_to: row.get("in_reply_to")?,
        references: row.get("references_header")?,
        body_html: row.get("body_html")?,
        attachments: crate::application::attaching::split(&row.get::<_, String>("attachments")?),
        protection: Choice::from_stored(row.get::<_, Option<String>>("protection")?.as_deref()),
        from_address: row.get("from_address")?,
        from_name: row.get("from_name")?,
    })
}

impl MessageCache {
    /// Save a draft to cache
    pub fn save_draft(&self, draft: &CachedDraft) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT OR REPLACE INTO drafts (id, account_id, to_addr, cc, bcc, subject, body, created_at, updated_at, in_reply_to, references_header, body_html, attachments, protection, from_address, from_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
                     COALESCE((SELECT created_at FROM drafts WHERE id = ?1), ?8), ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                draft.id,
                draft.account_id,
                draft.to_addr,
                draft.cc,
                draft.bcc,
                draft.subject,
                draft.body,
                draft.created_at.clone(),
                now,
                draft.in_reply_to,
                draft.references,
                draft.body_html,
                crate::application::attaching::joined(&draft.attachments),
                draft.protection.as_stored(),
                draft.from_address,
                draft.from_name,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save draft: {}", e)))?;

        Ok(())
    }

    /// Load all drafts for an account
    pub fn load_drafts(&self, account_id: &str) -> Result<Vec<CachedDraft>> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT {DRAFT_COLUMNS}
                     FROM drafts
                     WHERE account_id = ?1
                     ORDER BY updated_at DESC"
            ))
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let drafts = stmt
            .query_map(params![account_id], a_draft)
            .map_err(|e| Error::Other(format!("Failed to query drafts: {}", e)))?;

        let mut result = Vec::new();
        for draft in drafts {
            result.push(draft.map_err(|e| Error::Other(format!("Failed to read draft: {}", e)))?);
        }

        Ok(result)
    }

    /// Load a specific draft by ID
    pub fn load_draft(&self, draft_id: &str) -> Result<Option<CachedDraft>> {
        let result = self
            .conn
            .query_row(
                &format!("SELECT {DRAFT_COLUMNS} FROM drafts WHERE id = ?1"),
                params![draft_id],
                a_draft,
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to load draft: {}", e)))?;

        Ok(result)
    }

    /// Delete a draft
    pub fn delete_draft(&self, draft_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM drafts WHERE id = ?1", params![draft_id])
            .map_err(|e| Error::Other(format!("Failed to delete draft: {}", e)))?;

        Ok(())
    }

    /// Clear all drafts for an account
    pub fn clear_drafts(&self, account_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM drafts WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| Error::Other(format!("Failed to clear drafts: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    #[test]
    fn test_the_files_and_the_formatting_come_back_with_the_draft() {
        // Save Draft threw the files away without a word: it announced that
        // the draft was saved, and reopening it showed nothing attached and
        // sent without them. The formatted half had nowhere to live either,
        // so the editor's markup went into the plain-text body and the filed
        // copy declared tags as though somebody had typed them.
        let cache = a_cache("draft_files_and_formatting");
        let mut draft = CachedDraft {
            id: "draft-with-things".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "ada@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Notes".to_string(),
            body: "Hi Ada".to_string(),
            body_html: Some("<div>Hi Ada</div>".to_string()),
            attachments: vec![
                std::path::PathBuf::from("C:/notes/one.pdf"),
                std::path::PathBuf::from("C:/notes/two.png"),
            ],
            in_reply_to: None,
            references: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            protection: Choice::Plain,
            from_address: None,
            from_name: None,
        };
        cache.save_draft(&draft).expect("the draft to save");

        let back = cache
            .load_drafts("acc-1")
            .expect("the drafts to load")
            .into_iter()
            .find(|d| d.id == draft.id)
            .expect("the draft is there");

        assert_eq!(
            back.attachments, draft.attachments,
            "the files did not come back with the draft"
        );
        assert_eq!(
            back.body_html.as_deref(),
            Some("<div>Hi Ada</div>"),
            "the formatted half did not come back"
        );
        assert_eq!(back.body, "Hi Ada", "the plain half is not the plain text");

        // And taking them off is kept too, rather than the older values
        // surviving because nothing was written over them.
        draft.attachments.clear();
        draft.body_html = None;
        cache.save_draft(&draft).expect("the draft to save again");
        let back = cache
            .load_drafts("acc-1")
            .expect("the drafts to load")
            .into_iter()
            .find(|d| d.id == draft.id)
            .expect("the draft is still there");
        assert!(back.attachments.is_empty(), "a removed file came back");
        assert_eq!(back.body_html, None, "removed formatting came back");
    }

    #[test]
    fn test_a_picture_and_its_description_come_back_out_of_a_real_database() {
        // The storage stage of the trip a picture takes between being inserted
        // and being seen again. The other stages are unit-testable strings and
        // live in `presentation::html_renderer`; this one needs a database,
        // and the phase's criterion says "survives a draft save and reload",
        // so a test that stopped at the sanitiser would be proving the wrong
        // half.
        //
        // Both descriptions here are awkward on purpose. A quote and an angle
        // bracket are what the escaping is for, and SQLite is the one stage
        // that could return them as something else.
        use crate::application::pictures::{WhatThePictureSays, a_picture_to_send};

        let bytes = vec![0x89, b'P', b'N', b'G', 1, 2, 3, 4];
        let chart = a_picture_to_send(
            "image/png",
            &bytes,
            &WhatThePictureSays::InWords(r#"A chart of "sales" < 2026"#.to_string()),
        )
        .expect("a described picture");
        let furniture = format!(r#"<img src="data:image/png;base64,{}" alt="">"#, {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(&bytes)
        });
        let written = crate::presentation::editor_document::body_from_editor(
            &serde_json::to_string(&format!("<p>{chart}</p><p>{furniture}</p>"))
                .expect("a body a page could answer with"),
        );

        let cache = a_cache("draft_pictures");
        cache
            .save_draft(&CachedDraft {
                id: "draft-with-pictures".to_string(),
                account_id: "acc-1".to_string(),
                to_addr: "ada@example.com".to_string(),
                cc: None,
                bcc: None,
                subject: "Sales".to_string(),
                body: "See the chart".to_string(),
                body_html: Some(written.clone()),
                attachments: Vec::new(),
                in_reply_to: None,
                references: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                protection: Choice::Plain,
                from_address: None,
                from_name: None,
            })
            .expect("the draft to save");

        let back = cache
            .load_draft("draft-with-pictures")
            .expect("the draft to load")
            .expect("the draft to be there")
            .body_html
            .expect("the formatted half to come back");

        assert_eq!(
            back, written,
            "the database did not give the body back as it was stored"
        );
        assert!(
            back.contains("data:image/png;base64,"),
            "the picture itself did not survive the database: {back}"
        );
        assert!(
            back.contains("&quot;sales&quot;") && back.contains("&lt; 2026"),
            "the escaped description did not survive the database: {back}"
        );
        assert!(
            back.contains(r#"alt="""#),
            "the decorative mark did not survive the database: {back}"
        );
    }

    #[test]
    fn test_a_reply_saved_as_a_draft_still_knows_what_it_answers() {
        // Otherwise Save Draft on a reply loses its place in the thread
        // silently: it comes back looking complete and goes out as the start of
        // a new conversation, which is the same shape of invisible loss the
        // missing Cc was.
        let cache = a_cache("thread");
        let draft = CachedDraft {
            id: "draft-reply".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "ada@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Re: Notes".to_string(),
            body: "Half a thought".to_string(),
            in_reply_to: Some("<c@x>".to_string()),
            references: Some("<a@x> <c@x>".to_string()),
            body_html: None,
            attachments: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            protection: Choice::Plain,
            from_address: None,
            from_name: None,
        };
        cache.save_draft(&draft).expect("the draft to save");

        let back = cache
            .load_draft("draft-reply")
            .expect("the draft to load")
            .expect("a draft");
        assert_eq!(back.in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(back.references.as_deref(), Some("<a@x> <c@x>"));

        let listed = cache.load_drafts("acc-1").expect("the drafts to list");
        assert_eq!(listed[0].in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(listed[0].references.as_deref(), Some("<a@x> <c@x>"));
    }

    #[test]
    fn test_a_draft_saved_before_there_was_a_conversation_still_opens() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache
                .save_draft(&CachedDraft {
                    id: "draft-old".to_string(),
                    account_id: "acc-1".to_string(),
                    to_addr: "ada@example.com".to_string(),
                    cc: None,
                    bcc: None,
                    subject: "Written long ago".to_string(),
                    body: "Body".to_string(),
                    in_reply_to: None,
                    references: None,
                    body_html: None,
                    attachments: Vec::new(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    protection: Choice::Plain,
                    from_address: None,
                    from_name: None,
                })
                .expect("the draft to save");
            for column in ["in_reply_to", "references_header"] {
                cache
                    .conn
                    .execute(&format!("ALTER TABLE drafts DROP COLUMN {column}"), [])
                    .expect("the column to come off, making this an older database");
            }
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        let back = reopened
            .load_draft("draft-old")
            .expect("the draft to load")
            .expect("the draft to survive");
        assert_eq!(back.subject, "Written long ago");
        assert!(back.in_reply_to.is_none());
    }

    fn a_draft_protected(choice: Choice) -> CachedDraft {
        CachedDraft {
            id: "draft-private".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "grace@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Private".to_string(),
            body: "Only for Grace".to_string(),
            body_html: None,
            attachments: Vec::new(),
            in_reply_to: None,
            references: None,
            protection: choice,
            from_address: None,
            from_name: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_a_draft_keeps_whether_it_is_to_go_signed_or_encrypted() {
        // A draft reopened and sent without its Encrypt box is a private
        // message sent in the clear.
        let cache = a_cache("draft_protection");
        for choice in [
            Choice::Signed,
            Choice::Encrypted,
            Choice::SignedAndEncrypted,
            Choice::Plain,
        ] {
            cache
                .save_draft(&a_draft_protected(choice))
                .expect("the draft to save");
            let opened = cache
                .load_draft("draft-private")
                .expect("the draft to load")
                .expect("the draft is there");
            let listed = cache.load_drafts("acc-1").expect("the drafts to list");
            assert_eq!(opened.protection, choice, "opened");
            assert_eq!(listed[0].protection, choice, "listed");
        }
    }

    #[test]
    fn test_a_draft_saved_before_there_was_a_protection_opens_plain() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache
                .save_draft(&a_draft_protected(Choice::Plain))
                .expect("the draft to save");
            cache
                .conn
                .execute("ALTER TABLE drafts DROP COLUMN protection", [])
                .expect("the column to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        let back = reopened
            .load_draft("draft-private")
            .expect("the draft to load")
            .expect("the draft to survive");
        assert_eq!(back.protection, Choice::Plain);
    }

    #[test]
    fn test_draft_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let draft = CachedDraft {
            id: "draft-123".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: Some("cc@example.com".to_string()),
            bcc: None,
            subject: "Draft Subject".to_string(),
            body: "Draft body content".to_string(),
            in_reply_to: None,
            references: None,
            body_html: None,
            attachments: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            protection: Choice::Plain,
            from_address: None,
            from_name: None,
        };

        cache.save_draft(&draft).unwrap();

        let loaded = cache.load_draft("draft-123").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().subject, "Draft Subject");

        let drafts = cache.load_drafts("test@example.com").unwrap();
        assert_eq!(drafts.len(), 1);

        cache.delete_draft("draft-123").unwrap();
        let deleted = cache.load_draft("draft-123").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_draft_update() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let mut draft = CachedDraft {
            id: "draft-456".to_string(),
            account_id: "test@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            bcc: None,
            subject: "Original Subject".to_string(),
            body: "Original body".to_string(),
            in_reply_to: None,
            references: None,
            body_html: None,
            attachments: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            protection: Choice::Plain,
            from_address: None,
            from_name: None,
        };

        cache.save_draft(&draft).unwrap();

        draft.subject = "Updated Subject".to_string();
        draft.body = "Updated body".to_string();
        cache.save_draft(&draft).unwrap();

        let loaded = cache.load_draft("draft-456").unwrap();
        assert!(loaded.is_some());
        let loaded_draft = loaded.unwrap();
        assert_eq!(loaded_draft.subject, "Updated Subject");
        assert_eq!(loaded_draft.body, "Updated body");
    }
}
