//! Checking one radio item in a real menu unchecks the rest of its group, and
//! a separator is where a group ends.
//!
//! `tests/one_sort_is_checked.rs` reads the Sort submenu's builder chain and
//! holds it to seven radio items with nothing between them. That reading rests
//! on a claim about wxWidgets, that one group means one tick, and a reading of
//! this project's source cannot see whether the claim is true. This builds the
//! same shape in a real menu bar and asks. The plan for #39 wanted this as a
//! look at the running program, sort by sender from the menu and then by date
//! from a column header; a look is made once and a test is made on every run
//! of the whole gate, and the question is the same: after checking one item
//! and then another by id, which items does the menu say are checked?
//!
//! One `#[test]` function building real windows, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per process
//! and `cargo test` runs each file under `tests/` as its own process. The old
//! shape, with a separator between each pair, is built in the same process as
//! the companion, and the reading is that it shows four ticks, one per group
//! from the moment it is made, where the new shape shows one.

use std::sync::{Arc, Mutex};
use wxdragon::prelude::*;

/// One check that failed: what it was, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// Seven ids that collide with nothing else in this process.
const IDS: [i32; 7] = [9101, 9102, 9103, 9104, 9105, 9106, 9107];

/// A menu bar whose one menu holds the seven items as radio items, with a
/// separator after the second, fourth and sixth when `split` is on, which is
/// the shape the tester met.
fn a_menu_bar_of_seven(frame: &Frame, split: bool) -> MenuBar {
    let mut chain = Menu::builder();
    for (at, id) in IDS.iter().enumerate() {
        chain = chain.append_radio_item(*id, &format!("Sort {at}"), "");
        if split && at % 2 == 1 && at < 6 {
            chain = chain.append_separator();
        }
    }
    frame.set_menu_bar(MenuBar::builder().append(chain.build(), "&Sort").build());
    // Read back through the frame, which is how `sync_sort_menu` reaches it.
    frame
        .get_menu_bar()
        .expect("the frame holds the menu bar it was just given")
}

/// Which of the seven the menu bar says are checked, by position.
fn which_are_checked(menu_bar: &MenuBar) -> Vec<usize> {
    IDS.iter()
        .enumerate()
        .filter(|(_, id)| {
            menu_bar
                .find_item(**id)
                .is_some_and(|item| item.is_checked())
        })
        .map(|(at, _)| at)
        .collect()
}

#[test]
fn test_checking_one_sort_in_one_group_leaves_one_checked_and_a_separator_leaves_two() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();

            // The new shape: one group. As built, before any sort, wxWidgets
            // checks the first item of a radio group, so one tick.
            let frame = Frame::builder().build();
            let one_group = a_menu_bar_of_seven(&frame, false);
            let checked = which_are_checked(&one_group);
            if checked != [0] {
                wrong.push((
                    "one group, as built".to_string(),
                    format!("wanted only item 0 checked, the menu says {checked:?}"),
                ));
            }
            // Sort by sender, then by date, as sync_sort_menu does on the
            // column-header path, and read back.
            one_group.check_item(IDS[2], true);
            one_group.check_item(IDS[0], true);
            let checked = which_are_checked(&one_group);
            if checked != [0] {
                wrong.push((
                    "one group, sender then date".to_string(),
                    format!("wanted only item 0 checked, the menu says {checked:?}"),
                ));
            }
            // And the other way round, so the answer is not the first item's
            // default tick.
            one_group.check_item(IDS[5], true);
            let checked = which_are_checked(&one_group);
            if checked != [5] {
                wrong.push((
                    "one group, then subject descending".to_string(),
                    format!("wanted only item 5 checked, the menu says {checked:?}"),
                ));
            }
            frame.destroy();

            // The companion: the tester's shape, four groups, the same two
            // checks, and the menu says four are checked. Without this, a
            // menu bar that answered nothing checked, or one, for any reason
            // at all would look like the reading above holding.
            //
            // Four and not two, which this expected when it was written:
            // wxWidgets checks the first item of every radio group as it is
            // made, so the menu the tester met showed four ticks from the
            // moment it was built, before anybody had sorted anything, and
            // sorting only moved the tick within one of the four.
            let frame = Frame::builder().build();
            let four_groups = a_menu_bar_of_seven(&frame, true);
            let as_built = which_are_checked(&four_groups);
            four_groups.check_item(IDS[2], true);
            four_groups.check_item(IDS[0], true);
            let checked = which_are_checked(&four_groups);
            if as_built != [0, 2, 4, 6] || checked != [0, 2, 4, 6] {
                wrong.push((
                    "four groups, as built and then sender then date".to_string(),
                    format!(
                        "wanted items 0, 2, 4 and 6 checked both times, which is the shape the \
                         tester met; the menu says {as_built:?} as built and {checked:?} after"
                    ),
                ));
            }
            frame.destroy();

            drop(wrong);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "a real menu did not answer as the reading assumes:\n{}",
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
