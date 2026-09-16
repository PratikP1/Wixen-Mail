//! What the installer script and the release workflow say, read as text.
//!
//! `installer/Wixen-Mail-Setup.iss` is the artefact and it cannot be run from
//! here. Nothing in this repository compiles it with ISCC, installs anything,
//! or looks at a shortcut on a real machine, so every test in this file is a
//! claim about what the script says and not one of them is a claim about what
//! an install does. A green run here does not mean a shortcut on somebody's
//! desktop shows the right picture. `.planning/WINDOWS.md` carries that as a
//! verification nobody has run.
//!
//! The same applies twice over to `.github/workflows/release.yml`, added to
//! this file on 2026-09-12. No release has ever been cut from this repository:
//! `git tag` returns nothing and so does `git ls-remote --tags origin`, so the
//! tag branch in `scripts/build-installer.sh` has never been taken and not one
//! of the globs the workflow publishes has ever been matched against a real
//! `dist/`. What the tests below hold is that the names the workflow promises
//! and the names the build really writes agree on paper. Whether GitHub then
//! behaves as the file says is unknown until somebody dispatches a release.
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
//!
//! # The census says what needs signing, not that anything is signed
//!
//! The last section of this file counts the things a signature has to name. It
//! is a reading of two lists, so it knows what the installer carries and what
//! the release publishes, and it knows nothing whatever about a certificate.
//! **Nothing in this project is signed.** There is no key: the account that
//! would issue one is plan 07-08's task 2 and only a person can create it. A
//! green census is the question stated correctly, which is worth having
//! before task 3 answers it and worthless if mistaken for the answer.

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

// ---------------------------------------------------------------------------
// The release workflow, and the files it promises
// ---------------------------------------------------------------------------

/// The name of the step that stops a release which cannot keep its promises.
///
/// Named once, because the rule below and the message it prints when the step
/// is missing both have to say the same string, and a step renamed in the
/// workflow without this moving would read as the step having been deleted.
const THE_CHECK: &str = "Check that every promised file exists";

/// The release workflow, read from the repository root.
fn the_release_workflow() -> String {
    std::fs::read_to_string(".github/workflows/release.yml").expect("the release workflow")
}

/// This package's version, off its own manifest.
///
/// `scripts/build-installer.sh:12` reads the same line the same way and hands
/// the number to ISCC, so this is what ends up in the setup executable's name.
fn the_version(manifest: &str) -> String {
    manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = "))
        .map(|value| value.trim().trim_matches('"').to_string())
        .expect("a version at the top of Cargo.toml")
}

/// The lines of a YAML block scalar, given the key that opens it.
///
/// A block runs from `key: |` to the first line indented no further than the
/// key itself. Blank lines and comments are dropped, so the list can be
/// commented without a comment being read as a member of it.
///
/// Absent key, empty answer. Every caller treats that as a failure with its own
/// message, because a reading that found nothing and a file that promises
/// nothing read alike.
fn block_scalar<'a>(yaml: &'a str, key: &str) -> Vec<&'a str> {
    let opener = format!("{key}: |");
    let mut lines = yaml.lines().skip_while(|line| line.trim() != opener);
    let Some(header) = lines.next() else {
        return Vec::new();
    };
    let depth = header.len() - header.trim_start().len();
    lines
        .take_while(|line| line.trim().is_empty() || line.len() - line.trim_start().len() > depth)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// Every file the release promises to publish, taken off the one list the
/// publishing step really reads.
///
/// The indirection is followed rather than assumed. `files:` either opens a
/// block of its own or names an `env` entry that does, and the publishing step
/// reads whichever it is. Following it is what lets the workflow keep a single
/// copy of the list, which is the only way the step that checks the files exist
/// and the step that publishes them cannot drift apart.
fn the_promised_files(yaml: &str) -> Vec<String> {
    let named_env = yaml.lines().find_map(|line| {
        let value = line.trim().strip_prefix("files:")?.trim();
        let inside = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
        inside.strip_prefix("env.").map(str::to_string)
    });
    let key = named_env.unwrap_or_else(|| "files".to_string());
    block_scalar(yaml, &key)
        .iter()
        .map(|line| (*line).to_string())
        .collect()
}

/// The lines of one named workflow step, from its own `- name:` to the next.
fn step<'a>(yaml: &'a str, name: &str) -> Vec<&'a str> {
    let header = format!("- name: {name}");
    yaml.lines()
        .skip_while(|line| line.trim() != header)
        .skip(1)
        .take_while(|line| !line.trim().starts_with("- name:"))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

/// The steps of the workflow in the order they run.
fn the_steps(yaml: &str) -> Vec<&str> {
    yaml.lines()
        .filter_map(|line| line.trim().strip_prefix("- name: "))
        .collect()
}

/// Every `dist/` file name the release really writes, with the tag and the
/// version filled in.
///
/// Both halves are read off the files that write them rather than kept as a
/// second copy here. A second copy is the failure this whole rule is about: two
/// lists that agree until one of them is edited.
fn the_names_the_release_writes(yaml: &str, script: &str, tag: &str, version: &str) -> Vec<String> {
    let mut written = Vec::new();

    // The setup executable. Inno writes it into `OutputDir` under
    // `OutputBaseFilename` and appends the extension itself, and
    // `scripts/build-installer.sh` hands it the version off `Cargo.toml`. At a
    // tag that version carries no `+build` suffix, because build-installer.sh
    // appends one only when HEAD is not a tag.
    if let (Some(folder), Some(base)) = (
        setup_directive(script, "OutputDir"),
        setup_directive(script, "OutputBaseFilename"),
    ) {
        let folder = folder.trim_start_matches("..\\").replace('\\', "/");
        let base = base.replace("{#AppVersion}", version);
        written.push(format!("{folder}/{base}.exe"));
    }

    // The portable copy and its zip, off the step that writes them.
    for line in step(yaml, "Prepare the portable download") {
        written.extend(
            line.split('"')
                .filter(|piece| piece.starts_with("dist/"))
                .map(|piece| piece.replace("$tag", tag)),
        );
    }

    written
}

/// Whether a glob matches a name, with `*` standing for any run of characters
/// including none.
///
/// The patterns in the workflow use nothing else, and `*` here does not stop at
/// a path separator because none of them needs it to.
fn glob_matches(glob: &str, name: &str) -> bool {
    let mut parts = glob.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = name.strip_prefix(first) else {
        return false;
    };

    let parts: Vec<&str> = parts.collect();
    let Some((last, middles)) = parts.split_last() else {
        // No wildcard at all, so the whole glob had to be the whole name.
        return rest.is_empty();
    };
    for middle in middles {
        match rest.find(middle) {
            Some(at) => rest = &rest[at + middle.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

/// Whether every file the release promises is one the release really produces,
/// and what is wrong when one is not.
///
/// **Two categories, not one, and the second names exactly what it excuses.**
/// Three of the four promised files are built and then published; the fourth,
/// `docs/changelog.md`, is a tracked file this repository already holds and
/// nothing builds it. A rule holding all four to "something writes this" fails
/// on the fourth forever, and a rule with a default category lets a fifth glob
/// added later join whichever category happens to be the default. So the
/// exception is a list of exact paths, and it is checked in both directions: a
/// path excused here that nothing publishes is as wrong as a glob nothing
/// builds, because that is how the exception rots without anybody seeing it.
///
/// The caller is left to say that an excused file really exists. This is where
/// the promise and the build are held together; whether a tracked file is on
/// the disk is a different question and reads better where it is asked.
fn every_promised_file_is_one_the_release_produces(
    promised: &[String],
    written: &[String],
    published_as_it_stands: &[&str],
) -> Result<(), String> {
    if promised.is_empty() {
        return Err(
            "no list of published files was read at all, so this rule looked \
                    at nothing"
                .to_string(),
        );
    }
    if written.is_empty() {
        return Err(
            "no name the release writes was read at all, so every glob would \
                    have been judged against nothing"
                .to_string(),
        );
    }

    let mut wrong: Vec<String> = promised
        .iter()
        .filter(|glob| !published_as_it_stands.contains(&glob.as_str()))
        .filter(|glob| !written.iter().any(|name| glob_matches(glob, name)))
        .map(|glob| {
            format!("{glob} matches nothing the release writes, and is not one of the files published as they stand")
        })
        .collect();

    wrong.extend(
        published_as_it_stands
            .iter()
            .filter(|excused| !promised.iter().any(|glob| glob == *excused))
            .map(|excused| {
                format!("{excused} is excused from being built and nothing publishes it")
            }),
    );

    if wrong.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the release writes these names:\n  {}\nand these promises do not line up with them:\n  {}",
        written.join("\n  "),
        wrong.join("\n  ")
    ))
}

/// Whether a release that cannot produce a file it promised stops before it
/// publishes anything, and what is wrong when it does not.
///
/// Two nets, and the order between them is the whole point. The first is a step
/// that checks the promised files exist, after they are built and before
/// anything is announced. The second is `fail_on_unmatched_files`, which fires
/// inside the publishing step itself, by which point the tag is on the remote
/// and the release exists; a missing asset caught only there is already a
/// half-published release. So the second is worth having and is not a
/// substitute for the first.
fn a_release_that_cannot_keep_a_promise_stops(yaml: &str) -> Result<(), String> {
    let steps = the_steps(yaml);
    let at = |name: &str| steps.iter().position(|step| *step == name);

    let Some(built) = at("Prepare the portable download") else {
        return Err(
            "no step called 'Prepare the portable download' was read, so this \
                    rule has no build step to be after"
                .to_string(),
        );
    };
    let Some(published) = at("Publish GitHub release assets") else {
        return Err(
            "no step called 'Publish GitHub release assets' was read, so this \
                    rule has no publishing step to be before"
                .to_string(),
        );
    };
    let Some(checked) = at(THE_CHECK) else {
        return Err(format!(
            "no step called '{THE_CHECK}' was read, so a release that cannot produce \
             a file it promised publishes the rest and says nothing"
        ));
    };

    if checked < built {
        return Err(format!(
            "'{THE_CHECK}' runs before the files it is checking for are built"
        ));
    }
    if checked > published {
        return Err(format!(
            "'{THE_CHECK}' runs after the release has been published, which is too \
             late to stop it"
        ));
    }
    if !yaml.contains("fail_on_unmatched_files: true") {
        return Err("fail_on_unmatched_files is not true, so a glob that stops \
                    matching is published as a silent absence"
            .to_string());
    }
    Ok(())
}

/// Every trigger the workflow carries, by name.
fn the_release_triggers(yaml: &str) -> Vec<String> {
    let body: Vec<&str> = yaml
        .lines()
        .skip_while(|line| *line != "on:")
        .skip(1)
        .take_while(|line| line.trim().is_empty() || line.starts_with(char::is_whitespace))
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect();

    let Some(depth) = body
        .iter()
        .map(|line| line.len() - line.trim_start().len())
        .min()
    else {
        return Vec::new();
    };
    body.iter()
        .filter(|line| line.len() - line.trim_start().len() == depth)
        .map(|line| line.trim().trim_end_matches(':').to_string())
        .collect()
}

/// Whether a release can still only be started by somebody asking for one.
///
/// Guardrail 7, and the comment at the top of the workflow says why it is
/// there rather than leaving it to be inferred: a push to `main` used to
/// trigger this file, which cut two releases nobody asked for and promoted an
/// alpha to beta.
fn nothing_starts_a_release_but_a_person(yaml: &str) -> Result<(), String> {
    let triggers = the_release_triggers(yaml);
    if triggers.is_empty() {
        return Err("no trigger was read at all, so this rule looked at nothing".to_string());
    }
    let uninvited: Vec<&str> = triggers
        .iter()
        .map(String::as_str)
        .filter(|trigger| *trigger != "workflow_dispatch")
        .collect();
    if uninvited.is_empty() {
        return Ok(());
    }
    Err(format!(
        "a release can be started by something other than somebody asking for one: {}",
        uninvited.join(", ")
    ))
}

// ---------------------------------------------------------------------------
// What the release promises against what it builds
// ---------------------------------------------------------------------------

/// Nothing here moves the published tag away from the shape one glob expects.
///
/// `dist/wixen-mail-$tag.exe` is written and `dist/wixen-mail-v*.exe` is
/// published, so the two agree only while the tag begins with `v`. No tag has
/// ever existed here, so the shape comes from cargo-release's defaults, whose
/// `tag-name` is `{{prefix}}v{{version}}` with an empty `tag-prefix` at a
/// repository root.
///
/// Those defaults apply only while nothing overrides them, and overriding them
/// is a one-line edit in a file this test names. **That is where the shape of
/// the string is really decided**, rather than in the code that produces the
/// value or in the glob that consumes it, and it is the half a reader would
/// not think to check. The assumption itself stays unverified until a release
/// is really cut, which `.planning/WINDOWS.md` carries.
///
/// Green when it was written, so it has no red half of its own and its break
/// is recorded in `guards/guards.toml` instead.
#[test]
fn test_nothing_here_moves_the_published_tag_away_from_the_shape_a_glob_expects() {
    for config in ["release.toml", ".cargo/release.toml"] {
        assert!(
            !std::path::Path::new(config).exists(),
            "{config} exists and can set tag-name or tag-prefix, so the published tag \
             may no longer begin with v, and dist/wixen-mail-v*.exe would then match \
             nothing"
        );
    }

    let manifest = std::fs::read_to_string("Cargo.toml").expect("the manifest");
    assert!(
        !manifest.contains("[package.metadata.release]"),
        "Cargo.toml carries a [package.metadata.release] section, which can set \
         tag-name or tag-prefix, so the published tag may no longer begin with v, and \
         dist/wixen-mail-v*.exe would then match nothing"
    );
}

/// Every file the release promises is one it really produces.
///
/// The two ends are read rather than compared by eye: the promises come off the
/// list the publishing step uses, and the names come off the installer script's
/// own output directives and off the step that writes the portable copy. Under
/// the tag shape the test above holds, they line up.
#[test]
fn test_every_file_the_release_promises_is_one_it_really_produces() {
    let workflow = the_release_workflow();
    let script = the_installer_script();
    let manifest = std::fs::read_to_string("Cargo.toml").expect("the manifest");

    let version = the_version(&manifest);
    let tag = format!("v{version}");
    let promised = the_promised_files(&workflow);
    let written = the_names_the_release_writes(&workflow, &script, &tag, &version);

    // The reading really found all four promises and all three names. Without
    // this the rule below is satisfied by a reading that found one of each, or
    // by an excuse list that grew to cover everything.
    assert_eq!(
        promised.len(),
        4,
        "expected four promised files, read {promised:?}"
    );
    assert_eq!(
        written.len(),
        3,
        "expected three names the release writes, read {written:?}"
    );

    if let Err(wrong) =
        every_promised_file_is_one_the_release_produces(&promised, &written, PUBLISHED_AS_IT_STANDS)
    {
        panic!("{wrong}");
    }

    // The other half of the excuse, asked where it belongs: a file published as
    // it stands has to be a file that stands there.
    for excused in PUBLISHED_AS_IT_STANDS {
        assert!(
            std::path::Path::new(excused).exists(),
            "{excused} is published as it stands and is not in the repository"
        );
    }
}

/// The files the release publishes without building them.
///
/// Exactly one today. `docs/changelog.md` is a tracked file and the release
/// hands out the copy already in the repository.
const PUBLISHED_AS_IT_STANDS: &[&str] = &["docs/changelog.md"];

/// The reading can tell a promise the release keeps from one it does not.
///
/// The companion the walk above cannot do without. That walk reads one
/// workflow, and while that workflow keeps its promises it passes whether the
/// reading works or has been narrowed until it can see nothing, which is how a
/// document guard in this tree came to prove nothing at all.
#[test]
fn test_the_reading_can_tell_a_promise_the_release_keeps_from_one_it_does_not() {
    let built = ["dist/Wixen-Mail-Setup-0.1.0.exe".to_string()];
    let excused: &[&str] = &["docs/changelog.md"];

    let kept = [
        "dist/Wixen-Mail-Setup-*.exe".to_string(),
        "docs/changelog.md".to_string(),
    ];
    assert_eq!(
        every_promised_file_is_one_the_release_produces(&kept, &built, excused),
        Ok(())
    );

    // The half-fix rather than the absent one, and the shape the real failure
    // would take: the glob is still there and still looks like a file name.
    let mistyped = [
        "dist/Wixen-Mail-Setup-v*.exe".to_string(),
        "docs/changelog.md".to_string(),
    ];
    let wrong = every_promised_file_is_one_the_release_produces(&mistyped, &built, excused)
        .expect_err("a glob that matches nothing the release writes has to be refused");
    assert!(wrong.contains("dist/Wixen-Mail-Setup-v*.exe"), "{wrong}");

    // An excuse that outlived the thing it excused.
    let dropped = ["dist/Wixen-Mail-Setup-*.exe".to_string()];
    let rotted = every_promised_file_is_one_the_release_produces(&dropped, &built, excused)
        .expect_err("an excused file nothing publishes has to be refused");
    assert!(rotted.contains("nothing publishes it"), "{rotted}");

    // Two readings that found nothing, which without these read exactly like a
    // release that keeps every promise it makes.
    let nothing_promised = every_promised_file_is_one_the_release_produces(&[], &built, &[])
        .expect_err("a reading that found no promises has to be refused");
    assert!(
        nothing_promised.contains("looked at nothing"),
        "{nothing_promised}"
    );

    let nothing_built = every_promised_file_is_one_the_release_produces(&kept, &[], excused)
        .expect_err("a reading that found no built names has to be refused");
    assert!(
        nothing_built.contains("judged against nothing"),
        "{nothing_built}"
    );
}

// ---------------------------------------------------------------------------
// Stopping rather than publishing what it has
// ---------------------------------------------------------------------------

/// A release that cannot produce a file it promised stops before it publishes.
#[test]
fn test_a_release_that_cannot_produce_a_promised_file_stops_before_it_publishes() {
    if let Err(wrong) = a_release_that_cannot_keep_a_promise_stops(&the_release_workflow()) {
        panic!("{wrong}");
    }
}

/// The reading can see a release that publishes before it checks.
#[test]
fn test_the_reading_can_see_a_release_that_publishes_before_it_checks() {
    let steps = |names: &[&str]| {
        names
            .iter()
            .map(|name| format!("      - name: {name}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let net = "          fail_on_unmatched_files: true";

    let in_order = format!(
        "{}\n{net}\n",
        steps(&[
            "Prepare the portable download",
            THE_CHECK,
            "Publish GitHub release assets",
        ])
    );
    assert_eq!(
        a_release_that_cannot_keep_a_promise_stops(&in_order),
        Ok(())
    );

    let too_late = format!(
        "{}\n{net}\n",
        steps(&[
            "Prepare the portable download",
            "Publish GitHub release assets",
            THE_CHECK,
        ])
    );
    let after = a_release_that_cannot_keep_a_promise_stops(&too_late)
        .expect_err("a check after the release is published has to be refused");
    assert!(after.contains("too late to stop it"), "{after}");

    let too_early = format!(
        "{}\n{net}\n",
        steps(&[
            THE_CHECK,
            "Prepare the portable download",
            "Publish GitHub release assets",
        ])
    );
    let before = a_release_that_cannot_keep_a_promise_stops(&too_early)
        .expect_err("a check before the files are built has to be refused");
    assert!(
        before.contains("before the files it is checking for are built"),
        "{before}"
    );

    let unchecked = format!(
        "{}\n{net}\n",
        steps(&[
            "Prepare the portable download",
            "Publish GitHub release assets",
        ])
    );
    let missing = a_release_that_cannot_keep_a_promise_stops(&unchecked)
        .expect_err("a release with no check at all has to be refused");
    assert!(
        missing.contains("publishes the rest and says nothing"),
        "{missing}"
    );

    // The second net on its own, with the first in place. Turning the flag off
    // is the half-fix: every step is where it should be and a glob that stops
    // matching is still published as an absence.
    let no_net = format!(
        "{}\n          fail_on_unmatched_files: false\n",
        steps(&[
            "Prepare the portable download",
            THE_CHECK,
            "Publish GitHub release assets",
        ])
    );
    let silent = a_release_that_cannot_keep_a_promise_stops(&no_net)
        .expect_err("an unmatched glob published as an absence has to be refused");
    assert!(silent.contains("silent absence"), "{silent}");
}

// ---------------------------------------------------------------------------
// When a release can happen
// ---------------------------------------------------------------------------

/// A release still happens only when somebody asks for one.
///
/// Green when it was written, and here so that it stays green: guardrail 7 is
/// the one this project has already been bitten by, and the bite was a trigger
/// added to this block. Its break is recorded in `guards/guards.toml`, because
/// a guard written over a rule that already holds has no red half of its own.
#[test]
fn test_a_release_still_happens_only_when_somebody_asks_for_one() {
    if let Err(wrong) = nothing_starts_a_release_but_a_person(&the_release_workflow()) {
        panic!("{wrong}");
    }
}

/// The reading can see a trigger that was widened.
#[test]
fn test_the_reading_can_see_a_release_trigger_that_was_widened() {
    const ASKED_FOR: &str = "on:\n  workflow_dispatch:\n    inputs:\n      release_level:\n        required: true\n\njobs:\n";
    const ON_A_PUSH: &str = "on:\n  workflow_dispatch:\n  push:\n    branches: [main]\n\njobs:\n";
    const NO_TRIGGER_READ: &str = "jobs:\n  build:\n";

    assert_eq!(nothing_starts_a_release_but_a_person(ASKED_FOR), Ok(()));

    let widened = nothing_starts_a_release_but_a_person(ON_A_PUSH)
        .expect_err("a release a push can start has to be refused");
    assert!(widened.contains("push"), "{widened}");

    let blind = nothing_starts_a_release_but_a_person(NO_TRIGGER_READ)
        .expect_err("a reading that found no trigger has to be refused");
    assert!(blind.contains("looked at nothing"), "{blind}");
}

// ---------------------------------------------------------------------------
// The four-number version Windows shows
// ---------------------------------------------------------------------------

/// The installer build script, read from the repository root.
fn the_installer_build_script() -> String {
    std::fs::read_to_string("scripts/build-installer.sh").expect("the installer build script")
}

/// What each prerelease word puts in the thousands of the fourth field, and
/// what a plain version gets, read off the script's own `case` arms.
#[derive(Debug, PartialEq, Eq)]
struct StageTable {
    alpha: u64,
    beta: u64,
    rc: u64,
    plain: u64,
}

/// The number one `case` arm assigns to `stage`, given the pattern that opens
/// the arm, or `None` when no such arm is in the script.
fn stage_arm(script: &str, pattern: &str) -> Option<u64> {
    script.lines().find_map(|line| {
        let assignment = line.trim().strip_prefix(pattern)?.trim();
        let value = assignment.strip_prefix("stage=")?;
        value.split(char::is_whitespace).next()?.parse().ok()
    })
}

/// The four arms the encoding needs, each read off the script, and the name of
/// the first one that is missing.
///
/// Read rather than copied, because a second copy of the arithmetic held to
/// nothing is the failure this file is about: two versions of one rule that
/// agree until one of them is edited. The multiplier and the cap are held by
/// the test that uses this, which asks the script for the expression itself.
fn the_stage_table(script: &str) -> Result<StageTable, String> {
    let arm = |pattern: &str| {
        stage_arm(script, pattern).ok_or_else(|| {
            format!(
                "no `{pattern} stage=N` arm was read off scripts/build-installer.sh, so the \
                 fourth field of a version of that shape is no longer computed"
            )
        })
    };
    Ok(StageTable {
        alpha: arm("*-alpha.*)")?,
        beta: arm("*-beta.*)")?,
        rc: arm("*-rc.*)")?,
        plain: arm("*)")?,
    })
}

/// The four numbers Windows shows for a version, computed the way the script
/// computes them: the three numbers as they are, and a fourth of
/// `stage * 1000 + step`, with the step capped at 999 so it can never read as
/// the next stage.
///
/// Ordered by the derived `Ord` on the array, which compares the four fields
/// as numbers left to right. That is the order Windows applies to a file
/// version, so a comparison of two of these is a comparison of the two
/// installs as Apps and Features would rank them.
fn windows_file_version(table: &StageTable, version: &str) -> [u64; 4] {
    let (numbers, prerelease) = match version.split_once('-') {
        Some((numbers, prerelease)) => (numbers, Some(prerelease)),
        None => (version, None),
    };
    let mut fields = numbers
        .split('.')
        .map(|field| field.parse::<u64>().expect("a whole number in a version"));
    let mut next = || fields.next().expect("three numbers in a version");
    let (major, minor, patch) = (next(), next(), next());

    let (stage, step) = match prerelease {
        None => (table.plain, 0),
        Some(prerelease) => {
            let (word, counter) = prerelease
                .split_once('.')
                .expect("a prerelease word, a dot and a counter");
            let stage = match word {
                "alpha" => table.alpha,
                "beta" => table.beta,
                "rc" => table.rc,
                other => panic!("{other} is not a prerelease word the script names"),
            };
            let step: u64 = counter
                .parse()
                .expect("a whole number after the prerelease word");
            (stage, step.min(999))
        }
    };
    [major, minor, patch, stage * 1000 + step]
}

/// The encoding here agrees with the script's own comment table, and with the
/// two versions this project moved between on 2026-09-16.
///
/// The table is the one at `scripts/build-installer.sh` above the `case`, and
/// this holds the reading to it row by row. The stage numbers come off the
/// script; the multiplier and the cap are asserted to still be in it, so a
/// script that stopped multiplying by a thousand, or stopped capping the
/// step, fails here by name rather than by a number that quietly disagrees.
#[test]
fn test_the_four_field_version_follows_the_scripts_own_table() {
    let script = the_installer_build_script();
    let table = the_stage_table(&script).unwrap_or_else(|missing| panic!("{missing}"));

    assert!(
        script.contains("$((stage * 1000 + step))"),
        "the fourth field is no longer stage * 1000 + step, so the reading here is a copy \
         of arithmetic the script no longer does"
    );
    assert!(
        script.contains(r#"[ "$step" -gt 999 ] && step=999"#),
        "the step is no longer capped at 999, so a step of 1000 would read as the next stage"
    );

    let encoded = |version: &str| windows_file_version(&table, version);
    assert_eq!(encoded("0.5.0"), [0, 5, 0, 4000]);
    assert_eq!(encoded("0.6.0-alpha.1"), [0, 6, 0, 1001]);
    assert_eq!(encoded("0.6.0-beta.2"), [0, 6, 0, 2002]);
    assert_eq!(encoded("0.6.0-rc.1"), [0, 6, 0, 3001]);
    assert_eq!(encoded("0.6.0"), [0, 6, 0, 4000]);

    assert_eq!(encoded("1.0.0-alpha.1"), [1, 0, 0, 1001]);
    assert_eq!(encoded("0.125.1"), [0, 125, 1, 4000]);
}

/// The first alpha of 1.0.0 is above the last 0.x version the way Windows
/// orders an upgrade.
///
/// Not obvious from the fourth field alone: a plain version lands on 4000 and
/// an alpha on 1001, so on that field the old version is the higher one. The
/// major decides first, which is what makes the step from `0.125.1` to
/// `1.0.0-alpha.1` an upgrade in Apps and Features rather than a downgrade
/// the installer would refuse to make.
#[test]
fn test_the_first_alpha_of_1_0_0_sits_above_the_last_0_x_version_the_way_windows_orders_it() {
    let script = the_installer_build_script();
    let table = the_stage_table(&script).unwrap_or_else(|missing| panic!("{missing}"));

    let first_alpha = windows_file_version(&table, "1.0.0-alpha.1");
    let last_of_0_x = windows_file_version(&table, "0.125.1");

    assert!(
        first_alpha[3] < last_of_0_x[3],
        "the fourth field alone would put the old version above the new one, which is what \
         makes the field-by-field order the thing worth holding: {first_alpha:?} against \
         {last_of_0_x:?}"
    );
    assert!(
        first_alpha > last_of_0_x,
        "Windows would rank {last_of_0_x:?} above {first_alpha:?}, and refuse the upgrade"
    );
}

/// The reading can see a stage arm that is gone.
///
/// The companion the two tests above cannot do without. They read one script,
/// and while that script has its four arms they pass whether the reading
/// works or has been narrowed until it finds a number anywhere, which is how
/// a document guard in this tree came to prove nothing at all.
#[test]
fn test_the_reading_can_see_a_stage_arm_that_is_gone() {
    const ALL_FOUR: &str = "case \"$VERSION\" in\n  *-alpha.*) stage=1 ;;\n  *-beta.*) stage=2 ;;\n  *-rc.*) stage=3 ;;\n  *-*) stage=0 ;;\n  *) stage=4 ;;\nesac\n";
    const NO_ALPHA: &str = "case \"$VERSION\" in\n  *-beta.*) stage=2 ;;\n  *-rc.*) stage=3 ;;\n  *-*) stage=0 ;;\n  *) stage=4 ;;\nesac\n";

    assert_eq!(
        the_stage_table(ALL_FOUR),
        Ok(StageTable {
            alpha: 1,
            beta: 2,
            rc: 3,
            plain: 4
        })
    );

    let missing =
        the_stage_table(NO_ALPHA).expect_err("a script with no alpha arm has to be refused");
    assert!(missing.contains("*-alpha.*)"), "{missing}");
}

/// The version the tree carries is a 1.x version.
///
/// Pratik decided on 2026-09-15 (#46) that the builds going to testers are
/// the alpha, beta and release-candidate stages of 1.0.0, so the scheme
/// cannot drift back to `0.x` without a test saying so. Red before the bump
/// to `1.0.0-alpha.1` and green after it.
#[test]
fn test_the_version_the_tree_carries_is_at_least_one_point_oh() {
    let manifest = std::fs::read_to_string("Cargo.toml").expect("the manifest");
    let version = the_version(&manifest);
    let major: u64 = version
        .split('.')
        .next()
        .and_then(|major| major.parse().ok())
        .unwrap_or_else(|| panic!("{version} does not start with a whole number"));

    assert!(
        major >= 1,
        "Cargo.toml says {version}, and the builds going to testers are the stages of 1.0.0"
    );
}

// ---------------------------------------------------------------------------
// What has to be signed
// ---------------------------------------------------------------------------

/// One thing this project hands to somebody, which a signature has to name.
#[derive(Debug, PartialEq, Eq)]
struct Artefact {
    /// Spelled the way the list it came off spells it: a `Source:` path for
    /// something inside the installer, a glob for something published beside
    /// it, and Inno's own wildcard for the uninstaller.
    ///
    /// Not reduced to a bare file name. A signing step has to name a path it
    /// can find the file at, and the same executable sits in `target` under one
    /// name and in `dist` under two others.
    name: String,
    origin: Origin,
}

/// Where an artefact comes from, which is what decides who can sign it.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Origin {
    /// A `[Files]` entry. Built here, carried inside the setup executable, and
    /// run from the machine it lands on.
    InsideTheInstaller,
    /// A glob in the list the release publishes. Downloaded on its own, so
    /// nothing else vouches for it.
    PublishedBesideIt,
    /// Written by Inno rather than shipped by anything, so it is in neither
    /// list. It is the one a census taken off the two lists misses.
    WrittenByInno,
}

/// Everything this project hands to somebody that a signature has to name.
///
/// **Derived from the two lists that decide it, never typed out.** A test
/// holding seven strings passes when an eighth executable is added to the
/// installer, and says nothing about it; this one grows and names it. That is
/// the whole reason this is a parse rather than a constant, and SHIP-01's own
/// wording is the reason it is needed: it says "the installer and the
/// executable inside it", which is two of the seven.
fn the_artefacts_that_need_signing(script: &str, yaml: &str) -> Vec<Artefact> {
    let mut census: Vec<Artefact> = section(script, "Files")
        .iter()
        .filter_map(|entry| parameter(entry, "Source"))
        .filter(|source| authenticode_signs_it(source))
        .map(|name| Artefact {
            name,
            origin: Origin::InsideTheInstaller,
        })
        .collect();

    census.extend(
        the_promised_files(yaml)
            .into_iter()
            .filter(|glob| a_download_that_carries_code(glob))
            .map(|name| Artefact {
                name,
                origin: Origin::PublishedBesideIt,
            }),
    );

    // Inno writes an uninstaller whether or not the script mentions one, so
    // this asks whether the script turned it off rather than whether it asked
    // for it. `Uninstallable` is absent here and there is an uninstaller all
    // the same, which is exactly how the seventh artefact came to be missing
    // from a requirement that listed the other two it could see.
    //
    // Gated on `[Setup]` really having been read. Without that gate a script
    // this function cannot parse still reports an uninstaller, and a census
    // that answers "one thing" for a file it could not read is the
    // reading-found-nothing failure the rest of this file guards against
    // everywhere else.
    let setup = section(script, "Setup");
    if !setup.is_empty() && setup_directive(script, "Uninstallable").as_deref() != Some("no") {
        census.push(Artefact {
            // Inno's own spelling, from the help topic for SignedUninstaller.
            // The digits are the install's serial number, so no single name
            // exists to write down.
            name: "unins???.exe".to_string(),
            origin: Origin::WrittenByInno,
        });
    }

    census
}

/// The file extensions Authenticode embeds a signature into.
///
/// Two, and the shortness is the point rather than an oversight. Authenticode
/// writes its signature into a PE file's certificate table, and on Windows a
/// PE file is an `.exe` or a `.dll`. Every file this project ships that Windows
/// loads and runs is one of the two.
///
/// **What it deliberately does not cover, and what that costs.** A `.ps1`, a
/// `.cat` or an `.msi` can also carry a signature, by a different mechanism in
/// each case, and none of the three is signed the way a PE file is. This
/// project ships none of them today. If one ever arrives it will not be
/// counted here, and that is a hole rather than a decision: it is recorded in
/// `.planning/WINDOWS.md` so somebody meets it rather than discovering it in a
/// download. Widening this list without also settling how each of those is
/// signed would be worse, because it would make the census look complete while
/// pairing a file with a signing step that cannot sign it.
const WHAT_AUTHENTICODE_SIGNS: [&str; 2] = [".exe", ".dll"];

/// Whether a file inside the installer is one Authenticode signs.
///
/// Read off the end of the name, because that is all a `Source:` line gives.
/// The entries this drops are `..\LICENSE`, `..\README.md`, `..\assets\icon.ico`
/// and `..\docs\*.md`: prose and a picture, none of it code, and nothing
/// Windows will ever execute.
///
/// **One thing this cannot see.** A `Source:` entry may be a wildcard, and
/// `..\docs\*.md` already is. A wildcard naming executables would be one census
/// entry standing for however many files it matched, so the count would be
/// right about the line and wrong about the artefacts. None exists today and
/// this says so rather than pretending the reading is finer than it is.
fn authenticode_signs_it(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    WHAT_AUTHENTICODE_SIGNS
        .iter()
        .any(|extension| lowered.ends_with(extension))
}

/// Whether a published download is code, or an archive whose contents are.
///
/// The archive is the awkward one and it is counted on purpose. A `.zip` cannot
/// carry an Authenticode signature at all, so the entry stands for the
/// executable inside it rather than for the container: what has to be true is
/// that the file somebody extracts is signed. Leaving it out of the census
/// because the container cannot be signed is how a download comes to be the one
/// unsigned thing on the release page.
///
/// `docs/changelog.md` is what this drops. It is published as it stands, it is
/// prose, and nothing runs it.
fn a_download_that_carries_code(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    authenticode_signs_it(&lowered) || lowered.ends_with(".zip")
}

/// Seven things have to be signed, and SHIP-01's own wording says two.
///
/// Three are carried inside the setup executable and run from the machine it
/// installs them on. Three are published beside it and downloaded on their own.
/// The seventh is written by Inno and named in neither list, which is why a
/// census taken only off the two lists would count six and read as complete.
///
/// The per-origin assertions are also this test's guard against a reading that
/// found nothing: an empty census fails three of them by name rather than
/// passing as a tree with nothing in it.
#[test]
fn test_the_census_of_what_has_to_be_signed_counts_seven_things() {
    let script = the_installer_script();
    let workflow = the_release_workflow();
    let census = the_artefacts_that_need_signing(&script, &workflow);

    let from = |origin: Origin| -> Vec<&str> {
        census
            .iter()
            .filter(|artefact| artefact.origin == origin)
            .map(|artefact| artefact.name.as_str())
            .collect()
    };

    assert_eq!(
        from(Origin::InsideTheInstaller),
        [
            r"..\target\release\wixen-mail.exe",
            r"..\search-handler\target\release\wixen_mail_search.dll",
            r"..\search-handler\target\release\wixen-mail-search-setup.exe",
        ],
        "the [Files] entries that are code have changed, and each one is \
         something a person runs from an installed folder"
    );

    assert_eq!(
        from(Origin::PublishedBesideIt),
        [
            "dist/Wixen-Mail-Setup-*.exe",
            "dist/wixen-mail-v*.exe",
            "dist/Wixen-Mail-*-windows.zip",
        ],
        "the published files that are code have changed, and each one is \
         something a person downloads on its own"
    );

    assert_eq!(
        from(Origin::WrittenByInno),
        ["unins???.exe"],
        "the uninstaller is the seventh and is in neither list, so a census \
         that stops reading the two lists loses it silently"
    );

    assert_eq!(
        census.len(),
        7,
        "seven things need signing, and this census reads {census:?}"
    );
}

/// The reading can tell a census that sees everything from one that has been
/// narrowed until it cannot.
///
/// The companion the census cannot do without. The census reads one script and
/// one workflow, and while those two hold seven things it passes whether the
/// parse works or has been narrowed to see six, which is how a document guard
/// in this tree came to prove nothing at all. These fixtures make the
/// difference visible, and the second is the break `guards/guards.toml`
/// applies.
#[test]
fn test_the_census_can_see_an_artefact_that_arrived_without_being_signed() {
    const A_SCRIPT: &str = r#"
[Setup]
OutputDir=..\dist

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"
"#;

    const AN_EIGHTH_SOURCE: &str = r#"
[Setup]
OutputDir=..\dist

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\tools\target\release\wixen-mail-helper.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"
"#;

    const NOTHING_TO_UNINSTALL: &str = r#"
[Setup]
OutputDir=..\dist
Uninstallable=no

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion
"#;

    const A_WORKFLOW: &str = r#"
    env:
      RELEASE_ASSETS: |
        dist/Wixen-Mail-Setup-*.exe
        docs/changelog.md

    steps:
      - name: Publish GitHub release assets
        with:
          files: ${{ env.RELEASE_ASSETS }}
"#;

    const A_WIDER_WORKFLOW: &str = r#"
    env:
      RELEASE_ASSETS: |
        dist/Wixen-Mail-Setup-*.exe
        dist/wixen-mail-debugger-*.exe
        docs/changelog.md

    steps:
      - name: Publish GitHub release assets
        with:
          files: ${{ env.RELEASE_ASSETS }}
"#;

    let names = |script: &str, yaml: &str| -> Vec<String> {
        the_artefacts_that_need_signing(script, yaml)
            .into_iter()
            .map(|artefact| artefact.name)
            .collect()
    };

    assert_eq!(
        the_artefacts_that_need_signing(A_SCRIPT, A_WORKFLOW),
        vec![
            Artefact {
                name: r"..\target\release\wixen-mail.exe".to_string(),
                origin: Origin::InsideTheInstaller,
            },
            Artefact {
                name: "dist/Wixen-Mail-Setup-*.exe".to_string(),
                origin: Origin::PublishedBesideIt,
            },
            Artefact {
                name: "unins???.exe".to_string(),
                origin: Origin::WrittenByInno,
            },
        ],
        "the README is prose and the changelog is prose, and neither is \
         something Authenticode can sign or Windows can run"
    );

    // An executable added to the installer without anybody thinking about
    // signing it. This is the real shape: somebody ships a new helper, the
    // installer carries it, and nothing anywhere says it is unsigned.
    let eighth = names(AN_EIGHTH_SOURCE, A_WORKFLOW);
    assert!(
        eighth.contains(&r"..\tools\target\release\wixen-mail-helper.exe".to_string()),
        "an executable added to [Files] has to appear in the census: {eighth:?}"
    );

    // The same on the other side: a new published download.
    let wider = names(A_SCRIPT, A_WIDER_WORKFLOW);
    assert!(
        wider.contains(&"dist/wixen-mail-debugger-*.exe".to_string()),
        "an executable added to the published list has to appear in the \
         census: {wider:?}"
    );

    // The uninstaller is read off the script rather than assumed, so a script
    // that generates none does not carry a phantom seventh.
    assert!(
        !names(NOTHING_TO_UNINSTALL, A_WORKFLOW).contains(&"unins???.exe".to_string()),
        "Uninstallable=no means Inno writes no uninstaller, so there is \
         nothing there to sign"
    );

    // Two readings that found nothing, which without these read exactly like a
    // project that hands nobody anything.
    assert!(
        names(A_SCRIPT, "").is_empty()
            || !names(A_SCRIPT, "").iter().any(|n| n.starts_with("dist/")),
        "a workflow that could not be read must not contribute published names"
    );
    assert!(
        names("", A_WORKFLOW)
            .iter()
            .all(|name| !name.ends_with(".dll") && name != "unins???.exe"),
        "a script that could not be read must not contribute an installed \
         file or an uninstaller"
    );
}
