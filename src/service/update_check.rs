//! Asking GitHub whether there is a newer version of Wixen Mail.
//!
//! A `GET` and nothing else. Nothing at anybody's account changes, nothing is
//! signed in to, and no identifier for a person leaves this computer. What does
//! leave is the request itself, which GitHub associates with the address it
//! came from: "Unauthenticated requests are associated with the originating IP
//! address, not with the user or application that made the request."
//! [docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api,
//! read 2026-09-12]. `docs/privacy.md` says so in the words somebody using this
//! would read, and [`WHERE_THE_ASKING_GOES`] is tied to that page by a test.
//!
//! # The shape, and why it is split this way
//!
//! [`ask`] is a thin shell: it builds a URL, sends one request, and hands what
//! came back to [`what_the_answer_means`], which is pure. Every rule about what
//! an answer means is therefore a rule about text, testable with no socket
//! open. The transport is the half no test here covers, which is stated rather
//! than hidden.
//!
//! # Nothing is downloaded and nothing is run
//!
//! This module reads a version number and says a sentence. When there is a
//! newer version somebody is offered the releases page in a browser. Fetching
//! an installer and running it is plan 07-09's, after signing lands in 07-08,
//! and nothing here is a step toward doing it without being asked.

use crate::common::version::{self, Offer, ReleaseChannel, WhichUpdates};
use serde::Deserialize;

/// The heading the one update setting sits under on the settings screen.
///
/// A constant rather than a string typed into the screen, because sentences
/// elsewhere send somebody to it by name: the answer this check gives on the
/// public channel names the setting that would show test versions, and
/// `docs/privacy.md` says where the check is switched on. Two copies of a
/// heading is how "Allow Changes" came to be labelled "Allowed Changes", near
/// enough to look right and far enough that somebody stops to check.
pub const SETTINGS_SECTION: &str = "New versions";

/// What the one update control is labelled, without its keyboard mark.
pub const WHICH_UPDATES_LABEL: &str = "Tell me about new versions";

/// What the control says about what choosing a version kind will mean.
///
/// The consent is given here or it is not given. Somebody choosing either kind
/// is agreeing that this program will later fetch an installer, several
/// megabytes of it, without asking again, and a warning that lives only in a
/// document is a warning nobody gets.
///
/// It says "will" rather than "does" because nothing is downloaded yet, and a
/// control claiming a capability this build does not have is the same defect
/// one step along.
pub const WHICH_UPDATES_DESCRIPTION: &str = "Wixen Mail asks GitHub which versions have been published. Nothing about you is sent. \
     When updating is finished, choosing either kind of version will mean the installer for \
     it is downloaded without asking you again; nothing is downloaded yet.";

/// The one host this module ever asks.
///
/// Named once so `docs/privacy.md` can be tied to it rather than to a phrase
/// somebody may reword.
pub const WHERE_THE_ASKING_GOES: &str = "api.github.com";

/// Who asked, which decides whether a check happens and on which channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoAsked {
    /// Somebody chose the Help menu item.
    ByHand,
    /// The program started, and nobody pressed anything.
    AtStart,
}

/// Which channel a check asks, and whether it happens at all.
///
/// Two rules in one place rather than two callers matching on three values.
///
/// A check at start happens only where somebody chose a kind of version, which
/// is what keeps "nothing is fetched unless somebody asked" true.
///
/// A check asked for by hand always happens, per D-16, because that is the
/// deliberate path and it must not depend on the automatic one. With nothing
/// chosen it asks the public channel: that is the conservative answer, since
/// the public channel never offers a test version and a test version is
/// exactly what somebody who never opted in should not be handed. Asking which
/// channel at that moment was the other candidate and it is a question put in
/// front of somebody who has already asked for the thing, every time.
pub const fn channel_for(setting: WhichUpdates, asked: WhoAsked) -> Option<ReleaseChannel> {
    match asked {
        WhoAsked::AtStart => setting.channel(),
        // `ReleaseChannel::default()` rather than the variant by name, because
        // 07-04 already decided that a channel nobody chose is the one that
        // offers least and said so in that type's own doc comment.
        WhoAsked::ByHand => Some(match setting.channel() {
            Some(chosen) => chosen,
            None => ReleaseChannel::PublicReleases,
        }),
    }
}

/// What came back, reduced to the three things any rule here reads.
///
/// A struct of its own rather than a `reqwest::Response`, so every rule below
/// is a rule about text and a number and can be driven from a fixture. The
/// crate's own types stop at [`ask`].
#[derive(Debug, Clone)]
pub struct Reply {
    /// The status GitHub answered with.
    pub status: u16,
    /// `x-ratelimit-remaining`, which GitHub documents as being zero when a
    /// rate limit is what refused the request. Absent when the header was not
    /// sent or could not be read as a number.
    pub requests_left: Option<u64>,
    /// The body, as text.
    pub body: String,
}

/// What a check learned, and nothing it did not.
///
/// Separate variants rather than a version and a flag, because the whole point
/// is that "there is nothing newer" and "I could not find out" are different
/// things to be told and only one of them means somebody is up to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// A published version newer than this build, on the channel asked.
    ANewerVersion {
        /// The tag as GitHub published it, which carries a leading `v`.
        version: String,
        /// The page a person can read about it on.
        page: String,
        /// Which channel was asked.
        channel: ReleaseChannel,
    },
    /// Something is published, and none of it is newer than this build.
    ThisIsTheNewest {
        /// Which channel was asked.
        channel: ReleaseChannel,
    },
    /// The request did not happen, or came back refused.
    CouldNotBeFetched,
}

impl Answer {
    /// The sentence somebody hears and reads.
    ///
    /// Built here rather than at the screen, so the wording can be argued about
    /// in a test and so no caller can invent a sixth thing to say.
    pub fn said(&self) -> String {
        match self {
            Self::ANewerVersion {
                version, channel, ..
            } => format!(
                "Wixen Mail {version} has been published. You are running {}. Asked on: {}.",
                version::current(),
                which_was_asked(*channel)
            ),
            Self::ThisIsTheNewest { channel } => format!(
                "You are running the newest published version, {}. Asked on: {}.",
                version::current(),
                which_was_asked(*channel)
            ),
            Self::CouldNotBeFetched => "Wixen Mail could not ask GitHub which versions have \
                 been published, so it does not know whether there is a newer one. Your \
                 version has not changed."
                .to_string(),
        }
    }
}

/// What a channel is called where somebody reads which one was asked.
///
/// The same words the control offers, so a person told which channel was asked
/// can look for it on the settings screen under the name they just heard.
fn which_was_asked(channel: ReleaseChannel) -> &'static str {
    match channel {
        ReleaseChannel::PublicReleases => WhichUpdates::PublicReleases.words(),
        ReleaseChannel::DevelopmentReleases => WhichUpdates::DevelopmentReleases.words(),
    }
}

/// The most releases considered on the development channel.
///
/// GitHub's list endpoint is paginated, `per_page` and `page`, thirty by
/// default and a hundred at most, so this asks for the most one page will give.
/// One page rather than several: the answer wanted is the newest version, and a
/// project whose newest release is not among its hundred most recent has other
/// problems. Somebody in that position is told this is the newest, which is
/// wrong, and the bound is written down here so the next person can see the
/// case rather than discover it.
const MOST_RELEASES_CONSIDERED: u32 = 100;

/// Where a channel's question goes.
///
/// Two endpoints rather than one with a parameter, because they behave
/// differently enough that a parameter would hide it. `releases/latest`
/// "retrieves the most recent non-prerelease, non-draft release"
/// [docs.github.com/en/rest/releases/releases], so the public channel gets its
/// whole rule from GitHub. The development channel cannot use it, because it
/// would never see an alpha, so it asks for the list and puts it in order here.
fn endpoint(channel: ReleaseChannel) -> String {
    let repository = which_repository();
    match channel {
        ReleaseChannel::PublicReleases => {
            format!("https://{WHERE_THE_ASKING_GOES}/repos/{repository}/releases/latest")
        }
        ReleaseChannel::DevelopmentReleases => format!(
            "https://{WHERE_THE_ASKING_GOES}/repos/{repository}/releases?per_page={MOST_RELEASES_CONSIDERED}"
        ),
    }
}

/// Whose releases are asked about, as `owner/name`.
///
/// Taken from the manifest rather than written out, so this cannot come to
/// point at somebody else's project after a move. `Cargo.toml`'s `repository`
/// is the one place that address is already kept.
fn which_repository() -> String {
    env!("CARGO_PKG_REPOSITORY")
        .trim_end_matches('/')
        .trim_start_matches("https://github.com/")
        .to_string()
}

/// What this program calls itself to GitHub.
///
/// Not ceremony. GitHub: "Requests with no `User-Agent` header will be
/// rejected. If you provide an invalid `User-Agent` header, you will receive a
/// `403 Forbidden` response."
/// [docs.github.com/en/rest/using-the-rest-api/getting-started-with-the-rest-api].
/// Measured the same day rather than taken on trust: a request sent with an
/// empty agent came back `403` with a body naming this requirement.
fn who_is_asking() -> String {
    format!("wixen-mail/{}", env!("CARGO_PKG_VERSION"))
}

/// One published release, reduced to the two things this reads.
///
/// serde passes over the twenty other fields GitHub sends, which was checked
/// against a response caught off the wire rather than against the reference
/// page: see `A_REAL_RELEASE` in the tests below.
#[derive(Debug, Clone, Deserialize)]
struct Published {
    /// The tag, which carries a leading `v` on anything this project publishes.
    tag_name: String,
    /// The page a person reads about the release on.
    html_url: String,
}

/// What a reply means for the person in front of us.
///
/// Pure. Every rule about an answer is a rule about a status, a header and a
/// body, so none of this needs a socket to be tested and none of it can be
/// right only against a server that happens to be up.
pub fn what_the_answer_means(channel: ReleaseChannel, running: &str, reply: &Reply) -> Answer {
    if reply.status != 200 {
        return Answer::CouldNotBeFetched;
    }
    match channel {
        ReleaseChannel::PublicReleases => match serde_json::from_str::<Published>(&reply.body) {
            Ok(published) => whether_that_one_is_an_offer(&published, running, channel),
            Err(_) => Answer::CouldNotBeFetched,
        },
        ReleaseChannel::DevelopmentReleases => {
            match serde_json::from_str::<Vec<Published>>(&reply.body) {
                Ok(published) => the_newest_offer_among(&published, running, channel),
                Err(_) => Answer::CouldNotBeFetched,
            }
        }
    }
}

/// Whether one published release is something to tell somebody about.
///
/// The channel rule is [`version::whether_to_offer`]'s and is not repeated
/// here: a prerelease is never an offer on the public channel, and that lives
/// in one place for the reason 07-04 gives about two callers deciding
/// independently.
fn whether_that_one_is_an_offer(
    published: &Published,
    running: &str,
    channel: ReleaseChannel,
) -> Answer {
    match version::whether_to_offer(&published.tag_name, running, channel) {
        Offer::Yes => Answer::ANewerVersion {
            version: published.tag_name.clone(),
            page: published.html_url.clone(),
            channel,
        },
        // A tag this program cannot read is not a newer version, and saying so
        // is the honest answer for one release: something is published and none
        // of it is an offer. The list below counts what it could not read,
        // because there the difference decides whether the answer is worth
        // anything.
        Offer::NothingNewer | Offer::CouldNotRead => Answer::ThisIsTheNewest { channel },
    }
}

/// The newest offer in a list, by this project's own ordering.
///
/// Not element zero. GitHub documents `per_page` and `page` for this endpoint
/// and documents no ordering for it at all, and an order nobody has written
/// down can change without an announcement. Measured on 2026-09-12 as well as
/// read: the first entry of a real list was a rolling `nightly` tag, which is a
/// prerelease and is not a version this program can put in order, so element
/// zero was the wrong answer twice over on the first list anybody looked at.
fn the_newest_offer_among(
    published: &[Published],
    running: &str,
    channel: ReleaseChannel,
) -> Answer {
    let mut best: Option<&Published> = None;
    for candidate in published {
        if version::whether_to_offer(&candidate.tag_name, running, channel) != Offer::Yes {
            continue;
        }
        let newer_than_best = best.is_none_or(|held| {
            version::compare(&candidate.tag_name, &held.tag_name) == version::Compared::Newer
        });
        if newer_than_best {
            best = Some(candidate);
        }
    }
    match best {
        Some(offer) => Answer::ANewerVersion {
            version: offer.tag_name.clone(),
            page: offer.html_url.clone(),
            channel,
        },
        None => Answer::ThisIsTheNewest { channel },
    }
}

/// Ask GitHub, and say what came back.
///
/// The whole of the transport, which is why it is this short: everything that
/// decides anything is in [`what_the_answer_means`] above.
///
/// Built through [`crate::service::outward::Outward::read_only`] rather than on
/// a client of its own. This only ever reads, so the constructor that cannot
/// change anything is the honest one, and the gate is then a fact about the
/// value rather than a promise in a comment.
pub async fn ask(channel: ReleaseChannel) -> Answer {
    let gate = crate::service::outward::Outward::read_only(reqwest::Client::new());
    let sent = gate
        .reading(&endpoint(channel))
        .header(reqwest::header::USER_AGENT, who_is_asking())
        .send()
        .await;
    let Ok(response) = sent else {
        return Answer::CouldNotBeFetched;
    };
    let status = response.status().as_u16();
    let requests_left = response
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|left| left.to_str().ok())
        .and_then(|left| left.parse().ok());
    let Ok(body) = response.text().await else {
        return Answer::CouldNotBeFetched;
    };
    what_the_answer_means(
        channel,
        &version::current(),
        &Reply {
            status,
            requests_left,
            body,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real response, caught off the wire on 2026-09-12 and trimmed.
    ///
    /// Not written from what this code expects, which would only prove the code
    /// agrees with itself. The command was:
    ///
    /// ```text
    /// curl -H "User-Agent: wixen-mail-dev" \
    ///   https://api.github.com/repos/rust-lang/rust-analyzer/releases/latest
    /// ```
    ///
    /// Trimmed by taking out the eighteen assets, the author block and the
    /// reactions block, and shortening the body: every top-level field this
    /// module could read is here with the value GitHub really sent. Another
    /// repository, because this one has published nothing.
    const A_REAL_RELEASE: &str = r#"{"url":"https://api.github.com/repos/rust-lang/rust-analyzer/releases/383853727","assets_url":"https://api.github.com/repos/rust-lang/rust-analyzer/releases/383853727/assets","upload_url":"https://uploads.github.com/repos/rust-lang/rust-analyzer/releases/383853727/assets{?name,label}","html_url":"https://github.com/rust-lang/rust-analyzer/releases/tag/2026-09-07","id":383853727,"node_id":"RE_kwDOBttd2s4W4SSf","tag_name":"2026-09-07","target_commitish":"master","name":"2026-09-07","draft":false,"immutable":false,"prerelease":false,"created_at":"2026-09-06T08:11:37Z","updated_at":"2026-09-07T06:24:20Z","published_at":"2026-09-07T05:45:17Z","tarball_url":"https://api.github.com/repos/rust-lang/rust-analyzer/tarball/2026-09-07","zipball_url":"https://api.github.com/repos/rust-lang/rust-analyzer/zipball/2026-09-07","body":"Commit: 9074e9b","author":{"login":"github-actions[bot]"},"assets":[{"name":"rust-analyzer-aarch64-apple-darwin.gz","browser_download_url":"https://github.com/rust-lang/rust-analyzer/releases/download/2026-09-07/rust-analyzer-aarch64-apple-darwin.gz"}]}"#;

    /// The same shape with a tag this project could really publish.
    ///
    /// The shape is the real one above, field for field. Only `tag_name` and
    /// `html_url` are substituted, because no tag has ever been published from
    /// this repository and one had to be invented to have a newer version at
    /// all. Which of the two halves is real is said here so nobody reads the
    /// whole fixture as evidence.
    fn a_release_tagged(tag: &str) -> String {
        A_REAL_RELEASE
            .replace(
                "\"tag_name\":\"2026-09-07\"",
                &format!("\"tag_name\":\"{tag}\""),
            )
            .replace("releases/tag/2026-09-07", &format!("releases/tag/{tag}"))
    }

    fn answered(body: &str) -> Reply {
        Reply {
            status: 200,
            requests_left: Some(59),
            body: body.to_string(),
        }
    }

    #[test]
    fn test_a_published_release_newer_than_this_build_is_offered_and_named() {
        let answer = what_the_answer_means(
            ReleaseChannel::PublicReleases,
            "0.115.0",
            &answered(&a_release_tagged("v0.116.0")),
        );
        assert_eq!(
            answer,
            Answer::ANewerVersion {
                version: "v0.116.0".to_string(),
                page: "https://github.com/rust-lang/rust-analyzer/releases/tag/v0.116.0"
                    .to_string(),
                channel: ReleaseChannel::PublicReleases,
            },
            "a published version newer than this build is the whole point of asking"
        );
    }

    #[test]
    fn test_a_published_release_that_is_not_an_offer_is_reported_as_this_being_the_newest() {
        // Older, which is the ordinary case between releases.
        assert_eq!(
            what_the_answer_means(
                ReleaseChannel::PublicReleases,
                "0.115.0",
                &answered(&a_release_tagged("v0.114.0")),
            ),
            Answer::ThisIsTheNewest {
                channel: ReleaseChannel::PublicReleases
            }
        );
        // The same version, which is what somebody running the newest gets.
        assert_eq!(
            what_the_answer_means(
                ReleaseChannel::PublicReleases,
                "0.115.0+g64c73dd",
                &answered(&a_release_tagged("v0.115.0")),
            ),
            Answer::ThisIsTheNewest {
                channel: ReleaseChannel::PublicReleases
            },
            "a build identifier is not part of the version, so this is the same version"
        );
    }

    #[test]
    fn test_a_real_published_release_reads_as_a_version_and_a_page() {
        // The fixture nothing here wrote. Its tag is `2026-09-07`, which is a
        // date and not a version this program can read, so the answer is that
        // it is not newer rather than that it is. That is the honest reading:
        // the comparison refuses what it cannot put in order, and this is what
        // a real feed can really contain.
        let answer = what_the_answer_means(
            ReleaseChannel::PublicReleases,
            "0.115.0",
            &answered(A_REAL_RELEASE),
        );
        assert_ne!(
            answer,
            Answer::CouldNotBeFetched,
            "a response that arrived is not a response that never came back"
        );
        // And the reading really reached the fields, rather than passing the
        // whole thing over: substituting a readable tag into the same bytes
        // changes the answer.
        assert_eq!(
            what_the_answer_means(
                ReleaseChannel::PublicReleases,
                "0.115.0",
                &answered(&a_release_tagged("v9.9.9")),
            ),
            Answer::ANewerVersion {
                version: "v9.9.9".to_string(),
                page: "https://github.com/rust-lang/rust-analyzer/releases/tag/v9.9.9".to_string(),
                channel: ReleaseChannel::PublicReleases,
            }
        );
    }

    #[test]
    fn test_the_answers_say_different_things_and_name_the_channel_they_asked() {
        let newer = Answer::ANewerVersion {
            version: "v0.116.0".to_string(),
            page: "https://example.invalid/r".to_string(),
            channel: ReleaseChannel::PublicReleases,
        }
        .said();
        let newest = Answer::ThisIsTheNewest {
            channel: ReleaseChannel::PublicReleases,
        }
        .said();
        let unfetched = Answer::CouldNotBeFetched.said();

        for (one, other) in [
            (&newer, &newest),
            (&newer, &unfetched),
            (&newest, &unfetched),
        ] {
            assert_ne!(
                one, other,
                "two answers that say the same thing are one answer"
            );
        }

        // Each of the two that asked says which channel it asked, so somebody
        // who expected a test version and was told they are current can tell
        // why. The one that never got an answer does not, because it has
        // nothing to attribute.
        for said in [&newer, &newest] {
            assert!(
                said.contains(WhichUpdates::PublicReleases.words()),
                "an answer that does not say which channel it asked leaves \
                 somebody on the wrong one with no way to tell: {said}"
            );
        }
    }

    #[test]
    fn test_each_kind_of_version_somebody_can_choose_has_its_own_words() {
        let words: Vec<&str> = WhichUpdates::ALL.iter().map(|kind| kind.words()).collect();
        for said in &words {
            assert!(
                !said.is_empty(),
                "a choice with no words is a choice nobody can read"
            );
        }
        for (at, one) in words.iter().enumerate() {
            for other in &words[at + 1..] {
                assert_ne!(one, other, "two choices reading the same are one choice");
            }
        }
    }

    #[test]
    fn test_the_two_channels_ask_different_endpoints_and_the_public_one_excludes_test_versions() {
        let public = endpoint(ReleaseChannel::PublicReleases);
        let development = endpoint(ReleaseChannel::DevelopmentReleases);

        assert!(
            public.ends_with("/releases/latest"),
            "the public channel gets its whole prerelease rule from GitHub by asking \
             the endpoint that excludes them, and this asks {public}"
        );
        assert!(
            development.contains("/releases?"),
            "the development channel has to ask for the list, because the latest \
             endpoint would never show it an alpha, and this asks {development}"
        );
        assert_ne!(public, development);
        for asked in [&public, &development] {
            assert!(
                asked.contains(WHERE_THE_ASKING_GOES),
                "{asked} does not go to the one host this module is allowed to ask"
            );
            assert!(
                asked.contains("Wixen-Mail"),
                "{asked} does not name this repository, so it asks about somebody else's releases"
            );
        }
    }

    #[test]
    fn test_the_request_says_which_program_is_asking() {
        // GitHub rejects a request with no agent outright, which was measured
        // rather than read: an empty agent came back 403 with a body naming
        // this requirement.
        let asking = who_is_asking();
        assert!(
            asking.contains("wixen-mail"),
            "GitHub rejects a request that does not name the program asking, and \
             this sends {asking:?}"
        );
        assert!(
            asking.contains(env!("CARGO_PKG_VERSION")),
            "the agent does not carry the version asking, so a bad answer cannot be \
             matched to a build: {asking:?}"
        );
    }

    #[test]
    fn test_a_check_at_start_happens_only_where_somebody_chose_a_kind_of_version() {
        assert_eq!(
            channel_for(WhichUpdates::NotLooking, WhoAsked::AtStart),
            None,
            "nothing is fetched for somebody who did not ask for it"
        );
        assert_eq!(
            channel_for(WhichUpdates::PublicReleases, WhoAsked::AtStart),
            Some(ReleaseChannel::PublicReleases)
        );
        assert_eq!(
            channel_for(WhichUpdates::DevelopmentReleases, WhoAsked::AtStart),
            Some(ReleaseChannel::DevelopmentReleases)
        );
    }

    #[test]
    fn test_a_check_asked_for_by_hand_asks_the_public_channel_when_nobody_chose() {
        assert_eq!(
            channel_for(WhichUpdates::NotLooking, WhoAsked::ByHand),
            Some(ReleaseChannel::PublicReleases),
            "the Help menu item works whatever the setting says, and with nothing \
             chosen it asks the channel that never offers a test version"
        );
        // And it still follows a choice that was made, or it would be a second
        // setting wearing the first one's label.
        assert_eq!(
            channel_for(WhichUpdates::DevelopmentReleases, WhoAsked::ByHand),
            Some(ReleaseChannel::DevelopmentReleases)
        );
    }

    #[test]
    fn test_the_settings_screen_offers_the_one_update_setting_by_its_constants() {
        // The companion `every_setting_is_acted_on`'s mirror guard cannot be.
        // That one is satisfied by the field's name appearing anywhere in the
        // settings screen, including in a comment, so it would pass for a
        // control that shows a fixed value and is read back into nothing.
        let screen = std::fs::read_to_string("src/presentation/wx_settings.rs")
            .expect("the settings screen");
        let ships = crate::common::what_ships::what_ships(&screen);
        assert!(
            !ships.is_empty(),
            "the settings screen could not be read, so this proves nothing"
        );

        for named in [
            "update_check::SETTINGS_SECTION",
            "update_check::WHICH_UPDATES_LABEL",
            "update_check::WHICH_UPDATES_DESCRIPTION",
        ] {
            assert!(
                ships.contains(named),
                "the settings screen does not name {named}, so the heading, the label \
                 or the warning about downloading is a second copy that can drift from \
                 the sentence that sends somebody here"
            );
        }

        // And it is a control rather than a mention: built from the three
        // answers, showing the stored one, and read back.
        assert!(
            ships.contains("WhichUpdates::ALL"),
            "the control is not built from the three answers, so an answer added later \
             would not appear on it"
        );
        assert!(
            ships.contains("config.which_updates"),
            "the control does not show the stored answer, so somebody who chose one is \
             shown whatever it was built with"
        );
        assert!(
            ships.contains("cfg.which_updates ="),
            "the control's value is not read back into the configuration, so choosing \
             changes nothing"
        );

        // The reading can see a violation. A check over one obedient file
        // passes whether it works or has been narrowed until it sees nothing,
        // and this project has shipped a document guard disarmed exactly that
        // way.
        assert!(
            !ships.contains("update_check::A_CONSTANT_THAT_DOES_NOT_EXIST"),
            "the reading answers yes for a name nothing writes, so it would answer \
             yes for every name"
        );
    }

    #[test]
    fn test_the_settings_screen_does_not_write_the_heading_or_the_label_out_itself() {
        // The section name and the label each exist once. Typed a second time
        // they drift, which is how a sync came to say "turn on Allow Changes"
        // above a section headed "Allowed Changes".
        //
        // Homed here rather than added to `sections_named_by_a_constant()` in
        // `tests/house_style.rs`: that file is fingerprinted by eighteen guard
        // records, and this file by none, so the same protection is free here
        // and costs about half an hour of re-measurement there.
        let screen = std::fs::read_to_string("src/presentation/wx_settings.rs")
            .expect("the settings screen");
        for name in [SETTINGS_SECTION, WHICH_UPDATES_LABEL] {
            let written_out = format!("\"{name}\"");
            assert!(
                !screen.contains(&written_out),
                "{written_out} is typed into the settings screen, so there are two \
                 copies of it and only one of them follows a rename"
            );
        }
        // The reading can see a violation: a string that really is typed into
        // that file is found by the same test.
        assert!(
            screen.contains("\"Appearance\""),
            "the reading cannot find a heading that is typed into the screen, so it \
             would find nothing however many were"
        );
    }
}
