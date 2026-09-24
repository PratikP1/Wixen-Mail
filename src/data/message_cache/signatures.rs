//! Signature persistence operations

use super::{MessageCache, Signature};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

impl MessageCache {
    /// Create a new signature
    pub fn create_signature(&self, signature: &Signature) -> Result<()> {
        self.conn.execute(
            "INSERT INTO signatures (id, account_id, name, content_plain, content_html, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &signature.id, &signature.account_id, &signature.name,
                &signature.content_plain, &signature.content_html,
                &signature.is_default, &signature.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create signature: {}", e)))?;

        if signature.is_default {
            self.conn
                .execute(
                    "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                    params![&signature.account_id, &signature.id],
                )
                .map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        Ok(())
    }

    /// Get all signatures for an account
    pub fn get_signatures_for_account(&self, account_id: &str) -> Result<Vec<Signature>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 ORDER BY name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signatures = stmt
            .query_map(params![account_id], |row| {
                Ok(Signature {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    content_plain: row.get(3)?,
                    content_html: row.get(4)?,
                    is_default: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query signatures: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect signatures: {}", e)))?;
        Ok(signatures)
    }

    /// Get a specific signature by ID
    pub fn get_signature(&self, signature_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signature = stmt
            .query_row(params![signature_id], |row| {
                Ok(Signature {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    content_plain: row.get(3)?,
                    content_html: row.get(4)?,
                    is_default: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get signature: {}", e)))?;
        Ok(signature)
    }

    /// Get the default signature for an account
    pub fn get_default_signature(&self, account_id: &str) -> Result<Option<Signature>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, content_plain, content_html, is_default, created_at
             FROM signatures WHERE account_id = ?1 AND is_default = 1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let signature = stmt
            .query_row(params![account_id], |row| {
                Ok(Signature {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    content_plain: row.get(3)?,
                    content_html: row.get(4)?,
                    is_default: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get default signature: {}", e)))?;
        Ok(signature)
    }

    /// Update a signature, and say how many rows that touched.
    ///
    /// The count matters, for the same reason it does on a label. Updating a
    /// row that is not there is not an error in SQL, so a caller that creates
    /// only when the update fails would never create anything.
    pub fn update_signature(&self, signature: &Signature) -> Result<usize> {
        let touched = self
            .conn
            .execute(
                "UPDATE signatures
             SET name = ?1, content_plain = ?2, content_html = ?3, is_default = ?4
             WHERE id = ?5",
                params![
                    &signature.name,
                    &signature.content_plain,
                    &signature.content_html,
                    &signature.is_default,
                    &signature.id
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to update signature: {}", e)))?;

        if signature.is_default {
            self.conn
                .execute(
                    "UPDATE signatures SET is_default = 0 WHERE account_id = ?1 AND id != ?2",
                    params![&signature.account_id, &signature.id],
                )
                .map_err(|e| Error::Other(format!("Failed to update defaults: {}", e)))?;
        }
        Ok(touched)
    }

    /// Delete a signature
    pub fn delete_signature(&self, signature_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM signatures WHERE id = ?1",
                params![signature_id],
            )
            .map_err(|e| Error::Other(format!("Failed to delete signature: {}", e)))?;
        Ok(())
    }

    /// Every signature, whichever account was active when it was written,
    /// ordered by name.
    pub fn get_every_signature(&self) -> Result<Vec<Signature>> {
        Ok(Vec::new())
    }

    /// Give an account a signature, or with `None` take its assignment away so
    /// it uses the default.
    pub fn assign(&self, account_id: &str, signature_id: Option<&str>) -> Result<()> {
        let _ = (account_id, signature_id);
        Ok(())
    }

    /// The signature assigned to an account, if one is.
    pub fn assignment_for(&self, account_id: &str) -> Result<Option<String>> {
        let _ = account_id;
        Ok(None)
    }

    /// Make one signature the default for every account with none assigned,
    /// or with `None` leave no default at all.
    pub fn set_the_default(&self, signature_id: Option<&str>) -> Result<()> {
        let _ = signature_id;
        Ok(())
    }

    /// The signature an account's messages start with.
    pub fn signature_for_account(&self, account_id: &str) -> Result<Option<Signature>> {
        let _ = account_id;
        Ok(None)
    }

    /// Turn signatures kept per account into one set, once.
    pub fn make_signatures_one_set(&self) -> Result<()> {
        let _ = SIGNATURES_ARE_ONE_SET;
        Ok(())
    }
}

/// The name under which the once-only pass that made signatures one set is
/// recorded in `work_done_once`.
const SIGNATURES_ARE_ONE_SET: &str =
    "signatures made one set, each account keeping its own, 2026-09-24";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_sig_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    fn a_signature(id: &str, account_id: &str, name: &str) -> Signature {
        Signature {
            id: id.to_string(),
            account_id: account_id.to_string(),
            name: name.to_string(),
            content_plain: format!("Regards, {name}"),
            content_html: None,
            is_default: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    /// A row as a build before this one wrote it, where each account could
    /// have a default of its own, and the pass not yet run.
    fn written_before(
        cache: &MessageCache,
        id: &str,
        account_id: &str,
        is_default: bool,
        at: &str,
    ) {
        cache
            .conn
            .execute(
                "INSERT INTO signatures (id, account_id, name, content_plain, is_default, created_at)
                 VALUES (?1, ?2, ?1, ?1, ?3, ?4)",
                params![id, account_id, is_default, at],
            )
            .expect("a row written before");
        cache
            .conn
            .execute(
                "DELETE FROM work_done_once WHERE name = ?1",
                params![SIGNATURES_ARE_ONE_SET],
            )
            .expect("the pass not yet run");
    }

    fn defaults(cache: &MessageCache) -> Vec<String> {
        cache
            .get_every_signature()
            .expect("signatures to read")
            .into_iter()
            .filter(|s| s.is_default)
            .map(|s| s.id)
            .collect()
    }

    fn for_account(cache: &MessageCache, account_id: &str) -> Option<String> {
        cache
            .signature_for_account(account_id)
            .expect("a signature to resolve")
            .map(|s| s.id)
    }

    #[test]
    fn test_every_signature_is_listed_whichever_account_wrote_it() {
        let cache = a_cache();
        for signature in [
            a_signature("s1", "work", "Work"),
            a_signature("s2", "home", "Home"),
            a_signature("s3", "local", "Brief"),
        ] {
            cache
                .create_signature(&signature)
                .expect("a signature to save");
        }
        let names: Vec<String> = cache
            .get_every_signature()
            .expect("signatures to read")
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["Brief", "Home", "Work"]);
    }

    #[test]
    fn test_an_assignment_is_read_back_and_none_takes_it_away() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache.assign("work", Some("s1")).expect("an assignment");
        assert_eq!(
            cache.assignment_for("work").expect("read"),
            Some("s1".to_string())
        );
        cache
            .assign("work", None)
            .expect("the assignment taken away");
        assert_eq!(cache.assignment_for("work").expect("read"), None);
    }

    #[test]
    fn test_an_account_takes_its_assignment_before_the_default() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache
            .create_signature(&a_signature("s2", "work", "Home"))
            .expect("a signature");
        cache.set_the_default(Some("s2")).expect("a default");
        cache.assign("work", Some("s1")).expect("an assignment");
        assert_eq!(for_account(&cache, "work"), Some("s1".to_string()));
    }

    #[test]
    fn test_an_account_with_no_assignment_takes_the_default() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache.set_the_default(Some("s1")).expect("a default");
        assert_eq!(for_account(&cache, "home"), Some("s1".to_string()));
    }

    #[test]
    fn test_setting_the_default_clears_it_on_every_other_account() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache
            .create_signature(&a_signature("s2", "home", "Home"))
            .expect("a signature");
        cache.set_the_default(Some("s1")).expect("a default");
        cache.set_the_default(Some("s2")).expect("another default");
        assert_eq!(defaults(&cache), ["s2"]);
    }

    #[test]
    fn test_clearing_the_default_leaves_an_unassigned_account_with_none() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache.set_the_default(Some("s1")).expect("a default");
        assert_eq!(for_account(&cache, "home"), Some("s1".to_string()));
        cache.set_the_default(None).expect("the default cleared");
        assert_eq!(for_account(&cache, "home"), None);
        assert!(defaults(&cache).is_empty());
    }

    #[test]
    fn test_deleting_a_signature_takes_away_the_assignments_that_named_it() {
        let cache = a_cache();
        cache
            .create_signature(&a_signature("s1", "work", "Work"))
            .expect("a signature");
        cache.assign("work", Some("s1")).expect("an assignment");
        assert_eq!(
            cache.assignment_for("work").expect("read"),
            Some("s1".to_string())
        );
        cache.delete_signature("s1").expect("a delete");
        assert_eq!(cache.assignment_for("work").expect("read"), None);
    }

    #[test]
    fn test_the_pass_keeps_every_accounts_signature_and_one_default() {
        // Three accounts, each with the default it had before, and one
        // signature that was nobody's default.
        let cache = a_cache();
        written_before(&cache, "work-sig", "work", true, "2026-03-01T00:00:00Z");
        written_before(&cache, "home-sig", "home", true, "2026-01-01T00:00:00Z");
        written_before(&cache, "club-sig", "club", true, "2026-02-01T00:00:00Z");
        written_before(&cache, "spare", "work", false, "2025-01-01T00:00:00Z");

        cache.make_signatures_one_set().expect("the pass");

        for (account, signature) in [
            ("work", "work-sig"),
            ("home", "home-sig"),
            ("club", "club-sig"),
        ] {
            assert_eq!(
                for_account(&cache, account),
                Some(signature.to_string()),
                "{account} lost the signature it had"
            );
        }
        assert_eq!(
            defaults(&cache),
            ["home-sig"],
            "the oldest default is the one kept"
        );
    }

    #[test]
    fn test_the_pass_runs_once() {
        let cache = a_cache();
        written_before(&cache, "work-sig", "work", true, "2026-03-01T00:00:00Z");
        cache.make_signatures_one_set().expect("the pass");
        assert_eq!(
            cache.assignment_for("work").expect("read"),
            Some("work-sig".to_string())
        );

        // A default set per account after the pass is not a row from before
        // it, and a second open must not turn it into an assignment.
        cache
            .conn
            .execute(
                "INSERT INTO signatures (id, account_id, name, content_plain, is_default, created_at)
                 VALUES ('later', 'home', 'later', 'later', 1, '2026-04-01T00:00:00Z')",
                [],
            )
            .expect("a later row");
        cache.make_signatures_one_set().expect("the pass again");
        assert_eq!(cache.assignment_for("home").expect("read"), None);
    }

    #[test]
    fn test_signature_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let signature = Signature {
            id: "sig-work".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Work Signature".to_string(),
            content_plain: "Best regards,\nJohn Doe".to_string(),
            content_html: Some("<p>Best regards,<br><strong>John Doe</strong></p>".to_string()),
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&signature).unwrap();

        let loaded_sig = cache.get_signature("sig-work").unwrap();
        assert!(loaded_sig.is_some());
        assert_eq!(loaded_sig.unwrap().name, "Work Signature");

        let sigs = cache
            .get_signatures_for_account("test@example.com")
            .unwrap();
        assert_eq!(sigs.len(), 1);

        let default_sig = cache.get_default_signature("test@example.com").unwrap();
        assert!(default_sig.is_some());

        let mut updated_sig = signature.clone();
        updated_sig.name = "Updated Work Signature".to_string();
        updated_sig.content_plain = "Regards,\nJohn Doe, CEO".to_string();
        cache.update_signature(&updated_sig).unwrap();

        let loaded = cache.get_signature("sig-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Updated Work Signature");
        assert!(loaded.content_plain.contains("CEO"));

        cache.delete_signature("sig-work").unwrap();
        let deleted = cache.get_signature("sig-work").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_signature_default_switching() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let sig1 = Signature {
            id: "sig-1".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Signature 1".to_string(),
            content_plain: "Sig 1".to_string(),
            content_html: None,
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig1).unwrap();

        let sig2 = Signature {
            id: "sig-2".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Signature 2".to_string(),
            content_plain: "Sig 2".to_string(),
            content_html: None,
            is_default: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.create_signature(&sig2).unwrap();

        let default = cache.get_default_signature("test@example.com").unwrap();
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, "sig-2");

        let sig1_loaded = cache.get_signature("sig-1").unwrap().unwrap();
        assert!(!sig1_loaded.is_default);
    }
}
