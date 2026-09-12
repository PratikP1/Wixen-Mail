//! What the installer script says, read as text.
//!
//! `installer/Wixen-Mail-Setup.iss` is the artefact and it cannot be run from
//! here. Nothing in this repository compiles it with ISCC, installs anything,
//! or looks at a shortcut on a real machine, so every test in this file is a
//! claim about what the script says and not one of them is a claim about what
//! an install does. A green run here does not mean a shortcut on somebody's
//! desktop shows the right picture. `.planning/WINDOWS.md` carries that as a
//! verification nobody has run.
//!
//! # Why the script gets a target of its own
//!
//! Three tests already read this file and all three live in `src/` beside the
//! code they are really about: the mutex name in `application::running`, the
//! uninstall's erase step in `application::forget`, and the page the help
//! button opens in `presentation::first_run`. Each belongs where it is, because
//! each is half of an agreement between the script and a Rust constant.
//!
//! What had nowhere to live is a rule about the script on its own, where two of
//! its sections have to agree with each other and no Rust module owns either
//! side. That is what this file is for.
//!
//! Until 2026-09-12 a commit changing only the installer script ran none of the
//! tests that read it, this one included, because the scoped run maps a changed
//! file to a target by its path and an `.iss` matched nothing.
//! `scripts/which-checks.sh` now answers `all` for an `.iss`, which is what
//! makes this target run on the commits that could break it.

// ---------------------------------------------------------------------------
// The reading
// ---------------------------------------------------------------------------

/// One `[Icons]` entry: where the shortcut goes, what it names as its icon, and
/// the task that decides whether it is created at all.
#[derive(Debug)]
struct Shortcut {
    name: String,
    icon: Option<String>,
    task: Option<String>,
}

/// The installer script, read from the repository root.
fn the_installer_script() -> String {
    std::fs::read_to_string("installer/Wixen-Mail-Setup.iss").expect("the installer script")
}

/// The entry lines of one Inno section, with its comments and blank lines
/// dropped.
///
/// A section runs from its own header to the next one. A comment opens with a
/// semicolon at the start of a line, which is also the separator between two
/// parameters on an entry, so comments are dropped here rather than left for
/// the parameter reader to mistake for an entry.
///
/// Absent section, empty answer. Every caller below treats that as a failure
/// with its own message, because a reading that found nothing and a tree that
/// holds nothing read alike.
fn section<'a>(script: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("[{name}]");
    script
        .lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with(';'))
        .collect()
}

/// One `Key: "value"` parameter of an entry line.
///
/// The quotation marks are Inno's convention rather than its requirement:
/// `Flags: ignoreversion` and `Tasks: desktopicon` carry none, and both are
/// read here, so the marks are trimmed rather than required.
fn parameter(entry: &str, key: &str) -> Option<String> {
    entry.split(';').find_map(|part| {
        let (found, value) = part.split_once(':')?;
        found
            .trim()
            .eq_ignore_ascii_case(key)
            .then(|| value.trim().trim_matches('"').to_string())
    })
}

/// One `Key=value` directive from the `[Setup]` section, which is written with
/// an equals sign rather than with the colon the entry sections use.
fn setup_directive(script: &str, key: &str) -> Option<String> {
    section(script, "Setup").iter().find_map(|line| {
        let (found, value) = line.split_once('=')?;
        (found.trim() == key).then(|| value.trim().to_string())
    })
}

/// Where the installer puts the icon it ships, as the path that file will have
/// on the machine, or `None` when it ships no icon at all.
///
/// Both halves come off one `[Files]` line. `DestDir` gives the folder, and the
/// file name is `DestName` where the line sets one and the end of `Source`
/// where it does not, which is how Inno resolves it.
fn the_installed_icon(script: &str) -> Option<String> {
    section(script, "Files").iter().find_map(|entry| {
        let source = parameter(entry, "Source")?;
        if !source.to_ascii_lowercase().ends_with(".ico") {
            return None;
        }
        let folder = parameter(entry, "DestDir")?;
        let off_the_source = source.rsplit('\\').next().unwrap_or_default().to_string();
        let file = parameter(entry, "DestName").unwrap_or(off_the_source);
        Some(format!("{folder}\\{file}"))
    })
}

/// Every `[Icons]` entry the script carries.
fn the_shortcuts(script: &str) -> Vec<Shortcut> {
    section(script, "Icons")
        .iter()
        .filter_map(|entry| {
            Some(Shortcut {
                name: parameter(entry, "Name")?,
                icon: parameter(entry, "IconFilename"),
                task: parameter(entry, "Tasks"),
            })
        })
        .collect()
}

/// Whether every shortcut names the icon this script installs, and what is
/// wrong when one does not.
///
/// One function rather than the rule written out in the test below and again in
/// the companion beside it. Two readings of one rule is how a companion comes
/// to prove something the real check does not do, which is the failure this
/// project has already met in a document guard.
///
/// The comparison is between two whole paths and not between two presence
/// checks. Asking separately whether an icon is shipped and whether a shortcut
/// names one is satisfied by an icon installed in one folder and named in
/// another, which is the half-fix rather than the absent one and is exactly
/// what a person would see as a shortcut with no picture.
fn every_shortcut_names_the_installed_icon(script: &str) -> Result<(), String> {
    let Some(installed) = the_installed_icon(script) else {
        return Err(
            "the installer ships no .ico at all, so a shortcut has nothing to \
                    name and its picture is whatever the executable's resource table \
                    happens to hold"
                .to_string(),
        );
    };

    let shortcuts = the_shortcuts(script);
    if shortcuts.is_empty() {
        return Err("no [Icons] entry was read, so this rule looked at nothing".to_string());
    }

    let wrong: Vec<String> = shortcuts
        .iter()
        .filter(|shortcut| shortcut.icon.as_deref() != Some(installed.as_str()))
        .map(|shortcut| match &shortcut.icon {
            Some(named) => format!("{} names {named}", shortcut.name),
            None => format!("{} names no icon at all", shortcut.name),
        })
        .collect();

    if wrong.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the installer puts the icon at {installed}, and these shortcuts do not name it:\n  {}",
        wrong.join("\n  ")
    ))
}

// ---------------------------------------------------------------------------
// The shortcuts and their icon
// ---------------------------------------------------------------------------

/// Both shortcuts name the icon file the installer put there.
///
/// The picture this changes is nobody's: `build.rs` embeds `assets/icon.ico`
/// into the executable, and Inno's own help says that a shortcut with no
/// `IconFilename` gets the file's default icon, so both shortcuts already
/// showed it. What this holds is that the picture no longer depends on the
/// executable's resource table being right, which has failed here before:
/// `build.rs:30` records the executable having had no icon at all.
#[test]
fn test_the_icon_each_shortcut_names_is_the_one_the_installer_ships() {
    let script = the_installer_script();
    let shortcuts = the_shortcuts(&script);

    // Both of the shortcuts that exist were really read. Without this the rule
    // below is satisfied by a reading that found one of them, or none.
    for where_it_goes in ["{group}", "{autodesktop}"] {
        assert!(
            shortcuts
                .iter()
                .any(|shortcut| shortcut.name.starts_with(where_it_goes)),
            "no shortcut under {where_it_goes} was read, so this test is looking at nothing"
        );
    }

    if let Err(wrong) = every_shortcut_names_the_installed_icon(&script) {
        panic!("{wrong}");
    }
}

/// Apps and Features shows the same file.
///
/// `UninstallDisplayIcon` had the same dependence on the resource table that
/// the two shortcuts had, and it is the picture beside the entry somebody goes
/// to when they want the program gone.
#[test]
fn test_apps_and_features_shows_the_icon_the_installer_ships() {
    let script = the_installer_script();
    let installed = the_installed_icon(&script);

    // Before the comparison, because two absent values are equal and the
    // comparison alone would pass on a script that ships no icon and names
    // none.
    assert!(
        installed.is_some(),
        "the installer ships no .ico at all, so there is nothing for Apps and \
         Features to be shown"
    );

    assert_eq!(
        setup_directive(&script, "UninstallDisplayIcon").as_deref(),
        installed.as_deref(),
        "Apps and Features is pointed at a file the installer does not put there"
    );
}

/// The desktop shortcut is still somebody's choice.
///
/// Green when it was written and here so that it stays green. Nothing in this
/// plan touched the task, and an edit that gave the desktop entry its icon by
/// rewriting the line could take the gate off it without anybody noticing.
#[test]
fn test_the_desktop_shortcut_is_still_the_users_choice() {
    let script = the_installer_script();
    let shortcuts = the_shortcuts(&script);
    let desktop = shortcuts
        .iter()
        .find(|shortcut| shortcut.name.starts_with("{autodesktop}"));

    assert!(
        desktop.is_some(),
        "no desktop shortcut was read, so this test is looking at nothing"
    );
    assert_eq!(
        desktop.and_then(|shortcut| shortcut.task.as_deref()),
        Some("desktopicon"),
        "the desktop shortcut is no longer gated on the task, so unticking the \
         box would leave one there anyway"
    );
}

/// The reading can tell a shortcut that names the shipped icon from one that
/// does not.
///
/// The companion the walk above cannot do without. That walk reads one script,
/// and while that script obeys the rule it passes whether the reading works or
/// has been narrowed until it can see nothing, which is how a document guard in
/// this tree came to prove nothing at all. These four fixtures make the
/// difference visible, and the second is the break the guard record applies.
#[test]
fn test_the_reading_can_tell_a_shortcut_that_names_the_shipped_icon_from_one_that_does_not() {
    const SHIPPED_AND_NAMED: &str = r#"
[Files]
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"; IconFilename: "{app}\icon.ico"
"#;

    const SHIPPED_SOMEWHERE_ELSE: &str = r#"
[Files]
Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"; IconFilename: "{app}\icon.ico"
"#;

    const SHIPPED_AND_NOT_NAMED: &str = r#"
[Files]
Source: "..\assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"
"#;

    const NOT_SHIPPED_AT_ALL: &str = r#"
[Files]
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"; IconFilename: "{app}\icon.ico"
"#;

    assert_eq!(
        every_shortcut_names_the_installed_icon(SHIPPED_AND_NAMED),
        Ok(())
    );

    let somewhere_else = every_shortcut_names_the_installed_icon(SHIPPED_SOMEWHERE_ELSE)
        .expect_err("an icon installed in one folder and named in another has to be refused");
    assert!(
        somewhere_else.contains(r"{app}\assets\icon.ico"),
        "{somewhere_else}"
    );

    let not_named = every_shortcut_names_the_installed_icon(SHIPPED_AND_NOT_NAMED)
        .expect_err("a shortcut naming no icon has to be refused");
    assert!(not_named.contains("names no icon at all"), "{not_named}");

    let not_shipped = every_shortcut_names_the_installed_icon(NOT_SHIPPED_AT_ALL)
        .expect_err("a shortcut naming an icon nothing installs has to be refused");
    assert!(
        not_shipped.contains("ships no .ico at all"),
        "{not_shipped}"
    );
}
