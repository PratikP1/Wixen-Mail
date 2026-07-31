//! What kind of thing an event is.
//!
//! A calendar full of entries that are all just "event" makes somebody read
//! every one to find the two that matter. A birthday, a holiday and a dentist
//! appointment are different kinds of day and want telling apart, and telling
//! them apart by colour is telling them apart in a way this application's own
//! audience cannot use.
//!
//! Both halves of the same list. The ones offered to begin with are the ones
//! almost everybody has, and anything else somebody types is kept and offered
//! back, because a fixed list is a list that is wrong for everybody eventually.

/// The categories offered before anybody has added their own.
///
/// Short, and none of them a synonym of another. A long list is one somebody
/// arrows through, and two entries that mean nearly the same thing is a choice
/// nobody can make quickly.
pub const TO_BEGIN_WITH: [&str; 8] = [
    "Birthday",
    "Anniversary",
    "Holiday",
    "Appointment",
    "Meeting",
    "Travel",
    "Deadline",
    "Personal",
];

/// The word stored for an event with no category.
///
/// Empty rather than "None". Stored as a word it would be a category called
/// None, sorted among the real ones and read out on every uncategorised event.
pub const NONE: &str = "";

/// Clean up something somebody typed, or refuse it.
///
/// `None` for anything that is not a category: nothing, spaces, or a run of
/// punctuation. A blank category is worse than none, because it takes a place
/// in the list and announces nothing when it is reached.
///
/// Commas are taken out, because the list is stored as one comma-separated
/// column and a comma inside a name would split it into two categories, one of
/// which nobody meant.
pub fn tidy(typed: &str) -> Option<String> {
    let cleaned: String = typed
        .replace(',', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let trimmed = cleaned.trim();
    if trimmed.is_empty() || !trimmed.chars().any(char::is_alphanumeric) {
        return None;
    }
    // Long enough for anything anybody means and short enough that one cannot
    // fill a list row on its own.
    Some(
        trimmed
            .chars()
            .take(40)
            .collect::<String>()
            .trim()
            .to_string(),
    )
}

/// Every category to offer, which is the built-in ones plus any that have been
/// used, without repeats and in a stable order.
///
/// Compared without case, so "birthday" typed once does not sit in the list
/// beside "Birthday" for ever as a second entry meaning the same thing. The
/// first spelling seen wins, which for the built-in ones is theirs.
pub fn offered(also_used: &[String]) -> Vec<String> {
    let mut all: Vec<String> = TO_BEGIN_WITH.iter().map(|c| c.to_string()).collect();
    for used in also_used {
        let Some(tidied) = tidy(used) else {
            continue;
        };
        if !all.iter().any(|known| known.eq_ignore_ascii_case(&tidied)) {
            all.push(tidied);
        }
    }
    all
}

/// The categories on one item, read out of the stored column.
pub fn on(stored: &str) -> Vec<String> {
    stored.split(',').filter_map(tidy).collect()
}

/// The categories of one item, as the column stores them.
pub fn stored(categories: &[String]) -> String {
    categories
        .iter()
        .filter_map(|c| tidy(c))
        .collect::<Vec<_>>()
        .join(",")
}

/// What is said about an item's categories when it is read out.
///
/// Empty when it has none, so an uncategorised calendar costs nothing.
pub fn spoken(stored_value: &str) -> String {
    let categories = on(stored_value);
    match categories.len() {
        0 => String::new(),
        _ => categories.join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_ones_everybody_has_are_offered_first() {
        let offered = offered(&[]);

        assert_eq!(offered.len(), TO_BEGIN_WITH.len());
        assert_eq!(offered[0], "Birthday");
    }

    #[test]
    fn test_something_somebody_typed_is_kept_and_offered_back() {
        // A fixed list is one that is wrong for everybody eventually.
        let offered = offered(&["Book club".to_string()]);

        assert!(offered.contains(&"Book club".to_string()));
    }

    #[test]
    fn test_the_same_word_typed_differently_is_the_same_category() {
        // Otherwise "birthday" sits in the list beside "Birthday" for ever,
        // as a second entry meaning the same thing.
        let offered = offered(&["birthday".to_string(), "BIRTHDAY".to_string()]);

        assert_eq!(offered.len(), TO_BEGIN_WITH.len());
    }

    #[test]
    fn test_a_blank_category_is_refused() {
        // It would take a place in the list and announce nothing when reached.
        for nothing in ["", "   ", ",", " , , ", "..."] {
            assert_eq!(tidy(nothing), None, "for {nothing:?}");
        }
    }

    #[test]
    fn test_a_comma_cannot_smuggle_in_a_second_category() {
        // The column is comma separated, so a comma inside a name would split
        // it into two, one of which nobody meant.
        assert_eq!(tidy("Work, personal"), Some("Work personal".to_string()));
        assert_eq!(on("Birthday,Holiday").len(), 2);
    }

    #[test]
    fn test_spacing_is_tidied_rather_than_kept() {
        assert_eq!(tidy("  Book   club  "), Some("Book club".to_string()));
    }

    #[test]
    fn test_a_category_cannot_fill_a_row_on_its_own() {
        let long = "a".repeat(200);

        assert_eq!(tidy(&long).unwrap().chars().count(), 40);
    }

    #[test]
    fn test_the_categories_survive_the_trip_through_one_column() {
        let chosen = vec!["Birthday".to_string(), "Personal".to_string()];

        assert_eq!(on(&stored(&chosen)), chosen);
    }

    #[test]
    fn test_an_item_with_no_categories_says_nothing() {
        // An uncategorised calendar should cost nothing to listen to.
        assert_eq!(spoken(""), "");
        assert_eq!(spoken("  , ,"), "");
    }

    #[test]
    fn test_an_item_with_categories_reads_them_as_words() {
        assert_eq!(spoken("Birthday,Personal"), "Birthday, Personal");
    }

    #[test]
    fn test_nothing_stored_is_nothing_rather_than_the_word_none() {
        // Stored as a word it would be a category called None, sorted among
        // the real ones and read out on every uncategorised event.
        assert_eq!(NONE, "");
        assert!(on(NONE).is_empty());
    }
}
