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

/// What this project's installer is called, before its version number.
///
/// `installer/Wixen-Mail-Setup.iss` builds it as
/// `OutputBaseFilename=Wixen-Mail-Setup-{#AppVersion}` and the release workflow
/// publishes it under that name, so this is the one string that decides which
/// of a release's files is the one to run. Held to the script by
/// `test_the_name_this_looks_for_is_the_name_the_installer_is_built_under`,
/// because a release publishes two executables and the other one installs
/// nothing.
const WHAT_THE_INSTALLER_IS_CALLED: &str = "Wixen-Mail-Setup-";

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
    version: String,
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
    version: String,
}

impl Verified {
    /// Where the checked file is.
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// Which version it installs, as it was published.
    ///
    /// Carried on this value rather than passed beside it, so that everything
    /// a person is shown about an update takes one argument and that argument
    /// is the checked file. A version handed over separately is a second thing
    /// that can come from somewhere else.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Why an update did not get as far as installing itself.
///
/// Told apart from everything above because the file was fine: it arrived, it
/// was checked, it is this project's, and the handover is what did not work.
/// Each of these leaves the running version exactly as it was, which is what
/// every sentence below says, because somebody who has just watched an update
/// fail has every reason to wonder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoverFailed {
    /// The file was not where it had been checked.
    ItIsNoLongerThere,
    /// Windows would not start it.
    ItWouldNotStart(String),
}

impl HandoverFailed {
    /// The sentence somebody hears and reads.
    pub fn said(&self) -> String {
        let page = the_releases_page();
        match self {
            Self::ItIsNoLongerThere => format!(
                "The update Wixen Mail downloaded is no longer where it put it, so it was \
                 not started. Wixen Mail is still running and nothing on this computer has \
                 changed. You can download the new version yourself from {page}."
            ),
            Self::ItWouldNotStart(why) => format!(
                "Wixen Mail could not start the update it downloaded. The reason given was: \
                 {why}. Wixen Mail is still running and nothing on this computer has \
                 changed. You can download the new version yourself from {page}."
            ),
        }
    }
}

/// The one question this feature asks, in the words a person reads.
///
/// **It takes the checked file and nothing else, and that is the rule rather
/// than an ordering.** Nobody is ever asked to approve a file already known to
/// be bad, and a question implies the answer could reasonably be yes. Offering
/// somebody "this could not be verified, run it anyway?" moves a decision the
/// program is equipped to make on to a person who is not, at the exact moment
/// they are most likely to agree, because they asked for an update and this is
/// the thing standing between them and having one. Carried in the type, there
/// is no sequence to reorder, no later edit that can move the question earlier,
/// and nothing a review has to notice.
///
/// It also says the program is about to close. That belongs in the question and
/// not only in an announcement afterwards, because this is the last moment the
/// person can still say no.
pub fn the_question_about(installer: &Verified) -> String {
    format!(
        "Wixen Mail {} is ready to install. It was checked and it is signed by \
         {WHO_SIGNS_THIS}.\n\n\
         If you install it now, Wixen Mail will close and the installer will open in its \
         place. Your mail, accounts and settings are not touched.\n\n\
         Install it now?",
        installer.version()
    )
}

/// What is said in the moment before the window goes.
///
/// A screen reader user loses their bearings when a window disappears, and
/// being told a second beforehand is the difference between a handover and a
/// crash. This is said once, at high priority, straight after somebody answers
/// yes. It is not a second question: they have already agreed, and asking twice
/// about one thing teaches somebody to answer without reading.
pub fn what_is_said_before_the_window_closes(installer: &Verified) -> String {
    format!(
        "Installing Wixen Mail {}. Wixen Mail is closing now and the installer is opening.",
        installer.version()
    )
}

/// Somebody said no. Throw the installer away.
///
/// The file goes rather than waiting in case they change their mind. A declined
/// installer sitting on the disk is a file somebody can find and run by hand
/// having decided not to, and the next check will fetch it again in seconds if
/// they change their mind.
pub fn say_no(installer: Verified) -> Result<()> {
    if !installer.at.exists() {
        return Ok(());
    }
    std::fs::remove_file(&installer.at).map_err(|why| {
        Error::Config(format!(
            "Could not remove {}: {why}",
            installer.at.display()
        ))
    })
}

/// Start the installer and leave it to replace this program.
///
/// Takes the checked file, exactly as the question does.
///
/// **The installer is started first and this program closes afterwards.** The
/// other order, closing and then starting, was rejected: the process doing the
/// starting is the one closing, so the start has to outlive its parent, and if
/// it fails there is nothing left running to say so. Somebody who answered yes
/// would be left with no Wixen Mail and no installer and no sentence. This way
/// round, a failure to start is a message, and the program they were using is
/// still there.
///
/// What that trades away is a race this cannot remove. The installer refuses to
/// run while a copy of Wixen Mail holds the named mutex, which is what stops
/// the uninstaller deleting the data folder underneath a running program. So an
/// installer that reaches its own check before this process has finished
/// closing will say Wixen Mail is still open. That is the installer's own
/// dialog rather than a silence, which is the property that matters, and the
/// person's answer is to press retry. Nothing here waits on a bound, because
/// the wait would have to happen inside a process that is trying to exit.
pub fn hand_over_to(installer: &Verified) -> std::result::Result<(), HandoverFailed> {
    if !installer.at.exists() {
        return Err(HandoverFailed::ItIsNoLongerThere);
    }
    std::process::Command::new(&installer.at)
        .spawn()
        .map(|child| {
            tracing::info!(
                "The installer for {} was started as process {}",
                installer.version(),
                child.id()
            );
        })
        .map_err(|why| HandoverFailed::ItWouldNotStart(why.to_string()))
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
        let page = the_releases_page();
        match self {
            Self::NothingSignedIt => format!(
                "The update Wixen Mail downloaded is not signed by anyone, so Wixen Mail \
                 will not run it. The file has been deleted. Nothing on this computer has \
                 changed. You can download a version yourself from {page}."
            ),
            Self::TheSignatureIsNotValid { code } => format!(
                "The update Wixen Mail downloaded carries a signature that does not check \
                 out, so Wixen Mail will not run it. Windows reported {code}. The file has \
                 been deleted. Nothing on this computer has changed. You can download a \
                 version yourself from {page}."
            ),
            Self::SomebodyElseSignedIt { who } => format!(
                "The update Wixen Mail downloaded is signed by {who}, not by \
                 {WHO_SIGNS_THIS}, so Wixen Mail will not run it. A file can be correctly \
                 signed and still be someone else's program, which is why this is checked. \
                 The file has been deleted. Nothing on this computer has changed. You can \
                 download a version yourself from {page}."
            ),
            // The one refusal that does not mention a deleted file, because on
            // this path there is no file: a computer with no way to look at a
            // signature is told so before anything is fetched.
            Self::NoWayToCheckHere => format!(
                "Wixen Mail has no way to check who signed an update on this system, so it \
                 will not download or run one. Nothing has been downloaded and nothing on \
                 this computer has changed. You can download a version yourself from {page}."
            ),
            Self::TheFileCouldNotBeRead(why) => format!(
                "Wixen Mail could not read the update it downloaded to find out who signed \
                 it, so it will not run it. The reason given was: {why}. The file has been \
                 deleted. Nothing on this computer has changed. You can download a version \
                 yourself from {page}."
            ),
        }
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
    ///
    /// Six of them, because they lead somebody to do different things: wait,
    /// look at their connection, or go and fetch the new version by hand. All
    /// six say the current version is untouched, since a failed download is the
    /// one case where nothing at all has happened and somebody hearing about a
    /// failure assumes otherwise.
    pub fn said(&self) -> String {
        let page = the_releases_page();
        match self {
            Self::NoInstallerAmongTheFiles => format!(
                "There is a newer version of Wixen Mail, and it does not publish an \
                 installer Wixen Mail recognises, so nothing was downloaded. You are still \
                 running the version you were. You can look at what was published on {page}."
            ),
            Self::TheConnectionFailed => format!(
                "Wixen Mail could not download the newer version. You are still running the \
                 version you were, and nothing has changed. You can download it yourself \
                 from {page}."
            ),
            Self::ARedirectLeftHttps => format!(
                "The download of the newer version was sent to an address that is not \
                 secure, so Wixen Mail stopped it. You are still running the version you \
                 were, and nothing has changed. You can download it yourself from {page}."
            ),
            Self::TooManyRedirects => format!(
                "The download of the newer version was passed from one address to another \
                 too many times, so Wixen Mail stopped it. You are still running the \
                 version you were, and nothing has changed. You can download it yourself \
                 from {page}."
            ),
            Self::BiggerThanWeWillAccept => format!(
                "The download of the newer version was larger than Wixen Mail will accept, \
                 so it was stopped and thrown away. You are still running the version you \
                 were, and nothing has changed. You can download it yourself from {page}."
            ),
            Self::CouldNotBeKept(why) => format!(
                "Wixen Mail could not keep the newer version on this computer. The reason \
                 given was: {why}. You are still running the version you were, and nothing \
                 has changed. You can download it yourself from {page}."
            ),
        }
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

/// What a check's answer leads to.
///
/// A rule of its own rather than a branch buried in the screen, so that "an
/// offer is fetched without anybody being asked" is a sentence a test can hold
/// this to. Decision 16 is the whole of it: with a kind of version chosen, the
/// program checks, downloads and verifies on its own, and the only question is
/// whether to run what it ended up with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextStep {
    /// Fetch the installer from these files. Nobody is asked first.
    FetchIt(Vec<ReleaseFile>),
    /// Say what the check found and stop there.
    JustSayIt,
}

/// What to do about a check's answer.
///
/// An offer leads straight to a fetch. Every other answer leads to a sentence
/// and nothing else, because there is nothing to fetch.
pub fn what_to_do_about(answer: &crate::service::update_check::Answer) -> NextStep {
    match answer {
        crate::service::update_check::Answer::ANewerVersion { files, .. } => {
            NextStep::FetchIt(files.clone())
        }
        _ => NextStep::JustSayIt,
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
    files.iter().find(|file| {
        let name = file.name.to_ascii_lowercase();
        name.starts_with(&WHAT_THE_INSTALLER_IS_CALLED.to_ascii_lowercase())
            && name.ends_with(".exe")
    })
}

/// Whether a redirect may be followed.
///
/// Two rules. The next hop has to be HTTPS, because a release file redirected
/// onto plain HTTP is an executable anybody on the path can replace, and the
/// signature check would then be the only thing between a person and a
/// stranger's program. And the chain has to be short, because a server can
/// redirect for ever and a client with no bound will follow it for ever.
pub fn whether_a_redirect_may_be_followed(scheme: &str, already_followed: usize) -> Redirect {
    if !scheme.eq_ignore_ascii_case("https") {
        return Redirect::Stop(NotArrivedBecause::ItLeftHttps);
    }
    if already_followed >= MOST_REDIRECTS_FOLLOWED {
        return Redirect::Stop(NotArrivedBecause::ThereWereTooMany);
    }
    Redirect::Follow
}

/// Whether a download that has produced this many bytes may carry on.
///
/// Asked as the bytes arrive rather than once at the end, because the point of
/// a bound is to stop before the disk is full, and a check made afterwards
/// makes it after the damage.
pub fn whether_this_much_may_still_arrive(so_far: u64) -> std::result::Result<(), NotArrived> {
    if so_far > MOST_BYTES_ACCEPTED {
        return Err(NotArrived::BiggerThanWeWillAccept);
    }
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
    let name = who.name.trim();
    if name.is_empty() {
        return Err(Refused::NothingSignedIt);
    }
    if name.eq_ignore_ascii_case(WHO_SIGNS_THIS) {
        return Ok(());
    }
    Err(Refused::SomebodyElseSignedIt {
        who: who.name.clone(),
    })
}

/// Whether this computer can check a signature at all.
///
/// Asked before anything is downloaded rather than after, and the difference is
/// the whole of criterion 2's clause about it: where a signature cannot be
/// checked, **nothing is downloaded** and nothing is run, and the reason is
/// said. Fetching an executable and then discovering there is no way to look at
/// it leaves it on somebody's disk for the time it takes to refuse it, and
/// spends their connection to learn something this program already knew.
///
/// Its two arms sit beside [`who_signed`]'s two arms on purpose. They answer the
/// same question and a platform gaining one without the other is the drift this
/// arrangement makes visible.
#[cfg(target_os = "windows")]
pub fn whether_a_signature_can_be_checked_here() -> std::result::Result<(), Refused> {
    Ok(())
}

/// Whether this computer can check a signature at all, where it cannot.
#[cfg(not(target_os = "windows"))]
pub fn whether_a_signature_can_be_checked_here() -> std::result::Result<(), Refused> {
    Err(Refused::NoWayToCheckHere)
}

/// Read the signature off a file.
///
/// The half of this module no fixture can drive, so it is as thin as it can be:
/// it answers with a name and nothing else, and every rule about that name is
/// [`whether_that_signer_is_ours`] above.
#[cfg(target_os = "windows")]
fn who_signed(at: &Path) -> std::result::Result<WhoSignedIt, Refused> {
    whether_windows_trusts_it(at)?;
    the_name_on_the_signers_certificate(at)
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
    Err(Refused::NoWayToCheckHere)
}

/// A path as Windows wants it: wide characters, ending in a nought.
#[cfg(target_os = "windows")]
fn as_wide(at: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    at.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Whether Windows itself trusts the signature on a file.
///
/// The first half of the check and the smaller half. This answers "is this file
/// validly signed by somebody a trusted root vouches for", which millions of
/// files are. On its own it is worth nothing here, and a module that stopped at
/// it would report success over a stranger's program. The half that matters is
/// [`the_name_on_the_signers_certificate`] below.
///
/// **Revocation is not checked, deliberately.** Checking the whole chain reaches
/// the network, and this runs on a computer that has just failed to download
/// something, or on one behind a connection that allows GitHub and nothing else.
/// A revocation server that cannot be reached is not a bad signature, and
/// treating it as one would refuse every genuine update on an aeroplane. What
/// this trades away is a certificate revoked after it was issued: Windows still
/// applies whatever it holds in its own local cache, and a stolen key that had
/// been revoked would pass here until that cache caught up. The narrower risk is
/// accepted rather than turned into a check that fails offline, and it is
/// written down here so the next person can weigh it rather than rediscover it.
#[cfg(target_os = "windows")]
fn whether_windows_trusts_it(at: &Path) -> std::result::Result<(), Refused> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Security::WinTrust::{
        WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO,
        WTD_CHOICE_FILE, WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY,
        WTD_UI_NONE, WinVerifyTrust,
    };

    /// `TRUST_E_NOSIGNATURE`, which Windows answers for a file nothing signed.
    ///
    /// Told apart from every other failure because they are different things to
    /// be told: an unsigned build is what this project ships today, and a broken
    /// signature is a file that was interfered with.
    const NOTHING_SIGNED_IT: i32 = -2_146_762_496;
    /// What it answers when the file is fine.
    const IT_IS_FINE: i32 = 0;

    let wide = as_wide(at);
    let mut file = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: windows::core::PCWSTR(wide.as_ptr()),
        ..Default::default()
    };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &raw mut file,
        },
        dwStateAction: WTD_STATEACTION_VERIFY,
        ..Default::default()
    };
    let mut which_check = WINTRUST_ACTION_GENERIC_VERIFY_V2;

    // SAFETY: both structures are filled in above, live for the whole of both
    // calls, and say their own size. The window handle is null because
    // WTD_UI_NONE means nothing is shown.
    let verdict = unsafe {
        WinVerifyTrust(
            HWND::default(),
            &raw mut which_check,
            (&raw mut data).cast(),
        )
    };

    // The provider keeps state for the verify above and hands it back only on a
    // second call. Leaving this out is the easiest mistake in this function and
    // it leaks a handle per download, which nothing would ever report.
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    // SAFETY: as above, with the same two structures still alive.
    unsafe {
        WinVerifyTrust(
            HWND::default(),
            &raw mut which_check,
            (&raw mut data).cast(),
        )
    };

    match verdict {
        IT_IS_FINE => Ok(()),
        NOTHING_SIGNED_IT => Err(Refused::NothingSignedIt),
        code => Err(Refused::TheSignatureIsNotValid { code }),
    }
}

/// The name on the certificate that signed a file.
///
/// **This is the check.** Without it any validly signed file passes, which means
/// any installer anybody bought a certificate for. The first half asks whether
/// the signature is real; only this one asks whose it is.
#[cfg(target_os = "windows")]
fn the_name_on_the_signers_certificate(at: &Path) -> std::result::Result<WhoSignedIt, Refused> {
    use windows::Win32::Security::Cryptography::{
        CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED, CERT_QUERY_FORMAT_FLAG_BINARY,
        CERT_QUERY_OBJECT_FILE, CertCloseStore, CryptMsgClose, CryptQueryObject, HCERTSTORE,
    };

    let wide = as_wide(at);
    let mut store = HCERTSTORE::default();
    let mut message: *mut core::ffi::c_void = std::ptr::null_mut();

    // SAFETY: the path lives for the call, and both handles are written only on
    // success, which is what the `?` below depends on.
    unsafe {
        CryptQueryObject(
            CERT_QUERY_OBJECT_FILE,
            wide.as_ptr().cast(),
            CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
            CERT_QUERY_FORMAT_FLAG_BINARY,
            0,
            None,
            None,
            None,
            Some(&raw mut store),
            Some(&raw mut message),
            None,
        )
    }
    .map_err(|why| Refused::TheFileCouldNotBeRead(why.message()))?;

    let found = the_name_in(store, message);

    // Closed however the reading went. An early return that skipped this would
    // leak a store and a message per refused download, and a refused download is
    // the case this module expects to be common.
    // SAFETY: both were opened by the call above and are closed exactly once.
    unsafe {
        let _ = CryptMsgClose(Some(message));
        let _ = CertCloseStore(Some(store), 0);
    }
    found
}

/// The signer's name, given an open certificate store and message.
///
/// Split from its caller so the two handles are closed on one path rather than
/// on each of the five ways this can give up.
#[cfg(target_os = "windows")]
fn the_name_in(
    store: windows::Win32::Security::Cryptography::HCERTSTORE,
    message: *mut core::ffi::c_void,
) -> std::result::Result<WhoSignedIt, Refused> {
    use windows::Win32::Security::Cryptography::{
        CERT_FIND_SUBJECT_CERT, CERT_INFO, CERT_NAME_SIMPLE_DISPLAY_TYPE, CERT_QUERY_ENCODING_TYPE,
        CMSG_SIGNER_INFO, CMSG_SIGNER_INFO_PARAM, CertFindCertificateInStore,
        CertFreeCertificateContext, CertGetNameStringW, CryptMsgGetParam, PKCS_7_ASN_ENCODING,
        X509_ASN_ENCODING,
    };

    /// Longer than any certificate subject anybody issues, and bounded because
    /// this is a buffer a stranger's file decides the contents of.
    const ROOM_FOR_A_NAME: usize = 1024;

    let mut how_big: u32 = 0;
    // SAFETY: asking for the size with no buffer is how this call is defined to
    // be used.
    unsafe { CryptMsgGetParam(message, CMSG_SIGNER_INFO_PARAM, 0, None, &raw mut how_big) }
        .map_err(|why| Refused::TheFileCouldNotBeRead(why.message()))?;

    let mut held = vec![0_u8; how_big as usize];
    // SAFETY: the buffer is exactly the size the call above asked for.
    unsafe {
        CryptMsgGetParam(
            message,
            CMSG_SIGNER_INFO_PARAM,
            0,
            Some(held.as_mut_ptr().cast()),
            &raw mut how_big,
        )
    }
    .map_err(|why| Refused::TheFileCouldNotBeRead(why.message()))?;
    if held.len() < size_of::<CMSG_SIGNER_INFO>() {
        return Err(Refused::TheFileCouldNotBeRead(
            "the signature holds no signer".to_string(),
        ));
    }

    // SAFETY: the call above filled this buffer with a CMSG_SIGNER_INFO, and the
    // length was checked against that type's size on the line before.
    let signer = unsafe { &*held.as_ptr().cast::<CMSG_SIGNER_INFO>() };
    let looking_for = CERT_INFO {
        Issuer: signer.Issuer,
        SerialNumber: signer.SerialNumber,
        ..Default::default()
    };

    // SAFETY: the store is open and `looking_for` lives for the call.
    let certificate = unsafe {
        CertFindCertificateInStore(
            store,
            CERT_QUERY_ENCODING_TYPE(X509_ASN_ENCODING.0 | PKCS_7_ASN_ENCODING.0),
            0,
            CERT_FIND_SUBJECT_CERT,
            Some((&raw const looking_for).cast()),
            None,
        )
    };
    if certificate.is_null() {
        return Err(Refused::TheFileCouldNotBeRead(
            "the certificate that signed it is not in the file".to_string(),
        ));
    }

    let mut name = [0_u16; ROOM_FOR_A_NAME];
    // SAFETY: the context came from the call above and the buffer is this
    // stack frame's, with its length handed over so the call cannot overrun it.
    let written = unsafe {
        CertGetNameStringW(
            certificate,
            CERT_NAME_SIMPLE_DISPLAY_TYPE,
            0,
            None,
            Some(&mut name),
        )
    };
    // SAFETY: freed exactly once, and nothing reads it after this. The answer
    // is discarded because this call is documented as always answering true.
    let _ = unsafe { CertFreeCertificateContext(Some(certificate)) };

    // The count includes the closing nought, so a name of one character answers
    // two and an empty answer answers one.
    let letters = (written as usize).saturating_sub(1);
    Ok(WhoSignedIt {
        name: String::from_utf16_lossy(&name[..letters.min(ROOM_FOR_A_NAME)]),
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
        Ok(()) => Ok(Verified {
            at: downloaded.at,
            version: downloaded.version,
        }),
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
    let kept: String = published
        .chars()
        .filter(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '.' | '_' | '-' | '+'))
        .collect();
    let kept = kept.trim_matches('.').to_string();
    if kept.is_empty() {
        "installer.exe".to_string()
    } else {
        kept
    }
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
pub async fn fetch(version: &str, files: &[ReleaseFile], paths: &AppPaths) -> Fetched {
    // Asked first, before a single byte is fetched. Downloading an executable
    // and then finding out there is no way to look at it leaves it on somebody's
    // disk for as long as it takes to refuse, and spends their connection to
    // learn something this program already knew.
    if let Err(no_way) = whether_a_signature_can_be_checked_here() {
        return Fetched::Refused(no_way);
    }
    let Some(installer) = the_installer_among(files) else {
        return Fetched::NotArrived(NotArrived::NoInstallerAmongTheFiles);
    };
    match bring_it_down(version, installer, paths).await {
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
    version: &str,
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

    Ok(Downloaded {
        at,
        version: version.to_string(),
    })
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

    /// A checked installer, made the only way anything outside this module
    /// could not.
    ///
    /// Inside the test module the private fields are reachable, which is the
    /// whole reason these tests can exist at all and the whole reason no other
    /// module can do this.
    fn a_checked_installer(at: PathBuf) -> Verified {
        Verified {
            at,
            version: "v0.117.0".to_string(),
        }
    }

    #[test]
    fn test_saying_whether_a_signature_can_be_checked_agrees_with_the_check_itself() {
        // Two answers that have to be the same answer. One decides whether to
        // download at all and the other decides what a downloaded file is; a
        // platform where they disagree either downloads an executable it can
        // never look at, or refuses to download one it could have checked.
        let dir = TempDir::new().unwrap();
        let at = dir.path().join("nothing-signed-this.exe");
        std::fs::write(&at, b"MZ and then nothing that means anything").unwrap();

        match whether_a_signature_can_be_checked_here() {
            Ok(()) => assert!(
                !matches!(who_signed(&at), Err(Refused::NoWayToCheckHere)),
                "this says a signature can be checked here and the check says it cannot"
            ),
            Err(why) => {
                assert_eq!(why, Refused::NoWayToCheckHere);
                assert_eq!(
                    who_signed(&at),
                    Err(Refused::NoWayToCheckHere),
                    "this says a signature cannot be checked here and the check disagrees"
                );
            }
        }
    }

    /// Whether the shipped `fetch` asks whether a signature can be checked
    /// before it downloads anything.
    ///
    /// Split out so the reading can be shown a violation as well as the tree.
    /// Read rather than driven, because on a machine that can check a signature
    /// the order is invisible to any test: both orders succeed, and the one
    /// that is wrong is only wrong somewhere this suite does not run.
    fn it_asks_before_it_downloads(source: &str) -> bool {
        let Some((_, after)) = source.split_once("pub async fn fetch(") else {
            return false;
        };
        let body = &after[..after.find("\n}\n").unwrap_or(after.len())];
        match (
            body.find("whether_a_signature_can_be_checked_here"),
            body.find("bring_it_down"),
        ) {
            (Some(asked), Some(fetched)) => asked < fetched,
            _ => false,
        }
    }

    #[test]
    fn test_where_a_signature_cannot_be_checked_nothing_is_even_downloaded() {
        assert!(
            it_asks_before_it_downloads(&what_this_module_ships()),
            "an executable is fetched onto somebody's disk before this program asks \
             whether it has any way of looking at it"
        );
    }

    #[test]
    fn test_that_reading_can_see_a_fetch_that_downloads_first() {
        let downloads_first = "pub async fn fetch(files: &[ReleaseFile]) -> Fetched {\n\
             \x20   let got = bring_it_down(files).await;\n\
             \x20   whether_a_signature_can_be_checked_here()?;\n\
             \x20   got\n\
             }\n";
        assert!(
            !it_asks_before_it_downloads(downloads_first),
            "the reading above cannot see a fetch that downloads before it asks"
        );
        let asks_first = "pub async fn fetch(files: &[ReleaseFile]) -> Fetched {\n\
             \x20   whether_a_signature_can_be_checked_here()?;\n\
             \x20   bring_it_down(files).await\n\
             }\n";
        assert!(
            it_asks_before_it_downloads(asks_first),
            "the reading above cannot see a fetch that does the right thing either, so \
             it answers no to everything"
        );
    }

    #[test]
    fn test_the_question_is_about_running_and_says_the_program_will_close() {
        let installer = a_checked_installer(PathBuf::from("Wixen-Mail-Setup-0.117.0.exe"));
        let asked = the_question_about(&installer);

        assert!(
            asked.contains("v0.117.0"),
            "a person asked whether to install something should be told what: {asked}"
        );
        assert!(
            asked.to_lowercase().contains("close"),
            "the last moment somebody can decline is the moment they have to be told \
             the window is going: {asked}"
        );
        // Not about downloading. That already happened, with consent given at
        // the setting, and a second question about it teaches somebody to
        // answer without reading.
        assert!(
            !asked.to_lowercase().contains("download"),
            "the only question this feature asks is whether to run it: {asked}"
        );
    }

    #[test]
    fn test_what_is_said_before_the_window_closes_names_the_closing() {
        let installer = a_checked_installer(PathBuf::from("Wixen-Mail-Setup-0.117.0.exe"));
        let said = what_is_said_before_the_window_closes(&installer);
        assert!(!said.is_empty(), "the window goes with nothing said");
        assert!(
            said.to_lowercase().contains("clos"),
            "a screen reader user loses their bearings when a window disappears, so \
             the sentence before it has to say that is what is happening: {said}"
        );
    }

    #[test]
    fn test_saying_no_deletes_the_installer_and_leaves_this_version_running() {
        let dir = TempDir::new().unwrap();
        let at = dir.path().join("Wixen-Mail-Setup-0.117.0.exe");
        std::fs::write(&at, b"MZ").unwrap();

        say_no(a_checked_installer(at.clone())).expect("declining to work");

        assert!(
            !at.exists(),
            "a declined installer was left where somebody can find it and run it by hand, \
             having just decided not to"
        );
    }

    #[test]
    fn test_a_handover_that_could_not_start_says_so_and_leaves_this_version_running() {
        // The half of the handover a test can reach. That an installer really
        // starts, and really replaces the files, is nothing here has ever seen.
        let dir = TempDir::new().unwrap();
        let never_there = dir.path().join("Wixen-Mail-Setup-0.117.0.exe");

        let outcome = hand_over_to(&a_checked_installer(never_there));

        assert!(
            outcome.is_err(),
            "a handover to a file that is not there reported success, so somebody would \
             be told their update had started and watch the window close on nothing"
        );
    }

    #[test]
    fn test_a_failed_handover_says_the_running_version_is_untouched() {
        for why in [
            HandoverFailed::ItIsNoLongerThere,
            HandoverFailed::ItWouldNotStart("access is denied".to_string()),
        ] {
            let said = why.said();
            assert!(!said.is_empty(), "{why:?} says nothing at all");
            assert!(
                said.contains(&the_releases_page()),
                "{why:?} leaves somebody with nowhere to go: {said}"
            );
        }
        assert_ne!(
            HandoverFailed::ItIsNoLongerThere.said(),
            HandoverFailed::ItWouldNotStart("access is denied".to_string()).said(),
            "the two ways a handover fails say the same thing"
        );
    }

    /// Whether the module's shipped half lets anything but the check make a
    /// checked installer.
    ///
    /// Split out so the reading can be shown a violation as well as the tree.
    /// The rule is that `Verified` is built in exactly one place, which is
    /// `verify`, and that no public function hands one out. A constructor taking
    /// a path would compile, would break nothing at any call site, and would
    /// quietly undo the whole design.
    fn anything_but_the_check_can_make_one(source: &str) -> bool {
        // `struct Verified {` and `impl Verified {` say the same ten characters
        // and neither builds one. Counting those as constructions was the first
        // version of this and it put the real tree exactly on the threshold,
        // which is the way a reading over text goes wrong: right about the
        // answer, wrong about what it counted.
        let built = source
            .match_indices("Verified {")
            .filter(|(at, _)| {
                let before = source[..*at].trim_end();
                !before.ends_with("struct") && !before.ends_with("impl")
            })
            .count();
        let handed_out = source
            .lines()
            .filter(|line| {
                let line = line.trim();
                line.starts_with("pub fn")
                    && !line.starts_with("pub fn verify(")
                    && line
                        .split_once("->")
                        .is_some_and(|(_, returns)| returns.contains("Verified"))
            })
            .count();
        // Exactly one construction, which is `verify`'s, and nothing public
        // handing one out. Not "at most one": nought means this reading has
        // stopped finding the construction it is about, which is a broken check
        // rather than a clean tree.
        built != 1 || handed_out > 0
    }

    #[test]
    fn test_only_the_check_can_make_a_checked_installer() {
        assert!(
            !anything_but_the_check_can_make_one(&what_this_module_ships()),
            "something other than the check can produce a checked installer, so the \
             question and the handover can both be handed a file nothing looked at"
        );
    }

    #[test]
    fn test_that_reading_can_see_a_second_way_to_make_one() {
        // The whole module in miniature, with one extra constructor in it. That
        // constructor compiles, breaks nothing at any call site, and undoes the
        // entire design of this file, which is exactly why it is the violation
        // worth proving the reading can see.
        let doctored = "pub struct Verified { at: PathBuf }\n\
             impl Verified {\n\
             \x20   pub fn from_a_path(at: PathBuf) -> Self { Verified { at } }\n\
             }\n\
             pub fn verify(d: Downloaded) -> Result<Verified, Refused> {\n\
             \x20   Ok(Verified { at: d.at })\n\
             }\n";
        assert!(
            anything_but_the_check_can_make_one(doctored),
            "the reading above cannot see the thing it exists to refuse"
        );
    }

    #[test]
    fn test_that_reading_can_see_a_checked_installer_handed_out() {
        let doctored = "pub struct Verified { at: PathBuf }\n\
             pub fn verify(d: Downloaded) -> Result<Verified, Refused> {\n\
             \x20   Ok(Verified { at: d.at })\n\
             }\n\
             pub fn trust_me(at: PathBuf) -> Verified { from_somewhere(at) }\n";
        assert!(
            anything_but_the_check_can_make_one(doctored),
            "a public function handing out a checked installer walks past this reading"
        );
    }

    #[test]
    fn test_an_offer_is_fetched_without_anybody_being_asked() {
        use crate::common::version::ReleaseChannel;
        use crate::service::update_check::Answer;

        let offer = Answer::ANewerVersion {
            version: "v0.117.0".to_string(),
            page: "https://github.com/PratikP1/Wixen-Mail/releases/tag/v0.117.0".to_string(),
            channel: ReleaseChannel::PublicReleases,
            files: a_real_release(),
        };
        assert_eq!(
            what_to_do_about(&offer),
            NextStep::FetchIt(a_real_release()),
            "a kind of version was chosen, so the fetch is the program's own doing"
        );
    }

    #[test]
    fn test_every_other_answer_leads_to_a_sentence_and_nothing_else() {
        use crate::common::version::ReleaseChannel;
        use crate::service::update_check::{Answer, NotFetched};

        for answer in [
            Answer::ThisIsTheNewest {
                channel: ReleaseChannel::PublicReleases,
            },
            Answer::NothingPublishedYet {
                channel: ReleaseChannel::PublicReleases,
            },
            Answer::CouldNotBeFetched {
                why: NotFetched::NoAnswer,
            },
            Answer::CouldNotBeUnderstood { entries: 2 },
        ] {
            assert_eq!(
                what_to_do_about(&answer),
                NextStep::JustSayIt,
                "{answer:?} is not an offer, so there is nothing to fetch"
            );
        }
    }

    #[test]
    fn test_the_name_this_looks_for_is_the_name_the_installer_is_built_under() {
        // Read from the script that builds it. A release publishes two
        // executables and only one of them installs anything, so which name
        // this looks for is the whole of the choice, and a copy of it that
        // drifted would quietly start fetching the portable build.
        let script = std::fs::read_to_string("installer/Wixen-Mail-Setup.iss")
            .expect("the installer script to be readable");
        let built_as = script
            .lines()
            .find_map(|line| line.trim().strip_prefix("OutputBaseFilename="))
            .map(str::trim)
            .expect("the installer script to name what it builds");
        assert_eq!(
            built_as.replace("{#AppVersion}", ""),
            WHAT_THE_INSTALLER_IS_CALLED
        );
    }

    #[test]
    fn test_the_reasons_nothing_arrived_do_not_all_say_the_same_thing() {
        // Six ways a download does not happen, and they lead somebody to do
        // different things: wait, check their connection, or go and fetch it by
        // hand. One sentence for all six would be a status line saying only
        // that something went wrong.
        let every = [
            NotArrived::NoInstallerAmongTheFiles,
            NotArrived::TheConnectionFailed,
            NotArrived::ARedirectLeftHttps,
            NotArrived::TooManyRedirects,
            NotArrived::BiggerThanWeWillAccept,
            NotArrived::CouldNotBeKept("the disk is full".to_string()),
        ];
        let said: Vec<String> = every.iter().map(NotArrived::said).collect();
        for sentence in &said {
            assert!(!sentence.is_empty(), "one of these says nothing at all");
            assert_eq!(
                said.iter().filter(|other| *other == sentence).count(),
                1,
                "two of these say the same thing: {sentence}"
            );
            assert!(
                sentence.contains(&the_releases_page()),
                "a download that did not happen leaves somebody with nowhere to go: {sentence}"
            );
        }
    }

    /// Whether the module's shipped half names a way of asking somebody
    /// something.
    ///
    /// Split out so the reading can be shown a violation as well as the tree.
    fn it_asks_somebody_something(source: &str) -> bool {
        ["MessageDialog", "show_modal", "wxdragon"]
            .iter()
            .any(|way| source.contains(way))
    }

    #[test]
    fn test_nothing_in_this_module_asks_anybody_anything() {
        // The only question in this feature is whether to run an installer, and
        // that question belongs to a file which has already passed both checks.
        // A question asked from here could be asked about a file nothing had
        // looked at yet, which is the one thing the design of this module
        // exists to make impossible.
        assert!(
            !it_asks_somebody_something(&what_this_module_ships()),
            "this module puts a question in front of somebody, and it has no business doing so"
        );
    }

    #[test]
    fn test_that_reading_can_see_a_question_being_asked() {
        assert!(
            it_asks_somebody_something(
                "let answered = MessageDialog::builder(frame).show_modal();"
            ),
            "the reading above cannot see the thing it exists to refuse"
        );
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

        let refused = verify(Downloaded {
            at: at.clone(),
            version: "v0.117.0".to_string(),
        });

        assert!(refused.is_err(), "an unsigned file was accepted");
        assert!(
            !at.exists(),
            "a refused installer was left where somebody can find it and run it by hand"
        );
    }

    /// Files somebody else really signed, to be tried in order.
    ///
    /// The two easy refusals, nothing signed it and the signature is broken,
    /// are the ones a naive implementation already fails. The one it passes is
    /// a validly signed executable somebody else published, so this is the
    /// fixture that decides whether any of this is worth having.
    ///
    /// **Not `notepad.exe`, and the reason is worth writing down.** That was
    /// the obvious choice and it is wrong: almost every Windows system binary
    /// is signed through a catalogue file rather than with a signature inside
    /// the file, and `WinVerifyTrust` asked about a file does not go looking in
    /// catalogues. So `notepad.exe` comes back as `TRUST_E_NOSIGNATURE`, which
    /// would have made this test pass for the wrong reason if the test had been
    /// written a little more loosely. These were found by asking Windows which
    /// files in `System32` carry an embedded signature, on 2026-09-12:
    ///
    /// ```text
    /// Get-ChildItem 'C:\Windows\System32\*.exe' |
    ///   ForEach-Object { Get-AuthenticodeSignature $_.FullName } |
    ///   Where-Object { $_.SignatureType -eq 'Authenticode' }
    /// ```
    ///
    /// More than one, because none of them ships on literally every machine and
    /// a fixture missing from one is not a reason for the check to go untested
    /// on the rest.
    #[cfg(target_os = "windows")]
    const SIGNED_BY_SOMEBODY_ELSE: [&str; 4] = [
        r"C:\Windows\System32\microsoft.windows.softwarelogo.showdesktop.exe",
        r"C:\Windows\System32\MpSigStub.exe",
        r"C:\Windows\System32\MRT.exe",
        r"C:\Windows\System32\appverif.exe",
    ];

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_file_somebody_else_signed_is_refused_for_that_reason() {
        let mut tried = Vec::new();
        for candidate in SIGNED_BY_SOMEBODY_ELSE {
            let at = Path::new(candidate);
            if !at.exists() {
                tried.push(format!("{candidate}: not on this machine"));
                continue;
            }
            match who_signed(at) {
                Ok(who) => {
                    assert!(
                        matches!(
                            whether_that_signer_is_ours(&who),
                            Err(Refused::SomebodyElseSignedIt { .. })
                        ),
                        "{candidate} is signed by {} and was accepted as this project's own",
                        who.name
                    );
                    assert!(
                        !who.name.is_empty(),
                        "{candidate} is validly signed and the signer's name came back empty, \
                         so the reading of the certificate is broken and every file would be \
                         refused as unsigned"
                    );
                    return;
                }
                Err(why) => tried.push(format!("{candidate}: {why:?}")),
            }
        }
        // Said rather than passed over, because this is the one refusal a naive
        // implementation passes and a silent skip reads as a green result.
        panic!(
            "no file with a signature inside it could be found, so the refusal that matters \
             is untested on this machine:\n  {}",
            tried.join("\n  ")
        );
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
