//! Which row the spelling language picker shows for a stored tag, and so
//! which tag OK writes back.
//!
//! The checker and the settings screen answer two different questions about
//! one stored value. The checker asks which dictionary to open, and may
//! settle for the nearest one it has: a stored `en-AU` on a machine with no
//! Australian dictionary is checked in `en-US`, and that is right for the
//! checker. The screen asks which row to show, and what it shows is what OK
//! writes back, so a screen that showed the checker's answer rewrote the
//! choice: on a machine that offers only `en-US`, a stored `en-AU` was shown
//! as English (United States) and saved as that the first time Settings was
//! saved. GitHub's runner is such a machine, and CI run 35336142985 on
//! 2026-09-18 is where it was seen; the machine the testing happens on
//! offers `en-AU`, which is why nobody saw it there. A regression of 09-02's
//! fix for #21, which put the resolver in front of the screen for a bare
//! tag and did not stop it answering for a tag with a region.
//!
//! So the rule has two halves. A tag that names a region is kept exactly as
//! chosen, whether or not this machine can check it: its own row if the
//! machine lists one, available or not, and a row added at the end if not.
//! A bare tag, which is what every profile written before 2026-09-03 holds,
//! is resolved the way the checker resolves it, to this machine's own region
//! when it is in the family, which is #21's fix; and when the resolver has no
//! answer the tag's own row stands, as it did before, so a bare tag the
//! machine lists without a dictionary is shown as itself and not added a
//! second time.
//!
//! Pure over the stored tag, the machine's tag and the offered rows, so each
//! half is held by a case that a runner without the dictionary would fail
//! too.

use crate::service::spellcheck::{LanguageChoice, language_to_use};

/// The row the picker shows for a stored tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowToShow {
    /// One of the rows this machine offers, by its index in the offered list.
    Existing(usize),
    /// No offered row spells it: a row is added at the end carrying this tag,
    /// so what is stored is visible and pressing OK keeps it.
    Added(String),
}

/// Which row the picker shows for `stored`, given what this machine is set
/// to and the rows it offers.
///
/// A tag with a region is kept as chosen; a bare tag is resolved as the
/// checker resolves it. The module comment says why the two differ.
pub fn which_row_shows(
    stored: &str,
    this_machine: Option<&str>,
    rows: &[LanguageChoice],
) -> RowToShow {
    let row_of = |tag: &str| {
        rows.iter()
            .position(|row| row.tag.eq_ignore_ascii_case(tag))
    };
    let own_row = row_of(stored);
    let shown = if names_a_region(stored) {
        own_row
    } else {
        language_to_use(stored, this_machine, rows)
            .and_then(|resolved| row_of(&resolved))
            .or(own_row)
    };
    shown.map_or_else(|| RowToShow::Added(stored.to_string()), RowToShow::Existing)
}

/// Whether a tag names a region: `en-AU` does, `en` and `en-` do not.
fn names_a_region(tag: &str) -> bool {
    tag.split_once('-')
        .is_some_and(|(_, region)| !region.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(tag: &str, available: bool) -> LanguageChoice {
        LanguageChoice {
            tag: tag.to_string(),
            name: tag.to_string(),
            available,
        }
    }

    /// The runner's case: the machine is set to en-US and lists en-AU
    /// without a dictionary for it. The checker would answer en-US; the
    /// screen shows the row that says en-AU.
    #[test]
    fn test_a_tag_with_a_region_is_shown_as_its_own_row_even_without_a_dictionary() {
        let rows = [row("en-US", true), row("en-AU", false)];
        assert_eq!(
            which_row_shows("en-AU", Some("en-US"), &rows),
            RowToShow::Existing(1)
        );
    }

    /// The runner's other shape: the machine does not list en-AU at all.
    /// The tag is added as a row rather than resolved to en-US.
    #[test]
    fn test_a_tag_with_a_region_the_machine_does_not_list_is_added_as_stored() {
        let rows = [row("en-US", true)];
        assert_eq!(
            which_row_shows("en-AU", Some("en-US"), &rows),
            RowToShow::Added("en-AU".to_string())
        );
    }

    /// Windows spells a tag en-AU; a hand-edited file may not.
    #[test]
    fn test_a_tag_with_a_region_finds_its_row_whatever_its_case() {
        let rows = [row("en-US", true), row("en-AU", true)];
        assert_eq!(
            which_row_shows("EN-au", Some("en-US"), &rows),
            RowToShow::Existing(1)
        );
    }

    /// #21: a bare "en" from an older profile on a machine set to en-US is
    /// this machine's own region, not the Caribbean Windows lists first.
    #[test]
    fn test_a_bare_tag_is_resolved_to_the_row_the_checker_would_use() {
        let rows = [row("en-029", true), row("en-US", true)];
        assert_eq!(
            which_row_shows("en", Some("en-US"), &rows),
            RowToShow::Existing(1)
        );
    }

    /// A bare tag the checker cannot place, but the machine lists without a
    /// dictionary, is shown as that row and not added a second time.
    #[test]
    fn test_a_bare_tag_the_checker_cannot_place_stands_as_its_own_row_when_listed() {
        let rows = [row("en-US", true), row("fr", false)];
        assert_eq!(
            which_row_shows("fr", Some("en-US"), &rows),
            RowToShow::Existing(1)
        );
    }

    /// A tag nothing offers is added as stored under either half.
    #[test]
    fn test_a_tag_nothing_offers_is_added_as_stored() {
        let rows = [row("en-US", true)];
        assert_eq!(
            which_row_shows("zz-ZZ", Some("en-US"), &rows),
            RowToShow::Added("zz-ZZ".to_string())
        );
        assert_eq!(
            which_row_shows("zz", Some("en-US"), &rows),
            RowToShow::Added("zz".to_string())
        );
    }
}
