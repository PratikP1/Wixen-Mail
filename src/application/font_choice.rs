//! Which typeface the lists are drawn in.
//!
//! # Why a missing font is said out loud
//!
//! Windows does not fail when asked for a typeface that is not installed. It
//! quietly draws something else. So a font uninstalled after it was chosen, or
//! a settings file carried to another computer, produces a program that looks
//! wrong for no stated reason, and the person most likely to notice is the one
//! who chose a particular typeface because the default was hard for them to
//! read.
//!
//! That is the whole reason this module exists rather than the setting being a
//! string passed straight to the toolkit. A choice is checked against what the
//! machine has, and when it is not there the sentence says so and the drawing
//! falls back openly rather than silently.
//!
//! # Why the list is not curated
//!
//! It would be easy to put the typefaces that are easier to read at the top.
//! Alphabetical is better: a list whose order somebody cannot predict is one
//! they have to read all of, every time, and predictable ordering is worth more
//! than a recommendation nobody asked for. What this application can honestly
//! say about readability belongs in a sentence, not in a sort order.

/// What is stored when nobody has chosen, and what means "whatever Windows
/// would use".
///
/// The empty string rather than a name, because the toolkit already treats an
/// empty face name that way, and because storing the name of today's system
/// font would freeze it: somebody who changes their Windows font afterwards
/// would keep the old one here with nothing saying why.
pub const THE_SYSTEM_FONT: &str = "";

/// What the settings list shows first, standing for [`THE_SYSTEM_FONT`].
pub const THE_SYSTEM_FONT_LABEL: &str = "Whatever Windows uses";

/// Every row of the settings list, in the order it is shown.
pub fn what_the_list_offers(installed: &[String]) -> Vec<String> {
    let mut offered = Vec::with_capacity(installed.len() + 1);
    offered.push(THE_SYSTEM_FONT_LABEL.to_string());
    offered.extend(installed.iter().cloned());
    offered
}

/// Which row is the stored choice.
///
/// A choice that is not installed lands on the system font, because that is
/// what is going to be drawn. Showing the missing name as though it were
/// selected would be the settings screen agreeing with a state the machine
/// cannot produce.
pub fn which_row_is_chosen(stored: &str, installed: &[String]) -> usize {
    if stored == THE_SYSTEM_FONT {
        return 0;
    }
    installed
        .iter()
        .position(|name| name.eq_ignore_ascii_case(stored))
        .map_or(0, |at| at + 1)
}

/// The stored value for a row of the list.
pub fn what_a_row_stores(row: usize, installed: &[String]) -> String {
    if row == 0 {
        return THE_SYSTEM_FONT.to_string();
    }
    installed
        .get(row - 1)
        .cloned()
        .unwrap_or_else(|| THE_SYSTEM_FONT.to_string())
}

/// The face name to draw with, which is not always the one that was chosen.
///
/// Empty means let the toolkit decide. A chosen font that is not installed
/// comes back empty rather than being passed on, so what is drawn is the
/// system font on purpose rather than whatever Windows would have substituted.
pub fn face_to_draw_with(stored: &str, installed: &[String]) -> String {
    if stored == THE_SYSTEM_FONT {
        return THE_SYSTEM_FONT.to_string();
    }
    installed
        .iter()
        .find(|name| name.eq_ignore_ascii_case(stored))
        .cloned()
        .unwrap_or_else(|| THE_SYSTEM_FONT.to_string())
}

/// What to say about a stored choice, or nothing when there is nothing to say.
///
/// Named for what it reports rather than for when it is empty, because the
/// empty case is the ordinary one and a caller should be able to show this
/// without asking a second question first.
pub fn what_is_wrong_with_the_choice(stored: &str, installed: &[String]) -> String {
    if stored == THE_SYSTEM_FONT {
        return String::new();
    }
    if installed
        .iter()
        .any(|name| name.eq_ignore_ascii_case(stored))
    {
        return String::new();
    }
    format!(
        "{stored} is not installed on this computer any more, so lists are drawn \
         in the font Windows uses. Choose another to stop this message."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed() -> Vec<String> {
        vec![
            "Arial".to_string(),
            "Consolas".to_string(),
            "Verdana".to_string(),
        ]
    }

    #[test]
    fn test_the_system_font_is_the_first_thing_offered() {
        // First because it is the answer for almost everybody, and because a
        // list somebody has to arrow to the end of to find "leave it alone" is
        // a list that argues for changing something.
        let offered = what_the_list_offers(&installed());

        assert_eq!(offered[0], THE_SYSTEM_FONT_LABEL);
        assert_eq!(offered.len(), 4);
    }

    #[test]
    fn test_nothing_chosen_lands_on_the_system_font() {
        assert_eq!(which_row_is_chosen(THE_SYSTEM_FONT, &installed()), 0);
    }

    #[test]
    fn test_a_chosen_font_lands_on_its_own_row() {
        // One past its place in the installed list, because the system font
        // sits in front of them. Getting this wrong selects the neighbour,
        // which reads plausibly and is wrong for every font.
        assert_eq!(which_row_is_chosen("Consolas", &installed()), 2);
        assert_eq!(what_a_row_stores(2, &installed()), "Consolas");
    }

    #[test]
    fn test_a_row_and_its_stored_value_agree_all_the_way_along() {
        // Both directions, over every row, because an off-by-one in either
        // would still pass a test that only checked one font.
        let installed = installed();
        for row in 0..what_the_list_offers(&installed).len() {
            let stored = what_a_row_stores(row, &installed);
            assert_eq!(
                which_row_is_chosen(&stored, &installed),
                row,
                "row {row} stores {stored:?} which comes back as a different row"
            );
        }
    }

    #[test]
    fn test_a_font_that_has_been_uninstalled_is_not_drawn_with() {
        // Windows substitutes silently for a face it does not have, so passing
        // the name on would draw something nobody chose and say nothing. The
        // empty answer means the toolkit decides, which is the same thing on
        // purpose rather than by accident.
        assert_eq!(face_to_draw_with("Comic Sans MS", &installed()), "");
    }

    #[test]
    fn test_a_font_that_is_installed_is_drawn_with() {
        assert_eq!(face_to_draw_with("Verdana", &installed()), "Verdana");
    }

    #[test]
    fn test_a_stored_name_is_matched_however_it_was_capitalised() {
        // A settings file written by hand, or carried from a machine that
        // spelled it differently. Windows compares face names without case and
        // so does this, or a perfectly installed font reads as missing.
        assert_eq!(face_to_draw_with("verdana", &installed()), "Verdana");
        assert!(what_is_wrong_with_the_choice("VERDANA", &installed()).is_empty());
    }

    #[test]
    fn test_a_missing_font_is_said_rather_than_quietly_replaced() {
        // The failure this module exists for. The person most likely to meet
        // it is the one who chose a particular typeface because the default
        // was hard for them to read.
        let said = what_is_wrong_with_the_choice("Comic Sans MS", &installed());

        assert!(said.contains("Comic Sans MS"), "{said}");
        assert!(said.contains("not installed"), "{said}");
        assert!(
            said.contains("Choose another"),
            "the sentence does not say what to do about it: {said}"
        );
    }

    #[test]
    fn test_nothing_is_said_when_the_choice_is_fine() {
        // Including the ordinary case of never having chosen, which must not
        // report a problem nobody has.
        assert!(what_is_wrong_with_the_choice(THE_SYSTEM_FONT, &installed()).is_empty());
        assert!(what_is_wrong_with_the_choice("Arial", &installed()).is_empty());
    }

    #[test]
    fn test_a_machine_that_would_not_list_its_fonts_falls_back_rather_than_complaining() {
        // An empty list is what a caller has when the enumeration failed. Every
        // font then reads as missing, and a settings screen full of complaints
        // about fonts that are really there would be worse than the fallback.
        assert_eq!(face_to_draw_with("Arial", &[]), "");
        assert_eq!(which_row_is_chosen("Arial", &[]), 0);
        assert_eq!(what_the_list_offers(&[]).len(), 1);
    }
}
