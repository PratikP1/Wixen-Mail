//! Where user files live.
//!
//! One module owns every path the application writes to, so there is a single
//! answer to "where is my mail" for backups, for support, and for the
//! uninstaller.
//!
//! Everything sits under one root, `%LOCALAPPDATA%\wixen-mail` by default:
//!
//! ```text
//! config\        settings, one file per account, oauth.toml
//! cache\         message_cache.db and its SQLite sidecars
//! logs\          the running log and crash.log
//! logs\feedback\ a copy of each feedback report sent, and its log excerpt
//! sound_schemes\ imported sound-scheme packs, one subdirectory each
//! updates\       an installer fetched for an update, while it waits
//! security.key   the fallback key, used when the credential store refuses
//! ```
//!
//! Two more sit beside those and are written by WebView2 rather than by this
//! program, which is why they are named here rather than handed out by an
//! accessor:
//!
//! ```text
//! EBWebView\     the browser profile the message preview and the formatted
//!                message window share, one per process and this is the one
//! pages\EBWebView\
//!                the browser profile a separate window uses, which is a
//!                process of its own (#80)
//! ```
//!
//! Both follow the Windows local data folder rather than this root, because
//! wxWidgets reads it from the known-folder API and `WIXEN_MAIL_DATA` does
//! not reach it. With the variable unset they are inside the root and the
//! erase takes them with the rest; with it set they stay where they are, and
//! [`page_profile_dir`] is what the erase asks so it can remove the second
//! anyway.
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
const DATA_DIR_ENV: &str = "WIXEN_MAIL_DATA";

/// Folder created inside the platform's local application data directory.
const FOLDER: &str = "wixen-mail";

/// The folder inside the root that a page process keeps its browser profile
/// in.
const PAGE_PROFILE_FOLDER: &str = "pages";

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
    fn resolve_with(chosen: Option<OsString>, local_data: Option<PathBuf>) -> Result<Self> {
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

    /// The cached mail database. Not encrypted, and rebuildable from the server.
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// The running log and the crash log.
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// The copy of each feedback report sent, and its log excerpt, kept so a
    /// report that bounces can be sent again (#64).
    pub fn feedback_dir(&self) -> PathBuf {
        self.logs_dir().join("feedback")
    }

    /// Fallback encryption key, used only when the credential store refuses.
    pub fn security_key(&self) -> PathBuf {
        self.root.join("security.key")
    }

    /// OAuth client credentials, supplied by whoever built the application.
    pub fn oauth_toml(&self) -> PathBuf {
        self.config_dir().join("oauth.toml")
    }

    /// Imported sound-scheme packs, one subdirectory per scheme. Built-in
    /// schemes ship inside the application itself and are not stored here;
    /// this is only ever written to by importing a zip.
    pub fn sound_schemes_dir(&self) -> PathBuf {
        self.root.join("sound_schemes")
    }

    /// Where an installer fetched for an update waits until it is used.
    ///
    /// Under this application's own root rather than the shared temporary
    /// folder, and that is a security decision before it is a tidy one. A
    /// downloaded executable sitting where another person using this computer
    /// can write is a file they can swap after it was checked and before it is
    /// run, so the check would have been made against something that is no
    /// longer there. Under the root, only this user can write.
    ///
    /// It also keeps `docs/privacy.md`'s sentence about uninstalling removing
    /// everything true: the uninstaller clears this root wholesale, so an
    /// installer waiting here goes with it. In the temporary folder it would
    /// not, and that page would have needed weakening to stay honest.
    ///
    /// It holds one file at most, it exists only while an update is being
    /// fetched, and it is emptied three times over: after a handover, after any
    /// refusal, and at the next start, which is the one that covers a program
    /// that died in between.
    pub fn updates_dir(&self) -> PathBuf {
        self.root.join("updates")
    }

    /// Make the folders. Safe to call on every start.
    pub fn create(&self) -> Result<()> {
        for dir in [
            self.config_dir(),
            self.cache_dir(),
            self.logs_dir(),
            self.sound_schemes_dir(),
        ] {
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

/// The application name a page process runs under.
///
/// A separate Wixen Mail window is a process of its own (#80), and the one
/// lever this toolkit gives for a browser profile is the application name:
/// `wxStandardPathsBase::AppendAppInfo` appends it to the local data folder,
/// and `wxWebViewConfigurationImplEdge` takes that folder as its data path.
/// The name is appended as written, separator and all, so a name with one in
/// it puts the page profile a level down, inside the root this module
/// promises, where the erase and the uninstaller already reach.
///
/// A `String` rather than a constant because the separator is the platform's.
pub fn page_profile_app_name() -> String {
    format!("{FOLDER}{}{PAGE_PROFILE_FOLDER}", std::path::MAIN_SEPARATOR)
}

/// Where a page process's browser profile really goes on this computer.
///
/// Not under [`AppPaths::root`], and that is the point of its being here.
/// WebView2's data path comes from wxWidgets, which reads the local data
/// folder from the Windows known-folder API; `WIXEN_MAIL_DATA` moves
/// everything this program writes for itself and moves nothing the browser
/// writes. So with the variable unset this is inside the root and the erase
/// reaches it with the rest, and with the variable set it is a second place
/// the erase has to name. It names it.
pub fn page_profile_dir() -> Option<PathBuf> {
    page_profile_dir_in(dirs::data_local_dir())
}

/// The decision [`page_profile_dir`] makes, with its one input handed in so
/// it can be tested without reading this machine's profile.
fn page_profile_dir_in(local_data: Option<PathBuf>) -> Option<PathBuf> {
    local_data.map(|dir| dir.join(FOLDER).join(PAGE_PROFILE_FOLDER))
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

    #[test]
    fn test_only_the_settings_files_are_collected() {
        // Found by mutation testing: the two halves of the account-file test
        // were never checked apart, so `and` could become `or` unnoticed. With
        // `or`, every file whose name merely ends in .json is a settings file,
        // and a migration sweeps up whatever else somebody kept in that folder.
        for name in ["app_config.json", "account_work.json", "account_.json"] {
            assert!(
                is_settings_file(&OsString::from(name)),
                "{name} was not recognised as a settings file"
            );
        }
        for name in [
            // Ends in .json but is not ours.
            "notes.json",
            // Starts with account_ but is not JSON, which is how a backup made
            // by hand usually looks.
            "account_work.json.bak",
            "account_work.txt",
            "message_cache.db",
            "",
        ] {
            assert!(
                !is_settings_file(&OsString::from(name)),
                "{name} was collected and should not have been"
            );
        }
    }

    #[test]
    fn test_a_report_is_empty_only_when_nothing_happened() {
        // It decides whether a migration is worth saying anything about, so
        // one that always answers "nothing happened" means files quietly moved
        // with no record of where they went.
        let mut report = MigrationReport::default();
        assert!(report.is_empty());

        report.moved.push(PathBuf::from("account_work.json"));
        assert!(
            !report.is_empty(),
            "a report with a move in it read as empty"
        );

        let mut failed = MigrationReport::default();
        failed
            .failed
            .push((PathBuf::from("oauth.toml"), "in use".to_string()));
        assert!(
            !failed.is_empty(),
            "a report with a failure in it read as empty"
        );
    }

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
        assert_eq!(paths.sound_schemes_dir(), root.join("sound_schemes"));
        assert_eq!(paths.feedback_dir(), root.join("logs").join("feedback"));
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
        assert!(paths.sound_schemes_dir().is_dir());
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

    /// The two pages that list what Wixen Mail leaves on somebody's disk.
    ///
    /// They carry the same listing almost word for word and nothing checked
    /// that they agreed. For the paths in that listing, the test below is now
    /// what checks it.
    const PAGES_LISTING_WHAT_IS_STORED: [&str; 2] = ["docs/installing.md", "docs/privacy.md"];

    /// Every path this module hands out, with the accessor that hands it out.
    ///
    /// Built by calling each accessor rather than by reading this file for
    /// `self.root.join(`, which is the version that looks right and is wrong:
    /// `oauth_toml` joins onto `config_dir` rather than onto the root, so that
    /// reading would miss it and pass while missing it. A value cannot be
    /// missed the way a pattern can.
    ///
    /// That leaves one hole and
    /// `test_the_enumeration_reaches_every_accessor_this_module_has` is what
    /// fills it: this list is written by hand, so an accessor added later would
    /// not be in it. That test reads the source for the definitions and fails
    /// when this list falls behind them.
    fn every_path_handed_out(paths: &AppPaths) -> Vec<(&'static str, PathBuf)> {
        vec![
            ("root", paths.root().to_path_buf()),
            ("config_dir", paths.config_dir()),
            ("cache_dir", paths.cache_dir()),
            ("logs_dir", paths.logs_dir()),
            ("feedback_dir", paths.feedback_dir()),
            ("security_key", paths.security_key()),
            ("oauth_toml", paths.oauth_toml()),
            ("sound_schemes_dir", paths.sound_schemes_dir()),
            ("updates_dir", paths.updates_dir()),
        ]
    }

    /// The last part of each path, which is what a page can name.
    fn the_name_at_the_end(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string()
    }

    /// Which of those paths a page does not name.
    ///
    /// Split out from the test so a page that resolved to nothing can be driven
    /// as a case rather than waited for.
    fn paths_not_named_in(page: &str, text: &str, paths: &AppPaths) -> Vec<String> {
        every_path_handed_out(paths)
            .into_iter()
            .filter(|(_, path)| !text.contains(&the_name_at_the_end(path)))
            .map(|(accessor, path)| {
                format!(
                    "{page} does not name {}, which {accessor} hands out",
                    the_name_at_the_end(&path)
                )
            })
            .collect()
    }

    #[test]
    fn test_every_path_this_module_hands_out_is_named_on_both_pages() {
        // Somebody backing up a machine, or wiping one, reads those two pages
        // to find out what is there. A path on neither is a path they do not
        // know about, and `security.key` was on neither for the whole of this
        // project's life.
        //
        // What this cannot see, said here so a green build is not read as
        // "every path is written down". Three writes go to the temporary folder
        // through no accessor at all, so this check cannot reach any of them:
        //
        //     src/common/logging.rs:79      %TEMP%\wixen-mail\logs, when
        //                                   AppPaths::resolve() fails
        //     src/main.rs:307               %TEMP%\wixen-mail-uninstall.log,
        //                                   every uninstall
        //     src/presentation/help_page.rs:97
        //                                   %TEMP%\wixen-mail-help, when the
        //                                   folder holding the documents will
        //                                   not take a file
        //
        // Only the second is on a page. They are found with, and this is the
        // command to re-run rather than trust:
        //
        //     grep -rn 'temp_dir()' src/ --include=*.rs | grep -v 'TempDir::new\|tempfile::'
        //
        // They are not given accessors to make this check see them.
        // `logging.rs` falls back there precisely when `AppPaths` cannot
        // answer, so an accessor for it would be circular, and `help_page.rs`
        // writes there only when the install folder will not take a file.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        let mut missing = Vec::new();
        let mut pages_read = 0;

        for page in PAGES_LISTING_WHAT_IS_STORED {
            let Ok(text) = fs::read_to_string(page) else {
                continue;
            };
            pages_read += 1;
            missing.extend(paths_not_named_in(page, &text, &paths));
        }

        assert_eq!(
            pages_read,
            PAGES_LISTING_WHAT_IS_STORED.len(),
            "only {pages_read} of the pages listing what is stored could be \
             read, so this compared against nothing"
        );
        assert!(
            missing.is_empty(),
            "these paths are left on somebody's disk and are on no page they \
             can read:\n  {}",
            missing.join("\n  ")
        );
    }

    #[test]
    fn test_the_enumeration_reaches_every_accessor_this_module_has() {
        // The list above is written by hand, so without this an accessor added
        // later would sit outside the check and its path would never have to be
        // written down. This is the half that makes a new path fail the build.
        //
        // Read from the source rather than from a running program, because
        // nothing can call a function it only knows the name of. The test
        // module is cut off first: it calls these accessors rather than
        // defining them.
        let source = fs::read_to_string("src/common/paths.rs")
            .expect("this module to be readable")
            .replace("\r\n", "\n");
        let definitions = &source[..source.find("#[cfg(test)]").unwrap_or(source.len())];

        let defined: Vec<&str> = definitions
            .lines()
            .filter_map(|line| {
                let after = line.trim_start().strip_prefix("pub fn ")?;
                let (name, rest) = after.split_once('(')?;
                // A path handed out, rather than a constructor or `create`.
                (rest.contains("-> PathBuf") || rest.contains("-> &Path")).then_some(name)
            })
            .collect();

        assert!(
            defined.len() > 3,
            "only {} path accessors were found in the source, so the reading is \
             broken rather than the list",
            defined.len()
        );

        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        let enumerated: Vec<&str> = every_path_handed_out(&paths)
            .into_iter()
            .map(|(accessor, _)| accessor)
            .collect();

        let unlisted: Vec<&&str> = defined
            .iter()
            .filter(|name| !enumerated.contains(name))
            .collect();
        let gone: Vec<&&str> = enumerated
            .iter()
            .filter(|name| !defined.contains(name))
            .collect();

        assert!(
            unlisted.is_empty(),
            "these accessors hand out a path that nothing checks is written \
             down: {unlisted:?}"
        );
        assert!(
            gone.is_empty(),
            "these are enumerated and no longer exist: {gone:?}"
        );
    }

    #[test]
    fn test_a_page_that_could_not_be_read_is_a_failure_rather_than_a_clean_tree() {
        // A page renamed or moved would otherwise make the check above pass by
        // finding nothing to compare against, which is the way a check that
        // reads documents really fails.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));

        let against_nothing = paths_not_named_in("gone.md", "", &paths);
        assert_eq!(
            against_nothing.len(),
            every_path_handed_out(&paths).len(),
            "a page with nothing in it has to come back naming every path, or \
             an empty read reads as a clean tree"
        );

        for page in PAGES_LISTING_WHAT_IS_STORED {
            assert!(
                fs::read_to_string(page).is_ok_and(|text| !text.is_empty()),
                "{page} could not be read, and it is what the check above \
                 compares against"
            );
        }
    }

    #[test]
    fn test_a_page_processs_profile_sits_under_the_root_rather_than_beside_the_previews() {
        // #80's third place is a process of its own so that a page it shows
        // cannot read what the message preview's browser stored. Two
        // processes with one profile folder would be two processes with one
        // cookie jar, which is the thing that route exists to prevent.
        let local = PathBuf::from("C:/Users/somebody/AppData/Local");
        let profile = page_profile_dir_in(Some(local.clone())).expect("a local data folder");

        assert_eq!(profile, local.join(FOLDER).join(PAGE_PROFILE_FOLDER));
        // Under the root the rest of this module promises, so the erase and
        // the uninstaller reach it without learning a second place.
        assert!(profile.starts_with(AppPaths::under(local.join(FOLDER)).root()));
        // And not the root itself, which is where the message preview's
        // browser profile goes.
        assert_ne!(profile, local.join(FOLDER));
    }

    #[test]
    fn test_the_application_name_a_page_process_takes_is_the_folder_it_lands_in() {
        // wxWidgets appends the application name to the local data folder as
        // written, separator and all, so the name and the folder are one fact
        // and are built from one pair of parts rather than written twice.
        let local = PathBuf::from("C:/Users/somebody/AppData/Local");
        let name = page_profile_app_name();

        assert_eq!(
            local.join(&name),
            page_profile_dir_in(Some(local)).expect("a local data folder")
        );
        assert!(name.starts_with(FOLDER), "{name}");
        assert!(name.ends_with(PAGE_PROFILE_FOLDER), "{name}");
    }

    #[test]
    fn test_with_no_local_data_folder_there_is_no_page_profile_to_erase() {
        // The erase asks this and removes what it names. Nothing named is not
        // an error: a machine with no local data folder never had a page
        // process either.
        assert_eq!(page_profile_dir_in(None), None);
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
