//! Two threads reaching the credential store at once both find it ready.
//!
//! `keyring` 4.1.5 sets up the platform's store lazily, inside the first
//! `keyring::Entry::new` of the process (`keyring-4.1.5/src/v1.rs:47-55`). The
//! thread that wins a `compare_exchange` on an `AtomicBool` builds the Windows
//! store and only then installs it; every thread that loses goes straight on to
//! `keyring_core::Entry::new`, which answers "No default store has been set" if
//! the winner has not finished. Ledger 374 found it on 2026-09-13 as the cause of
//! ledger 365's unexplained red, and it refused three of the twenty-eight full
//! gates measured on 2026-09-23. In the program it is an account that fails to
//! sign in at start with "Could not reach the credential store".
//!
//! The race needs a process whose flag is still unset, so this test starts its
//! own binary again, once per attempt, with an environment variable marking the
//! child. A child releases sixteen threads together from a barrier and each asks
//! the store for an entry under a service name no program stores under, so each
//! answer is "nothing there" and nothing is written. A child fails if any thread
//! got an error, and the parent fails naming every child that did.
//!
//! Windows only. The race is in `keyring`'s Windows store setup, and on
//! `other-platforms.yml`'s headless Linux runner `keyring` reaches for the
//! Secret Service (`v1.rs:93-107`), which is not there, so every read would be
//! an error and this would be red for a reason that is not the race.
#![cfg(windows)]

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

/// Set in a child's environment, so the test body knows it is the child.
const THE_CHILD_MARKER: &str = "WIXEN_CREDENTIAL_RACE_CHILD";

/// This test's own name, handed to the child with `--exact`.
const THIS_TEST: &str = "test_threads_reaching_the_credential_store_at_once_all_find_it_ready";

/// Fresh processes per run. The race is a probability; ledger 374 measured
/// about one run in three at ten unsynchronised tests.
const CHILDREN: usize = 20;

/// Threads released together in each child.
const THREADS_PER_CHILD: usize = 16;

/// Longer than a child needs by far, so passing it means the child hung.
const A_CHILD_DEADLINE: Duration = Duration::from_secs(60);

/// How often the parent looks to see whether a child has exited.
const A_LOOK_AT_THE_CHILD: Duration = Duration::from_millis(50);

/// A service name no part of the program stores under, so every read answers
/// that nothing is there and nothing in a real store is touched.
const A_SERVICE_NOTHING_STORES_UNDER: &str = "wixen-mail-tests-the-credential-store-race";

/// What the harness prints when the one test it was asked for ran and passed.
/// A filter that matches nothing also exits zero, so this is what proves the
/// child ran anything at all.
const THE_CHILD_RAN_ITS_TEST: &str = "test result: ok. 1 passed";

#[test]
fn test_threads_reaching_the_credential_store_at_once_all_find_it_ready() {
    if std::env::var_os(THE_CHILD_MARKER).is_some() {
        reach_the_store_from_every_thread_at_once();
        return;
    }

    let failures: Vec<String> = (1..=CHILDREN)
        .filter_map(|child| run_one_child(child).err())
        .collect();

    assert!(
        failures.is_empty(),
        "{} of {CHILDREN} fresh processes had a thread find the credential store \
         not ready:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The child's half: every thread asks at once, and any error fails the child.
fn reach_the_store_from_every_thread_at_once() {
    let barrier = Arc::new(Barrier::new(THREADS_PER_CHILD));
    let threads: Vec<_> = (0..THREADS_PER_CHILD)
        .map(|index| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                wixen_mail::service::secret_store::read(
                    A_SERVICE_NOTHING_STORES_UNDER,
                    &format!("thread-{index}"),
                )
                .map(|_| ())
                .map_err(|e| format!("thread {index}: {e}"))
            })
        })
        .collect();

    let errors: Vec<String> = threads
        .into_iter()
        .filter_map(|thread| match thread.join() {
            Ok(answer) => answer.err(),
            Err(_) => Some("a thread panicked".to_string()),
        })
        .collect();

    assert!(
        errors.is_empty(),
        "{} of {THREADS_PER_CHILD} threads could not reach the store: {errors:?}",
        errors.len()
    );
}

/// Start one child, wait for it within the deadline, and say what went wrong.
fn run_one_child(child_number: usize) -> Result<(), String> {
    let this_binary = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let mut child = Command::new(this_binary)
        .args(["--exact", THIS_TEST, "--test-threads=1"])
        .env(THE_CHILD_MARKER, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("child {child_number} did not start: {e}"))?;

    let stdout = read_in_the_background(child.stdout.take());
    let stderr = read_in_the_background(child.stderr.take());
    let exited = wait_within_the_deadline(&mut child, child_number)?;
    let said = stdout.join().unwrap_or_default();
    let complained = stderr.join().unwrap_or_default();

    if !exited.success() {
        return Err(format!(
            "child {child_number} exited {exited}:\n{said}\n{complained}"
        ));
    }
    if !said.contains(THE_CHILD_RAN_ITS_TEST) {
        return Err(format!(
            "child {child_number} exited cleanly without running its test, \
             which is a filter matching nothing:\n{said}"
        ));
    }
    Ok(())
}

/// Drain a pipe on its own thread, so a child that writes a lot cannot block on
/// a full pipe while the parent waits for it to exit.
fn read_in_the_background<R: Read + Send + 'static>(pipe: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    })
}

/// A child that never returns would hang this target, which no suite reports,
/// so it is killed at the deadline and named.
fn wait_within_the_deadline(
    child: &mut Child,
    child_number: usize,
) -> Result<std::process::ExitStatus, String> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if started.elapsed() > A_CHILD_DEADLINE => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "child {child_number} was still running after {A_CHILD_DEADLINE:?} and was killed"
                ));
            }
            Ok(None) => thread::sleep(A_LOOK_AT_THE_CHILD),
            Err(e) => return Err(format!("child {child_number} could not be waited on: {e}")),
        }
    }
}
