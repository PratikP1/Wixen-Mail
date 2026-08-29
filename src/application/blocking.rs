//! Never hearing from this sender again.
//!
//! # A block is a rule
//!
//! The rules engine in [`crate::application::filters`] can already move mail
//! from an address into a folder, which is what blocking somebody means. So
//! this makes one of its rules rather than building a second list beside it.
//! Nothing new runs at sync time, a block shows up in the rule list where
//! somebody expects to find it, and there is one answer to "why did this
//! message move" instead of two.
//!
//! What this adds is the part that would otherwise have to be typed correctly
//! by hand: the right pattern, the right folder, a name that says what it is,
//! and the questions worth asking before the rule exists.

use crate::application::allowed::Allowed;
use crate::common::types::FolderType;
use crate::common::{Error, Result};
use crate::data::message_cache::MessageFilterRule;

/// Who is being blocked.
///
/// Two cases rather than one string and a flag, because they are matched
/// differently and mixing them up is expensive in one direction: a domain
/// block written as an address block does nothing, and an address block
/// written as a domain block sends a whole company's mail to Junk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// One person, at exactly this address.
    ThisAddress(String),
    /// Everyone writing from this domain, and from anything under it.
    EveryoneAt(String),
}

impl Block {
    /// The address or the domain, as it is stored and shown.
    pub fn as_written(&self) -> &str {
        match self {
            Block::ThisAddress(address) => address,
            Block::EveryoneAt(domain) => domain,
        }
    }
}

/// Block the person a message came from, and nobody else.
pub fn just_this_sender(from: &str) -> Result<Block> {
    Ok(Block::ThisAddress(the_address_in(from)?))
}

/// Block everybody writing from the same place as this sender.
///
/// One unwanted sender is usually one of many at the same domain, and the
/// next one arrives under a different name.
pub fn everyone_at_the_senders_domain(from: &str) -> Result<Block> {
    let address = the_address_in(from)?;
    let Some((_, domain)) = address.rsplit_once('@') else {
        // Unreachable: `the_address_in` refuses anything with no domain half.
        // Written as a refusal rather than an unwrap so that a future change
        // to that function cannot turn into a panic here.
        return Err(no_address(from));
    };
    Ok(Block::EveryoneAt(domain.to_string()))
}

/// The address inside a `From` header, or a refusal.
///
/// The stored `from` is whatever the sender wrote, so it arrives either as a
/// bare address or as a display name wrapped around one. The same parser the
/// rest of this program reads headers with is used here, rather than a second
/// hand-rolled one that might disagree with it about the same text.
///
/// Folded to lower case, so that blocking `Ada@Example.com` and blocking
/// `ada@example.com` are one block rather than two that both claim to be the
/// only one. ASCII folding, matching how addresses are compared everywhere
/// else in this program.
fn the_address_in(from: &str) -> Result<String> {
    let written = from.trim();
    let address = crate::service::mime::parse_addresses(written)
        .into_iter()
        .next()
        .map(|parsed| parsed.address)
        // A value the parser makes nothing of may still be a bare address,
        // and refusing before looking at it would refuse the ordinary case.
        .unwrap_or_else(|| written.to_string())
        .trim()
        .to_ascii_lowercase();

    // Split at the last `@`, because a quoted local part may hold one and the
    // domain never can.
    let Some((local_part, domain)) = address.rsplit_once('@') else {
        return Err(no_address(from));
    };
    if local_part.is_empty() || !a_domain_could_look_like_this(domain) {
        return Err(no_address(from));
    }
    Ok(address)
}

/// Whether this is shaped like the domain half of an address.
///
/// Deliberately not a full check of what a domain may be: the only job here
/// is to refuse text that would make a nonsense rule. A domain holding a
/// space, an angle bracket or a second `@` is text somebody's mail program
/// mangled, not a place mail comes from.
fn a_domain_could_look_like_this(domain: &str) -> bool {
    !domain.is_empty()
        && !domain.contains(['@', '<', '>', ',', ';', '"'])
        && !domain.chars().any(char::is_whitespace)
}

/// What to say when there is nothing there to block.
///
/// A block built from text with no address in it would become a rule that
/// matches everything, which files the whole mailbox into Junk.
fn no_address(from: &str) -> Error {
    Error::Other(format!(
        "There is no email address in \"{}\", so there is nothing to block. \
         Open the message and block it from there, or type the address itself.",
        from.trim()
    ))
}

/// The start of the name every block's rule is given.
///
/// The name is the readable half of a block's identity, and it is what makes
/// a block findable again: the rules table keeps one name per account, so two
/// blocks cannot collide and the same block twice is the same row.
const BLOCKED: &str = "Blocked: ";

/// How a domain block reads in that name.
const EVERYONE_AT: &str = "everyone at ";

/// Turn a block into the rule that carries it out.
///
/// `made_at` is passed in rather than read from the clock here, so that what
/// this produces depends only on what it was given.
pub fn a_rule_that_blocks(
    account_id: &str,
    block: &Block,
    into_folder: &str,
    made_at: &str,
) -> MessageFilterRule {
    MessageFilterRule {
        id: an_id_for(account_id, block),
        account_id: account_id.to_string(),
        name: the_name_of(block),
        field: "from".to_string(),
        match_type: "regex".to_string(),
        pattern: a_pattern_matching(block),
        case_sensitive: false,
        action_type: "move_to_folder".to_string(),
        action_value: Some(into_folder.to_string()),
        enabled: true,
        created_at: made_at.to_string(),
    }
}

/// What a block's rule is called.
fn the_name_of(block: &Block) -> String {
    match block {
        Block::ThisAddress(address) => format!("{BLOCKED}{address}"),
        Block::EveryoneAt(domain) => format!("{BLOCKED}{EVERYONE_AT}{domain}"),
    }
}

/// The row this block occupies, which is the same row every time it is made.
///
/// Worked out from the account and the block rather than made up fresh, so
/// blocking somebody twice writes the same row instead of two rows that both
/// claim to be the block. The account is part of it because the rules table
/// keeps one row per id across every account.
///
/// Read by nothing. It is an identifier, and the readable half of a block
/// lives in its name.
fn an_id_for(account_id: &str, block: &Block) -> String {
    let (kind, target) = match block {
        Block::ThisAddress(address) => ("address", address),
        Block::EveryoneAt(domain) => ("domain", domain),
    };
    format!("block:{account_id}:{kind}:{target}")
}

/// What has to follow a blocked address or domain in a `From` header.
///
/// The end of the field, the bracket that closes an address, or a separator.
/// This is what keeps a block on `example.com` off `example.com.evil.test`,
/// which is the whole reason a block is a bounded pattern and not a search
/// for the text somewhere in the field.
const AND_THEN_IT_ENDS: &str = "(?:$|[\\s>,;])";

/// The pattern that matches this block and nothing else.
///
/// A pattern rather than a plain comparison, because the stored `from` is a
/// display name and an address together when the sender gave a name, and the
/// address alone when they did not. "Equals" matches neither reliably and
/// "contains" matches far too much: blocking `ada@example.com` with a
/// contains rule also blocks `notada@example.com`.
///
/// Every character of the address is escaped first. An address may hold a
/// full stop or a plus, both of which mean something in a pattern, and left
/// alone `a.b@example.com` would also match `axb@example.com`, who is
/// somebody else.
fn a_pattern_matching(block: &Block) -> String {
    match block {
        // Preceded by the start of the field, the bracket that opens an
        // address, or a separator.
        Block::ThisAddress(address) => {
            format!("(?:^|[\\s<,;]){}{AND_THEN_IT_ENDS}", regex::escape(address))
        }
        // Preceded by the `@` of an address at this domain, or by the dot of
        // a name under it. Subdomains are included on purpose: senders nobody
        // wants arrive from a new one each week, and blocking the domain is
        // what somebody meant. A domain that merely ends the same way, like
        // `notexample.com`, is not preceded by either character.
        Block::EveryoneAt(domain) => {
            format!("[@.]{}{AND_THEN_IT_ENDS}", regex::escape(domain))
        }
    }
}

/// Where a blocked sender's mail is filed.
///
/// Three answers rather than a folder or nothing, because "this account has
/// no junk folder" and "nobody has asked this account what folders it has
/// yet" need different things done about them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedMailGoesTo<'a> {
    /// File it here. Out of the inbox, still readable, still recoverable.
    TheJunkFolder(&'a str),
    /// This account has folders and none of them is its junk folder, so
    /// nothing is blocked and [`NO_JUNK_FOLDER_FOUND`] says what to do.
    NoJunkFolderFound,
    /// This account has never been asked what folders it has.
    NoFoldersKnownYet,
}

/// What to say when no folder on the account reads as its junk folder.
///
/// The trash is not offered as a second best. On most servers the trash
/// empties itself after a few weeks, so filing there is deleting with a
/// delay, and a person who asked for a sender to be filed away did not ask
/// for their mail to be destroyed on a timer.
pub const NO_JUNK_FOLDER_FOUND: &str = "Nothing has been blocked. This account does not say which of its folders it keeps junk mail \
     in, so there is nowhere to file blocked mail. Make a folder for it on the account, check for \
     mail once so this program can see it, and try again.";

/// What to say when the account has never been asked what folders it has.
pub const NO_FOLDERS_KNOWN_YET: &str = "Nothing has been blocked. This account has not learned what folders it has yet. Check for \
     mail once, and blocking will be able to file mail into the junk folder from then on.";

/// Where a block should file the mail it catches.
///
/// # Why junk and not deleted
///
/// Junk, on all four counts that matter.
///
/// A block is a filing decision. Somebody saying "never this address again"
/// is saying they do not want it in front of them, not that they want it
/// destroyed, and those are different requests with different costs when the
/// answer turns out to be wrong.
///
/// Blocks are wrong at the edges more often than anything else here. Blocking
/// a domain catches a colleague at the same provider, and blocking one
/// address catches somebody who matters six months later. The junk folder is
/// where people already look when mail has gone missing, and it is where
/// every mail provider's own blocking puts it, so it is where somebody thinks
/// to look without being told.
///
/// Deleting would not even mean what it looks like. The delete a rule can
/// carry out in [`crate::application::filters`] removes the copy on this
/// computer and leaves the message at the server, so a blocked message would
/// be gone here and still in the inbox on a phone. The same mailbox would
/// answer differently depending on which device somebody opened.
///
/// And the junk folder already exists, is already recognised and is already
/// searched, so nothing has to be created on somebody's server for a block to
/// work.
///
/// It was not already downloaded, which this used to say it was. On a server
/// account the sync leaves it out by default, so the folder blocked mail is
/// filed into was one this program never fetched and never listed. See
/// [`TheJunkFolder`], which is what blocking now does about that.
pub fn where_blocked_mail_goes<'a>(
    folders: impl IntoIterator<Item = (&'a str, FolderType)>,
) -> BlockedMailGoesTo<'a> {
    let mut folders = folders.into_iter().peekable();
    if folders.peek().is_none() {
        return BlockedMailGoesTo::NoFoldersKnownYet;
    }
    match folders.find(|(_, kind)| *kind == FolderType::Spam) {
        Some((path, _)) => BlockedMailGoesTo::TheJunkFolder(path),
        None => BlockedMailGoesTo::NoJunkFolderFound,
    }
}

/// What blocking has to do about the junk folder before a block can work.
///
/// # Why a block switches a folder on
///
/// Filing mail into Junk only helps if Junk is a folder this program
/// downloads, and on a server account it is not: the sync leaves it out by
/// default, because downloading a spam folder costs the whole of it. The
/// folder tree leaves out what is not downloaded, for its own good reason, so
/// the two together sent blocked mail to a folder that was neither filled nor
/// listed. The recovery route this feature promises was empty inside the
/// program.
///
/// That lands exactly on the case blocking was designed around. A block on a
/// whole domain catches a colleague sooner or later, and the person then goes
/// looking in Junk, which is the point of filing there rather than deleting.
///
/// So blocking switches the folder on. It is a change nobody asked for in so
/// many words, and it is the change that makes the one they did ask for mean
/// anything: mail filed where it cannot be opened has been destroyed as far as
/// they are concerned. It is announced in the same breath, it is undone in one
/// place, and it is never done over the top of somebody who said no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheJunkFolder {
    /// Already downloaded, or kept on this computer. Nothing to do.
    AlreadyKeptUpToDate,
    /// Nobody has said either way, and a server account does not download its
    /// junk folder unless somebody does, so blocking switches it on.
    IsSwitchedOnByBlocking,
    /// Not downloaded, and blocking has not changed that. Either somebody
    /// switched it off, which blocking does not overrule, or switching it on
    /// did not work. What they have to do about it is the same either way.
    IsNotBeingDownloaded,
}

/// What blocking has to do about the junk folder on this account.
///
/// `already_chosen` is what somebody has said about that folder. `None` is
/// "never asked", which is not the same as "asked and said no": they look the
/// same as a `false` and mean opposite things, which is the distinction
/// [`crate::application::mail_sync::FolderChoices`] exists to keep.
pub fn what_the_junk_folder_needs(
    junk_folder: &str,
    already_chosen: Option<bool>,
) -> TheJunkFolder {
    // A folder on this computer is always there to be opened. The choice only
    // decides what is downloaded from a server, and this folder has none.
    if crate::application::local_folders::is_local(junk_folder) {
        return TheJunkFolder::AlreadyKeptUpToDate;
    }
    match already_chosen {
        Some(true) => TheJunkFolder::AlreadyKeptUpToDate,
        Some(false) => TheJunkFolder::IsNotBeingDownloaded,
        None => TheJunkFolder::IsSwitchedOnByBlocking,
    }
}

/// What is already true when somebody asks to block a sender.
///
/// Gathered by the caller rather than looked up here, so that deciding
/// whether a block makes sense stays something that can be run without a
/// database, a mailbox or a window.
#[derive(Debug, Clone, Copy)]
pub struct WhatIsAlreadyTrue<'a> {
    /// Every address this person receives mail at, across their accounts.
    pub their_own_addresses: &'a [String],
    /// The rules this account already has, which is where its blocks live.
    pub rules_already_there: &'a [MessageFilterRule],
    /// What the message's `List-Unsubscribe` header said, when it had one.
    ///
    /// Its presence is what says the message came from a mailing list. The
    /// value is a list of places to write or visit, wrapped in angle
    /// brackets, and some lists give an empty one.
    pub how_to_leave_the_list: Option<&'a str>,
    /// The sender this block was made from, when it was made from a message.
    ///
    /// Only used to write a better sentence: a block on a whole domain has
    /// forgotten which sender started it, and "block that one address
    /// instead" is worth saying with the address in it.
    pub the_message_was_from: Option<&'a str>,
}

/// Whether this block should be made, and what to say either way.
///
/// Three answers rather than yes and no, because the mailing list case is
/// neither. Blocking a list works exactly as asked and is still usually the
/// wrong tool, so the answer is yes with something worth reading first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MayBlock {
    /// Nothing stands in the way.
    Yes,
    /// Go ahead, once they have read this.
    YesButFirst(String),
    /// No. The sentence says why, and what to do instead.
    No(String),
}

/// Ask whether this block makes sense before making it.
pub fn may_block(account_id: &str, block: &Block, known: &WhatIsAlreadyTrue<'_>) -> MayBlock {
    // Their own first. It is the answer that is about them rather than about
    // the state of a rule list, and it is the one that costs most to get
    // wrong.
    if let Some(refusal) = blocking_themselves(block, known) {
        return MayBlock::No(refusal);
    }
    if let Some(refusal) = already_covered(account_id, block, known.rules_already_there) {
        return MayBlock::No(refusal);
    }
    match known.how_to_leave_the_list {
        Some(header) => MayBlock::YesButFirst(a_list_goes_on_sending(header)),
        None => MayBlock::Yes,
    }
}

/// Whether this block would catch the person making it, and why that is a no.
fn blocking_themselves(block: &Block, known: &WhatIsAlreadyTrue<'_>) -> Option<String> {
    let theirs: Vec<String> = known
        .their_own_addresses
        .iter()
        .filter_map(|written| the_address_in(written).ok())
        .collect();

    match block {
        Block::ThisAddress(address) => theirs.iter().any(|own| own == address).then(|| {
            format!(
                "{address} is your own address. Blocking it would send your own mail to Junk: the \
                 copies of messages you send, and anything you write to yourself. Nothing has \
                 been blocked."
            )
        }),
        Block::EveryoneAt(domain) => theirs
            .iter()
            .any(|own| is_at_or_under(the_domain_of(own), domain))
            .then(|| {
                format!(
                    "{domain} is the domain your own mail arrives at. Blocking it would send mail \
                     from everyone there to Junk, including your own. Nothing has been blocked. \
                     {}",
                    block_just_the_one_sender_instead(known.the_message_was_from)
                )
            }),
    }
}

/// The other half of the domain refusal: what to do instead.
fn block_just_the_one_sender_instead(sender: Option<&str>) -> String {
    match sender.map(str::trim).filter(|from| !from.is_empty()) {
        Some(from) => format!("To stop this one sender, block {from} on its own instead."),
        None => "To stop one sender, block that address on its own instead.".to_string(),
    }
}

/// Whether a block already stored on this account does this one's job.
///
/// The sentence names the folder that block really files into, taken from the
/// rule rather than assumed. Not every account calls it Junk, and a sentence
/// naming a folder somebody does not have sends them looking for one that is
/// not there.
fn already_covered(
    account_id: &str,
    wanted: &Block,
    rules: &[MessageFilterRule],
) -> Option<String> {
    let (covering, goes_to) = rules
        .iter()
        .filter(|rule| rule.account_id == account_id)
        .find_map(|rule| {
            let held = the_block_in(rule)?;
            covers(&held, wanted).then(|| (held, rule.action_value.clone().unwrap_or_default()))
        })?;

    Some(match (&covering, wanted) {
        (Block::EveryoneAt(domain), Block::ThisAddress(address)) => format!(
            "Mail from {address} already goes to {goes_to}, because you block everyone at \
             {domain}. Nothing has changed."
        ),
        _ => format!(
            "You already block {}, and its mail goes to {goes_to}. Nothing has changed. Blocked \
             senders are in the rule list for this account, where you can take one off again.",
            covering.as_written()
        ),
    })
}

/// Whether a block already in place does the job of one being asked for.
///
/// A domain block covers every address at that domain and every address
/// under it. An address block covers only itself: somebody who blocked one
/// address and now asks to block the whole domain is widening the block, and
/// that is a real thing to ask for.
fn covers(existing: &Block, wanted: &Block) -> bool {
    match (existing, wanted) {
        (Block::ThisAddress(held), Block::ThisAddress(asked)) => held == asked,
        (Block::EveryoneAt(held), Block::ThisAddress(asked)) => {
            is_at_or_under(the_domain_of(asked), held)
        }
        (Block::EveryoneAt(held), Block::EveryoneAt(asked)) => is_at_or_under(asked, held),
        (Block::ThisAddress(_), Block::EveryoneAt(_)) => false,
    }
}

/// The domain half of an address, or the whole of it when there is no `@`.
fn the_domain_of(address: &str) -> &str {
    address
        .rsplit_once('@')
        .map(|(_, domain)| domain)
        .unwrap_or(address)
}

/// Whether a domain is `under` itself or sits beneath it.
///
/// `mail.example.com` is under `example.com`. `notexample.com` is not, which
/// is the comparison a plain `ends_with` gets wrong.
fn is_at_or_under(domain: &str, under: &str) -> bool {
    domain == under || domain.ends_with(&format!(".{under}"))
}

/// What to say when the message being blocked came from a mailing list.
///
/// A block works exactly as asked here and still leaves the list sending.
/// Junk fills up, the list has no idea, and the address they signed up with
/// goes on being on it.
fn a_list_goes_on_sending(unsubscribe_header: &str) -> String {
    let leaving = "This message came from a mailing list. Blocking files it into Junk and the \
                   list carries on sending it.";
    match where_to_write_to_leave(unsubscribe_header) {
        Some(address) => {
            format!("{leaving} To stop it at the source, unsubscribe by writing to {address}.")
        }
        None => format!(
            "{leaving} The message gives no address to unsubscribe at, so look for a link to \
             leave the list at the bottom of it."
        ),
    }
}

/// The address to write to to leave a list, out of a `List-Unsubscribe`
/// header.
///
/// The header holds one or more places wrapped in angle brackets, and only
/// the `mailto:` one can be acted on by writing a message. A header that
/// offers only a web page gives nothing to name here, and saying so is better
/// than naming a link somebody then has to read out character by character.
fn where_to_write_to_leave(header: &str) -> Option<String> {
    header
        .split(',')
        .map(str::trim)
        .filter_map(|entry| entry.strip_prefix('<')?.strip_suffix('>'))
        .find_map(|uri| uri.strip_prefix("mailto:"))
        // A `mailto:` may carry a subject after a `?`, which is machinery
        // rather than something to read out.
        .map(|address| address.split('?').next().unwrap_or(address).trim())
        .filter(|address| !address.is_empty())
        .map(str::to_string)
}

/// Who this block catches, as a phrase a sentence can be built around.
fn whose_mail(block: &Block) -> String {
    match block {
        Block::ThisAddress(address) => format!("Mail from {address}"),
        Block::EveryoneAt(domain) => format!("Mail from everyone at {domain}"),
    }
}

/// The two things blocking here does not do.
///
/// Both are assumptions somebody will otherwise make, and both are wrong in a
/// way that takes weeks to notice. Nothing is reported to the mail provider,
/// so the provider goes on accepting the mail and nothing about the sender's
/// standing changes anywhere else. And nothing already in the mailbox moves,
/// because a rule is run on mail as it arrives.
const WHAT_IT_DOES_NOT_DO: &str = "This does not tell your mail provider anything, so the mail is still accepted and still \
     arrives here. Messages that already arrived stay where they are.";

/// What to say before a block is made.
pub fn what_blocking_will_do(
    block: &Block,
    junk_folder: &str,
    allowed: Allowed,
    junk: TheJunkFolder,
) -> String {
    format!(
        "{} will go to {junk_folder} from now on.{} {WHAT_IT_DOES_NOT_DO}{}",
        whose_mail(block),
        what_will_happen_to_the_junk_folder(junk_folder, junk),
        but_mail_changes_are_off(allowed)
    )
}

/// What to say once it has been made.
pub fn what_blocking_did(
    block: &Block,
    junk_folder: &str,
    allowed: Allowed,
    junk: TheJunkFolder,
) -> String {
    format!(
        "{} now goes to {junk_folder}.{} {WHAT_IT_DOES_NOT_DO}{}",
        whose_mail(block),
        what_happened_to_the_junk_folder(junk_folder, junk),
        but_mail_changes_are_off(allowed)
    )
}

/// Where somebody chooses which folders are downloaded.
///
/// Named once, because every sentence here sends them to it and a menu path
/// written out three times is a menu path that drifts.
const WHERE_FOLDERS_ARE_CHOSEN: &str = "File, then Folders to Keep Up to Date";

/// Why a junk folder nobody downloads makes a block worth nothing.
const A_FOLDER_NOT_DOWNLOADED_CANNOT_BE_READ: &str =
    "Blocked mail filed into a folder that is not downloaded cannot be read here at all.";

/// What to say about the junk folder, once the block has been made.
fn what_happened_to_the_junk_folder(junk_folder: &str, junk: TheJunkFolder) -> String {
    match junk {
        // Nothing. A line that counts the nothings teaches somebody to stop
        // listening to the one that matters.
        TheJunkFolder::AlreadyKeptUpToDate => String::new(),
        TheJunkFolder::IsSwitchedOnByBlocking => format!(
            " {junk_folder} was not being downloaded to this computer, so it has been switched \
             on. {A_FOLDER_NOT_DOWNLOADED_CANNOT_BE_READ} To stop downloading it, use \
             {WHERE_FOLDERS_ARE_CHOSEN}."
        ),
        TheJunkFolder::IsNotBeingDownloaded => it_is_not_being_downloaded(junk_folder),
    }
}

/// What to say about the junk folder, before the block is made.
fn what_will_happen_to_the_junk_folder(junk_folder: &str, junk: TheJunkFolder) -> String {
    match junk {
        TheJunkFolder::AlreadyKeptUpToDate => String::new(),
        TheJunkFolder::IsSwitchedOnByBlocking => format!(
            " {junk_folder} is not being downloaded to this computer, so blocking will switch it \
             on. {A_FOLDER_NOT_DOWNLOADED_CANNOT_BE_READ} To stop downloading it, use \
             {WHERE_FOLDERS_ARE_CHOSEN}."
        ),
        TheJunkFolder::IsNotBeingDownloaded => it_is_not_being_downloaded(junk_folder),
    }
}

/// What to say when the junk folder stays undownloaded either way.
///
/// The same before and after, because it is a state rather than something
/// that happened: somebody switched that folder off, or switching it on did
/// not work, and what they have to do about it is the same.
fn it_is_not_being_downloaded(junk_folder: &str) -> String {
    format!(
        " {junk_folder} is not being downloaded to this computer, so blocked mail is filed there \
         at the server and you will not be able to read it here. To download that folder, use \
         {WHERE_FOLDERS_ARE_CHOSEN}."
    )
}

/// What to say once a block has been taken off.
pub fn what_unblocking_did(block: &Block) -> String {
    format!(
        "{} will arrive in your inbox again. Anything already filed away stays where it is until \
         you move it back.",
        whose_mail(block)
    )
}

/// The sentence a block needs when it cannot actually be carried out yet.
///
/// A block files mail into a folder on the server, which is a change to
/// somebody's mail, and every change to somebody's mail is behind the same
/// permission. With that permission off the rule is stored and the move is
/// held back every time it matches, so the block exists and does nothing.
/// Saying nothing here would leave somebody believing it works.
fn but_mail_changes_are_off(allowed: Allowed) -> &'static str {
    if allowed.mail {
        ""
    } else {
        " This will not happen yet, because moving mail on the server is switched off in Allowed \
         Changes. The rule is saved and starts working when you turn mail changes on."
    }
}

/// One entry in the list of who is blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocked {
    /// The address or domain this block is about.
    pub what: Block,
    /// The rule to delete to undo it.
    pub rule_id: String,
    /// Where this block files the mail it catches.
    pub goes_to: String,
    /// Whether the rule is switched on.
    ///
    /// A rule can be switched off in the rule editor, and a block that is
    /// switched off catches nothing. A list that showed it as though it were
    /// working would be worse than no list.
    pub still_on: bool,
}

/// Everybody blocked on this account, in the order their rules are stored.
///
/// A block somebody cannot find is a trap. Mail stops arriving, nothing says
/// why, and the rule doing it is one row among however many rules they have.
pub fn everyone_blocked(account_id: &str, rules: &[MessageFilterRule]) -> Vec<Blocked> {
    rules
        .iter()
        .filter(|rule| rule.account_id == account_id)
        .filter_map(|rule| {
            Some(Blocked {
                what: the_block_in(rule)?,
                rule_id: rule.id.clone(),
                goes_to: rule.action_value.clone().unwrap_or_default(),
                still_on: rule.enabled,
            })
        })
        .collect()
}

/// The stored rule that is exactly this block, for undoing it.
///
/// Exactly this block, and never a wider one that happens to cover it.
/// Unblocking one address by deleting the domain block that catches it would
/// unblock everybody at that domain, which is not what was asked and cannot
/// be undone by asking again.
pub fn the_rule_that_blocks<'a>(
    account_id: &str,
    block: &Block,
    rules: &'a [MessageFilterRule],
) -> Option<&'a MessageFilterRule> {
    rules
        .iter()
        .filter(|rule| rule.account_id == account_id)
        .find(|rule| the_block_in(rule).as_ref() == Some(block))
}

/// The block a stored rule carries, when it carries one.
///
/// Both the name and the shape have to agree. The name is what makes a block
/// findable, and the shape is what stops an ordinary rule that happens to
/// file a sender into a folder from being read as a block and offered for
/// unblocking.
///
/// A block whose rule somebody has renamed by hand stops reading as a block.
/// That is the honest answer: they have turned it into an ordinary rule, and
/// it still does what it did.
fn the_block_in(rule: &MessageFilterRule) -> Option<Block> {
    if rule.field != "from" || rule.action_type != "move_to_folder" {
        return None;
    }
    let target = rule.name.strip_prefix(BLOCKED)?.trim();
    if target.is_empty() {
        return None;
    }
    Some(match target.strip_prefix(EVERYONE_AT) {
        Some(domain) => Block::EveryoneAt(domain.to_string()),
        None => Block::ThisAddress(target.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filters::FilterEngine;
    use crate::data::message_cache::CachedMessage;

    fn message_from(from: &str) -> CachedMessage {
        CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".into(),
            subject: "Anything".into(),
            from_addr: from.into(),
            to_addr: "me@example.com".into(),
            cc: None,
            date: "2026-08-24".into(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    /// Run a block's own rule through the filter engine, which is the code
    /// that will really decide.
    fn the_rule_fires_on(block: &Block, from: &str) -> bool {
        let stored = a_rule_that_blocks("acct", block, "Junk", "2026-08-24T00:00:00Z");
        let Some(rule) = FilterEngine::from_persisted_rule(&stored) else {
            panic!("a block did not read back as a rule at all");
        };
        FilterEngine::matches(&rule, &message_from(from))
    }

    #[test]
    fn test_a_block_reads_the_address_out_of_a_from_header() {
        // Somebody blocks from a message, and what the message holds is a
        // display name and an address together.
        let block = just_this_sender("Ada Lovelace <ada@example.com>").expect("an address");
        assert_eq!(block.as_written(), "ada@example.com");
    }

    #[test]
    fn test_a_block_takes_a_bare_address_too() {
        let block = just_this_sender("ada@example.com").expect("an address");
        assert_eq!(block.as_written(), "ada@example.com");
    }

    #[test]
    fn test_blocking_the_domain_keeps_only_the_domain() {
        let block = everyone_at_the_senders_domain("Ada <ada@example.com>").expect("a domain");
        assert_eq!(block.as_written(), "example.com");
    }

    #[test]
    fn test_nothing_that_is_not_an_address_can_be_blocked() {
        // A block built from nothing would become a rule matching everything,
        // which files the whole mailbox into Junk.
        for nonsense in ["", "   ", "Ada Lovelace", "<>", "@", "ada@"] {
            assert!(
                just_this_sender(nonsense).is_err(),
                "{nonsense:?} was accepted as an address to block"
            );
            assert!(
                everyone_at_the_senders_domain(nonsense).is_err(),
                "{nonsense:?} was accepted as a domain to block"
            );
        }
    }

    #[test]
    fn test_a_blocked_sender_is_matched_however_the_header_is_written() {
        // The stored `from` carries a display name when the sender gave one
        // and nothing but the address when they did not, so a block has to
        // match both. An "equals" rule on the address matches neither when a
        // name is present.
        let block = just_this_sender("ada@example.com").expect("an address");

        for from in [
            "ada@example.com",
            "Ada Lovelace <ada@example.com>",
            "ADA@EXAMPLE.COM",
            "\"Lovelace, Ada\" <Ada@Example.com>",
            "ada@example.com, someone@elsewhere.example",
        ] {
            assert!(
                the_rule_fires_on(&block, from),
                "blocking ada@example.com did not catch {from:?}"
            );
        }
    }

    #[test]
    fn test_a_blocked_address_does_not_catch_a_different_address_that_contains_it() {
        // The reason a block is a bounded pattern and not a "contains". Every
        // one of these is somebody else, and sending their mail to Junk is
        // the failure that costs more than not blocking at all.
        let block = just_this_sender("ada@example.com").expect("an address");

        for from in [
            "notada@example.com",
            "ada@example.com.evil.example",
            "ada@example.company",
            "ada@example.com.au",
        ] {
            assert!(
                !the_rule_fires_on(&block, from),
                "blocking ada@example.com wrongly caught {from:?}"
            );
        }
    }

    #[test]
    fn test_blocking_a_domain_catches_everyone_at_it_and_below_it() {
        // Below it as well, because one sender is one of many and the many
        // arrive from a new subdomain each week.
        let block = everyone_at_the_senders_domain("spam@example.com").expect("a domain");

        for from in [
            "spam@example.com",
            "Someone Else <someone@example.com>",
            "bounce@mail.example.com",
            "x@a.b.example.com",
        ] {
            assert!(
                the_rule_fires_on(&block, from),
                "blocking example.com did not catch {from:?}"
            );
        }
    }

    #[test]
    fn test_blocking_a_domain_does_not_catch_a_domain_that_merely_ends_the_same_way() {
        let block = everyone_at_the_senders_domain("spam@example.com").expect("a domain");

        for from in [
            "someone@notexample.com",
            "someone@example.com.evil.example",
            "someone@example.communications",
        ] {
            assert!(
                !the_rule_fires_on(&block, from),
                "blocking example.com wrongly caught {from:?}"
            );
        }
    }

    #[test]
    fn test_an_address_full_of_pattern_characters_is_still_matched_as_itself() {
        // An address may hold characters that mean something in a pattern.
        // Left as they are, "a.b+c@example.com" would match "axb+c@..." and
        // the block would be about somebody else as well.
        let block = just_this_sender("a.b+c$d@example.com").expect("an address");

        assert!(the_rule_fires_on(&block, "a.b+c$d@example.com"));
        assert!(!the_rule_fires_on(&block, "axbXcYd@example.com"));
    }

    #[test]
    fn test_a_block_is_an_ordinary_rule_the_engine_already_understands() {
        // The whole design: a block is a filter rule, so nothing new has to
        // run it, and somebody can see it in the rule list beside the rest.
        let block = just_this_sender("ada@example.com").expect("an address");
        let rule = a_rule_that_blocks("acct", &block, "Junk", "2026-08-24T00:00:00Z");

        assert_eq!(rule.field, "from");
        assert_eq!(rule.action_type, "move_to_folder");
        assert_eq!(rule.action_value.as_deref(), Some("Junk"));
        assert!(rule.enabled);
        assert_eq!(rule.account_id, "acct");
        assert!(
            FilterEngine::from_persisted_rule(&rule).is_some(),
            "the engine could not read a block back as a rule"
        );
    }

    #[test]
    fn test_two_blocks_on_one_account_are_two_rules_and_the_same_block_twice_is_one() {
        // The rules table keeps one row per id and refuses two rules with the
        // same name on one account, so a block made twice has to land on the
        // same row rather than failing to store.
        let ada = just_this_sender("ada@example.com").expect("an address");
        let bob = just_this_sender("bob@example.com").expect("an address");
        let made = |block: &Block, account: &str| a_rule_that_blocks(account, block, "Junk", "t");

        assert_eq!(made(&ada, "acct").id, made(&ada, "acct").id);
        assert_ne!(made(&ada, "acct").id, made(&bob, "acct").id);
        assert_ne!(made(&ada, "acct").name, made(&bob, "acct").name);
        // Two accounts can block the same person, and the ids are a primary
        // key across the whole table.
        assert_ne!(made(&ada, "acct").id, made(&ada, "other").id);
    }

    #[test]
    fn test_blocking_a_person_and_blocking_their_domain_are_two_different_blocks() {
        let one = just_this_sender("ada@example.com").expect("an address");
        let all = everyone_at_the_senders_domain("ada@example.com").expect("a domain");

        assert_ne!(
            a_rule_that_blocks("acct", &one, "Junk", "t").id,
            a_rule_that_blocks("acct", &all, "Junk", "t").id
        );
    }

    /// Blocking one address must not become blocking a whole domain because
    /// of one stray character, so the two are separate types rather than a
    /// flag, and this is the rule the engine really gets.
    #[test]
    fn test_blocking_one_person_leaves_the_rest_of_their_domain_alone() {
        let block = just_this_sender("ada@example.com").expect("an address");

        assert!(!the_rule_fires_on(&block, "someone-else@example.com"));
    }

    // ── Where blocked mail goes ─────────────────────────────────────────

    #[test]
    fn test_blocked_mail_goes_to_the_junk_folder() {
        let folders = [
            ("INBOX", FolderType::Inbox),
            ("Trash", FolderType::Trash),
            ("Junk E-mail", FolderType::Spam),
        ];

        assert_eq!(
            where_blocked_mail_goes(folders),
            BlockedMailGoesTo::TheJunkFolder("Junk E-mail")
        );
    }

    #[test]
    fn test_an_account_with_no_junk_folder_blocks_nobody_rather_than_guessing() {
        // Falling back to the trash would make a block delete mail, which is
        // not what the word means and is not recoverable in the same way.
        let folders = [("INBOX", FolderType::Inbox), ("Trash", FolderType::Trash)];

        assert_eq!(
            where_blocked_mail_goes(folders),
            BlockedMailGoesTo::NoJunkFolderFound
        );
    }

    #[test]
    fn test_an_account_that_has_never_synced_says_so_rather_than_saying_no_junk() {
        // Not knowing yet and knowing there is none are different, and what
        // to do about them is different.
        assert_eq!(
            where_blocked_mail_goes([]),
            BlockedMailGoesTo::NoFoldersKnownYet
        );
    }

    #[test]
    fn test_both_refusals_say_what_to_do_next() {
        for sentence in [NO_JUNK_FOLDER_FOUND, NO_FOLDERS_KNOWN_YET] {
            assert!(
                sentence.contains("Nothing has been blocked"),
                "a refusal did not say that nothing happened: {sentence}"
            );
            assert!(
                sentence.len() > 60,
                "a refusal did not say what to do next: {sentence}"
            );
        }
    }

    // ── What is asked before a block exists ─────────────────────────────

    fn mine(addresses: &[&str]) -> Vec<String> {
        addresses.iter().map(|a| a.to_string()).collect()
    }

    fn nothing_known(own: &[String]) -> WhatIsAlreadyTrue<'_> {
        WhatIsAlreadyTrue {
            their_own_addresses: own,
            rules_already_there: &[],
            how_to_leave_the_list: None,
            the_message_was_from: None,
        }
    }

    #[test]
    fn test_an_ordinary_sender_can_just_be_blocked() {
        let own = mine(&["me@work.example"]);
        let block = just_this_sender("spam@example.com").expect("an address");

        assert_eq!(
            may_block("acct", &block, &nothing_known(&own)),
            MayBlock::Yes
        );
    }

    #[test]
    fn test_blocking_somebody_already_blocked_says_so_rather_than_doing_it_twice() {
        // The rules table keeps one name per account, so a second attempt
        // would fail at the database with a message about a constraint.
        let own = mine(&["me@work.example"]);
        let block = just_this_sender("spam@example.com").expect("an address");
        let already = [a_rule_that_blocks("acct", &block, "Junk", "t")];

        let answer = may_block(
            "acct",
            &block,
            &WhatIsAlreadyTrue {
                rules_already_there: &already,
                ..nothing_known(&own)
            },
        );

        let MayBlock::No(sentence) = answer else {
            panic!("blocking somebody already blocked was not refused: {answer:?}");
        };
        assert!(sentence.contains("spam@example.com"), "{sentence}");
        assert!(sentence.contains("already"), "{sentence}");
    }

    #[test]
    fn test_blocking_somebody_whose_whole_domain_is_blocked_says_where_that_comes_from() {
        // Otherwise it looks as though the block did not take: the address
        // never appears in the list, because the domain covers it.
        let own = mine(&["me@work.example"]);
        let domain = everyone_at_the_senders_domain("spam@example.com").expect("a domain");
        let already = [a_rule_that_blocks("acct", &domain, "Junk", "t")];
        let one = just_this_sender("someone@example.com").expect("an address");

        let answer = may_block(
            "acct",
            &one,
            &WhatIsAlreadyTrue {
                rules_already_there: &already,
                ..nothing_known(&own)
            },
        );

        let MayBlock::No(sentence) = answer else {
            panic!("a sender already covered by a domain block was not recognised: {answer:?}");
        };
        assert!(
            sentence.contains("example.com"),
            "the sentence did not name the domain block: {sentence}"
        );
    }

    #[test]
    fn test_the_already_blocked_sentence_names_the_folder_that_block_really_uses() {
        // Not every account calls it Junk, and a sentence naming a folder
        // somebody does not have sends them looking for one that is not
        // there.
        let own = mine(&["me@work.example"]);
        let domain = everyone_at_the_senders_domain("spam@example.com").expect("a domain");
        let already = [a_rule_that_blocks("acct", &domain, "Junk E-mail", "t")];
        let one = just_this_sender("someone@example.com").expect("an address");

        let MayBlock::No(sentence) = may_block(
            "acct",
            &one,
            &WhatIsAlreadyTrue {
                rules_already_there: &already,
                ..nothing_known(&own)
            },
        ) else {
            panic!("a sender already covered by a domain block was not recognised");
        };

        assert!(
            sentence.contains("Junk E-mail"),
            "the sentence did not name the folder the block files into: {sentence}"
        );
    }

    #[test]
    fn test_blocking_a_domain_is_still_allowed_when_one_address_at_it_is_blocked() {
        // Widening a block is a real thing to ask for, and refusing it would
        // leave somebody with no way to widen one but to find and undo the
        // narrow block first.
        let own = mine(&["me@work.example"]);
        let one = just_this_sender("spam@example.com").expect("an address");
        let already = [a_rule_that_blocks("acct", &one, "Junk", "t")];
        let domain = everyone_at_the_senders_domain("spam@example.com").expect("a domain");

        assert_eq!(
            may_block(
                "acct",
                &domain,
                &WhatIsAlreadyTrue {
                    rules_already_there: &already,
                    ..nothing_known(&own)
                }
            ),
            MayBlock::Yes
        );
    }

    #[test]
    fn test_somebody_cannot_block_themselves() {
        // Their own mail comes back to them: a copy of what they sent, a
        // message they sent to a list they are on, a note they wrote to
        // themselves. All of it would go to Junk.
        let own = mine(&["me@work.example", "old-me@work.example"]);

        for theirs in [
            "me@work.example",
            "Me <ME@Work.Example>",
            "old-me@work.example",
        ] {
            let block = just_this_sender(theirs).expect("an address");
            let answer = may_block("acct", &block, &nothing_known(&own));
            let MayBlock::No(sentence) = answer else {
                panic!("blocking {theirs} was allowed: {answer:?}");
            };
            assert!(
                sentence.contains("your own"),
                "the sentence did not say whose address it is: {sentence}"
            );
        }
    }

    #[test]
    fn test_somebody_cannot_block_the_domain_they_receive_mail_at() {
        // At work this is everybody they work with, plus themselves. It reads
        // as one click and it is the whole company.
        let own = mine(&["me@work.example"]);
        let block = everyone_at_the_senders_domain("annoying@work.example").expect("a domain");

        let answer = may_block(
            "acct",
            &block,
            &WhatIsAlreadyTrue {
                the_message_was_from: Some("annoying@work.example"),
                ..nothing_known(&own)
            },
        );

        let MayBlock::No(sentence) = answer else {
            panic!("blocking their own domain was allowed: {answer:?}");
        };
        assert!(
            sentence.contains("annoying@work.example"),
            "the sentence did not offer blocking the one sender instead: {sentence}"
        );
    }

    #[test]
    fn test_the_domain_refusal_still_says_what_to_do_when_no_sender_is_known() {
        // Blocking a domain typed by hand rather than clicked from a message.
        let own = mine(&["me@work.example"]);
        let block = everyone_at_the_senders_domain("annoying@work.example").expect("a domain");

        let MayBlock::No(sentence) = may_block("acct", &block, &nothing_known(&own)) else {
            panic!("blocking their own domain was allowed");
        };
        assert!(
            sentence.contains("block that address on its own"),
            "the sentence did not say what to do instead: {sentence}"
        );
    }

    #[test]
    fn test_blocking_a_subdomain_of_their_own_domain_is_refused_too() {
        // A block on `work.example` catches `mail.work.example`, so the same
        // check has to read the same way round.
        let own = mine(&["me@mail.work.example"]);
        let block = everyone_at_the_senders_domain("annoying@work.example").expect("a domain");

        assert!(matches!(
            may_block("acct", &block, &nothing_known(&own)),
            MayBlock::No(_)
        ));
    }

    #[test]
    fn test_blocking_a_mailing_list_warns_that_it_keeps_arriving() {
        // Blocking a list keeps it out of the inbox and does nothing about
        // the list itself, which goes on sending. Leaving is the real answer,
        // and the message says where to write to do it.
        let own = mine(&["me@work.example"]);
        let block = just_this_sender("birds@lists.example").expect("an address");

        let answer = may_block(
            "acct",
            &block,
            &WhatIsAlreadyTrue {
                how_to_leave_the_list: Some("<mailto:birds-leave@lists.example>"),
                ..nothing_known(&own)
            },
        );

        let MayBlock::YesButFirst(sentence) = answer else {
            panic!("blocking a mailing list gave no warning: {answer:?}");
        };
        assert!(sentence.contains("birds-leave@lists.example"), "{sentence}");
        assert!(
            sentence.to_lowercase().contains("unsubscribe")
                || sentence.to_lowercase().contains("leave"),
            "the sentence did not say what to do instead: {sentence}"
        );
    }

    #[test]
    fn test_a_mailing_list_with_no_way_out_still_warns() {
        // Some lists give no address to write to. The warning is still worth
        // having, because the mail keeps coming either way.
        let own = mine(&["me@work.example"]);
        let block = just_this_sender("birds@lists.example").expect("an address");

        assert!(matches!(
            may_block(
                "acct",
                &block,
                &WhatIsAlreadyTrue {
                    how_to_leave_the_list: Some("   "),
                    ..nothing_known(&own)
                }
            ),
            MayBlock::YesButFirst(_)
        ));
    }

    #[test]
    fn test_a_rule_from_another_account_does_not_count_as_a_block_here() {
        // Rules are per account, and reading another account's blocks would
        // say somebody is already blocked when they are not.
        let own = mine(&["me@work.example"]);
        let block = just_this_sender("spam@example.com").expect("an address");
        let elsewhere = [a_rule_that_blocks("other", &block, "Junk", "t")];

        assert_eq!(
            may_block(
                "acct",
                &block,
                &WhatIsAlreadyTrue {
                    rules_already_there: &elsewhere,
                    ..nothing_known(&own)
                }
            ),
            MayBlock::Yes
        );
    }

    // ── Finding a block again, and undoing it ───────────────────────────

    #[test]
    fn test_every_block_can_be_listed() {
        // A block nobody can find is a trap: mail goes missing and there is
        // nothing to look at that says why.
        let ada = just_this_sender("ada@example.com").expect("an address");
        let noisy = everyone_at_the_senders_domain("x@noisy.example").expect("a domain");
        let rules = [
            a_rule_that_blocks("acct", &ada, "Junk", "t"),
            a_rule_that_blocks("acct", &noisy, "Junk", "t"),
        ];

        let listed = everyone_blocked("acct", &rules);

        assert_eq!(
            listed.iter().map(|b| b.what.clone()).collect::<Vec<_>>(),
            vec![ada, noisy]
        );
    }

    #[test]
    fn test_the_list_says_where_the_mail_goes_and_whether_the_block_is_on() {
        // A rule can be switched off in the rule editor, and a block that is
        // switched off is a block that does nothing. Saying so is the whole
        // difference between a list and a list somebody can trust.
        let ada = just_this_sender("ada@example.com").expect("an address");
        let mut off = a_rule_that_blocks("acct", &ada, "Junk E-mail", "t");
        off.enabled = false;

        let listed = everyone_blocked("acct", &[off]);

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].goes_to, "Junk E-mail");
        assert!(!listed[0].still_on);
    }

    #[test]
    fn test_the_list_holds_only_this_accounts_blocks_and_only_blocks() {
        let ada = just_this_sender("ada@example.com").expect("an address");
        let ordinary = MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "Newsletters".into(),
            field: "from".into(),
            match_type: "contains".into(),
            pattern: "news@example.com".into(),
            case_sensitive: false,
            action_type: "move_to_folder".into(),
            action_value: Some("Reading".into()),
            enabled: true,
            created_at: "t".into(),
        };
        let rules = [
            a_rule_that_blocks("other", &ada, "Junk", "t"),
            ordinary,
            a_rule_that_blocks("acct", &ada, "Junk", "t"),
        ];

        let listed = everyone_blocked("acct", &rules);

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].what, ada);
    }

    #[test]
    fn test_unblocking_names_the_rule_to_take_away() {
        // Undoing a block is deleting its rule, so what unblocking has to
        // produce is the identifier of the row to delete.
        let ada = just_this_sender("ada@example.com").expect("an address");
        let rule = a_rule_that_blocks("acct", &ada, "Junk", "t");
        let rules = [rule.clone()];

        assert_eq!(
            the_rule_that_blocks("acct", &ada, &rules).map(|found| found.id.as_str()),
            Some(rule.id.as_str())
        );
    }

    #[test]
    fn test_unblocking_somebody_who_is_not_blocked_finds_nothing() {
        let ada = just_this_sender("ada@example.com").expect("an address");
        let bob = just_this_sender("bob@example.com").expect("an address");
        let rules = [a_rule_that_blocks("acct", &bob, "Junk", "t")];

        assert!(the_rule_that_blocks("acct", &ada, &rules).is_none());
    }

    #[test]
    fn test_unblocking_an_address_does_not_reach_for_the_domain_block_that_covers_it() {
        // Deleting the domain block would unblock everybody at it, which is
        // not what was asked and is not recoverable by repeating the action.
        let domain = everyone_at_the_senders_domain("x@example.com").expect("a domain");
        let one = just_this_sender("ada@example.com").expect("an address");
        let rules = [a_rule_that_blocks("acct", &domain, "Junk", "t")];

        assert!(the_rule_that_blocks("acct", &one, &rules).is_none());
    }

    // ── What is said before and after ───────────────────────────────────

    /// An account whose junk folder is already downloaded, which is the case
    /// where blocking has nothing to say about the folder at all.
    const ALREADY_THERE: TheJunkFolder = TheJunkFolder::AlreadyKeptUpToDate;

    #[test]
    fn test_blocking_says_that_it_switched_the_junk_folder_on() {
        // Switching something on for somebody without saying so is a change
        // they find out about when their folder list has grown and mail they
        // did not ask for is in it. Said in the same breath as the block,
        // with what it is for and where to undo it.
        let block = just_this_sender("spam@example.com").expect("an address");
        let sentence = what_blocking_did(
            &block,
            "Junk",
            Allowed::EVERYTHING,
            TheJunkFolder::IsSwitchedOnByBlocking,
        );

        assert!(
            sentence.contains("switched on"),
            "the sentence did not say the folder was switched on: {sentence}"
        );
        assert!(
            sentence.contains("Folders to Keep Up to Date"),
            "the sentence did not say where to undo it: {sentence}"
        );
    }

    #[test]
    fn test_blocking_says_when_the_junk_folder_is_not_being_downloaded() {
        // Somebody who switched it off keeps their choice and is told what it
        // now costs: blocked mail is filed at the server and never arrives
        // here, so it cannot be read or got back in this program.
        let block = just_this_sender("spam@example.com").expect("an address");
        let sentence = what_blocking_did(
            &block,
            "Junk",
            Allowed::EVERYTHING,
            TheJunkFolder::IsNotBeingDownloaded,
        );

        assert!(
            sentence.contains("not being downloaded"),
            "the sentence did not say the folder is not downloaded: {sentence}"
        );
        assert!(
            sentence.contains("Folders to Keep Up to Date"),
            "the sentence did not say how to turn it on: {sentence}"
        );
    }

    #[test]
    fn test_a_junk_folder_already_being_downloaded_is_not_mentioned_at_all() {
        // A line that says the nothings teaches somebody to stop listening to
        // the one that matters.
        let block = just_this_sender("spam@example.com").expect("an address");
        let sentence = what_blocking_did(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE);

        assert!(
            !sentence.contains("Folders to Keep Up to Date"),
            "somebody was told to go and change a setting for no reason: {sentence}"
        );
    }

    #[test]
    fn test_what_is_said_before_and_after_both_say_what_blocking_does_not_do() {
        // Two things somebody will otherwise assume, and be wrong about for
        // as long as it takes them to notice. Nothing is reported to the mail
        // provider, and nothing already in the mailbox moves.
        let block = just_this_sender("spam@example.com").expect("an address");

        for sentence in [
            what_blocking_will_do(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE),
            what_blocking_did(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE),
        ] {
            assert!(sentence.contains("spam@example.com"), "{sentence}");
            assert!(sentence.contains("Junk"), "{sentence}");
            assert!(
                sentence.contains("provider"),
                "the sentence did not say that the provider is not told: {sentence}"
            );
            assert!(
                sentence.contains("already"),
                "the sentence did not say what happens to mail already here: {sentence}"
            );
        }
    }

    #[test]
    fn test_the_two_sentences_are_written_for_before_and_for_after() {
        // The same facts, and not the same words: one is about what will
        // happen and one is about what has happened.
        let block = just_this_sender("spam@example.com").expect("an address");
        let before = what_blocking_will_do(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE);
        let after = what_blocking_did(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE);

        assert_ne!(before, after);
        assert!(before.contains("will go"), "{before}");
        assert!(after.contains("now goes"), "{after}");
    }

    #[test]
    fn test_a_domain_block_says_it_covers_everyone_there() {
        let block = everyone_at_the_senders_domain("spam@example.com").expect("a domain");
        let sentence = what_blocking_will_do(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE);

        assert!(sentence.contains("everyone"), "{sentence}");
        assert!(sentence.contains("example.com"), "{sentence}");
    }

    #[test]
    fn test_blocking_says_so_when_mail_changes_are_switched_off() {
        // A block files mail into a folder on the server, and that is behind
        // the same permission as every other change to somebody's mail. With
        // it off, the rule is stored and never carried out, and saying
        // nothing would leave somebody believing the block works.
        let block = just_this_sender("spam@example.com").expect("an address");

        for sentence in [
            what_blocking_will_do(&block, "Junk", Allowed::NOTHING, ALREADY_THERE),
            what_blocking_did(&block, "Junk", Allowed::NOTHING, ALREADY_THERE),
        ] {
            assert!(
                sentence.contains("Allowed Changes"),
                "the sentence did not name the setting that is holding it back: {sentence}"
            );
        }
        assert!(
            !what_blocking_will_do(&block, "Junk", Allowed::EVERYTHING, ALREADY_THERE)
                .contains("Allowed Changes"),
            "the warning was shown when nothing is holding the block back"
        );
    }

    // ── The folder blocked mail is filed into ───────────────────────────

    #[test]
    fn test_blocking_switches_the_junk_folder_on_when_nobody_has_said_either_way() {
        // The whole recovery route this feature promises. A server account
        // does not download its junk folder unless somebody says so, and the
        // folder tree leaves out what is not downloaded, so blocked mail went
        // to a folder that was neither filled nor listed. A block on a whole
        // domain catches a colleague sooner or later, and that is the case
        // where the person goes looking in Junk and finds nothing there.
        assert_eq!(
            what_the_junk_folder_needs("INBOX/Junk", None),
            TheJunkFolder::IsSwitchedOnByBlocking
        );
    }

    #[test]
    fn test_somebody_who_switched_the_junk_folder_off_is_not_overruled() {
        // "Never asked" and "asked and said no" look the same as a `false`
        // and mean opposite things, which is why the answer is an `Option`.
        // Blocking a sender is not a reason to undo a choice somebody made in
        // Folders to Keep Up to Date; it is a reason to say what that choice
        // now costs them.
        assert_eq!(
            what_the_junk_folder_needs("INBOX/Junk", Some(false)),
            TheJunkFolder::IsNotBeingDownloaded
        );
    }

    #[test]
    fn test_a_junk_folder_already_being_downloaded_needs_nothing() {
        assert_eq!(
            what_the_junk_folder_needs("INBOX/Junk", Some(true)),
            TheJunkFolder::AlreadyKeptUpToDate
        );
    }

    #[test]
    fn test_a_junk_folder_on_this_computer_needs_nothing_switching_on() {
        // A POP account keeps its junk folder here, and the choice only
        // decides what is downloaded from a server. Reading it as "never
        // asked" would switch on a folder that is always there anyway and
        // say so for no reason.
        //
        // The path comes from the module that makes it rather than being
        // typed out here: it starts with a character no mailbox name carries,
        // and a hand-written copy of it would be an ordinary path that this
        // test would then pass against for the wrong reason.
        let junk_here =
            crate::application::local_folders::for_account(crate::common::types::Protocol::Pop3)
                .iter()
                .find(|folder| folder.kind == FolderType::Spam)
                .expect("a POP account keeps a junk folder on this computer")
                .path();

        assert_eq!(
            what_the_junk_folder_needs(&junk_here, None),
            TheJunkFolder::AlreadyKeptUpToDate
        );
    }

    #[test]
    fn test_unblocking_says_what_happens_to_the_mail_already_filed() {
        // Mail already in Junk stays in Junk. Somebody expecting it to come
        // back to the inbox goes looking in the wrong place.
        let block = just_this_sender("spam@example.com").expect("an address");
        let sentence = what_unblocking_did(&block);

        assert!(sentence.contains("spam@example.com"), "{sentence}");
        assert!(sentence.contains("already"), "{sentence}");
    }

    #[test]
    fn test_no_sentence_in_this_module_uses_a_dash_where_a_comma_would_do() {
        // House style, checked rather than remembered.
        let one = just_this_sender("spam@example.com").expect("an address");
        let all = everyone_at_the_senders_domain("spam@example.com").expect("a domain");
        let mut every_sentence = vec![
            NO_JUNK_FOLDER_FOUND.to_string(),
            NO_FOLDERS_KNOWN_YET.to_string(),
            what_unblocking_did(&one),
        ];
        for block in [&one, &all] {
            for allowed in [Allowed::EVERYTHING, Allowed::NOTHING] {
                for junk in [
                    TheJunkFolder::AlreadyKeptUpToDate,
                    TheJunkFolder::IsSwitchedOnByBlocking,
                    TheJunkFolder::IsNotBeingDownloaded,
                ] {
                    every_sentence.push(what_blocking_will_do(block, "Junk", allowed, junk));
                    every_sentence.push(what_blocking_did(block, "Junk", allowed, junk));
                }
            }
        }

        for sentence in every_sentence {
            assert!(
                !sentence.contains('\u{2014}') && !sentence.contains('\u{2013}'),
                "a sentence uses a dash: {sentence}"
            );
        }
    }

    #[test]
    fn test_a_block_made_and_then_listed_reads_back_as_the_same_block() {
        // The round trip, which is what makes a block undoable at all: the
        // name written when it is made is the name read when it is listed.
        for written in ["ada@example.com", "a.b+c@sub.example.co.uk"] {
            let one = just_this_sender(written).expect("an address");
            let all = everyone_at_the_senders_domain(written).expect("a domain");
            let rules = [
                a_rule_that_blocks("acct", &one, "Junk", "t"),
                a_rule_that_blocks("acct", &all, "Junk", "t"),
            ];

            let listed = everyone_blocked("acct", &rules);

            assert_eq!(listed.len(), 2, "{written} did not read back");
            assert_eq!(listed[0].what, one);
            assert_eq!(listed[1].what, all);
        }
    }

    #[test]
    fn test_an_ordinary_rule_that_happens_to_file_a_sender_is_not_read_as_a_block() {
        // Somebody who files a newsletter into a folder has not blocked it,
        // and reading their rule as a block would offer to unblock a sender
        // they never blocked.
        let own = mine(&["me@work.example"]);
        let filing = MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "Newsletters".into(),
            field: "from".into(),
            match_type: "contains".into(),
            pattern: "news@example.com".into(),
            case_sensitive: false,
            action_type: "move_to_folder".into(),
            action_value: Some("Reading".into()),
            enabled: true,
            created_at: "t".into(),
        };
        let block = just_this_sender("news@example.com").expect("an address");

        assert_eq!(
            may_block(
                "acct",
                &block,
                &WhatIsAlreadyTrue {
                    rules_already_there: &[filing],
                    ..nothing_known(&own)
                }
            ),
            MayBlock::Yes
        );
    }
}
