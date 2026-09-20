//! What a picture nobody described is called when a message is read here.
//!
//! The tester's words in #28, on 2026-09-15: "Photos without descriptions
//! should automatically be given "" as the alt by default unless the user
//! specifically chooses either 'image' or 'photo' in settings." A sender who
//! wrote no description left a gap, and a screen reader meeting a picture
//! with no name says whatever it says for one, which on a newsletter with
//! forty pictures is forty times "graphic" with nothing after it. This module
//! holds the person's choice of what stands in that gap: nothing, so the
//! picture is passed over the way a decorative one is, or one word that says
//! a picture is there.
//!
//! It is the reader's answer and never the sender's. A description the
//! sender wrote is untouched by it, a picture inside a link takes the link's
//! words first (`pictures::the_links_text_as_a_description`), and nothing
//! here reaches a message on its way out: the word is written where a
//! message is shown, in `presentation::html_renderer`'s reading path, and
//! nowhere else.

/// What stands in for the description a sender did not write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UndescribedPicture {
    /// An empty description, so a screen reader passes over the picture.
    /// The default, by the tester's decision, and what an older settings
    /// file with no such key answers.
    #[default]
    Nothing,
    /// The word "image".
    Image,
    /// The word "photo".
    Photo,
}

/// The label beside the choice on the Reading tab. The ampersand is the
/// keyboard accelerator, on "P" for picture, which no other control on that
/// page claims.
pub const UNDESCRIBED_PICTURES_LABEL: &str = "An undescribed &picture is read as:";

/// The sentence under the choice: what the choice does not change, because
/// three answers cannot say on their own what stays the same under all of
/// them.
pub const WHAT_THE_CHOICE_LEAVES_ALONE: &str = "A picture the sender described keeps its \
     description. A picture inside a link takes the link's words.";

impl UndescribedPicture {
    /// The choices offered, in the order they are offered: the default
    /// first, then the two words.
    pub const ALL: [UndescribedPicture; 3] = [
        UndescribedPicture::Nothing,
        UndescribedPicture::Image,
        UndescribedPicture::Photo,
    ];

    /// What the choice is called: what will happen, in plain words. The
    /// first says what "nothing" means to somebody listening, because an
    /// empty description is a mechanism and being passed over is what is
    /// heard.
    pub fn label(self) -> &'static str {
        match self {
            UndescribedPicture::Nothing => "Nothing, so it is passed over",
            UndescribedPicture::Image => "The word image",
            UndescribedPicture::Photo => "The word photo",
        }
    }

    /// How it is written in the settings file: a word a person could read
    /// there, and for the two words the word itself.
    pub fn as_stored(self) -> String {
        match self {
            UndescribedPicture::Nothing => "nothing",
            UndescribedPicture::Image => "image",
            UndescribedPicture::Photo => "photo",
        }
        .to_string()
    }

    /// Read the stored setting.
    ///
    /// Anything unreadable is nothing, the default, because the other two
    /// answers each write a word on every undescribed picture, and a word
    /// written because the file was garbled is a description this program
    /// invented (guardrail 9).
    pub fn from_stored(value: &str) -> Self {
        match value.trim() {
            "image" => UndescribedPicture::Image,
            "photo" => UndescribedPicture::Photo,
            _ => UndescribedPicture::Nothing,
        }
    }

    /// What the stored settings say, read from the file.
    ///
    /// For the note reader, `long_text::spoken`, whose callers hold no
    /// setting. The message renderer reads the same field itself, once, with
    /// its two other picture answers, so a message costs one file read and
    /// not three. A settings file that cannot be read at all answers the
    /// default, nothing, which is the safe way to be wrong here: a picture
    /// passed over is a picture a reader can still ask about, and a word
    /// written because the file was broken would be a description this
    /// program invented.
    pub fn from_stored_settings() -> Self {
        crate::data::config::ConfigManager::load_stored()
            .map(|stored| Self::from_stored(&stored.app_config().undescribed_pictures_read_as))
            .unwrap_or_default()
    }

    /// The description written on a picture that has none: empty, or the
    /// one word chosen.
    pub fn description(self) -> &'static str {
        match self {
            UndescribedPicture::Nothing => "",
            UndescribedPicture::Image => "image",
            UndescribedPicture::Photo => "photo",
        }
    }
}

/// Which entry of the offered list a stored choice selects.
///
/// A garbled stored value reads as the default, so the entry it selects is
/// the default's, and saving that back writes the default.
pub fn offered_index(stored: &str) -> usize {
    let wanted = UndescribedPicture::from_stored(stored);
    UndescribedPicture::ALL
        .iter()
        .position(|choice| *choice == wanted)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_choices_are_nothing_then_image_then_photo() {
        assert_eq!(
            UndescribedPicture::ALL,
            [
                UndescribedPicture::Nothing,
                UndescribedPicture::Image,
                UndescribedPicture::Photo,
            ],
            "the default first, then the two words, which is the order a person \
             arrowing down a list meets them"
        );
    }

    #[test]
    fn test_the_default_is_nothing_so_an_undescribed_picture_is_passed_over() {
        // The tester's decision in #28: "" unless somebody chooses a word.
        assert_eq!(UndescribedPicture::default(), UndescribedPicture::Nothing);
        assert_eq!(UndescribedPicture::default().description(), "");
    }

    #[test]
    fn test_each_choice_says_what_will_happen_in_plain_words() {
        assert_eq!(
            UndescribedPicture::Nothing.label(),
            "Nothing, so it is passed over"
        );
        assert_eq!(UndescribedPicture::Image.label(), "The word image");
        assert_eq!(UndescribedPicture::Photo.label(), "The word photo");
    }

    #[test]
    fn test_the_description_is_empty_or_the_one_word_chosen() {
        assert_eq!(UndescribedPicture::Nothing.description(), "");
        assert_eq!(UndescribedPicture::Image.description(), "image");
        assert_eq!(UndescribedPicture::Photo.description(), "photo");
    }

    #[test]
    fn test_a_choice_is_stored_in_a_word_a_person_could_read_in_the_file() {
        assert_eq!(UndescribedPicture::Nothing.as_stored(), "nothing");
        assert_eq!(UndescribedPicture::Image.as_stored(), "image");
        assert_eq!(UndescribedPicture::Photo.as_stored(), "photo");
    }

    #[test]
    fn test_every_offered_choice_survives_being_stored_and_read_back() {
        for choice in UndescribedPicture::ALL {
            assert_eq!(
                UndescribedPicture::from_stored(&choice.as_stored()),
                choice,
                "{choice:?} did not survive the settings file"
            );
        }
    }

    #[test]
    fn test_anything_unreadable_reads_as_nothing_because_a_word_nobody_chose_would_be_invented() {
        // A garbled value, an empty one and a word from another setting all
        // fall to the default: writing "image" on every picture because the
        // file was hand-edited would be this program inventing a description,
        // which is the one thing guardrail 9 forbids.
        for garbled in ["", "  ", "picture", "IMAGE ", "nothing at all", "1"] {
            assert_eq!(
                UndescribedPicture::from_stored(garbled),
                UndescribedPicture::Nothing,
                "{garbled:?} read as something other than nothing"
            );
        }
        assert_eq!(
            UndescribedPicture::from_stored(" photo "),
            UndescribedPicture::Photo
        );
    }

    #[test]
    fn test_a_stored_choice_selects_its_entry_and_a_garbled_one_selects_the_default() {
        assert_eq!(offered_index("nothing"), 0);
        assert_eq!(offered_index("image"), 1);
        assert_eq!(offered_index("photo"), 2);
        assert_eq!(offered_index("garbled"), 0);
    }
}
