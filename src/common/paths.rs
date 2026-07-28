//! Where user files live.
//!
//! One module owns every path the application writes to, so there is a single
//! answer to "where is my mail" for backups, for support, and for the
//! uninstaller.
//!
//! Everything sits under one root, `%LOCALAPPDATA%\wixen-mail` by default:
//!
//! ```text
//! config\      settings, one file per account, oauth.toml
//! cache\       message_cache.db and its SQLite sidecars
//! logs\        the running log and crash.log
//! security.key the fallback key, used when the credential store refuses
//! ```
//!
//! Earlier versions spread these across three profile folders and roamed the
//! key that decrypts the mail while leaving the mail itself behind.
//! [`AppPaths::migrate_legacy`] collects them on the next start.

use crate::common::{Error, Result};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

/// Environment variable that puts every user file somewhere else.
///
/// Set it to run from a memory stick, or to keep mail off a roaming profile
/// without moving the whole application data folder.
pub const DATA_DIR_ENV: &str = "WIXEN_MAIL_DATA";

/// Folder created inside the platform's local application data directory.
const FOLDER: &str = "wixen-mail";

/// The cache database and every sidecar SQLite may have left beside it.
///
/// Moving the database without its write-ahead log loses whatever had not been
/// checkpointed into the main file yet.
const DATABASE_FILES: [&str; 4] = [
    "message_cache.db",
    "message_cache.db-wal",
    "message_cache.db-shm",
    "message_cache.db-journal",
];

/// Every path the application writes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    root: PathBuf,
}

impl AppPaths {
    /// Work out where the files belong, without touching the disk.
    pub fn resolve() -> Result<Self> {
        Self::resolve_with(std::env::var_os(DATA_DIR_ENV), dirs::data_local_dir())
    }

    /// The decision [`resolve`](Self::resolve) makes, with its two inputs
    /// handed in so it can be tested without setting process environment.
    pub fn resolve_with(chosen: Option<OsString>, local_data: Option<PathBuf>) -> Result<Self> {
        if let Some(chosen) = chosen.filter(|value| !value.is_empty()) {
            return Ok(Self::under(chosen));
        }

        local_data.map(|dir| Self::under(dir.join(FOLDER))).ok_or_else(|| {
            Error::Config(format!(
                "Could not find the local application data folder. Set {DATA_DIR_ENV} to choose where Wixen Mail keeps its files."
            ))
        })
    }

    /// Root the files at a directory of the caller's choosing.
    pub fn under(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The one folder holding everything.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Settings and one file per account.
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }

    /// The cached mail database. Encrypted, and rebuildable from the server.
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// The running log and the crash log.
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Fallback encryption key, used only when the credential store refuses.
    pub fn security_key(&self) -> PathBuf {
        self.root.join("security.key")
    }

    /// OAuth client credentials, supplied by whoever built the application.
    pub fn oauth_toml(&self) -> PathBuf {
        self.config_dir().join("oauth.toml")
    }

    /// Make the folders. Safe to call on every start.
    pub fn create(&self) -> Result<()> {
        for dir in [self.config_dir(), self.cache_dir(), self.logs_dir()] {
            fs::create_dir_all(&dir)
                .map_err(|e| Error::Config(format!("Could not create {}: {e}", dir.display())))?;
        }
        Ok(())
    }

    /// Collect files earlier versions left in other folders.
    ///
    /// A file already in its new place is left alone and the old copy is not
    /// touched, so a migration that goes wrong can still be inspected.
    pub fn migrate_legacy(&self, legacy: &LegacyLocations) -> MigrationReport {
        let mut report = MigrationReport::default();

        if let Some(roaming) = &legacy.roaming {
            report.bring(roaming.join("security.key"), self.security_key());
        }
        if let Some(dotdir) = &legacy.home_dotdir {
            report.bring(dotdir.join("oauth.toml"), self.oauth_toml());
        }
        for name in DATABASE_FILES {
            report.bring(self.root.join(name), self.cache_dir().join(name));
        }
        for name in self.settings_left_in_the_root() {
            report.bring(self.root.join(&name), self.config_dir().join(&name));
        }

        report
    }

    fn settings_left_in_the_root(&self) -> Vec<OsString> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name())
            .filter(is_settings_file)
            .collect()
    }
}

fn is_settings_file(name: &OsString) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    name == "app_config.json" || (name.starts_with("account_") && name.ends_with(".json"))
}

/// Folders earlier versions wrote to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyLocations {
    /// `%APPDATA%\wixen-mail`, which held the security key.
    pub roaming: Option<PathBuf>,
    /// `%USERPROFILE%\.wixen-mail`, which held oauth.toml.
    pub home_dotdir: Option<PathBuf>,
}

impl LegacyLocations {
    /// Where those folders are on this machine.
    pub fn detect() -> Self {
        Self {
            roaming: dirs::data_dir().map(|dir| dir.join(FOLDER)),
            home_dotdir: dirs::home_dir().map(|home| home.join(".wixen-mail")),
        }
    }
}

/// What a migration did, so it can be logged rather than guessed at.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    /// Where each collected file now is.
    pub moved: Vec<PathBuf>,
    /// Files left where they were, and what stopped them.
    pub failed: Vec<(PathBuf, String)>,
}

impl MigrationReport {
    /// Nothing needed collecting, which is the normal case.
    pub fn is_empty(&self) -> bool {
        self.moved.is_empty() && self.failed.is_empty()
    }

    fn bring(&mut self, from: PathBuf, to: PathBuf) {
        if !from.is_file() || to.exists() {
            return;
        }
        match move_file(&from, &to) {
            Ok(()) => self.moved.push(to),
            Err(reason) => self.failed.push((from, reason)),
        }
    }
}

fn move_file(from: &Path, to: &Path) -> std::result::Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    // Renaming only works within a volume, and profile folders are redirected
    // to a network share often enough to matter. Copy, then drop the original.
    fs::copy(from, to).map_err(|e| e.to_string())?;
    // A source that will not delete leaves a stale duplicate behind, not a lost
    // file, so it is not worth reporting as a failed migration.
    let _ = fs::remove_file(from);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn nowhere() -> LegacyLocations {
        LegacyLocations {
            roaming: None,
            home_dotdir: None,
        }
    }

    #[test]
    fn test_every_kind_of_file_gets_its_own_place_under_one_root() {
        let root = PathBuf::from("root");
        let paths = AppPaths::under(&root);

        assert_eq!(paths.root(), root);
        assert_eq!(paths.config_dir(), root.join("config"));
        assert_eq!(paths.cache_dir(), root.join("cache"));
        assert_eq!(paths.logs_dir(), root.join("logs"));
        assert_eq!(paths.security_key(), root.join("security.key"));
        assert_eq!(paths.oauth_toml(), root.join("config").join("oauth.toml"));
    }

    #[test]
    fn test_the_key_sits_under_the_same_root_as_the_mail_it_unlocks() {
        // The key used to roam while the encrypted database stayed local, so a
        // domain profile carried the key across the network every logon and
        // left behind the only thing it was protecting.
        let paths = AppPaths::under("root");
        assert!(paths.security_key().starts_with(paths.root()));
        assert!(paths.cache_dir().starts_with(paths.root()));
    }

    #[test]
    fn test_the_chosen_directory_wins_over_the_platform_one() {
        let paths = AppPaths::resolve_with(
            Some(OsString::from("chosen")),
            Some(PathBuf::from("platform")),
        )
        .unwrap();
        assert_eq!(paths.root(), Path::new("chosen"));
    }

    #[test]
    fn test_the_platform_directory_is_used_when_nothing_overrides_it() {
        let paths = AppPaths::resolve_with(None, Some(PathBuf::from("platform"))).unwrap();
        assert_eq!(paths.root(), Path::new("platform").join(FOLDER));
    }

    #[test]
    fn test_an_empty_override_is_ignored_rather_than_taken_as_the_root() {
        // A variable that is unset and one set to nothing look identical to
        // somebody editing a shortcut, and rooting the data at "" would scatter
        // it into whichever directory the shortcut happened to start in.
        let paths =
            AppPaths::resolve_with(Some(OsString::new()), Some(PathBuf::from("platform"))).unwrap();
        assert_eq!(paths.root(), Path::new("platform").join(FOLDER));
    }

    #[test]
    fn test_resolving_fails_rather_than_writing_to_the_current_directory() {
        // Logging and the crash handler both used to fall back to ".", which
        // dropped logs wherever the shortcut's working directory pointed and
        // made them impossible to ask somebody for.
        assert!(AppPaths::resolve_with(None, None).is_err());
    }

    #[test]
    fn test_creating_the_directories_is_safe_to_repeat() {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));

        paths.create().unwrap();
        paths.create().unwrap();

        assert!(paths.config_dir().is_dir());
        assert!(paths.cache_dir().is_dir());
        assert!(paths.logs_dir().is_dir());
    }

    #[test]
    fn test_the_security_key_is_brought_in_from_the_roaming_profile() {
        let dir = TempDir::new().unwrap();
        let roaming = dir.path().join("Roaming").join("wixen-mail");
        write(&roaming.join("security.key"), "key");
        let paths = AppPaths::under(dir.path().join("Local").join("wixen-mail"));

        let report = paths.migrate_legacy(&LegacyLocations {
            roaming: Some(roaming.clone()),
            home_dotdir: None,
        });

        assert_eq!(fs::read_to_string(paths.security_key()).unwrap(), "key");
        assert!(!roaming.join("security.key").exists());
        assert_eq!(report.moved, vec![paths.security_key()]);
        assert!(report.failed.is_empty());
    }

    #[test]
    fn test_oauth_credentials_come_in_from_the_home_dotfolder() {
        let dir = TempDir::new().unwrap();
        let dotdir = dir.path().join(".wixen-mail");
        write(&dotdir.join("oauth.toml"), "[gmail]");
        let paths = AppPaths::under(dir.path().join("wixen-mail"));

        let report = paths.migrate_legacy(&LegacyLocations {
            roaming: None,
            home_dotdir: Some(dotdir),
        });

        assert_eq!(fs::read_to_string(paths.oauth_toml()).unwrap(), "[gmail]");
        assert_eq!(report.moved, vec![paths.oauth_toml()]);
    }

    #[test]
    fn test_the_message_database_moves_with_its_write_ahead_log() {
        // Moving the database and leaving the -wal behind loses every change
        // SQLite had not yet checkpointed into the main file.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        let sidecars = [
            "message_cache.db",
            "message_cache.db-wal",
            "message_cache.db-shm",
        ];
        for name in sidecars {
            write(&paths.root().join(name), name);
        }

        paths.migrate_legacy(&nowhere());

        for name in sidecars {
            assert_eq!(
                fs::read_to_string(paths.cache_dir().join(name)).unwrap(),
                name,
                "{name} did not arrive"
            );
            assert!(!paths.root().join(name).exists(), "{name} was left behind");
        }
    }

    #[test]
    fn test_settings_move_out_of_the_root_into_the_config_folder() {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        write(&paths.root().join("app_config.json"), "{}");
        write(&paths.root().join("account_abc123.json"), "{\"id\":1}");

        let report = paths.migrate_legacy(&nowhere());

        assert_eq!(
            fs::read_to_string(paths.config_dir().join("app_config.json")).unwrap(),
            "{}"
        );
        assert_eq!(
            fs::read_to_string(paths.config_dir().join("account_abc123.json")).unwrap(),
            "{\"id\":1}"
        );
        assert_eq!(report.moved.len(), 2);
        assert!(report.failed.is_empty());
    }

    #[test]
    fn test_a_file_already_in_its_new_place_is_never_overwritten() {
        // Whatever is already in the new location is the one in use. The stale
        // copy stays where it is rather than being destroyed, so a migration
        // that went wrong can still be inspected.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        write(&paths.root().join("app_config.json"), "old");
        write(&paths.config_dir().join("app_config.json"), "current");

        let report = paths.migrate_legacy(&nowhere());

        assert_eq!(
            fs::read_to_string(paths.config_dir().join("app_config.json")).unwrap(),
            "current"
        );
        assert_eq!(
            fs::read_to_string(paths.root().join("app_config.json")).unwrap(),
            "old"
        );
        assert!(report.is_empty());
    }

    #[test]
    fn test_a_fresh_install_has_nothing_to_report() {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        paths.create().unwrap();

        assert!(paths.migrate_legacy(&nowhere()).is_empty());
    }

    #[test]
    fn test_a_legacy_folder_that_never_existed_is_not_a_failure() {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));

        let report = paths.migrate_legacy(&LegacyLocations {
            roaming: Some(dir.path().join("no-such-folder")),
            home_dotdir: Some(dir.path().join("also-missing")),
        });

        assert!(report.is_empty());
    }
}
