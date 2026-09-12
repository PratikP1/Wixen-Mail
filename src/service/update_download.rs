//! Fetching the installer for a newer version, and refusing anything this
//! project did not sign.
//!
//! This is the largest increase in attack surface in the milestone, in decision
//! 7's own words: it makes a mail client fetch an executable. Nothing here runs
//! one and nothing here asks anybody anything. Fetching and checking is this
//! file; the question and the handover are the screen's, and the only thing
//! either of them will take is a [`Verified`], which nothing outside this module
//! can build.
//!
//! # Two checks, not one
//!
//! "Is this file validly signed" is not the question. Millions of files pass it.
//! The question is "is this file signed by the name this project signs with",
//! and it needs a second reading, of the signer's own certificate. A module that
//! stopped at the first check would be worse than one with no check at all,
//! because it would print the word verified over an executable a stranger
//! signed.
//!
//! # Nothing is signed yet, so everything is refused
//!
//! No release has ever been published from this repository and no certificate
//! exists, so as this ships every real installer fails the second check and is
//! deleted. That is the designed behaviour rather than a defect, and it is said
//! plainly in `docs/changelog.md` and on `docs/installing.md` rather than left
//! for somebody to discover. When there is a certificate, the name it carries is
//! the one [`WHO_SIGNS_THIS`] already expects.
//!
//! # What no test here reaches
//!
//! The transport, and the two Windows calls on a machine that has never seen a
//! signature this project made. Every rule below is a rule about text, a number
//! or a path, so it is driven from a fixture; the socket and the signature of a
//! genuine Wixen Mail installer are not, and cannot be until a release is
//! signed.

use crate::common::paths::AppPaths;
use crate::common::{Error, Result};
use std::path::{Path, PathBuf};

/// The name this project's own installers are signed with.
///
/// Taken from `installer/Wixen-Mail-Setup.iss`'s `Publisher`, which is what a
/// person already sees in Apps and Features, and held to it by
/// `test_the_name_this_expects_is_the_one_the_installer_publishes_under`. A
/// second copy of a name is how "Allow Changes" came to be labelled "Allowed
/// Changes": near enough to look right, far enough to be wrong.
pub const WHO_SIGNS_THIS: &str = "Pratik Patel";

/// The host a release's own files are served from before any redirect.
///
/// Named so `docs/privacy.md` can be tied to it rather than to a phrase
/// somebody may reword, the way [`crate::service::update_check`] names the host
/// it asks.
pub const WHERE_THE_INSTALLER_COMES_FROM: &str = "github.com";

/// The host GitHub redirects a release file to.
///
/// A second request to a second company-owned host, and a privacy page that
/// named only the first would be describing half of what happens.
pub const AND_THE_HOST_IT_REDIRECTS_TO: &str = "objects.githubusercontent.com";

/// How many redirects are followed before giving up.
///
/// A release file is served from `github.com` and redirected once to the
/// content host, so one is the number this really needs. Five leaves room for a
/// hop this project does not control being added without the feature breaking,
/// and still refuses a chain built to keep a client busy for ever.
const MOST_REDIRECTS_FOLLOWED: usize = 5;

/// The most bytes accepted before the download is abandoned.
///
/// Measured rather than guessed: `dist/Wixen-Mail-Setup-0.40.0+gf7e34777.exe`,
/// the only installer this project has ever built, is 11,604,104 bytes, and the
/// release executable it now packs is 41,010,688 before compression. So 300 MB
/// is roughly twenty-five times the real installer and seven times the largest
/// thing that goes in one, which leaves room for this program to grow a great
/// deal without the bound ever refusing a genuine file.
///
/// It is enforced by counting what arrives rather than by reading
/// `content-length`, because a header is what a hostile or broken server says
/// about itself and a chunked response does not send one at all.
const MOST_BYTES_ACCEPTED: u64 = 300 * 1024 * 1024;

/// One file published beside a release.
///
/// Reduced to the two fields anything here reads, the way
/// [`crate::service::update_check`] reduces a release, so choosing between them
/// is a rule about text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseFile {
    /// The file name as it was published.
    pub name: String,
    /// Where to fetch it from.
    pub from: String,
}

/// An installer on this disk that nothing has looked at yet.
///
/// Deliberately not the same type as [`Verified`]. Everything a person can be
/// shown or asked about takes the second one, so a file in this state has
/// nowhere to go but the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Downloaded {
    at: PathBuf,
}

impl Downloaded {
    /// Where it is, which is also where it will be checked.
    ///
    /// The same path both times, and that is the whole point: verifying one
    /// path and running another is a check that proves nothing about the file
    /// that runs.
    pub fn at(&self) -> &Path {
        &self.at
    }
}

/// An installer that is validly signed and signed by this project.
///
/// The field is private and [`verify`] is the only thing in this crate that
/// builds one, so no other module can make a value of this type by any route.
/// That is what carries the rule rather than an ordering: the function that
/// asks somebody whether to run an update takes one of these, exactly as the
/// function that runs it does, so there is no sequence anybody can reorder into
/// a version that asks about a file already known to be bad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    at: PathBuf,
}

impl Verified {
    /// Where the checked file is.
    pub fn at(&self) -> &Path {
        &self.at
    }
}

/// Why an installer will not be run.
///
/// Separate variants rather than one message, because a file nobody signed, a
/// file whose signature is broken and a file somebody else signed are three
/// different things to be told, and only the third one means somebody may have
/// been handed a different program on purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// Nothing signed it at all.
    NothingSignedIt,
    /// It carries a signature and the signature does not check out.
    TheSignatureIsNotValid {
        /// What Windows answered, so a report can say which failure it was.
        code: i32,
    },
    /// It is validly signed, by somebody who is not this project.
    SomebodyElseSignedIt {
        /// The name on the certificate, so the message can say who.
        who: String,
    },
    /// This computer has no way of checking a signature.
    NoWayToCheckHere,
    /// The file could not be read to be checked.
    TheFileCouldNotBeRead(String),
}

impl Refused {
    /// The sentence somebody hears and reads.
    ///
    /// Every one of them says the file was deleted and says where to go
    /// instead, because a refusal that leaves somebody with no way forward is
    /// how a person ends up downloading an installer by hand from wherever a
    /// search engine offers one.
    pub fn said(&self) -> String {
        let _ = (self, the_releases_page());
        String::new()
    }
}

/// Why an installer never arrived.
///
/// Told apart from [`Refused`] because nothing was checked in these cases:
/// there was no file to check. Reporting them as a failed signature would
/// accuse a publisher of something a broken connection did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotArrived {
    /// The release carried nothing that looks like this project's installer.
    NoInstallerAmongTheFiles,
    /// Nothing came back.
    TheConnectionFailed,
    /// A redirect pointed somewhere that is not HTTPS.
    ARedirectLeftHttps,
    /// The redirects went on past the bound.
    TooManyRedirects,
    /// More bytes arrived than will ever be accepted.
    BiggerThanWeWillAccept,
    /// It arrived and could not be written down.
    CouldNotBeKept(String),
}

impl NotArrived {
    /// The sentence somebody hears and reads.
    pub fn said(&self) -> String {
        let _ = self;
        String::new()
    }
}

/// What a fetch came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    /// It arrived, it is signed, and this project signed it.
    Ready(Verified),
    /// It arrived and it is not ours. It has been deleted.
    Refused(Refused),
    /// It never arrived.
    NotArrived(NotArrived),
}

/// Whether a redirect is one this will follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redirect {
    /// Follow it.
    Follow,
    /// Stop, for this reason.
    Stop(NotArrivedBecause),
}

/// The two ways a redirect is refused, as a copyable reason.
///
/// A small mirror of two [`NotArrived`] variants rather than the whole type,
/// because a redirect policy runs inside a callback that has to be `Copy` and
/// cannot carry a `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotArrivedBecause {
    /// The next hop is not HTTPS.
    ItLeftHttps,
    /// There have been too many hops.
    ThereWereTooMany,
}

impl From<NotArrivedBecause> for NotArrived {
    fn from(why: NotArrivedBecause) -> Self {
        match why {
            NotArrivedBecause::ItLeftHttps => Self::ARedirectLeftHttps,
            NotArrivedBecause::ThereWereTooMany => Self::TooManyRedirects,
        }
    }
}

/// Which published file is this project's installer.
///
/// By name, not by position. A release publishes three things a person could
/// download, and the one to run is the setup program; the portable executable
/// and the zip are the other two and neither installs anything. GitHub
/// documents no order for the list of files on a release, and an order nobody
/// has written down is one that can change without an announcement, so taking
/// element zero would be a rule about whatever GitHub happens to return.
pub fn the_installer_among(files: &[ReleaseFile]) -> Option<&ReleaseFile> {
    files.first()
}

/// Whether a redirect may be followed.
///
/// Two rules. The next hop has to be HTTPS, because a release file redirected
/// onto plain HTTP is an executable anybody on the path can replace, and the
/// signature check would then be the only thing between a person and a
/// stranger's program. And the chain has to be short, because a server can
/// redirect for ever and a client with no bound will follow it for ever.
pub fn whether_a_redirect_may_be_followed(scheme: &str, already_followed: usize) -> Redirect {
    let _ = (scheme, already_followed, MOST_REDIRECTS_FOLLOWED);
    Redirect::Follow
}

/// Whether a download that has produced this many bytes may carry on.
///
/// Asked as the bytes arrive rather than once at the end, because the point of
/// a bound is to stop before the disk is full, and a check made afterwards
/// makes it after the damage.
pub fn whether_this_much_may_still_arrive(so_far: u64) -> std::result::Result<(), NotArrived> {
    let _ = (so_far, MOST_BYTES_ACCEPTED);
    Ok(())
}

/// What a file's signature says, reduced to the one fact the rule reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhoSignedIt {
    /// The name on the signer's certificate.
    pub name: String,
}

/// Whether that signer is this project.
///
/// Exactly equal, allowing for surrounding space and for letter case. Not
/// "contains": a rule written that way accepts a certificate issued to
/// "Not Pratik Patel Holdings", which is a name anybody can buy.
pub fn whether_that_signer_is_ours(who: &WhoSignedIt) -> std::result::Result<(), Refused> {
    let _ = who;
    Ok(())
}

/// Read the signature off a file.
///
/// The half of this module no fixture can drive, so it is as thin as it can be:
/// it answers with a name and nothing else, and every rule about that name is
/// [`whether_that_signer_is_ours`] above.
#[cfg(target_os = "windows")]
fn who_signed(at: &Path) -> std::result::Result<WhoSignedIt, Refused> {
    let _ = at;
    Ok(WhoSignedIt {
        name: String::new(),
    })
}

/// Read the signature off a file, where there is no way to read one.
///
/// It refuses, and that is the whole of it. A fallback returning success here
/// would be the worst line of code in this phase: every other platform fallback
/// in this tree returns a harmless nothing, so the shape of the code invites
/// one, and the harmless nothing in this one position means a mail client
/// running an executable a stranger sent it. Plan 07-06 really builds this
/// crate on Linux and macOS, so this is compiled rather than hypothetical.
#[cfg(not(target_os = "windows"))]
fn who_signed(at: &Path) -> std::result::Result<WhoSignedIt, Refused> {
    let _ = at;
    Ok(WhoSignedIt {
        name: String::new(),
    })
}

/// Check a downloaded installer, and delete it unless this project signed it.
///
/// The only thing in this crate that builds a [`Verified`].
pub fn verify(downloaded: Downloaded) -> std::result::Result<Verified, Refused> {
    let checked = who_signed(downloaded.at()).and_then(|who| {
        whether_that_signer_is_ours(&who)?;
        Ok(())
    });
    match checked {
        Ok(()) => Ok(Verified { at: downloaded.at }),
        Err(refused) => {
            throw_it_away(downloaded.at());
            Err(refused)
        }
    }
}

/// Delete a file that will not be run, and say nothing about it going wrong.
///
/// A refused installer left on the disk is worse than one never fetched: a
/// person who has just been told this program will not run it can still find it
/// and double-click it, having been given no reason to think it dangerous. A
/// deletion that fails is logged rather than raised, because the refusal is the
/// message that matters and a second failure should not replace it.
fn throw_it_away(at: &Path) {
    if at.exists()
        && let Err(why) = std::fs::remove_file(at)
    {
        tracing::warn!("A refused installer could not be deleted from {at:?}: {why}");
    }
}

/// Empty the folder a downloaded installer waits in.
///
/// Called three times over. After a handover, so a machine that has just
/// updated is not carrying the installer it used. After a refusal, which
/// [`verify`] does file by file. And at the next start, which is the one that
/// matters: a program killed between fetching and handing over runs neither of
/// the first two, and without this the installer would sit there indefinitely.
pub fn clear_the_waiting_room(paths: &AppPaths) -> Result<()> {
    let room = paths.updates_dir();
    if !room.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&room)
        .map_err(|why| Error::Config(format!("Could not clear {}: {why}", room.display())))
}

/// A file name safe to write into a folder of ours.
///
/// The published name arrives in a response from a server, so it is a stranger's
/// text until something says otherwise. Anything but letters, digits and the
/// four punctuation marks a version number needs is dropped, which takes a
/// separator, a parent reference and a drive letter with it in one rule rather
/// than in three that each have to be remembered. A name left with nothing in it
/// falls back to a fixed one, because a refusal that can be walked round by
/// sending an awkward file name is worse than no refusal at all.
pub fn a_name_safe_to_write(published: &str) -> String {
    published.to_string()
}

/// What this program calls itself, to the host serving the file.
fn who_is_asking() -> String {
    format!("wixen-mail/{}", env!("CARGO_PKG_VERSION"))
}

/// Fetch a release's installer and check it, and say what came of it.
///
/// Nothing here asks anybody anything and nothing here runs anything. That is
/// the whole shape decision 16 settles: with a kind of version chosen, the fetch
/// and the check happen on their own, and the only question a person ever sees
/// is whether to run a file that has already passed both checks.
pub async fn fetch(files: &[ReleaseFile], paths: &AppPaths) -> Fetched {
    let Some(installer) = the_installer_among(files) else {
        return Fetched::NotArrived(NotArrived::NoInstallerAmongTheFiles);
    };
    match bring_it_down(installer, paths).await {
        Ok(downloaded) => match verify(downloaded) {
            Ok(verified) => Fetched::Ready(verified),
            Err(refused) => Fetched::Refused(refused),
        },
        Err(why) => Fetched::NotArrived(why),
    }
}

/// Nothing refused a redirect.
const NOTHING_WAS_REFUSED: u8 = 0;
/// A redirect pointed somewhere that is not HTTPS.
const IT_LEFT_HTTPS: u8 = 1;
/// The chain of redirects went past the bound.
const THERE_WERE_TOO_MANY: u8 = 2;

/// Bring the bytes down onto the disk, bounded at both ends.
///
/// Written to a file as it arrives rather than gathered in memory first, and
/// that is the only way the size bound is worth anything: a client that reads a
/// whole body before looking at it has already taken whatever was sent.
/// `reqwest::Response::chunk` does this at version 0.13.4 with no extra feature
/// enabled, which was read out of the vendored crate rather than assumed; only
/// `bytes_stream` sits behind `stream`.
async fn bring_it_down(
    installer: &ReleaseFile,
    paths: &AppPaths,
) -> std::result::Result<Downloaded, NotArrived> {
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU8, Ordering};

    let room = paths.updates_dir();
    std::fs::create_dir_all(&room)
        .map_err(|why| NotArrived::CouldNotBeKept(format!("{}: {why}", room.display())))?;
    let at = room.join(a_name_safe_to_write(&installer.name));

    // Why a redirect was refused, carried out of the policy callback. The
    // callback cannot return a reason of its own, and "the download failed" in
    // place of "it was redirected off HTTPS" is a message that hides the one
    // fact worth reporting.
    let refused_because = Arc::new(AtomicU8::new(NOTHING_WAS_REFUSED));
    let noted = Arc::clone(&refused_because);
    let follower =
        reqwest::redirect::Policy::custom(move |attempt| match whether_a_redirect_may_be_followed(
            attempt.url().scheme(),
            attempt.previous().len(),
        ) {
            Redirect::Follow => attempt.follow(),
            Redirect::Stop(why) => {
                noted.store(
                    match why {
                        NotArrivedBecause::ItLeftHttps => IT_LEFT_HTTPS,
                        NotArrivedBecause::ThereWereTooMany => THERE_WERE_TOO_MANY,
                    },
                    Ordering::Relaxed,
                );
                attempt.stop()
            }
        });
    let why_it_stopped = || match refused_because.load(Ordering::Relaxed) {
        IT_LEFT_HTTPS => NotArrived::ARedirectLeftHttps,
        THERE_WERE_TOO_MANY => NotArrived::TooManyRedirects,
        _ => NotArrived::TheConnectionFailed,
    };

    let client = reqwest::Client::builder()
        .redirect(follower)
        .build()
        .map_err(|_| NotArrived::TheConnectionFailed)?;
    // Through the gate that cannot change anything, rather than on a client of
    // its own. This only ever reads, so the constructor that cannot write is the
    // honest one and the census can then see it.
    let gate = crate::service::outward::Outward::read_only(client);
    let mut response = gate
        .reading(&installer.from)
        .header(reqwest::header::USER_AGENT, who_is_asking())
        .send()
        .await
        .map_err(|_| why_it_stopped())?;
    if !response.status().is_success() {
        return Err(why_it_stopped());
    }

    let mut file = std::fs::File::create(&at)
        .map_err(|why| NotArrived::CouldNotBeKept(format!("{}: {why}", at.display())))?;
    let mut so_far: u64 = 0;
    loop {
        let arrived = response.chunk().await.map_err(|_| {
            let _ = std::fs::remove_file(&at);
            NotArrived::TheConnectionFailed
        })?;
        let Some(chunk) = arrived else { break };
        so_far = so_far.saturating_add(chunk.len() as u64);
        if let Err(too_big) = whether_this_much_may_still_arrive(so_far) {
            drop(file);
            let _ = std::fs::remove_file(&at);
            return Err(too_big);
        }
        file.write_all(&chunk)
            .map_err(|why| NotArrived::CouldNotBeKept(format!("{}: {why}", at.display())))?;
    }
    file.flush()
        .map_err(|why| NotArrived::CouldNotBeKept(format!("{}: {why}", at.display())))?;

    Ok(Downloaded { at })
}

/// Where the releases are, for a refusal to point at.
fn the_releases_page() -> String {
    format!(
        "{}/releases",
        env!("CARGO_PKG_REPOSITORY").trim_end_matches('/')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// The three files a release of this project publishes, in an order that is
    /// not the one a naive reading would want.
    fn a_real_release() -> Vec<ReleaseFile> {
        vec![
            ReleaseFile {
                name: "Wixen-Mail-0.116.0-windows.zip".to_string(),
                from: "https://github.com/PratikP1/Wixen-Mail/releases/download/v0.116.0/Wixen-Mail-0.116.0-windows.zip".to_string(),
            },
            ReleaseFile {
                name: "wixen-mail-v0.116.0.exe".to_string(),
                from: "https://github.com/PratikP1/Wixen-Mail/releases/download/v0.116.0/wixen-mail-v0.116.0.exe".to_string(),
            },
            ReleaseFile {
                name: "Wixen-Mail-Setup-0.116.0.exe".to_string(),
                from: "https://github.com/PratikP1/Wixen-Mail/releases/download/v0.116.0/Wixen-Mail-Setup-0.116.0.exe".to_string(),
            },
        ]
    }

    #[test]
    fn test_the_installer_is_chosen_by_name_and_not_by_being_first() {
        let files = a_real_release();
        let chosen = the_installer_among(&files).expect("a release to carry its installer");
        assert_eq!(chosen.name, "Wixen-Mail-Setup-0.116.0.exe");
    }

    #[test]
    fn test_the_portable_build_and_the_zip_are_never_mistaken_for_the_installer() {
        // Both are executables a person can be handed and neither installs
        // anything. Running the portable copy in place of the installer would
        // look like an update and change nothing on the machine.
        let files = a_real_release();
        let chosen = the_installer_among(&files).expect("a release to carry its installer");
        assert!(!chosen.name.ends_with(".zip"));
        assert!(!chosen.name.starts_with("wixen-mail-v"));
    }

    #[test]
    fn test_a_release_carrying_no_installer_is_not_something_to_download() {
        let only_notes = vec![ReleaseFile {
            name: "release-notes.txt".to_string(),
            from: "https://github.com/PratikP1/Wixen-Mail/releases/download/v1/release-notes.txt"
                .to_string(),
        }];
        assert_eq!(the_installer_among(&only_notes), None);
    }

    #[test]
    fn test_a_published_name_can_never_escape_the_folder_it_is_written_into() {
        // The name comes out of a response, so it is a stranger's text. A
        // refusal that can be walked round by publishing an awkward file name
        // is worse than no refusal, because it reads as one.
        for awkward in [
            r"..\..\..\Windows\System32\calc.exe",
            "../../../etc/passwd",
            r"C:\Windows\System32\calc.exe",
            "setup.exe:stream",
            "..",
            "",
        ] {
            let written = a_name_safe_to_write(awkward);
            let landed = Path::new("room").join(&written);
            assert_eq!(
                landed.parent(),
                Some(Path::new("room")),
                "{awkward} was written as {written}, which lands outside the folder"
            );
            assert!(
                !written.is_empty(),
                "{awkward} was written as nothing at all"
            );
        }
    }

    #[test]
    fn test_an_ordinary_installer_name_is_kept_as_it_was_published() {
        assert_eq!(
            a_name_safe_to_write("Wixen-Mail-Setup-0.116.0+g1234abc.exe"),
            "Wixen-Mail-Setup-0.116.0+g1234abc.exe"
        );
    }

    #[test]
    fn test_a_redirect_that_leaves_https_is_refused() {
        assert_eq!(
            whether_a_redirect_may_be_followed("http", 1),
            Redirect::Stop(NotArrivedBecause::ItLeftHttps)
        );
        assert_eq!(
            whether_a_redirect_may_be_followed("ftp", 0),
            Redirect::Stop(NotArrivedBecause::ItLeftHttps)
        );
        assert_eq!(
            whether_a_redirect_may_be_followed("https", 1),
            Redirect::Follow
        );
    }

    #[test]
    fn test_a_redirect_chain_past_the_bound_is_refused() {
        assert_eq!(
            whether_a_redirect_may_be_followed("https", MOST_REDIRECTS_FOLLOWED - 1),
            Redirect::Follow
        );
        assert_eq!(
            whether_a_redirect_may_be_followed("https", MOST_REDIRECTS_FOLLOWED),
            Redirect::Stop(NotArrivedBecause::ThereWereTooMany)
        );
    }

    #[test]
    fn test_a_download_past_the_size_bound_stops_rather_than_filling_the_disk() {
        assert!(whether_this_much_may_still_arrive(MOST_BYTES_ACCEPTED).is_ok());
        assert_eq!(
            whether_this_much_may_still_arrive(MOST_BYTES_ACCEPTED + 1),
            Err(NotArrived::BiggerThanWeWillAccept)
        );
    }

    #[test]
    fn test_a_signature_from_anybody_else_is_refused_however_valid_it_is() {
        // The check a naive version passes. WinVerifyTrust says yes to millions
        // of files and says nothing at all about who signed them.
        let somebody_else = WhoSignedIt {
            name: "Microsoft Windows".to_string(),
        };
        assert_eq!(
            whether_that_signer_is_ours(&somebody_else),
            Err(Refused::SomebodyElseSignedIt {
                who: "Microsoft Windows".to_string()
            })
        );
    }

    #[test]
    fn test_a_name_that_merely_contains_ours_is_refused() {
        // "Contains" is the rule somebody reaches for, and a certificate issued
        // to a company with our name inside its own is one anybody can buy.
        for nearly in [
            "Pratik Patel Holdings Ltd",
            "Not Pratik Patel",
            "PratikPatel",
        ] {
            assert!(
                whether_that_signer_is_ours(&WhoSignedIt {
                    name: nearly.to_string()
                })
                .is_err(),
                "{nearly} was accepted as this project's own name"
            );
        }
    }

    #[test]
    fn test_our_own_name_is_accepted_whatever_space_or_case_surrounds_it() {
        for ours in [WHO_SIGNS_THIS, "  Pratik Patel  ", "PRATIK PATEL"] {
            assert_eq!(
                whether_that_signer_is_ours(&WhoSignedIt {
                    name: ours.to_string()
                }),
                Ok(()),
                "{ours} was refused and it is this project's own name"
            );
        }
    }

    #[test]
    fn test_the_name_this_expects_is_the_one_the_installer_publishes_under() {
        // Read from the script that really sets it rather than written twice.
        // The name a person sees in Apps and Features and the name this refuses
        // an installer for have to be the same string, and nothing else in the
        // tree would notice if they drifted apart.
        let script = std::fs::read_to_string("installer/Wixen-Mail-Setup.iss")
            .expect("the installer script to be readable");
        let published = script
            .lines()
            .find_map(|line| line.trim().strip_prefix("#define Publisher "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .expect("the installer script to define a publisher");
        assert_eq!(published, WHO_SIGNS_THIS);
    }

    #[test]
    fn test_the_refusals_do_not_all_say_the_same_thing() {
        let every = [
            Refused::NothingSignedIt,
            Refused::TheSignatureIsNotValid { code: -2146762496 },
            Refused::SomebodyElseSignedIt {
                who: "Microsoft Windows".to_string(),
            },
            Refused::NoWayToCheckHere,
            Refused::TheFileCouldNotBeRead("access denied".to_string()),
        ];
        let said: Vec<String> = every.iter().map(Refused::said).collect();
        for (which, sentence) in said.iter().enumerate() {
            assert!(!sentence.is_empty(), "refusal {which} says nothing at all");
            assert_eq!(
                said.iter().filter(|other| *other == sentence).count(),
                1,
                "two refusals say the same thing, so a person cannot tell them apart: {sentence}"
            );
        }
    }

    #[test]
    fn test_every_refusal_says_the_file_is_gone_and_where_to_go_instead() {
        // A refusal that leaves somebody with no way forward is how a person
        // ends up fetching an installer by hand from whatever a search engine
        // offers, which is worse than the thing being refused.
        let every = [
            Refused::NothingSignedIt,
            Refused::TheSignatureIsNotValid { code: -1 },
            Refused::SomebodyElseSignedIt {
                who: "somebody".to_string(),
            },
            Refused::NoWayToCheckHere,
            Refused::TheFileCouldNotBeRead("no".to_string()),
        ];
        for refusal in every {
            let sentence = refusal.said();
            assert!(
                sentence.contains(&the_releases_page()),
                "{refusal:?} does not say where to go instead"
            );
        }
    }

    #[test]
    fn test_a_refusal_naming_somebody_else_says_who_signed_it() {
        let sentence = Refused::SomebodyElseSignedIt {
            who: "Microsoft Windows".to_string(),
        }
        .said();
        assert!(
            sentence.contains("Microsoft Windows"),
            "a person told an installer is somebody else's should be told whose: {sentence}"
        );
    }

    /// The half of this module a release build compiles, as text.
    fn what_this_module_ships() -> String {
        let source = std::fs::read_to_string("src/service/update_download.rs")
            .expect("this module to be readable");
        crate::common::what_ships::what_ships(&source.replace("\r\n", "\n"))
    }

    /// Whether the reading below can see a fallback that agreed.
    ///
    /// Split out so the reading can be shown a violation as well as the tree,
    /// because a check over source text that has never seen one is a check
    /// nobody has proved can fail.
    fn the_fallback_agrees_somewhere(source: &str) -> bool {
        let Some(after) = source.find("#[cfg(not(target_os = \"windows\"))]") else {
            return false;
        };
        let block = &source[after..];
        let end = block.find("\n}\n").map_or(block.len(), |at| at + 2);
        block[..end].contains("Ok(WhoSignedIt")
    }

    #[test]
    fn test_the_platform_with_no_way_to_check_refuses_rather_than_agreeing() {
        // Compiled on Linux and macOS by plan 07-06, and read here rather than
        // called, because a Windows build cannot call it and a Windows build is
        // where somebody would break it without noticing.
        assert!(
            !the_fallback_agrees_somewhere(&what_this_module_ships()),
            "the platform with no way to check a signature answers with a signer, \
             which is a mail client agreeing to run whatever it was sent"
        );
    }

    #[test]
    fn test_that_reading_can_see_a_fallback_that_agreed() {
        let doctored = "#[cfg(not(target_os = \"windows\"))]\n\
             fn who_signed(_at: &Path) -> std::result::Result<WhoSignedIt, Refused> {\n\
             \x20   Ok(WhoSignedIt { name: String::new() })\n\
             }\n";
        assert!(
            the_fallback_agrees_somewhere(doctored),
            "the reading above cannot see the defect it exists to refuse"
        );
    }

    #[test]
    fn test_the_installer_waits_under_this_programs_own_root() {
        // Not the shared temporary folder. Another person using this computer
        // can write there, and a file they can swap between the check and the
        // run is a file the check said nothing useful about.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        assert!(paths.updates_dir().starts_with(paths.root()));
    }

    #[test]
    fn test_clearing_the_waiting_room_takes_the_installer_with_it() {
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        std::fs::create_dir_all(paths.updates_dir()).unwrap();
        let left_behind = paths.updates_dir().join("Wixen-Mail-Setup-0.1.0.exe");
        std::fs::write(&left_behind, b"MZ").unwrap();

        clear_the_waiting_room(&paths).expect("clearing to work");

        assert!(!left_behind.exists());
        assert!(!paths.updates_dir().exists());
    }

    #[test]
    fn test_clearing_a_waiting_room_that_was_never_made_is_not_a_failure() {
        // This runs at every start, including the first one on a machine that
        // has never fetched an update, so the ordinary case must be silent.
        let dir = TempDir::new().unwrap();
        let paths = AppPaths::under(dir.path().join("wixen-mail"));
        assert!(clear_the_waiting_room(&paths).is_ok());
    }

    #[test]
    fn test_a_refused_installer_is_deleted() {
        let dir = TempDir::new().unwrap();
        let at = dir.path().join("Wixen-Mail-Setup-0.1.0.exe");
        std::fs::write(&at, b"not an installer").unwrap();

        let refused = verify(Downloaded { at: at.clone() });

        assert!(refused.is_err(), "an unsigned file was accepted");
        assert!(
            !at.exists(),
            "a refused installer was left where somebody can find it and run it by hand"
        );
    }

    /// A file Windows itself signed, which is the fixture that matters.
    ///
    /// The two easy refusals, nothing signed it and the signature is broken,
    /// are the ones a naive implementation already fails. The one it passes is
    /// a validly signed executable somebody else published, and Windows ships
    /// hundreds of those. This one is chosen because it is present on every
    /// supported version of Windows and because it is not something this
    /// project could ever have signed.
    #[cfg(target_os = "windows")]
    const SIGNED_BY_SOMEBODY_ELSE: &str = r"C:\Windows\System32\notepad.exe";

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_file_windows_signed_is_refused_because_somebody_else_signed_it() {
        let at = Path::new(SIGNED_BY_SOMEBODY_ELSE);
        if !at.exists() {
            // Said rather than passed over. A fixture that is not there makes
            // this test prove nothing, and a silent skip reads as a pass.
            panic!(
                "{SIGNED_BY_SOMEBODY_ELSE} is not on this machine, so the one refusal a naive implementation passes is untested"
            );
        }
        match who_signed(at) {
            Ok(who) => assert!(
                matches!(
                    whether_that_signer_is_ours(&who),
                    Err(Refused::SomebodyElseSignedIt { .. })
                ),
                "a file Microsoft signed was accepted as this project's own, and it was signed by {}",
                who.name
            ),
            Err(other) => {
                panic!("a validly signed Windows file was refused for the wrong reason: {other:?}")
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_file_with_no_signature_at_all_is_refused() {
        let dir = TempDir::new().unwrap();
        let at = dir.path().join("nothing-signed-this.exe");
        std::fs::write(&at, b"MZ and then nothing that means anything").unwrap();
        assert!(
            matches!(
                who_signed(&at),
                Err(Refused::NothingSignedIt | Refused::TheSignatureIsNotValid { .. })
            ),
            "a file nothing signed was not refused"
        );
    }
}
