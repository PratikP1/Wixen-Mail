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
