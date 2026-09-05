//! Every control in the window that asks which copy to keep carries a name
//! that reaches the accessibility tree, and the window can be reached.
//!
//! # Why this reads the source
//!
//! Building the window needs a display and a running application, so nothing in
//! the library reaches it. That is the same position plan 03-02's sign-in
//! census and plan 03-07's whole-folder census were in, and this follows both:
//! the assertion is a reading of the file, and `guards/guards.toml` couples it
//! to the source it is about so it runs on the commits that could break it
//! rather than only on the commits that change this file.
//!
//! # Why the distinction it draws is the one that matters
//!
//! Windows has two accessibility channels. `set_accessible_name` writes through
//! `wxAccessible` to MSAA, which is what NVDA reads for a native control.
//! `set_name` sets an internal wxWidgets identifier and reaches neither
//! channel: nothing a screen reader ever asks for.
//!
//! `CLAUDE.md` records what that cost. Sixteen widgets were "named" with
//! `set_name`. It compiled and it passed 324 tests, because no test asked which
//! call had been used and neither does a build.
//!
//! So this counts the controls the window builds and requires a name for each
//! through the call that reaches the tree, and refuses `set_name` outright in
//! that file. A count rather than a list of names, because a control added
//! later is the case this exists to catch and a list of names cannot see one.

use std::fs;

/// The file that builds the window.
const THE_WINDOW: &str = "src/presentation/wx_conflict_choice.rs";

fn the_window() -> String {
    fs::read_to_string(THE_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_WINDOW} could not be read: {why}"))
}

/// How many controls a file builds, by the builders that make one.
///
/// A sizer is not a control and neither is the dialog itself: neither is
/// focusable and neither is read out on its own.
fn how_many_controls_are_built(source: &str) -> usize {
    [
        "Button::builder",
        "StaticText::builder",
        "ListCtrl::builder",
    ]
    .iter()
    .map(|builder| source.matches(builder).count())
    .sum()
}

/// How many of them are named through the call that reaches the tree.
fn how_many_are_named_where_it_counts(source: &str) -> usize {
    source.matches("set_accessible_name(").count()
}

#[test]
fn test_every_control_in_the_choosing_window_is_named_where_a_screen_reader_reads() {
    let source = the_window();
    let built = how_many_controls_are_built(&source);
    let named = how_many_are_named_where_it_counts(&source);
    assert!(
        built > 0,
        "no control was found at all, so this check asked nothing. Either the \
         window stopped building controls or the builders it looks for were \
         renamed"
    );
    assert_eq!(
        named, built,
        "{THE_WINDOW} builds {built} controls and names {named} of them through \
         set_accessible_name. A control with no accessible name is a control a \
         screen reader reads as its type and nothing else: 'button', 'list'"
    );
}

#[test]
fn test_the_choosing_window_never_reaches_for_the_call_that_names_nothing() {
    let source = the_window();
    assert!(
        !source.contains(".set_name("),
        "{THE_WINDOW} calls set_name. That sets an internal wxWidgets \
         identifier and reaches neither MSAA nor UI Automation, so the name \
         never gets to a screen reader. Sixteen widgets were once named that \
         way here; it compiled and passed 324 tests"
    );
}

#[test]
fn test_the_two_copies_are_labelled_by_which_copy_they_are() {
    // The labels themselves are built and tested in
    // `application::conflict_choice`. What this asks is that the window uses
    // them, rather than heading the two lists with something of its own.
    let source = the_window();
    assert!(
        source.contains("copies.label_for(which)"),
        "the window heads the two copies with something other than the label \
         the application layer builds, so the two can come to disagree and the \
         tested one is not the one anybody hears"
    );
}

#[test]
fn test_leaving_the_window_alone_is_the_answer_it_opens_on() {
    // Nothing is the default answer. A window that opens with a copy-keeping
    // button focused turns Enter pressed by reflex into a choice, and one of
    // the two copies goes.
    let source = the_window();
    assert!(
        source.contains("leave_it.set_focus()"),
        "the window opens with one of the two answers focused, so Enter \
         pressed by reflex chooses a copy and loses the other"
    );
}

#[test]
fn test_the_window_can_be_reached_from_the_menu() {
    // Done means it runs. A window nothing opens is a window nobody has.
    //
    // This asks about the arm rather than about the file, and the difference
    // is the whole test. Written as "the file mentions
    // wx_conflict_choice::ask_which_copy_to_keep", it passed with the menu arm
    // emptied out, because the mention lives in the function the arm used to
    // call and that function was still there. Measured on 2026-09-05 by taking
    // the break by hand and watching all five of these stay green.
    let app = fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let arm = "_ if id == ID_CHOOSE_WHICH_COPY => {";
    let at = app.find(arm).unwrap_or_else(|| {
        panic!(
            "no menu arm handles ID_CHOOSE_WHICH_COPY, so the sync says \
             'open Contacts to choose' about a window with no door"
        )
    });
    let body = &app[at + arm.len()..(at + arm.len() + 400).min(app.len())];
    assert!(
        body.contains("choose_which_copy_to_keep("),
        "the menu item is handled by an arm that does not open the choosing \
         window. What follows the arm:\n{body}"
    );
    assert!(
        app.contains("wx_conflict_choice::ask_which_copy_to_keep"),
        "nothing in the main window opens the choosing window"
    );
    assert!(
        app.contains("cache.settle_a_held_conflict("),
        "the window asks and the answer reaches nothing, so choosing changes \
         only what is on the screen"
    );
}
