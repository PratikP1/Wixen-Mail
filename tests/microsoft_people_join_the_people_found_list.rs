//! Microsoft is asked with the permission each feature needs, and only where
//! it should be.
//!
//! Read out of the source, because the question is which token a worker asks
//! for and which account it asks for it on, and neither is anything a running
//! test can see without an account at Microsoft. Each reading has a companion
//! that hands it a planted text with the fault in it, so a reading that stopped
//! reading would say so rather than pass.

/// The body of one function in a source file, from its signature to the line
/// that closes it at the left margin.
fn the_body_of(file: &str, signature: &str) -> String {
    let source = std::fs::read_to_string(file).unwrap_or_else(|why| panic!("{file}: {why}"));
    the_body_in(&source, signature)
}

fn the_body_in(source: &str, signature: &str) -> String {
    let after = source
        .split(signature)
        .nth(1)
        .unwrap_or_else(|| panic!("no {signature:?} to read"));
    after.split("\n}\n").next().unwrap_or_default().to_string()
}

// ── The tasks sync (ledger 282) ─────────────────────────────────────────────

/// Whether a tasks sync asks for the token carrying the tasks permission, and
/// not the shared one that has never carried it.
fn asks_for_the_tasks_token(body: &str) -> bool {
    body.contains("a_tasks_token_for(") && !body.contains("get_valid_graph_token")
}

#[test]
fn test_the_tasks_sync_asks_for_the_token_that_carries_the_tasks_permission() {
    let body = the_body_of("src/presentation/wx_app.rs", "fn spawn_tasks_sync(");

    assert!(
        asks_for_the_tasks_token(&body),
        "the tasks sync writes Microsoft tasks with a token that was never asked for \
         the tasks permission, so every change is refused (ledger 282):\n{body}"
    );
}

#[test]
fn test_the_reading_of_the_tasks_sync_sees_the_shared_token_when_it_is_there() {
    let planted = "fn spawn_tasks_sync(app: AppHandles<'_>) {\n    \
                   match handle.block_on(auth.get_valid_graph_token()) {\n}\n";

    assert!(!asks_for_the_tasks_token(&the_body_in(
        planted,
        "fn spawn_tasks_sync("
    )));
}

// ── Microsoft's people search ───────────────────────────────────────────────

/// Every Rust file under a folder, with its path and its text.
fn every_source_file_under(folder: &std::path::Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(folder).unwrap_or_else(|why| panic!("{folder:?}: {why}"));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_source_file_under(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text =
                std::fs::read_to_string(&path).unwrap_or_else(|why| panic!("{path:?}: {why}"));
            found.push((path.display().to_string().replace('\\', "/"), text));
        }
    }
    found
}

/// Whether a line opens a function.
fn opens_a_function(line: &str) -> bool {
    let line = line.trim_start();
    [
        "fn ",
        "pub fn ",
        "async fn ",
        "pub async fn ",
        "pub(crate) fn ",
    ]
    .iter()
    .any(|opening| line.starts_with(opening))
}

/// For each call to `call` in a file, the function it sits in, from the line
/// that opens that function up to the call.
fn the_functions_calling(source: &str, call: &str) -> Vec<String> {
    source
        .match_indices(call)
        .map(|(at, _)| {
            let before = &source[..at];
            let opens = before
                .rmatch_indices('\n')
                .map(|(newline, _)| newline + 1)
                .chain(std::iter::once(0))
                .find(|&line_start| opens_a_function(&source[line_start..]))
                .unwrap_or(0);
            source[opens..at].to_string()
        })
        .collect()
}

/// Every call to Microsoft's people search outside the client that defines
/// it, each with the function it sits in.
fn the_people_searches_in(files: &[(String, String)]) -> Vec<(String, String)> {
    files
        .iter()
        .filter(|(path, _)| !path.ends_with("src/service/microsoft_graph.rs"))
        .flat_map(|(path, text)| {
            the_functions_calling(text, ".people_matching(")
                .into_iter()
                .map(move |function| (path.clone(), function))
        })
        .collect()
}

/// The people searches whose token did not come from the one asked for
/// people search alone.
fn asked_with_some_other_token(searches: &[(String, String)]) -> Vec<String> {
    searches
        .iter()
        .filter(|(_, function)| !function.contains("a_people_token_for("))
        .map(|(path, function)| format!("{path}:\n{function}"))
        .collect()
}

#[test]
fn test_microsofts_people_search_is_asked_only_with_the_token_asked_for_it() {
    let searches = the_people_searches_in(&every_source_file_under(std::path::Path::new("src")));

    assert!(
        !searches.is_empty(),
        "nothing in the program asks Microsoft's people search, so typing a name on a \
         Microsoft account finds nobody from Microsoft"
    );
    let wrong = asked_with_some_other_token(&searches);
    assert!(
        wrong.is_empty(),
        "Microsoft's people search is asked with a token that was not asked for \
         People.Read alone:\n{}",
        wrong.join("\n\n")
    );
}

#[test]
fn test_the_reading_of_the_people_search_sees_a_search_asked_with_the_shared_token() {
    let planted = "fn the_people_microsoft_knows(account: &str) -> Vec<Somebody> {\n    \
                   let token = a_graph_token_for(account);\n    \
                   client.people_matching(&token, name, 50)\n}\n";
    let searches = the_people_searches_in(&[(
        "src/presentation/planted.rs".to_string(),
        planted.to_string(),
    )]);

    assert_eq!(searches.len(), 1, "{searches:?}");
    assert_eq!(asked_with_some_other_token(&searches).len(), 1);
}

/// Whether the search a window starts hands what Microsoft found to the list.
fn hands_microsofts_people_to_the_list(who_matches: &str) -> bool {
    who_matches.contains("the_people_microsoft_knows(")
        && who_matches
            .split("everybody_found(")
            .nth(1)
            .and_then(|call| call.split(')').next())
            .is_some_and(|arguments| arguments.contains("from_microsoft"))
}

#[test]
fn test_a_name_typed_on_a_recipient_line_reaches_microsoft_and_comes_back_into_the_list() {
    let who_matches = the_body_of("src/presentation/finding_people.rs", "fn who_matches(");

    assert!(
        hands_microsofts_people_to_the_list(&who_matches),
        "the search a compose window starts does not put Microsoft's people in the \
         list:\n{who_matches}"
    );
}

#[test]
fn test_the_reading_of_the_search_sees_microsofts_people_left_out_of_the_list() {
    let planted = "fn who_matches() {\n    \
                   let from_microsoft = the_people_microsoft_knows(account);\n    \
                   everybody_found(from_your_contacts, from_the_directory, Vec::new())\n}\n";

    assert!(!hands_microsofts_people_to_the_list(&the_body_in(
        planted,
        "fn who_matches("
    )));
}
