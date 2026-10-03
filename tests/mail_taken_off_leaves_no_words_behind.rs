//! Every check for mail lets the search index forget the words of mail taken
//! off this computer, and nothing else does (13-44.8, D32).
//!
//! # Why this exists
//!
//! Taking a message off this computer removes its entry from the search
//! index, but the index keeps the words on its pages until those pages are
//! rewritten, and rewriting them costs the whole index each time. So it is
//! done once a check, on the check's worker, in short steps. A call that
//! moved behind the check's early return would never run for somebody whose
//! accounts are all POP, because such a check always returns there; a second
//! caller would rewrite the index somewhere nobody measured.
//!
//! # Why it reads the source
//!
//! The call is inside the `spawn_blocking` closure of `spawn_mail_sync` in
//! `wx_app.rs`, which needs a window, a frame and a runtime to reach: the
//! position `the_trash_is_emptied_on_purpose` is in, and this follows it. A
//! companion plants the call after the return to prove the reading can see
//! it.

use std::fs;

/// The window's helper, called once a check.
const THE_CALL: &str = "the_search_index_forgets_what_was_taken_off(";

/// The store's function the helper calls.
const THE_COMPACTION: &str = "let_the_search_index_forget_what_was_taken_off(";

/// The lines of a source file before its first test module, comment lines
/// left out.
fn the_shipping_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .take_while(|line| line.trim() != "#[cfg(test)]")
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect()
}

/// Whether the check calls the helper once, after the last account's watch
/// is asked for and before the return a check that went through nowhere
/// takes.
fn every_check_reaches_it(window: &str) -> Result<(), String> {
    let lines = the_shipping_lines(window);
    let start = lines
        .iter()
        .position(|line| line.starts_with("fn spawn_mail_sync("))
        .ok_or("the check, spawn_mail_sync, is gone")?;
    let end = start
        + lines[start..]
            .iter()
            .position(|line| *line == "}")
            .ok_or("the check has no end")?;
    let calls: Vec<usize> = (start..end)
        .filter(|at| lines[*at].contains(THE_CALL) && !lines[*at].contains("fn "))
        .collect();
    let [call] = calls.as_slice() else {
        return Err(format!(
            "{THE_CALL} is on {} lines of the check, where it should be on one",
            calls.len()
        ));
    };
    let last_watch = (start..end)
        .rev()
        .find(|at| lines[*at].contains("say(UIUpdate::MailboxWatchRequested("))
        .ok_or("the check asks for no watch")?;
    let the_return = (start..end)
        .find(|at| lines[*at].contains("if nothing_went_through {"))
        .ok_or("the check has no return for a check that went through nowhere")?;
    if !(last_watch < *call && *call < the_return) {
        return Err(format!(
            "{THE_CALL} is not between the last account's watch and the return a check \
             that went through nowhere takes"
        ));
    }
    Ok(())
}

/// Every source file under `src`, by its path with forward slashes.
fn every_source_file() -> Vec<String> {
    fn walk(dir: &std::path::Path, into: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, into);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                into.push(path.display().to_string().replace('\\', "/"));
            }
        }
    }
    let mut found = Vec::new();
    walk(std::path::Path::new("src"), &mut found);
    found
}

#[test]
fn test_every_check_lets_the_search_index_forget_what_was_taken_off() {
    let window = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    assert_eq!(every_check_reaches_it(&window), Ok(()));
}

#[test]
fn test_the_reading_refuses_a_check_that_returns_before_it() {
    let planted = "fn spawn_mail_sync(\n\
                   \x20       for mut account in accounts {\n\
                   \x20           say(UIUpdate::MailboxWatchRequested(account.id.clone()));\n\
                   \x20       }\n\
                   \x20       if nothing_went_through {\n\
                   \x20           return;\n\
                   \x20       }\n\
                   \x20       the_search_index_forgets_what_was_taken_off();\n\
                   }\n";
    assert!(
        every_check_reaches_it(planted).is_err(),
        "a call after the return a POP check always takes was read as in its place"
    );
}

/// Whether a parent module declares this module under `#[cfg(test)]`, so no
/// line of its file ships.
///
/// A file can be test-only at the line that declares it rather than inside
/// itself: `what_forgetting_costs.rs` measures the compaction and is compiled
/// only for tests, and read as shipping it was a second caller that never
/// runs in the program.
fn declared_only_for_tests(parent: &str, module: &str) -> bool {
    let lines: Vec<&str> = parent.lines().map(str::trim).collect();
    lines.windows(2).any(|pair| {
        pair[0] == "#[cfg(test)]"
            && (pair[1] == format!("mod {module};") || pair[1] == format!("pub mod {module};"))
    })
}

/// Whether the file at this path is a module its parent compiles only for
/// tests.
fn ships_nothing(path: &str) -> bool {
    let Some((folder, file)) = path.rsplit_once('/') else {
        return false;
    };
    let Some(module) = file.strip_suffix(".rs") else {
        return false;
    };
    [format!("{folder}/mod.rs"), format!("{folder}.rs")]
        .iter()
        .filter_map(|parent| fs::read_to_string(parent).ok())
        .any(|parent| declared_only_for_tests(&parent, module))
}

#[test]
fn test_a_module_declared_only_for_tests_is_told_from_one_that_ships() {
    let parent = "pub mod shipped;\n#[cfg(test)]\nmod measured;\n";
    assert!(declared_only_for_tests(parent, "measured"));
    assert!(!declared_only_for_tests(parent, "shipped"));
}

#[test]
fn test_nothing_else_compacts_the_search_index() {
    let source_files = every_source_file();
    assert!(
        source_files.len() > 100,
        "only {} source files were found under src, so this read almost nothing",
        source_files.len()
    );
    let elsewhere: Vec<String> = source_files
        .into_iter()
        .filter(|path| {
            !path.ends_with("src/presentation/wx_app.rs")
                && !path.ends_with("src/data/message_cache/taken_off_this_computer.rs")
                && !ships_nothing(path)
        })
        .filter(|path| {
            fs::read_to_string(path).is_ok_and(|source| {
                the_shipping_lines(&source)
                    .iter()
                    .any(|line| line.contains(THE_COMPACTION))
            })
        })
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the search index is compacted from somewhere other than the check: {elsewhere:?}. \
         Read this file's comment before moving the call"
    );
}
