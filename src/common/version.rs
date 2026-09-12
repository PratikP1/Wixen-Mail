//! Which build of Wixen Mail this is.
//!
//! The version number moves when the software changes, not when a file
//! changes hands. That is the point of it, and it means several different
//! builds can share a version: three fixes on the way to 0.5.1 are all
//! 0.5.0 until 0.5.1 is cut.
//!
//! Which leaves a question somebody has to be able to answer. A bug report
//! arrives with a log saying "Starting Wixen Mail v0.5.0", and if six builds
//! said that, the report cannot be matched to the code it came from. So a
//! build made by `scripts/build-installer.sh` carries the commit it was built
//! from, and says so everywhere the version appears.
//!
//! Ordinary `cargo build` gets nothing extra, and that is deliberate: reading
//! the commit at compile time would mean relinking after every `git add`, for
//! a label nobody looks at in a development build. The script sets
//! `WIXEN_BUILD` and `build.rs` passes it through, so only the builds that are
//! handed to somebody pay for it.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

/// The version from `Cargo.toml`.
const NUMBER: &str = env!("CARGO_PKG_VERSION");

/// The commit this was built from, empty unless the installer script built it.
const BUILD: &str = env!("WIXEN_BUILD");

/// What separates a version from its build identifier.
///
/// One constant rather than the character written in two places. [`describe`]
/// joins on it and [`without_build`] splits on it, and the two are inverses,
/// so neither can be written in terms of the other and a literal in both is
/// one rule kept in two places.
const BUILD_SEPARATOR: char = '+';

/// What separates a version from the prerelease staging it.
///
/// Deliberately not [`BUILD_SEPARATOR`]: a build identifier put after this
/// would make every build a prerelease of the version it was built from and
/// sort them all below it.
const PRERELEASE_SEPARATOR: char = '-';

/// What a published tag carries in front of the version.
///
/// `cargo release` writes the tag, `.github/workflows/release.yml:115` names
/// the portable download after it, and `:133` publishes that download under
/// the glob `wixen-mail-v*.exe`. So the strings a release feed hands back
/// start with this and the number this crate is built from does not, and both
/// have to read as the same version.
const TAG_PREFIX: &str = "v";

/// What to call this build, wherever a person or a log will see it.
///
/// `0.5.0` from a plain build, `0.5.0+g64c73dd` from one the installer script
/// made. The `+` is deliberate: everything after it is build metadata, which
/// version ordering ignores, so two builds of the same version stay equal
/// while remaining tellable apart.
pub fn current() -> String {
    describe(NUMBER, BUILD)
}

/// Join a version and a build identifier.
///
/// Split out from [`current`] so it can be tested. The two values it reads are
/// fixed at compile time, so a test of `current` could only ever check one of
/// the two cases, and it would be whichever case the test run happened to be
/// built in.
fn describe(number: &str, build: &str) -> String {
    if build.is_empty() {
        number.to_string()
    } else {
        format!("{number}{BUILD_SEPARATOR}{build}")
    }
}

/// A version with its build identifier taken off.
///
/// Everything after the separator is build metadata, which version ordering
/// ignores, so two builds of one version stay equal. A separator with nothing
/// after it is neither a build identifier nor a version, so it is refused
/// rather than quietly read as the number in front of it.
fn without_build(version: &str) -> Option<&str> {
    match version.split_once(BUILD_SEPARATOR) {
        None => Some(version),
        Some((number, build)) if !build.is_empty() => Some(number),
        Some(_) => None,
    }
}

/// Where a version sits between its own prereleases and itself.
///
/// The order is the declaration order. `scripts/build-installer.sh` reached
/// the same one independently when it had to squeeze a prerelease into the
/// four numbers Windows shows, and this is where the two have to agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Alpha,
    Beta,
    Rc,
    /// Not a prerelease at all, which is why it is last: a release is newer
    /// than every prerelease that staged it.
    Release,
}

/// The three words `.github/workflows/release.yml` can put in a tag.
///
/// Anything else is refused rather than being given a position. A prerelease
/// spelling this project does not produce is a string somebody else chose,
/// and the thing downstream of this answer downloads an executable.
fn stage_named(word: &str) -> Option<Stage> {
    match word {
        "alpha" => Some(Stage::Alpha),
        "beta" => Some(Stage::Beta),
        "rc" => Some(Stage::Rc),
        _ => None,
    }
}

/// One field of a version, which is a run of digits and nothing else.
///
/// Not `str::parse` on its own, which accepts a leading plus and would read
/// `+5` as five in a string where a plus already means something else.
/// Refuses a number too large to hold rather than saturating it.
fn whole_number(field: &str) -> Option<u64> {
    if field.is_empty() || !field.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    field.parse().ok()
}

/// A version string this program can put in order.
///
/// Field order is comparison order, which is what the derived `Ord` reads, so
/// the three numbers decide first and the prerelease decides the ties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
    stage: Stage,
    /// The counter after the prerelease word, held as a number so that
    /// `alpha.10` is newer than `alpha.9` rather than sorting before it the
    /// way the text does. Zero for a release, which carries no counter and
    /// needs none, because `stage` has already put it above every prerelease
    /// of the same three numbers.
    step: u64,
}

/// Read a version string, or answer nothing if it is not one.
///
/// Total: no input makes this panic, overflow or hang, so [`compare`] has no
/// error to flow anywhere and every caller gets an answer. What it accepts is
/// what this project publishes and nothing more, because a string guessed at
/// is a string that can be guessed wrong.
fn parse(version: &str) -> Option<Version> {
    let named = version.strip_prefix(TAG_PREFIX).unwrap_or(version);
    let ordered = without_build(named)?;
    let (numbers, prerelease) = match ordered.split_once(PRERELEASE_SEPARATOR) {
        Some((numbers, prerelease)) => (numbers, Some(prerelease)),
        None => (ordered, None),
    };

    let mut fields = numbers.split('.');
    let major = whole_number(fields.next()?)?;
    let minor = whole_number(fields.next()?)?;
    let patch = whole_number(fields.next()?)?;
    if fields.next().is_some() {
        return None;
    }

    let (stage, step) = match prerelease {
        None => (Stage::Release, 0),
        Some(prerelease) => {
            let (word, counter) = prerelease.split_once('.')?;
            (stage_named(word)?, whole_number(counter)?)
        }
    };

    Some(Version {
        major,
        minor,
        patch,
        stage,
        step,
    })
}

/// How one version compares with another.
///
/// Four answers rather than three. "I could not read that" is not "older", and
/// a caller that has only a `bool` to read the answer out of has to decide
/// which of the two it means. Reading it as older tells everybody there is a
/// new version every time a server returns something unexpected, so the
/// unreadable case is its own answer and every caller has to say what it does
/// about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compared {
    /// The first version is newer than the second.
    Newer,
    /// The two are the same version. They may still be different builds of it.
    Same,
    /// The first version is older than the second.
    Older,
    /// One of the two is not a version this program can read.
    CouldNotRead,
}

/// Whether `version` is newer than, the same as, or older than `with`.
///
/// Both arguments are version strings as this project writes them: three
/// numbers, an optional prerelease that stages them, and an optional build
/// identifier that plays no part in the order.
pub fn compare(version: &str, with: &str) -> Compared {
    let (Some(left), Some(right)) = (parse(version), parse(with)) else {
        return Compared::CouldNotRead;
    };
    match left.cmp(&right) {
        Ordering::Greater => Compared::Newer,
        Ordering::Equal => Compared::Same,
        Ordering::Less => Compared::Older,
    }
}

/// Which releases somebody hears about.
///
/// Exactly two values, and every one of them is a real channel. "Is this
/// prerelease an offer for somebody who is not looking for updates" is not a
/// question with a true answer, so a third variant for that would force every
/// arm of [`whether_to_offer`] to invent one. What somebody chose is
/// [`WhichUpdates`], which has three answers and maps to this.
///
/// Never written to a settings file. Nothing stores a channel; the setting
/// stores [`WhichUpdates`] and this is derived from it, so there is no stored
/// spelling here for anybody to fix or to have to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReleaseChannel {
    /// Released versions only.
    ///
    /// The default, so a channel nobody chose is the one that offers least.
    #[default]
    PublicReleases,
    /// Released versions and the prereleases that stage them.
    DevelopmentReleases,
}

impl ReleaseChannel {
    /// Whether a prerelease is ever an offer on this channel.
    ///
    /// No catch-all arm, so a third channel would have to answer this rather
    /// than inheriting whichever answer happened to be the fallback.
    const fn offers_prereleases(self) -> bool {
        match self {
            Self::PublicReleases => false,
            Self::DevelopmentReleases => true,
        }
    }
}

/// What somebody chose about hearing of new versions.
///
/// Three answers, because D-16 makes it one control with three values rather
/// than a switch and a channel choice beside it. A control that greys out when
/// its parent is off is skipped in the tab order, so somebody moving by
/// keyboard does not meet it as unavailable, they do not meet it at all.
///
/// This is the value a settings file holds. [`ReleaseChannel`] is worked out
/// from it by [`WhichUpdates::channel`] and is never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhichUpdates {
    /// Nothing is asked for and nothing is fetched.
    ///
    /// The default, and the answer an unknown stored value reads as, so the
    /// only way anything is fetched is somebody choosing that it should be.
    #[default]
    NotLooking,
    /// Released versions.
    PublicReleases,
    /// Released versions and the prereleases that stage them.
    DevelopmentReleases,
}

impl WhichUpdates {
    /// Every answer, in the order a control should offer them.
    ///
    /// Least first, so the answer that asks for nothing is the one somebody
    /// meets first and each one after it fetches more. Written out rather than
    /// derived, for the reason [`Self::channel`] gives about catch-all arms: a
    /// fourth answer has to be put here by somebody, and a control built from
    /// this array is the one place that would say so.
    pub const ALL: [Self; 3] = [
        Self::NotLooking,
        Self::PublicReleases,
        Self::DevelopmentReleases,
    ];

    /// What this answer is called where somebody chooses it.
    ///
    /// Words a person would use rather than the variant's own name. "Test
    /// versions" rather than "prereleases", because the word this project
    /// publishes under is `alpha`, `beta` or `rc` and none of those is a word
    /// somebody scanning a settings screen by ear is looking for.
    pub const fn words(self) -> &'static str {
        match self {
            Self::NotLooking => "Do not look for new versions",
            Self::PublicReleases => "Released versions",
            Self::DevelopmentReleases => "Released versions and test versions",
        }
    }

    /// Which channel this answer asks, if it asks at all.
    ///
    /// One function with no catch-all arm, so a fourth answer added here has
    /// to say which channel it means rather than compiling into whichever
    /// arm came last. Two callers matching on three values independently is
    /// how one of them ends up treating "not looking" as the public channel.
    pub const fn channel(self) -> Option<ReleaseChannel> {
        match self {
            Self::NotLooking => None,
            Self::PublicReleases => Some(ReleaseChannel::PublicReleases),
            Self::DevelopmentReleases => Some(ReleaseChannel::DevelopmentReleases),
        }
    }

    /// The spelling that goes in the settings file.
    ///
    /// Fixed from the moment one settings file on one machine holds it, which
    /// is this project's rule about never renaming what shipped applied to a
    /// stored value rather than to a column. Lower case with underscores
    /// because every other key in that file is written that way and somebody
    /// may open it in a text editor.
    const fn as_stored(self) -> &'static str {
        match self {
            Self::NotLooking => "not_looking",
            Self::PublicReleases => "public_releases",
            Self::DevelopmentReleases => "development_releases",
        }
    }

    /// What a stored spelling means, including one this build does not know.
    ///
    /// Total, and it answers "not looking" for anything it does not
    /// recognise. That is the case a downgrade produces, and the two other
    /// answers are both worse: a channel would switch a fetch on for somebody
    /// who never chose it, and an error would fail the whole settings file and
    /// take every other setting on that machine back to its default.
    ///
    /// A named function rather than a serde attribute, for the reason
    /// `application::allowed` gives about its own default: the attribute that
    /// looks right answers for a missing key, and this is about a key that is
    /// present and holds something else.
    fn from_stored(stored: &str) -> Self {
        match stored {
            "public_releases" => Self::PublicReleases,
            "development_releases" => Self::DevelopmentReleases,
            _ => Self::NotLooking,
        }
    }
}

impl Serialize for WhichUpdates {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_stored())
    }
}

impl<'de> Deserialize<'de> for WhichUpdates {
    /// Written by hand so an answer this build does not know can load.
    ///
    /// A derived reader refuses one, and there is no serde attribute that
    /// covers it for an enum written as a plain string. What this does not
    /// cover is a stored value that is not a string at all, which still fails
    /// the read; the safe direction survives, because a failed read leaves
    /// every setting at its default and this one's default asks for nothing.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_stored(&String::deserialize(deserializer)?))
    }
}

/// Whether a published release is an offer for the person in front of us.
///
/// Three answers rather than a bool, for the reason [`Compared`] has four. A
/// caller that can see only "yes" and "no" maps a tag it could not read onto
/// one of them, and plan 07-09 acts on this answer by downloading an
/// executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offer {
    /// Newer than what is running, and of a kind this channel offers.
    Yes,
    /// Nothing here is worth telling anybody about.
    NothingNewer,
    /// The published version is not one this program can read.
    CouldNotRead,
}

/// Whether `candidate` is an offer for somebody running `running`.
///
/// Two questions, and the answer is the conjunction: is it newer, and is it
/// the kind of release this channel is for.
pub fn whether_to_offer(candidate: &str, running: &str, channel: ReleaseChannel) -> Offer {
    match compare(candidate, running) {
        Compared::CouldNotRead => Offer::CouldNotRead,
        Compared::Same | Compared::Older => Offer::NothingNewer,
        Compared::Newer if is_prerelease(candidate) && !channel.offers_prereleases() => {
            Offer::NothingNewer
        }
        Compared::Newer => Offer::Yes,
    }
}

/// Whether a version stages a release rather than being one.
///
/// The one place that knows what a prerelease is, so the channel rule asks it
/// rather than looking for a hyphen itself. A string this cannot read is not a
/// prerelease, and it never reaches the channel rule anyway: [`compare`] has
/// already answered [`Compared::CouldNotRead`] for it.
fn is_prerelease(version: &str) -> bool {
    parse(version).is_some_and(|version| !matches!(version.stage, Stage::Release))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_later_patch_minor_or_major_is_newer() {
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
        assert_eq!(compare("0.6.0", "0.5.9"), Compared::Newer);
        assert_eq!(compare("1.0.0", "0.99.0"), Compared::Newer);
        // The other direction, so a comparison that answers Newer for
        // everything cannot pass this.
        assert_eq!(compare("0.5.0", "0.5.1"), Compared::Older);
        assert_eq!(compare("0.99.0", "1.0.0"), Compared::Older);
    }

    #[test]
    fn test_the_same_version_is_neither_newer_nor_older() {
        assert_eq!(compare("0.5.0", "0.5.0"), Compared::Same);
    }

    #[test]
    fn test_a_build_identifier_after_a_plus_is_not_part_of_the_version() {
        // Both directions on purpose. A naive string comparison answers this
        // correctly in one direction, because the longer string sorts after,
        // and gets the other one backwards.
        assert_eq!(compare("0.5.0+g64c73dd", "0.5.0"), Compared::Same);
        assert_eq!(compare("0.5.0", "0.5.0+g64c73dd"), Compared::Same);
    }

    #[test]
    fn test_two_builds_of_one_version_are_the_same_version() {
        assert_eq!(compare("0.5.0+g64c73dd", "0.5.0+gaaaaaaa"), Compared::Same);
        assert_eq!(compare("0.5.0+gaaaaaaa", "0.5.0+g64c73dd"), Compared::Same);
    }

    #[test]
    fn test_a_release_is_newer_than_the_prerelease_that_staged_it() {
        // A prerelease stages the release rather than following it, so the
        // suffix sorts the version below the one it is a suffix of.
        assert_eq!(compare("0.6.0", "0.6.0-alpha.1"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.1", "0.6.0"), Compared::Older);
    }

    #[test]
    fn test_the_three_prerelease_words_sort_in_the_order_the_workflow_makes_them() {
        // alpha, beta and rc are the three levels
        // `.github/workflows/release.yml` offers, in that order.
        assert_eq!(compare("0.6.0-beta.1", "0.6.0-alpha.2"), Compared::Newer);
        assert_eq!(compare("0.6.0-rc.1", "0.6.0-beta.9"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.2", "0.6.0-beta.1"), Compared::Older);
    }

    #[test]
    fn test_the_prerelease_counter_is_compared_as_a_number() {
        // As text, "10" sorts before "9".
        assert_eq!(compare("0.6.0-alpha.10", "0.6.0-alpha.9"), Compared::Newer);
        assert_eq!(compare("0.6.0-alpha.9", "0.6.0-alpha.10"), Compared::Older);
    }

    #[test]
    fn test_a_published_tag_carries_a_v_and_is_still_a_version() {
        // What gets published is a tag rather than a bare version.
        // `.github/workflows/release.yml:115` names the portable copy after
        // the tag and `:133` publishes it with the glob `wixen-mail-v*.exe`,
        // so the tag starts with a `v` and that is the string a release feed
        // hands back. Refusing it would answer CouldNotRead for every real
        // release this project will ever cut.
        assert_eq!(compare("v0.5.1", "0.5.0"), Compared::Newer);
        assert_eq!(compare("v0.5.0", "v0.5.0"), Compared::Same);
        assert_eq!(compare("v0.6.0", "v0.6.0-alpha.1"), Compared::Newer);
    }

    #[test]
    fn test_a_version_it_cannot_read_is_refused_rather_than_treated_as_very_old() {
        // Paired with a readable case, so a comparison answering CouldNotRead
        // for everything could not pass this.
        assert_eq!(compare("banana", "0.5.0"), Compared::CouldNotRead);
        assert_eq!(compare("0.5.0", "banana"), Compared::CouldNotRead);
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
    }

    #[test]
    fn test_a_string_that_is_not_a_version_is_refused_rather_than_crashing() {
        // Every string below is one a server could return and none of them is
        // a version, so what this asserts is that the answer is an answer
        // rather than a crash, an overflow or a hang.
        for odd in [
            "",
            " ",
            "0",
            "0.5",
            "0.5.0.0",
            "-1.0.0",
            "0.5.0-",
            "0.5.0-alpha",
            "0.5.0-alpha.",
            "0.5.0+",
            "+",
            "...",
            "99999999999999999999999.0.0",
            "0.5.0-alpha.99999999999999999999999",
            "\u{1f600}",
            "0.5.0\n",
        ] {
            assert_eq!(compare(odd, "0.5.0"), Compared::CouldNotRead, "{odd:?}");
            assert_eq!(compare("0.5.0", odd), Compared::CouldNotRead, "{odd:?}");
        }
        // Paired with the ordinary case the whole module is for. Every
        // assertion above is satisfied by a comparison that refuses
        // everything, and that comparison is the one this pairing catches.
        assert_eq!(compare("0.5.1", "0.5.0"), Compared::Newer);
    }

    #[test]
    fn test_the_channel_somebody_has_not_chosen_is_the_released_only_one() {
        // The safe end, and the type says it rather than every caller
        // remembering to. Getting this one wrong opts somebody into
        // prereleases who never asked, and once 07-09 lands that means
        // fetching an installer built from one.
        assert_eq!(ReleaseChannel::default(), ReleaseChannel::PublicReleases);
    }

    #[test]
    fn test_the_setting_nobody_has_touched_is_not_looking() {
        assert_eq!(WhichUpdates::default(), WhichUpdates::NotLooking);
    }

    #[test]
    fn test_not_looking_asks_no_channel() {
        assert_eq!(WhichUpdates::NotLooking.channel(), None);
        // Paired with the other two, so a mapping answering None for
        // everything could not pass this.
        assert!(WhichUpdates::PublicReleases.channel().is_some());
        assert!(WhichUpdates::DevelopmentReleases.channel().is_some());
    }

    #[test]
    fn test_public_releases_asks_the_channel_that_leaves_prereleases_out() {
        assert_eq!(
            WhichUpdates::PublicReleases.channel(),
            Some(ReleaseChannel::PublicReleases)
        );
    }

    #[test]
    fn test_development_releases_asks_the_channel_that_takes_prereleases_in() {
        assert_eq!(
            WhichUpdates::DevelopmentReleases.channel(),
            Some(ReleaseChannel::DevelopmentReleases)
        );
    }

    #[test]
    fn test_each_answer_has_its_own_stored_spelling() {
        // These three strings are fixed from here on. Once one settings file
        // on one machine holds one of them, renaming it makes that machine's
        // answer unreadable, which is CLAUDE.md's rule about never renaming
        // what shipped applied to a stored value rather than to a column.
        assert_eq!(WhichUpdates::NotLooking.as_stored(), "not_looking");
        assert_eq!(WhichUpdates::PublicReleases.as_stored(), "public_releases");
        assert_eq!(
            WhichUpdates::DevelopmentReleases.as_stored(),
            "development_releases"
        );
    }

    #[test]
    fn test_every_answer_survives_being_written_and_read_back() {
        for chosen in [
            WhichUpdates::NotLooking,
            WhichUpdates::PublicReleases,
            WhichUpdates::DevelopmentReleases,
        ] {
            let written = serde_json::to_string(&chosen).expect("a string always serialises");
            let read_back: WhichUpdates =
                serde_json::from_str(&written).expect("what was just written reads back");
            assert_eq!(read_back, chosen, "{written}");
        }
    }

    #[test]
    fn test_a_stored_answer_this_build_does_not_know_is_not_looking() {
        // The case a downgrade produces. Reading it as either channel would
        // quietly switch the check on for somebody who never chose it, and
        // reading it as an error fails the whole settings file.
        let unknown: WhichUpdates =
            serde_json::from_str("\"nightly_releases\"").expect("an unknown answer still loads");
        assert_eq!(unknown, WhichUpdates::NotLooking);
        // Paired, so a read that answered NotLooking for everything could not
        // pass this.
        let known: WhichUpdates =
            serde_json::from_str("\"development_releases\"").expect("a known answer loads");
        assert_eq!(known, WhichUpdates::DevelopmentReleases);
    }

    #[test]
    fn test_whether_a_release_is_an_offer_depends_on_the_channel_somebody_chose() {
        // Every row is asked twice, so a rule that ignored its channel
        // argument could not pass. The two rows marked below are the ones
        // where the two channels give different answers; without them this
        // table would be green against exactly that rule.
        let rows = [
            // candidate, running, on the public channel, on the development channel
            (
                "0.6.0-alpha.1",
                "0.5.0",
                Offer::NothingNewer,
                Offer::Yes, // the two channels differ here
            ),
            (
                "0.6.0-alpha.3",
                "0.6.0-alpha.2",
                Offer::NothingNewer,
                Offer::Yes, // and here
            ),
            ("0.6.0", "0.6.0-alpha.1", Offer::Yes, Offer::Yes),
            ("0.5.1", "0.5.0", Offer::Yes, Offer::Yes),
            (
                "0.6.0-alpha.1",
                "0.6.0-alpha.2",
                Offer::NothingNewer,
                Offer::NothingNewer,
            ),
            (
                "0.6.0",
                "0.7.0-alpha.1",
                Offer::NothingNewer,
                Offer::NothingNewer,
            ),
            ("0.5.0", "0.5.0", Offer::NothingNewer, Offer::NothingNewer),
            (
                "0.5.0+g64c73dd",
                "0.5.0",
                Offer::NothingNewer,
                Offer::NothingNewer,
            ),
            ("banana", "0.5.0", Offer::CouldNotRead, Offer::CouldNotRead),
        ];

        for (candidate, running, on_public, on_development) in rows {
            assert_eq!(
                whether_to_offer(candidate, running, ReleaseChannel::PublicReleases),
                on_public,
                "{candidate} against {running} on the public channel"
            );
            assert_eq!(
                whether_to_offer(candidate, running, ReleaseChannel::DevelopmentReleases),
                on_development,
                "{candidate} against {running} on the development channel"
            );
        }
    }

    #[test]
    fn test_a_tag_it_cannot_read_is_a_different_answer_from_nothing_newer() {
        // A caller that mapped these two onto one answer would either say
        // nothing when the check has broken, or say there is an update when
        // there is not.
        for channel in [
            ReleaseChannel::PublicReleases,
            ReleaseChannel::DevelopmentReleases,
        ] {
            assert_eq!(
                whether_to_offer("release-candidate", "0.5.0", channel),
                Offer::CouldNotRead
            );
            assert_eq!(
                whether_to_offer("0.4.0", "0.5.0", channel),
                Offer::NothingNewer
            );
        }
    }

    #[test]
    fn test_a_plain_build_is_just_the_number() {
        assert_eq!(describe("0.5.0", ""), "0.5.0");
    }

    #[test]
    fn test_a_build_from_the_installer_script_says_which_commit() {
        // The whole reason this exists: a log line that can be matched to the
        // code it came from when several builds share a version.
        assert_eq!(describe("0.5.0", "g64c73dd"), "0.5.0+g64c73dd");
    }

    #[test]
    fn test_the_build_is_metadata_rather_than_part_of_the_version() {
        // After a `+`, which is what makes version ordering ignore it. Putting
        // it after a `-` would make every build a prerelease of the version it
        // was built from, and sort them below it.
        let described = describe("0.5.0", "g64c73dd");

        assert!(described.starts_with("0.5.0+"), "{described}");
        assert!(
            !described.contains("0.5.0-"),
            "a build identifier must not read as a prerelease: {described}"
        );
    }

    #[test]
    fn test_this_build_says_something() {
        // Guards against `WIXEN_BUILD` not being set at all, which would stop
        // the crate compiling, and against a version that came out empty.
        assert!(current().starts_with(NUMBER), "{}", current());
        assert!(!NUMBER.is_empty());
    }
}
