//! What the About dialog says, in one place (#78).
//!
//! The dialog reads its words from here and writes none of its own, so the
//! sentence somebody hears, the `LICENSE` file shipped beside the program and
//! the copyright Windows shows in the file's properties are one sentence with
//! several readers. `tests/the_about_dialog_names_its_owners_and_its_links.rs`
//! holds the three to each other.

/// Who holds the copyright, in the words `LICENSE` uses after its "(c)".
pub const COPYRIGHT: &str = "Copyright 2024-2026 Wixen Mail Contributors";

/// The licence the program is released under, as a sentence names it.
pub const LICENCE_NAME: &str = "";

/// The project's site.
pub const HOME_PAGE: &str = "";

/// Where somebody goes for help with the program.
pub const SUPPORT_PAGE: &str = "";

/// The dialog's four lines of text, top to bottom: the name, the version with
/// its build counter, what the program is, and who holds the copyright under
/// which licence.
pub fn lines() -> [String; 4] {
    Default::default()
}

/// What a link to `address` shows: the address without its scheme.
pub fn shown_as(address: &str) -> &str {
    address
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::html_renderer::HtmlRenderer;

    #[test]
    fn test_the_copyright_names_both_holders_and_the_other_contributors() {
        assert_eq!(
            COPYRIGHT,
            "Copyright 2024-2026 Pratik Patel and the Wixen Project, with other contributors"
        );
    }

    #[test]
    fn test_the_two_pages_pass_the_gate_every_link_takes_unchanged() {
        // Written as literals, not read back from the constants, so a constant
        // edited to another host is a failure here and not a new expectation.
        for (address, expected) in [
            (HOME_PAGE, "https://wixen.app"),
            (SUPPORT_PAGE, "https://wixen.app/support"),
        ] {
            assert_eq!(address, expected);
            assert_eq!(
                HtmlRenderer::safe_external_url(address).as_deref(),
                Some(expected),
                "{address:?} does not pass the gate every link takes, unchanged"
            );
        }
    }

    #[test]
    fn test_a_page_is_shown_as_its_address_without_the_scheme() {
        assert_eq!(shown_as("https://wixen.app"), "wixen.app");
        assert_eq!(shown_as("https://wixen.app/support"), "wixen.app/support");
    }

    #[test]
    fn test_the_version_line_carries_the_whole_build_string() {
        let version = lines()[1].clone();
        assert!(
            version.starts_with("Version "),
            "the second line is {version:?}"
        );
        assert!(
            version.ends_with(&crate::common::version::current()),
            "the second line {version:?} is not the whole build string {:?}",
            crate::common::version::current()
        );
    }

    #[test]
    fn test_the_lines_say_the_name_what_it_is_and_the_copyright_under_the_licence() {
        let lines = lines();
        assert_eq!(lines[0], "Wixen Mail");
        assert_eq!(
            lines[2],
            "A modern, accessible email client\nbuilt with Rust and wxWidgets."
        );
        assert_eq!(
            lines[3],
            "Copyright 2024-2026 Pratik Patel and the Wixen Project, with other contributors.\n\
             Released under the MIT licence."
        );
    }
}
