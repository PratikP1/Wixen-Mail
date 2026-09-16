//! The View menu's Sort submenu shows one tick for one sort.
//!
//! The tester's words on 2026-09-15 were "Sort options in the view menu are
//! confusing as multiple items are checked" (#39). The submenu was seven radio
//! items in four runs with a separator between each run, and wxWidgets ends a
//! radio group at a separator, so those were four groups: choosing in one run
//! left the checked item in each of the other three checked, and the menu
//! showed up to four ticks for one sort. A tick is a claim about the sort, and
//! four ticks are four claims.
//!
//! Read from the source rather than built, because a menu's grouping is a
//! property of the builder chain and the chain is one function's text: the
//! seven `append_radio_item` calls with nothing between them is what makes the
//! seven one group. Each reading is a function over that text with a companion
//! that plants the old shape and requires a complaint naming where, so a
//! reading that passes over anything is found here rather than by the next
//! tester.
//!
//! What this cannot see: whether wxWidgets really unchecks the other six when
//! `sync_sort_menu` checks one on the column-header path, which is a look at a
//! live menu and is recorded in the plan's summary rather than here.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

/// The seven ids the submenu appends, in the order the sorts are offered.
const THE_SORT_IDS: [&str; 7] = [
    "ID_SORT_DATE_NEWEST",
    "ID_SORT_DATE_OLDEST",
    "ID_SORT_SENDER_AZ",
    "ID_SORT_SENDER_ZA",
    "ID_SORT_SUBJECT_AZ",
    "ID_SORT_SUBJECT_ZA",
    "ID_SORT_UNREAD_FIRST",
];

/// The builder chain for the menu bound to `name`, from `let name = Menu::builder()`
/// to its `.build()`.
///
/// `None` when the file no longer builds a menu by that name, so the reading
/// complains about that rather than reading an empty string and passing.
fn menu_chain<'a>(ship: &'a str, name: &str) -> Option<&'a str> {
    let opens = ship.find(&format!("let {name} = Menu::builder()"))?;
    let chain = &ship[opens..];
    let closes = chain.find(".build()")?;
    Some(&chain[..closes])
}

/// One thing appended to a menu, in the order the chain appends them.
#[derive(Debug, PartialEq)]
enum Appended {
    Radio(String),
    Separator,
    Other,
}

/// Everything the chain appends, in order.
///
/// A radio item is named by the id that follows the bracket, after any
/// whitespace: `rustfmt` puts the id on the next line whenever the call is
/// long enough to wrap, which is every one of these.
fn what_the_chain_appends(chain: &str) -> Vec<Appended> {
    chain
        .match_indices(".append_")
        .map(|(at, _)| {
            let call = &chain[at + ".append_".len()..];
            if let Some(rest) = call.strip_prefix("radio_item(") {
                let id: String = rest
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                Appended::Radio(id)
            } else if call.starts_with("separator()") {
                Appended::Separator
            } else {
                Appended::Other
            }
        })
        .collect()
}

/// Whether the seven sort items are one radio group, said as a complaint
/// naming where the group is broken when they are not.
///
/// One group means the seven `append_radio_item` calls with nothing appended
/// between the first and the last of them: a separator or an ordinary item
/// there ends the group in wxWidgets, and the items after it are a second
/// group with a tick of their own.
fn whether_the_sort_items_are_one_group(app: &str) -> Result<(), String> {
    let ship = what_ships(app);
    let chain = menu_chain(&ship, "sort_menu")
        .ok_or("the Sort submenu is no longer built by that name, so this reads nothing")?;
    let appended = what_the_chain_appends(chain);

    let radios: Vec<&str> = appended
        .iter()
        .filter_map(|item| match item {
            Appended::Radio(id) => Some(id.as_str()),
            _ => None,
        })
        .collect();
    if radios != THE_SORT_IDS {
        return Err(format!(
            "the Sort submenu appends {} radio items, {radios:?}, and this expects the seven \
             sorts in order, so the reading no longer knows the menu",
            radios.len()
        ));
    }

    let first = appended
        .iter()
        .position(|item| matches!(item, Appended::Radio(_)))
        .expect("seven radio items were just counted");
    let last = appended
        .iter()
        .rposition(|item| matches!(item, Appended::Radio(_)))
        .expect("seven radio items were just counted");
    for (offset, item) in appended[first..=last].iter().enumerate() {
        if matches!(item, Appended::Radio(_)) {
            continue;
        }
        let after = appended[first..first + offset]
            .iter()
            .rev()
            .find_map(|item| match item {
                Appended::Radio(id) => Some(id.as_str()),
                _ => None,
            })
            .unwrap_or("nothing");
        let what = match item {
            Appended::Separator => "a separator",
            _ => "an item that is not a radio item",
        };
        return Err(format!(
            "{what} is appended after {after}, which ends the radio group there, so the \
             items after it keep a tick of their own and the menu shows more than one sort \
             as checked (#39)"
        ));
    }
    Ok(())
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The variants of `MailSortOption`, read from where it is declared.
fn the_sorts_there_are(ui_types: &str) -> Result<Vec<String>, String> {
    let decl = body_of(ui_types, "pub enum MailSortOption {")?;
    let variants: Vec<String> = decl
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//") && *line != "}")
        .map(|line| line.trim_end_matches(',').to_string())
        .collect();
    if variants.is_empty() {
        return Err("MailSortOption has no variants by this reading, so it reads nothing".into());
    }
    Ok(variants)
}

/// Whether `sync_sort_menu` names every sort there is, said as a complaint
/// naming the sort it does not when it does not.
///
/// The compiler already refuses a match with a variant missing, unless
/// somebody adds a wildcard arm, after which a sort added later is one the
/// menu silently stops following. So this reads for the wildcard as well as
/// for each name.
fn whether_the_menu_follows_every_sort(app: &str, ui_types: &str) -> Result<(), String> {
    let sync = body_of(&what_ships(app), "fn sync_sort_menu(")?;
    if sync.contains("_ =>") {
        return Err(
            "sync_sort_menu has a wildcard arm, so a sort added later is one the menu \
             silently stops following"
                .to_string(),
        );
    }
    for variant in the_sorts_there_are(ui_types)? {
        if !sync.contains(&format!("MailSortOption::{variant} =>")) {
            return Err(format!(
                "sync_sort_menu does not name MailSortOption::{variant}, so sorting that way \
                 leaves the menu ticking the previous sort"
            ));
        }
    }
    Ok(())
}

fn the_main_window() -> String {
    fs::read_to_string("src/presentation/wx_app.rs").expect("the main window")
}

fn the_ui_types() -> String {
    fs::read_to_string("src/presentation/ui_types.rs").expect("the ui types")
}

/// The chain text with one separator spliced back after the call appending
/// `id`, which is the shape the tester met.
fn with_a_separator_after(app: &str, id: &str) -> String {
    let call_opens = app
        .find(&format!(".append_radio_item(\n                {id},"))
        .expect("an append_radio_item naming the id, or the plant has nowhere to go");
    let call_closes = call_opens + app[call_opens..].find(')').expect("the call closes") + 1;
    format!(
        "{}\n            .append_separator(){}",
        &app[..call_closes],
        &app[call_closes..]
    )
}

#[test]
fn test_the_seven_sort_items_are_one_radio_group() {
    // The tester's sentence, held at the shape that causes it: nothing
    // appended between the first sort item and the last.
    let app = the_main_window();

    if let Err(why) = whether_the_sort_items_are_one_group(&app) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_a_separator_splits_the_group() {
    // The companion plants one separator back, after the second item, and
    // requires the complaint to say which item it follows. Without it, a
    // reading that found no chain, or read the wrong one, would look exactly
    // like one that holds the group whole.
    let planted = with_a_separator_after(&the_main_window(), "ID_SORT_DATE_OLDEST");

    let complaint = whether_the_sort_items_are_one_group(&planted)
        .expect_err("a separator was planted into the group and the reading did not notice");
    assert!(
        complaint.contains("a separator is appended after ID_SORT_DATE_OLDEST"),
        "the reading complained about something other than the planted separator: {complaint}"
    );
}

#[test]
fn test_the_reading_complains_when_a_sort_item_is_missing() {
    // The other way a reading can pass over a menu: fewer items than it
    // expects, which is what T-09-14 in the plan is about.
    let app = the_main_window();
    let call_opens = app
        .find(".append_radio_item(\n                ID_SORT_UNREAD_FIRST,")
        .expect("the Unread First item");
    let call_closes = call_opens + app[call_opens..].find(')').expect("the call closes") + 1;
    let planted = format!("{}{}", &app[..call_opens], &app[call_closes..]);

    let complaint = whether_the_sort_items_are_one_group(&planted)
        .expect_err("a sort item was taken out and the reading did not notice");
    assert!(
        complaint.contains("appends 6 radio items"),
        "the reading complained about something other than the missing item: {complaint}"
    );
}

#[test]
fn test_sync_sort_menu_follows_every_sort_there_is() {
    if let Err(why) = whether_the_menu_follows_every_sort(&the_main_window(), &the_ui_types()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_sync_sort_menu_gains_a_wildcard_arm() {
    // The companion swaps the last arm for a wildcard, which compiles and
    // stops the menu following any sort added after it.
    let app = the_main_window();
    let planted = app.replacen(
        "MailSortOption::UnreadFirst => ID_SORT_UNREAD_FIRST",
        "_ => ID_SORT_UNREAD_FIRST",
        1,
    );
    assert!(
        planted != app,
        "the plant changed nothing, so the arm is no longer written as this expects"
    );

    let complaint = whether_the_menu_follows_every_sort(&planted, &the_ui_types())
        .expect_err("a wildcard arm was planted and the reading did not notice");
    assert!(complaint.contains("wildcard"), "{complaint}");

    // And the other half: the arm gone and no wildcard, which the compiler
    // would refuse and this reading names anyway.
    let planted = app.replacen(
        "MailSortOption::UnreadFirst => ID_SORT_UNREAD_FIRST,",
        "",
        1,
    );
    let complaint = whether_the_menu_follows_every_sort(&planted, &the_ui_types())
        .expect_err("an arm was taken out and the reading did not notice");
    assert!(complaint.contains("UnreadFirst"), "{complaint}");
}
