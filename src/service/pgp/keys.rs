//! The OpenPGP crate, and the only place in this program that names it.
//!
//! Everything outside `src/service/pgp/` calls [`super`]'s functions and knows
//! no crate name, no packet, no armour header. The reason is not tidiness: a
//! cryptographic implementation is the one dependency here that may have to be
//! replaced at short notice, and a replacement that reaches every caller is one
//! nobody makes in a hurry.
//!
//! So the crate's own types stop here. What crosses out of this file is
//! [`WhatOpeningItFound`], [`WhatImportingAKeyFound`], [`WhatBecameOfAKey`]
//! and [`KeyListing`], which are this project's words.
//!
//! # Nothing here is logged
//!
//! Not the key, not the message, and not the crate's own error text. A private
//! key is the highest-value secret this program holds, a message body is
//! private mail from a stranger, and a decryption library's error text is
//! written for somebody reading a stack trace and can quote the bytes it choked
//! on. Those bytes are somebody's mail. `unwrap` and `expect` are out for the
//! same reason: a panic message is a log line nobody wrote.
//!
//! # What this build reads and what it does not
//!
//! Armour, and nothing that knows about MIME. Inline PGP is an armoured block
//! in the message's text, which `application::body_safety::what_the_form_says`
//! finds; PGP/MIME's armour is a part of its own, which
//! `application::opening_pgp::for_pgp_mime` takes out and hands here.
//!
//! Up to [`super::KEY_SLOTS`] private keys, kept in the credential store in
//! parts because Windows keeps 1,280 characters in one entry. A key locked
//! with a passphrase is kept as it came, still locked (13-17.1): the
//! passphrase is asked for by a reader window when a message first needs it,
//! and [`unlock`] holds it in memory until the program closes. Other people's
//! public keys are described and handed back here and kept by the caller,
//! since they are not secret.
//!
//! # A typed passphrase
//!
//! Held as the crate's own passphrase type, which overwrites its bytes when it
//! is dropped, in one map in this file, by fingerprint. Nothing hands it back
//! out, nothing writes it anywhere and nothing logs it. It goes when the
//! program closes, or when its key is removed.

use super::{
    KEY_SLOTS, KEYRING_PRIVATE_KEY, KEYRING_SERVICE, KeyListing, LockedKey, PARTS_PER_KEY,
    Unlocking, WhatBecameOfAKey, WhatImportingAKeyFound, WhatOpeningItFound, the_entry_for,
};
use crate::service::secret_store::{self, LONGEST_SECRET_ONE_ENTRY_HOLDS};
use pgp::composed::{
    ArmorOptions, Deserializable, Esk, Message, PublicOrSecret, SignedPublicKey, SignedSecretKey,
};
use pgp::errors::Error as OpenPgpError;
use pgp::packet::{KeyFlags, Signature, UserId};
use pgp::types::{KeyDetails, Password, SignedUser};
use std::collections::HashMap;
use std::io::Read;

/// The most of one decrypted message this will hold in memory.
///
/// Twenty-five megabytes, the same as the ceilings on a stored attachment and
/// on a kept signed message, so the three limits on message content are one
/// number. There is no measurement behind the exact figure and saying so is
/// more use than a justification that sounds like one.
///
/// It is here rather than left to the crate because a compressed OpenPGP
/// message can name a very large plaintext in very few bytes, and a stranger
/// chooses those bytes. rPGP's own default buffer is a gigabyte. A message over
/// this reads as damaged, which is honest: nothing of it was shown.
const MOST_A_MESSAGE_MAY_COST: u64 = 25 * 1024 * 1024;

/// Take an armoured private key and put it in the credential store: the File
/// menu's import, which speaks of one key.
///
/// Answered from [`import_keys`], by the first private key the text holds,
/// because that is the one it stored; a public key is the answer only when the
/// text holds nothing else. A key already here reads as imported, which is
/// what importing it again did before several keys were kept.
pub(super) fn import(armoured: &str) -> WhatImportingAKeyFound {
    let answers = import_keys(armoured);
    let about_a_private_key = answers
        .iter()
        .find(|answer| !matches!(answer, WhatBecameOfAKey::PublicKeyToKeep { .. }));
    match (about_a_private_key, answers.first()) {
        (Some(WhatBecameOfAKey::Imported(_) | WhatBecameOfAKey::AlreadyHere(_)), _) => {
            WhatImportingAKeyFound::Imported
        }
        (Some(WhatBecameOfAKey::CouldNotBeStored { reason, .. }), _) => {
            WhatImportingAKeyFound::CouldNotBeStored {
                reason: reason.clone(),
            }
        }
        (_, Some(_)) => WhatImportingAKeyFound::NotAPrivateKey,
        (_, None) => WhatImportingAKeyFound::NotAKey,
    }
}

/// Import every key an armoured text holds, answering for each.
pub(super) fn import_keys(armoured: &str) -> Vec<WhatBecameOfAKey> {
    keys_in(armoured)
        .into_iter()
        .map(|(key, armour)| what_becomes_of(key, &armour))
        .collect()
}

fn what_becomes_of(key: PublicOrSecret, armour: &str) -> WhatBecameOfAKey {
    let secret = match key {
        PublicOrSecret::Public(public) => {
            return WhatBecameOfAKey::PublicKeyToKeep {
                armour: armour.to_string(),
                listing: listing_of(&public, false),
            };
        }
        PublicOrSecret::Secret(secret) => secret,
    };
    let listing = listing_of_a_private_key(&secret);
    match slot_holding(&listing.fingerprint) {
        Ok(Some(_)) => return WhatBecameOfAKey::AlreadyHere(listing),
        Ok(None) => {}
        Err(problem) => {
            return WhatBecameOfAKey::CouldNotBeStored {
                listing,
                reason: problem.to_string(),
            };
        }
    }
    match keep(armour) {
        Ok(()) => WhatBecameOfAKey::Imported(listing),
        // The store's reason, which is about the store. Nothing from the file
        // travels in it; `secret_store` already holds itself to reasons and
        // never values.
        Err(not_kept) => WhatBecameOfAKey::CouldNotBeStored {
            listing,
            reason: not_kept.reason(),
        },
    }
}

/// Every key an armoured text holds, each with the armour it is kept as.
///
/// Which kind each key is, private or public, is the crate's answer rather
/// than a reading of the armour header: a header this program read for itself
/// would be a second opinion about what a key file is, and the crate is the one
/// that has to agree with it later. What is read here is only where one armoured
/// block ends and the next begins, because a file of several keys exported one
/// after another is several blocks and the crate reads one.
///
/// A block holding one key is kept as it arrived, byte for byte, rather than as
/// the crate's own re-serialisation of it. The same reasoning `signed_original`
/// gives about a signed message: anything that rewrites a cryptographic
/// document, even to tidy it, is a second chance to change what it says. A block
/// holding several has no one key's bytes to keep, so each is written out by the
/// crate.
fn keys_in(text: &str) -> Vec<(PublicOrSecret, String)> {
    let mut keys = Vec::new();
    for block in armoured_blocks(text) {
        let Ok((found, _)) = PublicOrSecret::from_armor_many(block.as_bytes()) else {
            continue;
        };
        let found: Vec<PublicOrSecret> = found.filter_map(Result::ok).collect();
        if let [_] = found.as_slice() {
            keys.extend(found.into_iter().map(|key| (key, block.to_string())));
            continue;
        }
        for key in found {
            if let Ok(armour) = key.to_armored_string(ArmorOptions::default()) {
                keys.push((key, armour));
            }
        }
    }
    keys
}

/// Each `-----BEGIN PGP ...` to the end of its `-----END PGP ...` line.
fn armoured_blocks(text: &str) -> Vec<&str> {
    const BEGINS: &str = "-----BEGIN PGP ";
    const ENDS: &str = "-----END PGP ";
    let mut blocks = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(BEGINS) {
        let from = &rest[start..];
        let Some(end) = from.find(ENDS) else {
            blocks.push(from);
            break;
        };
        let end_line = &from[end..];
        let through = end + end_line.find('\n').map_or(end_line.len(), |line| line + 1);
        blocks.push(&from[..through]);
        rest = &from[through..];
    }
    blocks
}

/// A key in this program's words.
fn listing_of(key: &SignedPublicKey, private: bool) -> KeyListing {
    let mut users: Vec<&SignedUser> = key.details.users.iter().collect();
    users.sort_by_key(|user| !user.is_primary());
    let primary_self_signature = users
        .first()
        .and_then(|user| user.signatures.first())
        .or(key.details.direct_signatures.first());
    let subkey_bindings = key
        .public_subkeys
        .iter()
        .filter_map(|subkey| subkey.signatures.first());
    let flags: Vec<KeyFlags> = primary_self_signature
        .into_iter()
        .chain(subkey_bindings)
        .map(Signature::key_flags)
        .collect();
    let created = in_utc(key.created_at().as_secs());
    KeyListing {
        user_ids: users.iter().map(|user| user_id(&user.id)).collect(),
        key_id: key.legacy_key_id().to_string().to_uppercase(),
        fingerprint: format!("{:X}", key.fingerprint()),
        created,
        // A key expiration time of zero means it never expires, RFC 9580
        // 5.2.3.13.
        expires: primary_self_signature
            .and_then(Signature::key_expiration_time)
            .filter(|lasts| lasts.as_secs() > 0)
            .map(|lasts| created + chrono::Duration::seconds(i64::from(lasts.as_secs()))),
        private,
        locked: false,
        can_encrypt: flags
            .iter()
            .any(|flags| flags.encrypt_comms() || flags.encrypt_storage()),
        can_sign: flags.iter().any(KeyFlags::sign),
    }
}

/// A private key in this program's words, saying whether it is locked.
fn listing_of_a_private_key(key: &SignedSecretKey) -> KeyListing {
    KeyListing {
        locked: a_passphrase_is_holding_it_shut(key),
        ..listing_of(&key.to_public_key(), true)
    }
}

fn user_id(id: &UserId) -> String {
    id.as_str().map_or_else(
        || String::from_utf8_lossy(id.id()).into_owned(),
        str::to_string,
    )
}

fn in_utc(seconds: u32) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(i64::from(seconds), 0).unwrap_or_default()
}

/// Why a key was not kept.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NotKept {
    /// Longer than the parts one key may use hold.
    TooLarge,
    /// Every slot already holds a key.
    NoRoomLeft,
    /// The credential store said no, and this is its reason.
    Refused(String),
}

impl NotKept {
    /// Why, in words that finish "Your key could not be saved: ...".
    fn reason(&self) -> String {
        match self {
            NotKept::TooLarge => format!(
                "it is longer than the {} characters Wixen Mail can keep for one key",
                with_commas(PARTS_PER_KEY * LONGEST_SECRET_ONE_ENTRY_HOLDS)
            ),
            NotKept::NoRoomLeft => {
                format!("Wixen Mail keeps {KEY_SLOTS} private keys and already holds {KEY_SLOTS}")
            }
            NotKept::Refused(reason) => reason.clone(),
        }
    }
}

/// A count with a thousands separator, so "10,240" is heard as one number.
fn with_commas(count: usize) -> String {
    let digits = count.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (from_the_end, digit) in digits.chars().rev().enumerate() {
        if from_the_end > 0 && from_the_end % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped.chars().rev().collect()
}

/// Put a private key's armour in the credential store, in the first free slot.
///
/// Split into parts Windows will keep, because one entry holds 1,280
/// characters and an ordinary RSA key is longer. Byte for byte: the parts
/// joined in order are the armour that arrived.
fn keep(armoured: &str) -> Result<(), NotKept> {
    move_the_old_entry_into_a_slot();
    keep_in_a_free_slot(armoured)
}

fn keep_in_a_free_slot(armoured: &str) -> Result<(), NotKept> {
    let parts = in_parts(armoured);
    if parts.len() > PARTS_PER_KEY {
        return Err(NotKept::TooLarge);
    }
    let slot = first_free_slot()?.ok_or(NotKept::NoRoomLeft)?;
    // Parts past this key's last first, so a removal the store refuses leaves
    // nothing half written, and a slot left with a longer key's tail never
    // reads back with that tail joined to this key.
    for stale in parts.len() + 1..=PARTS_PER_KEY {
        secret_store::remove(KEYRING_SERVICE, &the_entry_for(slot, stale)).map_err(refused)?;
    }
    for (index, part) in parts.iter().enumerate() {
        if let Err(problem) =
            secret_store::write(KEYRING_SERVICE, &the_entry_for(slot, index + 1), part)
        {
            forget_the_slot(slot);
            return Err(refused(problem));
        }
    }
    Ok(())
}

fn refused(problem: crate::common::Error) -> NotKept {
    NotKept::Refused(problem.to_string())
}

/// `text` in pieces of at most one entry's worth of UTF-16, cut only between
/// characters. Never empty: an empty text is one empty part.
fn in_parts(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut start, mut units) = (0, 0);
    for (at, character) in text.char_indices() {
        if units + character.len_utf16() > LONGEST_SECRET_ONE_ENTRY_HOLDS {
            parts.push(&text[start..at]);
            (start, units) = (at, 0);
        }
        units += character.len_utf16();
    }
    parts.push(&text[start..]);
    parts
}

/// The first slot whose first part is not there.
fn first_free_slot() -> Result<Option<usize>, NotKept> {
    for slot in 1..=KEY_SLOTS {
        if secret_store::read(KEYRING_SERVICE, &the_entry_for(slot, 1))
            .map_err(refused)?
            .is_none()
        {
            return Ok(Some(slot));
        }
    }
    Ok(None)
}

/// Remove every part of a slot, stopping at the first the store refuses.
///
/// The first part first, because a slot whose first part is gone reads as free,
/// and a later write there clears whatever parts the store refused to let go.
fn remove_the_slot(slot: usize) -> crate::common::Result<()> {
    for part in 1..=PARTS_PER_KEY {
        secret_store::remove(KEYRING_SERVICE, &the_entry_for(slot, part))?;
    }
    Ok(())
}

/// The same, taking back a write that failed partway, where the failure that
/// matters is the write's and this one is only said in the log.
fn forget_the_slot(slot: usize) {
    if let Err(problem) = remove_the_slot(slot) {
        tracing::warn!("A part of a private key could not be removed: {problem}");
    }
}

/// Every private key's armour in the credential store, in slot order.
fn keys_here() -> crate::common::Result<Vec<String>> {
    move_the_old_entry_into_a_slot();
    let mut keys = Vec::new();
    for slot in 1..=KEY_SLOTS {
        if let Some(key) = the_key_in(slot)? {
            keys.push(key);
        }
    }
    Ok(keys)
}

/// A slot's parts joined in order, up to the first that is not there.
fn the_key_in(slot: usize) -> crate::common::Result<Option<String>> {
    let mut joined: Option<String> = None;
    for part in 1..=PARTS_PER_KEY {
        let Some(piece) = secret_store::read(KEYRING_SERVICE, &the_entry_for(slot, part))? else {
            break;
        };
        joined.get_or_insert_with(String::new).push_str(&piece);
    }
    Ok(joined)
}

/// Move the one entry a build before 13-16 kept a key under into a slot.
///
/// On Windows that entry can only hold a key of 1,280 characters or fewer, so
/// it fits in one part. It is removed only once the slot holds it, and the
/// uninstaller names `private-key` for ever, so a move that fails leaves a key
/// that is still erased when the program goes.
fn move_the_old_entry_into_a_slot() {
    let old = match secret_store::read(KEYRING_SERVICE, KEYRING_PRIVATE_KEY) {
        Ok(Some(old)) => old,
        Ok(None) => return,
        Err(problem) => {
            tracing::warn!(
                "The credential store would not say whether an older key is here: {problem}"
            );
            return;
        }
    };
    let moved = keep_in_a_free_slot(&old)
        .and_then(|()| secret_store::remove(KEYRING_SERVICE, KEYRING_PRIVATE_KEY).map_err(refused));
    if let Err(not_kept) = moved {
        tracing::warn!(
            "The private key an older build kept could not be moved: {}",
            not_kept.reason()
        );
    }
}

/// Whether any part of a key that could decrypt is locked with a passphrase.
///
/// The primary key and every secret subkey, because a message is encrypted to
/// an encryption subkey where a key has one and to the primary where it does
/// not. Asking only the primary would accept a key whose encryption half is
/// locked, which opens nothing and reports the wrong reason for ever after.
fn a_passphrase_is_holding_it_shut(key: &SignedSecretKey) -> bool {
    key.primary_key.secret_params().is_encrypted()
        || key
            .secret_subkeys
            .iter()
            .any(|subkey| subkey.secret_params().is_encrypted())
}

/// Open an armoured message with the private keys this computer holds.
///
/// Every key here is offered, and the crate takes the one the message names:
/// each encrypted session key in a message says which key it was encrypted to,
/// so a message is opened by its own key whichever slot that is in.
pub(super) fn open(armour: &str) -> WhatOpeningItFound {
    let stored = match keys_here() {
        Ok(stored) if stored.is_empty() => return WhatOpeningItFound::NoKeyHere,
        Ok(stored) => stored,
        // The reason and never the entry. A locked-down credential store is
        // worth a log line; what it holds is not.
        Err(problem) => {
            tracing::warn!("The credential store would not give up the private keys: {problem}");
            return WhatOpeningItFound::TheKeyHereCouldNotBeRead;
        }
    };
    let keys: Vec<SignedSecretKey> = stored
        .iter()
        .filter_map(|armour| SignedSecretKey::from_string(armour).ok())
        .map(|(key, _)| key)
        .collect();
    // Import stores only what has already parsed, so a key that does not is
    // the store handing back something other than what went into it. The
    // message may have been for that key, so "no key here opens it" would be a
    // guess; it is the key here that could not be read.
    let some_could_not_be_read = keys.len() < stored.len();
    if some_could_not_be_read {
        tracing::warn!("A private key in the credential store no longer reads as a key");
    }
    match open_with(armour, &keys) {
        WhatOpeningItFound::TheKeyHereDoesNotOpenIt if some_could_not_be_read => {
            WhatOpeningItFound::TheKeyHereCouldNotBeRead
        }
        found => found,
    }
}

/// The same, once the keys are in hand.
///
/// **No error out of the crate is logged or passed on.** Not because the crate
/// is untrustworthy but because of what its error text can hold: a parser
/// refusing a message can quote the bytes it choked on, and those bytes are a
/// stranger's mail. What crosses out of here is one of the words this project
/// chose.
///
/// A key with no passphrase on it needs none, and the crate opens it without
/// one. A locked key is offered the passphrases typed this session, which is
/// the only way it opens; one nobody has typed for stays shut, and the answer
/// then says so if the message was encrypted to it.
fn open_with(armour: &str, keys: &[SignedSecretKey]) -> WhatOpeningItFound {
    if keys.is_empty() {
        return WhatOpeningItFound::TheKeyHereCouldNotBeRead;
    }
    let Ok((message, _)) = Message::from_armor(armour.as_bytes()) else {
        return WhatOpeningItFound::Damaged;
    };
    let shut = the_locked_key_it_names(&message, keys).map(the_locked_key);
    let every_key: Vec<&SignedSecretKey> = keys.iter().collect();
    let decrypted = with_the_held_passphrases(|held| {
        let typed: Vec<&Password> = keys
            .iter()
            .filter_map(|key| held.get(&fingerprint_of(key)))
            .collect();
        message.decrypt_with_keys(typed, every_key)
    });
    let opened = match decrypted {
        Ok(opened) => opened,
        // Nothing here opened it. Either it names a key here that is still
        // locked, which is the person's to open, or it names no key here and
        // was encrypted to somebody else.
        Err(OpenPgpError::MissingKey) => {
            return shut.map_or(
                WhatOpeningItFound::TheKeyHereDoesNotOpenIt,
                WhatOpeningItFound::TheKeyIsLocked,
            );
        }
        Err(_) => return WhatOpeningItFound::Damaged,
    };
    // Safe on a message that was never compressed: the crate hands those back
    // unchanged. Called unconditionally rather than after a test, because a
    // test for whether a message is compressed is a second opinion about a
    // shape the crate already knows.
    let Ok(mut opened) = opened.decompress() else {
        return WhatOpeningItFound::Damaged;
    };
    let mut words = Vec::new();
    if opened
        .by_ref()
        .take(MOST_A_MESSAGE_MAY_COST)
        .read_to_end(&mut words)
        .is_err()
    {
        return WhatOpeningItFound::Damaged;
    }
    // Lossy rather than refused. A message body is a stranger's, and mail that
    // is not UTF-8 is ordinary rather than hostile: an old client sending
    // Latin-1 is common. Refusing it would report a working message as damaged
    // and say the sender should send it again.
    WhatOpeningItFound::Opened(String::from_utf8_lossy(&words).into_owned())
}

/// The first locked key here the message says it was encrypted to.
///
/// Read from the message's own list of whom it was encrypted to, which is what
/// the crate reads to choose a key, so the two cannot disagree about which key
/// a message needs. A message sent to a hidden recipient names every key, and
/// then the first locked key here is the one asked about.
fn the_locked_key_it_names<'k>(
    message: &Message<'_>,
    keys: &'k [SignedSecretKey],
) -> Option<&'k SignedSecretKey> {
    let Message::Encrypted { esk, .. } = message else {
        return None;
    };
    let names = |key: &SignedSecretKey| {
        esk.iter().any(|sent_to| match sent_to {
            Esk::PublicKeyEncryptedSessionKey(sent_to) => {
                sent_to.match_identity(key.primary_key.public_key())
                    || key
                        .secret_subkeys
                        .iter()
                        .any(|subkey| sent_to.match_identity(subkey.public_key()))
            }
            Esk::SymKeyEncryptedSessionKey(_) => false,
        })
    };
    keys.iter()
        .filter(|key| a_passphrase_is_holding_it_shut(key))
        .find(|key| names(key))
}

/// A locked key as the reader window asks about it.
fn the_locked_key(key: &SignedSecretKey) -> LockedKey {
    let listing = listing_of_a_private_key(key);
    LockedKey {
        whose: listing
            .user_ids
            .first()
            .cloned()
            .unwrap_or_else(|| format!("key {}", listing.key_id)),
        fingerprint: listing.fingerprint,
    }
}

/// A key's fingerprint the way every listing writes it, in hexadecimal
/// capitals.
fn fingerprint_of(key: &SignedSecretKey) -> String {
    format!("{:X}", key.fingerprint())
}

/// Try a typed passphrase on the private key here with this fingerprint, and
/// hold it for the rest of the session if it opens every locked part.
///
/// The typed text becomes the crate's passphrase type here and nowhere else.
/// A passphrase that opens nothing is dropped at once and nothing of it is
/// kept, so the next message asks again.
pub(super) fn unlock(fingerprint: &str, typed: &str) -> Unlocking {
    let armour = match slot_holding(fingerprint) {
        Ok(Some((_, armour))) => armour,
        Ok(None) => return Unlocking::NoSuchKey,
        Err(problem) => {
            tracing::warn!("The credential store would not give up a key to unlock: {problem}");
            return Unlocking::TheKeyCouldNotBeRead;
        }
    };
    let Ok((key, _)) = SignedSecretKey::from_string(&armour) else {
        return Unlocking::TheKeyCouldNotBeRead;
    };
    let passphrase = Password::from(typed);
    if !it_opens_every_locked_part(&key, &passphrase) {
        return Unlocking::WrongPassphrase;
    }
    with_the_held_passphrases(|held| held.insert(fingerprint_of(&key), passphrase));
    Unlocking::Unlocked
}

/// Whether a passphrase opens the primary key and every secret subkey.
///
/// Every part, for [`a_passphrase_is_holding_it_shut`]'s reason: a message is
/// encrypted to a subkey where the key has one. A part with no lock on it
/// opens whatever is typed, so a key with none answers yes.
fn it_opens_every_locked_part(key: &SignedSecretKey, passphrase: &Password) -> bool {
    let opened = |tried: pgp::errors::Result<pgp::errors::Result<()>>| matches!(tried, Ok(Ok(())));
    opened(key.primary_key.unlock(passphrase, |_, _| Ok(())))
        && key
            .secret_subkeys
            .iter()
            .all(|subkey| opened(subkey.key.unlock(passphrase, |_, _| Ok(()))))
}

/// The passphrases typed this session, by fingerprint, for `work` to read or
/// change.
///
/// One map for the whole program, since a passphrase typed in one reader
/// window opens the same key's mail in the next. It lives as long as the
/// process and is never written anywhere.
#[cfg(not(test))]
fn with_the_held_passphrases<T>(work: impl FnOnce(&mut HashMap<String, Password>) -> T) -> T {
    use std::sync::{LazyLock, Mutex, PoisonError};
    static HELD: LazyLock<Mutex<HashMap<String, Password>>> = LazyLock::new(Default::default);
    // A thread that panicked while holding the map left it as it was: every
    // entry in it was a passphrase that opened its key.
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    work(&mut held)
}

/// The same under test, one map to a test, the way the credential store under
/// test is one to a test, so a passphrase one test types never opens a key in
/// another that runs beside it.
#[cfg(test)]
fn with_the_held_passphrases<T>(work: impl FnOnce(&mut HashMap<String, Password>) -> T) -> T {
    thread_local! {
        static HELD: std::cell::RefCell<HashMap<String, Password>> =
            std::cell::RefCell::new(HashMap::new());
    }
    HELD.with_borrow_mut(work)
}

/// Forget the passphrase typed for a key, when the key goes.
fn forget_the_passphrase_for(fingerprint: &str) {
    with_the_held_passphrases(|held| {
        held.retain(|kept, _| !kept.eq_ignore_ascii_case(fingerprint))
    });
}

/// Whether a passphrase is held for the key with this fingerprint.
#[cfg(test)]
fn a_passphrase_is_held_for(fingerprint: &str) -> bool {
    with_the_held_passphrases(|held| {
        held.keys()
            .any(|kept| kept.eq_ignore_ascii_case(fingerprint))
    })
}

/// Whether a private key has been imported on this computer.
///
/// From the credential store rather than from a stored flag, for the reason
/// [`super::keyring_entries`] gives: deciding from a flag whether a secret
/// exists is how secrets get left behind.
pub(super) fn a_key_is_here() -> bool {
    keys_here().is_ok_and(|keys| !keys.is_empty())
}

/// Every key an armoured text holds, described.
pub(super) fn describe(armour: &str) -> Vec<KeyListing> {
    keys_in(armour)
        .iter()
        .map(|(key, _)| match key {
            PublicOrSecret::Public(public) => listing_of(public, false),
            PublicOrSecret::Secret(secret) => listing_of_a_private_key(secret),
        })
        .collect()
}

/// Every private key here, described, in slot order.
///
/// A stored key that no longer reads as one is left out and said in the log,
/// which is all that can be said of it: there is nothing to describe.
pub(super) fn private_keys_here() -> crate::common::Result<Vec<KeyListing>> {
    let stored = keys_here()?;
    let described: Vec<KeyListing> = stored.iter().flat_map(|armour| describe(armour)).collect();
    if described.len() < stored.len() {
        tracing::warn!("A private key in the credential store no longer reads as a key");
    }
    Ok(described)
}

/// Remove the private key with this fingerprint, every part of it.
pub(super) fn remove_private_key(fingerprint: &str) -> crate::common::Result<bool> {
    let Some((slot, _)) = slot_holding(fingerprint)? else {
        return Ok(false);
    };
    remove_the_slot(slot)?;
    forget_the_passphrase_for(fingerprint);
    Ok(true)
}

/// A key's public half, armoured.
pub(super) fn public_half(armour: &str) -> Option<String> {
    let (key, _) = keys_in(armour).into_iter().next()?;
    let public = match key {
        PublicOrSecret::Public(public) => public,
        PublicOrSecret::Secret(secret) => secret.to_public_key(),
    };
    public.to_armored_string(ArmorOptions::default()).ok()
}

/// The public half of the private key here with this fingerprint.
pub(super) fn public_half_of_a_key_here(
    fingerprint: &str,
) -> crate::common::Result<Option<String>> {
    Ok(slot_holding(fingerprint)?.and_then(|(_, armour)| public_half(&armour)))
}

/// The slot holding the private key with this fingerprint, and its armour.
fn slot_holding(fingerprint: &str) -> crate::common::Result<Option<(usize, String)>> {
    move_the_old_entry_into_a_slot();
    for slot in 1..=KEY_SLOTS {
        let Some(armour) = the_key_in(slot)? else {
            continue;
        };
        if describe(&armour)
            .iter()
            .any(|listing| listing.fingerprint.eq_ignore_ascii_case(fingerprint))
        {
            return Ok(Some((slot, armour)));
        }
    }
    Ok(None)
}

/// A key and a message for the tests of other modules.
///
/// The same GnuPG-made fixtures this file's own tests open, for the reason
/// [`crate::service::signed_mail::for_tests`] gives about its S/MIME messages:
/// a surface that says it opens a PGP message has to be tested against one
/// that really was encrypted by something other than the crate behind this
/// file, or it agrees with itself and with nothing else. Armour and words
/// only; nothing here names a crate type.
#[cfg(test)]
pub(crate) mod for_tests {
    /// Alice's private key, as the armoured text a key file holds.
    pub(crate) fn alices_private_key() -> String {
        super::tests::armour(super::tests::ALICE_PRIVATE)
    }

    /// A message encrypted to Alice by GnuPG, as it sits in a text part.
    pub(crate) fn a_message_to_alice() -> String {
        super::tests::armour(super::tests::TO_ALICE)
    }

    /// The words inside it, exactly as opening it hands them back.
    pub(crate) fn what_alices_message_says() -> &'static str {
        "The meeting moved to Thursday at ten.\n"
    }

    /// A whole PGP/MIME message to Alice, as it arrives: `multipart/encrypted`
    /// with the control part and the encrypted part GnuPG made.
    pub(crate) fn a_pgp_mime_message_to_alice() -> Vec<u8> {
        super::tests::armour(super::tests::PGP_MIME_TO_ALICE).into_bytes()
    }

    /// The words of its plain half.
    pub(crate) fn what_the_pgp_mime_message_says() -> &'static str {
        "The figures are in the minutes. See you Thursday."
    }

    /// Bob's private key, which is a real key and not the one Alice's messages
    /// were encrypted to.
    pub(crate) fn bobs_private_key() -> String {
        super::tests::armour(super::tests::BOB_PRIVATE)
    }

    /// Alice's public key, as GnuPG exported it.
    pub(crate) fn alices_public_key() -> String {
        super::tests::armour(super::tests::ALICE_PUBLIC)
    }

    /// Carol's public key, as GnuPG exported it: somebody else's key, the
    /// kind kept so mail can be encrypted to them later.
    pub(crate) fn carols_public_key() -> String {
        super::tests::armour(super::tests::CAROL_PUBLIC)
    }

    /// Dave's private key, with the passphrase `correct horse` still on it.
    pub(crate) fn daves_locked_key() -> String {
        super::tests::armour(super::tests::DAVE_LOCKED)
    }

    /// The passphrase on Dave's key.
    pub(crate) const DAVES_PASSPHRASE: &str = "correct horse";

    /// A message encrypted to Dave by GnuPG.
    pub(crate) fn a_message_to_dave() -> String {
        super::tests::armour(super::tests::TO_DAVE)
    }

    /// The words inside it, exactly as opening it hands them back.
    pub(crate) fn what_daves_message_says() -> &'static str {
        "Dave, the passphrase opened it.\n"
    }

    /// Dave's fingerprint, as `gpg --list-keys --with-colons` gave it.
    pub(crate) const DAVES_FINGERPRINT: &str = "BC398E0D54261CA0642E99AD469C95C000B5CB12";

    /// Alice's fingerprint, as `gpg --list-keys --with-colons` gave it.
    pub(crate) const ALICES_FINGERPRINT: &str = "6EFD87D1527731DE679B8E1BA97E7BB74101FB3E";

    /// Carol's fingerprint, as `gpg --list-keys --with-colons` gave it.
    pub(crate) const CAROLS_FINGERPRINT: &str = "8DE4DEEC367D086637934A1C52B5C043A2C64173";
}

#[cfg(test)]
mod tests {
    use super::for_tests::{ALICES_FINGERPRINT, DAVES_FINGERPRINT};
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // ── Keys and a message to test against ───────────────────────────────
    //
    // **Made by GnuPG 2.4.9, not by the crate this file calls.** That is the
    // whole point of them. A key and a message generated by rPGP and read back
    // by rPGP in the same test prove that the crate agrees with itself, which
    // is worth having and is not evidence that this reads real PGP mail. These
    // came out of `gpg --quick-generate-key` and `gpg --encrypt`, and the
    // message was checked to open under `gpg --decrypt` before it was written
    // down here.
    //
    // Base64 encoded and decoded in the test that wants them, the same as the
    // S/MIME fixtures in `service::signed_mail` and for the same reason: an
    // armoured block held as text in a source file has its line endings
    // rewritten by any tool that touches the file, and a rewritten armour is a
    // fixture that fails in a way that looks exactly like a real defect.
    //
    // RSA 2048, no expiry, no passphrase. Two keys, so a message encrypted to
    // one of them can be offered to the other.

    /// Alice's private key, exported by GnuPG with no passphrase on it.
    pub(super) const ALICE_PRIVATE: &str = "
        LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbFFPWUJHcWRrWFFCQ0FD
        MkdHMER0U2FWb1N4UkZnMTJ1QlA0YzhqdnQzNjhiMC9RWXNKQVNPSjBlMnlWTTljeApwSUtt
        OTYvMmh2L1Z1d2w0MTRWMFJBSm5yd2tSdEdTSTVVTEYwK3l2Q09xWncrZDNScTRlL3FjajYw
        SnpuYzE0Ci9MTDVYWnNEbTJpR1I0ZzlWRFM2RTdMQTVsTmxuZmxaV0dVZ3hDZVVxZDZDRUs2
        a0dVSXpEaFZOZW41OVlxMmgKVkU4ZlhadzRaVW9GcGJQNENrR2lVcHM5RW0yMExxNjRyTUNx
        ZXFMZy9sZW9JY3ZQczhSQ1lrWE1mZ3k0WFZ2SQptL0p4SS9mTDNCcTVEOFFMMC9hVjJrQUpj
        bUE5c1AvQjJLeVFLRUtpYTU4UjA3MnVZd1RiUkdHME9SNXRaQVppCnpIZEdReTh4eWZuNUNh
        YWxnOGdrN0RPWGUvL2VPWFY4aGJhbkFCRUJBQUVBQi80MmNIeis4OFJ5VFhvYlQ5VjgKbmJI
        U3dJTGRMK1dpM2tCbFUzRXhtTmlpN0ZkZEQ5K1JCdGJNcGhZa1JOM3RmdnBvOXozOXNXdHFw
        Z2kzRTFCUApJUU5pYzJZNk9oY3hHMEZ6QmwxK0JMSGhhaTgyczRHL1h3VCt5ajVTeUw1cURx
        NnJieVpJVHlWTXlFODhmUXNUCjYvcG4zbHphOUNJQ2pvdzNvUm1LZS9aZ2IvSnVpbkp6STFU
        NjJOOU1RWWkybm05Z0xpWHo2NzRTbDlrNVJMN3UKUnFmWndvOGJZTDZGaDJGbmxycGdqaExz
        dURSZkx3cEVMa2lXQ0tsLy9pVUIzZXZwVExvTHdpRlZjOWwvRldUdQpIRHRCT3pJNVdvaWpE
        b016WklpZ1JzNXN3WDRscnVCNWQ1NGhnbmxZcDN3MWdGNFBKMmhlakEwMUJGemxsWDhGCnpO
        OUJCQURPS1cvdml3WTNLclVDN0VUMjZGb2hveUxCQlJ6U2NVWm1POE9iSWJyT2NDZmJLRlRY
        UlNBTmVuUTIKZ3ZQTG4ydDJjVzE2WGxjem44dHpDeSs5UWJVYk9VZkwvU0ZsYXEvbW5wTFdx
        WlRCZFJsOGJiV2M0Vk5tdldlaQpGNHhxaEtweXZiVjIvLzFyaG03Nkp2cjlremQxOFZ1U0tj
        TVRwZEpNQkhTL1U5dndod1FBNGgyYjEyYXp6QU92CnNSTmJiMlpzVFQ2SFZmQ2p5VU1XOERZ
        cFdqeVM0SXRUbDR4WU5QUXJqaUloL0d6Y3YwSVVzOWNURmxRK3phNisKZWxGSE5YOWRUVHBJ
        U05uUVpjT254T0hJQ3dZSlNYWWxyNVRudk1udW9CeVBhTFpFcG1FRFJaejcrVE8raGRYWgpv
        YVRzUkxSZG0rcG1pcGl0Y2o3QjNNekRRTmx5TU9FRC8xYUhXNWx4SmFJNXh4bXZyaEVPYTM0
        bkpjY1hiTEp2CmFkcWxpQ045UTB4NUwreHg5bm9IZ0hKaEJlNnFpSUZZTy92VXY0WGZoUVFj
        bGI5RS9adXhXcUFhVElZcEJ6RVAKdll3dk1yUHBnUHNKOFFLTlJpa2lmNkZ2eW1VQThEQU92
        ZXZKS1pnUFNScWp6UzJBNG45bTN6SkVSSmNqbzU0bgo2cEFQVjVlK0NIby9OWFcwSVVGc2FX
        TmxJRVY0WVcxd2JHVWdQR0ZzYVdObFFHVjRZVzF3YkdVdVkyOXRQb2tCClRnUVRBUW9BT0JZ
        aEJHNzloOUZTZHpIZVo1dU9HNmwrZTdkQkFmcytCUUpxblpGMEFoc05CUXNKQ0FjQ0JoVUsK
        Q1FnTEFnUVdBZ01CQWg0QkFoZUFBQW9KRUtsK2U3ZEJBZnMrMUpRSC8yL2FyNHV1QkprYndR
        UXBUeE9ySG13aQoxZDNqOHdkQ3NBLzJXSEdxblU5RmswU29iRFdIOUg3QU1sKzJDcExZby84
        Nnl5Mm1nSEJ0UEtOSFNmZm1uZ0ZOCjR2VHhaREc2Kysrd0cxTXQwUjE0M3ZQNk5uNStQYTZH
        MjNwRUpOenk0a21oZmpSS0grUjIwWk5xVUdUeVlzbjgKSjBTdzIyWkUwMzJhMmF4WkVTN29N
        SUF3d21hTzY4M1dHQXd3ZXRkTmV5RzNlSDUxemRUMTlaTG4xYjB2TkNVMQpWdWRvSDErLzFo
        KzI2U21kanNVSGZKaElUdkJNeS92cTVLSllqV3RzcVIrWUpiVWx5bU1IN1hnTDM0dzc2azZs
        CllNd1dwTml3YkhPaS9TQUFoLzFmSUl1ci9oYVFzNXFwSEdIUDJ6NnhpSUFua3BwcmdVTUZk
        QWpsUjcwRkpvMD0KPVRVNzQKLS0tLS1FTkQgUEdQIFBSSVZBVEUgS0VZIEJMT0NLLS0tLS0K";

    /// Bob's private key. A real key, and not the one the message below was
    /// encrypted to.
    pub(super) const BOB_PRIVATE: &str = "
        LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbFFPWUJHcWRrWFFCQ0FE
        UTFZcmdQYmF2MWxyenkvaEhBb3FiS2Y3YmdFMDZ3c0tXcm1aWlZoTnB3K0N4d3FlZgpwZ204
        RWNuWXNuQkJESkpVY01IVjRVS0R4MkFZdVFwemN0OG5Zd0xGMzh1WWtJc20xT1YwTTVGN2VM
        bm1ZblViClZrWlRsN3ZYdTYvN0tZVmNRcHJFVTV3VE9kQXZValhxZTN0ZEJoM1N1alJiM3ZX
        K0YvU000RE5Dd2sxN0pnMkIKUDZpQ0l0YzhCMDBkZHArazRIazB4Qng3OUJEQ3BrampBSmRI
        aE9HR01PYnpnTDcxL2NkekczenU0VmtiWDRpdQp5c1MzYXN3MWRIdDE4aElPM1g3eDRnOWsv
        dUNsNFY3dUZOMTVOb3BrY0UyWVcvTjJZSHRZZWtqVDFDTTRTU1RMCmRtbjBqMkxjZ1pEb25L
        dGdWeXZpczNxWStCblIyR2R3cDVSdEFCRUJBQUVBQi8wWExMZGtZcHpaSHd2SlZjZUgKMDgy
        ZllmWHRGR3dkbXZyZW4xeHV5Z2xOK2FWLy9JYS9CZjA3RUd0S242U3E1MUwreVdPTlVWbmhC
        MWxQN1FydQpuRjhNdnlWUjRaKzFuc2ErYUs1TTZxTVlwV0ZWNG5PMTlLbEp3Z01mc2UyWnpQ
        WUdjc0s3aWo2OStISzBxYVlmCmp4UDEwd1d3dXFhd1Vrb0pqaDErbTZ1OG11R05CY05QbU0y
        b291UUdaamRIWFY2N2UzV1lNdHJ3Ym0wWlN5d2sKNnBXTUZqTmpHQ1ZmbFRDYlhkZkoxTVhO
        Y1JqSzRXdUdsT3NwQXo4SWtJOWRXSE14VW9MS2FxY2NsNWtGc1gxOApFZ3Zycll0RHRua1JG
        ZFZRczlNQVZpd3cvL1NUcFR0WFN1SnJlckJhQXZLY1Bmb3U1UGFWZEFDQWsveTlISmI2Cjhk
        MFJCQURqNXoyOXZYMjkvcDVHeThZWllWM1ZYL25sUzB0QSttUlBBRXJpNWRDTDZYK1VnckhC
        YXFWK05qVzYKSjQ3a0dmQjVxZU1lUkkxNmRlVkFGeVRJTUJySmgrTDlOdUdlWjRDWGJtNDlw
        MmJkR0FrMDJVTjJhcnRUWnRWbwpoSGpmbkZuODBtTm4vcWJwWms4dnhPWjE4Z3dZUzBlQnhH
        WWUyLzBaVzhsNHBLQmgwUVFBNnBSMzZsZFVPV1laCkNCdWdMSVpWSXd4b0xCb2FQbnlscUxC
        c0E5SjdiMTk1QjdyYUN1eWFGMXhqTlNUaFJrRU5WMTdBanFLNE41ZjMKazJUU2ZPc3dyazZV
        eG16MDdoVDltOWpPT3RaL0haUXNscE5CQkpFQ1VUWXlWaFNuSFE1QXZyMVhOckFHMW5UOQp0
        RUtNU21vcXVlNEx0cytWV2oxSTBVNnp2RVp1czkwRUFMd3RSOHZtZDIzNUpRNlVhNWduL3Jz
        ZlVBdWlkakpLCldxdlliS1ZmOEQxdXJQaGtodmJhNlFnWStaR0xMa1RLNXhyZmJCVGpKWkow
        SERyMDFMMnNMT1RHUFZiUDVYNEIKalBjQjl2amJuT21yVkpsckZLN3I4djJncklheUUxa3NK
        TCsvbFVBc21PRkU5OGJ6eG1xcEFMazNJVkVIK2plKwppaVFCZ0Y5WWFmelFSbEswSFVKdllp
        QkZlR0Z0Y0d4bElEeGliMkpBWlhoaGJYQnNaUzVqYjIwK2lRRk9CQk1CCkNnQTRGaUVFOVR1
        RkliUzhBZmVMUm5HZ1pmYmEzYXd0SG4wRkFtcWRrWFFDR3cwRkN3a0lCd0lHRlFvSkNBc0MK
        QkJZQ0F3RUNIZ0VDRjRBQUNna1FaZmJhM2F3dEhuMlJtUWdBblBaTytyRWZZWWtiU0Rlbmhq
        WGVnc3QzM1dZUQpEV1VmbnAycExuRnhiRlBQRHBSUUVvZ3Y5Q0RuWTc5cGJvVy9Zb1NzOVJY
        akdEQzUwZlgyZWxvQmhqWkNZNkc0Cm1rRTB6WXg1UHlXWXZXOUlQaTNnV0ZqSDNYTjdlVTJo
        T3Qxc2c0VHVzOVlneXd0REJRczQvdkpXRC9rYlFGaHYKMW1HOXZ6dXcrSkpmV2xJa0hqZ2Vj
        Zksrc1g4a0VZNitOQjNJTTlXN0o3M2JhZEI5QU1GY3Exb1hqanpXV0xDdgpkcDBQSlNkenBJ
        NWRRSk9BdDFzTzVQTlp0ZHZlN3BhTG82Zk1nbGREcVJaaGZZa0dqaXNSc1V1aXZENUZ5U0Vn
        CkxMWUplNU01eW14ZmM0MU4vOWhNWUR2emFKeXpaa1M4MFlDVi9HZ0w3eTNiczBQZW1vR3RH
        SjFtb0E9PQo9MFZONAotLS0tLUVORCBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQo=";

    /// Alice's public key. A real key and not a private one, which is the
    /// mistake somebody makes at three in the morning.
    pub(super) const ALICE_PUBLIC: &str = "
        LS0tLS1CRUdJTiBQR1AgUFVCTElDIEtFWSBCTE9DSy0tLS0tCgptUUVOQkdxZGtYUUJDQUMy
        R0cwRHRTYVZvU3hSRmcxMnVCUDRjOGp2dDM2OGIwL1FZc0pBU09KMGUyeVZNOWN4CnBJS205
        Ni8yaHYvVnV3bDQxNFYwUkFKbnJ3a1J0R1NJNVVMRjAreXZDT3FadytkM1JxNGUvcWNqNjBK
        em5jMTQKL0xMNVhac0RtMmlHUjRnOVZEUzZFN0xBNWxObG5mbFpXR1VneENlVXFkNkNFSzZr
        R1VJekRoVk5lbjU5WXEyaApWRThmWFp3NFpVb0ZwYlA0Q2tHaVVwczlFbTIwTHE2NHJNQ3Fl
        cUxnL2xlb0ljdlBzOFJDWWtYTWZneTRYVnZJCm0vSnhJL2ZMM0JxNUQ4UUwwL2FWMmtBSmNt
        QTlzUC9CMkt5UUtFS2lhNThSMDcydVl3VGJSR0cwT1I1dFpBWmkKekhkR1F5OHh5Zm41Q2Fh
        bGc4Z2s3RE9YZS8vZU9YVjhoYmFuQUJFQkFBRzBJVUZzYVdObElFVjRZVzF3YkdVZwpQR0Zz
        YVdObFFHVjRZVzF3YkdVdVkyOXRQb2tCVGdRVEFRb0FPQlloQkc3OWg5RlNkekhlWjV1T0c2
        bCtlN2RCCkFmcytCUUpxblpGMEFoc05CUXNKQ0FjQ0JoVUtDUWdMQWdRV0FnTUJBaDRCQWhl
        QUFBb0pFS2wrZTdkQkFmcysKMUpRSC8yL2FyNHV1QkprYndRUXBUeE9ySG13aTFkM2o4d2RD
        c0EvMldIR3FuVTlGazBTb2JEV0g5SDdBTWwrMgpDcExZby84Nnl5Mm1nSEJ0UEtOSFNmZm1u
        Z0ZONHZUeFpERzYrKyt3RzFNdDBSMTQzdlA2Tm41K1BhNkcyM3BFCkpOenk0a21oZmpSS0gr
        UjIwWk5xVUdUeVlzbjhKMFN3MjJaRTAzMmEyYXhaRVM3b01JQXd3bWFPNjgzV0dBd3cKZXRk
        TmV5RzNlSDUxemRUMTlaTG4xYjB2TkNVMVZ1ZG9IMSsvMWgrMjZTbWRqc1VIZkpoSVR2Qk15
        L3ZxNUtKWQpqV3RzcVIrWUpiVWx5bU1IN1hnTDM0dzc2azZsWU13V3BOaXdiSE9pL1NBQWgv
        MWZJSXVyL2hhUXM1cXBIR0hQCjJ6NnhpSUFua3BwcmdVTUZkQWpsUjcwRkpvMD0KPURsOW0K
        LS0tLS1FTkQgUEdQIFBVQkxJQyBLRVkgQkxPQ0stLS0tLQo=";

    /// "The meeting moved to Thursday at ten.", encrypted to Alice by GnuPG.
    ///
    /// It opens under `gpg --decrypt` with the key above, which was checked
    /// before this was written down.
    pub(super) const TO_ALICE: &str = "
        LS0tLS1CRUdJTiBQR1AgTUVTU0FHRS0tLS0tCgpoUUVNQTZsK2U3ZEJBZnMrQVFmL1Yycyty
        TmxnS2VPQ2NXOG9QY1VLTUlYNHJHaWU5QnFac01KMXlmVmJPeERZClAxSmFSaWRPM051VVdW
        aUVDVk0raXEyemtmT0xuSk4zb3J1eWdzWWFNTmFmRWhWYW01Nk41T1FKdXFSSnladzMKWmYx
        VFM0eVh3cHZmbHNJZ1NZN0dMVURrMGpTT3lwWlYrRmtPU01aTkFyWEtVTzVBby8vR1dVS29J
        ZmFJZFpnagpRNC90WDlveFAyTTJxZHFndEVxckQ5VFdQZE9JZVhNdVBtUVQwcFY1STYwQXZk
        Um9BYzRvbWpDMFpoRnpaTHF6ClZ6UG55Vm1HTmZJUElhbHRzblZ6cFV4RHgxOUNQTzRVeUFX
        SGZzdjhrc1RGZkdsR25JNmpBeEs1eWFWMWIrWXkKZlFCQ0JERDNvQzN3NGorUUc4WlphR3FL
        dENSZFIwN0FUOTlVWGNYb2hOSmhBWUJEOGVSTGNlNlpJOUZWdkpRZwpoSTJvZVRtNHlXNlpo
        bXM5OHU5NmxQVGg5ZVBIcVZwZnd0M3NaMEd6eE5KNi9ScG1XQ2FhbUtmcVZ3dVlZM0lCCmZT
        a3NzSUFINVpKd3hqTGxDWWRmemIzd081bXJkbGxnMW5pKzE4dkJ3NFQ3ZVE9PQo9OFNXaAot
        LS0tLUVORCBQR1AgTUVTU0FHRS0tLS0tCg==";

    /// Carol's private key: Ed25519 with a Curve25519 encryption subkey, no
    /// expiry, no passphrase, 744 characters armoured, so it fits in one
    /// credential entry where Alice's and Bob's do not. Made by GnuPG 2.4.9 on
    /// 2026-09-27 in a short home directory:
    ///
    /// ```text
    /// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
    ///     --quick-generate-key 'Carol Example <carol@example.com>' ed25519 sign,cert never
    /// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
    ///     --quick-add-key 8DE4DEEC367D086637934A1C52B5C043A2C64173 cv25519 encr never
    /// GNUPGHOME=/c/g16 gpg --batch --pinentry-mode loopback --passphrase '' \
    ///     --armor --export-secret-keys carol@example.com
    /// ```
    const CAROL_PRIVATE: &str = "
        LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbEZnRWFyaVk1QllKS3dZ
        QkJBSGFSdzhCQVFkQStUc0c5RTZidWxlVmZrR1NmRDQ2R3U3WHRRbmEwN2R4SUZZTgpaSm9E
        Y1lJQUFQNHlZWFpvcnBsT3puVG9zMTdHa29FSm02RGlOMDY2YncyVTdYZUdBOU4xbUJGYXRD
        RkRZWEp2CmJDQkZlR0Z0Y0d4bElEeGpZWEp2YkVCbGVHRnRjR3hsTG1OdmJUNklrQVFURmdv
        QU9CWWhCSTNrM3V3MmZRaG0KTjVOS0hGSzF3RU9peGtGekJRSnF1SmprQWhzREJRc0pDQWND
        QmhVS0NRZ0xBZ1FXQWdNQkFoNEJBaGVBQUFvSgpFRksxd0VPaXhrRnp5b2dCQUxTeVFsN09S
        MjFpdTRPclBJTlZpQXR5YVBnQmYzN1ZvWUt2UU9aRVZMeXpBUDlyCkJMRzExQ3dlK0ZyS3o2
        TXhTS2tqMnVUVGpBLzFUaWZxVmFRUjFqZThDWnhkQkdxNG1PVVNDaXNHQVFRQmwxVUIKQlFF
        QkIwQkhjTGluQXp3QUp6WUx4WEpWMnVKdVBtTU9nOU5iajVhNHUzSlI5MEFDTndNQkNBY0FB
        UDljalE3dQpwRkY3ci8xbVNrNlZNbW5FbW5xR09TcnlDS0ltUk4wMGZoZ0EwQTlzaUhnRUdC
        WUtBQ0FXSVFTTjVON3NObjBJClpqZVRTaHhTdGNCRG9zWkJjd1VDYXJpWTVRSWJEQUFLQ1JC
        U3RjQkRvc1pCY3o0RkFRRENqWE1xMzRab2RVRFAKamV1Ni8vZ0pVeXpSd2hSdDNqSkhVMExH
        R2o2WHV3RUFvVyt2L0pnRmdVSXVvWk02NEQzWVlkK3p2V2FBTEZzbApZMkNFN3hGYzhRWT0K
        PUJ4eGUKLS0tLS1FTkQgUEdQIFBSSVZBVEUgS0VZIEJMT0NLLS0tLS0K";

    /// Carol's public key, from `gpg --armor --export carol@example.com` in
    /// the same home directory. GnuPG lists it as `scESC` with an `e`
    /// subkey: it signs, and mail can be encrypted to it.
    pub(super) const CAROL_PUBLIC: &str = "
        LS0tLS1CRUdJTiBQR1AgUFVCTElDIEtFWSBCTE9DSy0tLS0tCgptRE1FYXJpWTVCWUpLd1lC
        QkFIYVJ3OEJBUWRBK1RzRzlFNmJ1bGVWZmtHU2ZENDZHdTdYdFFuYTA3ZHhJRllOClpKb0Rj
        WUswSVVOaGNtOXNJRVY0WVcxd2JHVWdQR05oY205c1FHVjRZVzF3YkdVdVkyOXRQb2lRQkJN
        V0NnQTQKRmlFRWplVGU3RFo5Q0dZM2swb2NVclhBUTZMR1FYTUZBbXE0bU9RQ0d3TUZDd2tJ
        QndJR0ZRb0pDQXNDQkJZQwpBd0VDSGdFQ0Y0QUFDZ2tRVXJYQVE2TEdRWFBLaUFFQXRMSkNY
        czVIYldLN2c2czhnMVdJQzNKbytBRi9mdFdoCmdxOUE1a1JVdkxNQS8yc0VzYlhVTEI3NFdz
        clBvekZJcVNQYTVOT01EL1ZPSitwVnBCSFdON3dKdURnRWFyaVkKNVJJS0t3WUJCQUdYVlFF
        RkFRRUhRRWR3dUtjRFBBQW5OZ3ZGY2xYYTRtNCtZdzZEMDF1UGxyaTdjbEgzUUFJMwpBd0VJ
        QjRoNEJCZ1dDZ0FnRmlFRWplVGU3RFo5Q0dZM2swb2NVclhBUTZMR1FYTUZBbXE0bU9VQ0d3
        d0FDZ2tRClVyWEFRNkxHUVhNK0JRRUF3bzF6S3QrR2FIVkF6NDNydXYvNENWTXMwY0lVYmQ0
        eVIxTkN4aG8rbDdzQkFLRnYKci95WUJZRkNMcUdUT3VBOTJHSGZzNzFtZ0N4YkpXTmdoTzhS
        WFBFRwo9WmtSUwotLS0tLUVORCBQR1AgUFVCTElDIEtFWSBCTE9DSy0tLS0tCg==";

    /// Dave's private key, Ed25519 with a Curve25519 subkey, exported by
    /// GnuPG 2.4.9 on 2026-09-27 with the passphrase still on it: the same
    /// commands as Carol's with `--passphrase 'correct horse'` throughout.
    /// Nothing here can open it, which is what it is for.
    pub(super) const DAVE_LOCKED: &str = "
        LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbElZRWFyaWhNaFlKS3dZ
        QkJBSGFSdzhCQVFkQUNFWVUrTXQ1Z3BTdDNLKzM1UjR0V1dHdXBaQ1M0YStEQ21segowS2ZJ
        VS9iK0J3TUMwR0ZzRjA3a3A0L3oyclRvY2V5cFNIVEpJSDZqc1V0ODlGc2ZzbksxTGF0Rmt3
        cFh0V3BXCldzYmhaTXJTTy83QytFczcxTkRod2lYM1hLa2gwckVZYTdBNUtpci9mSnlNaWlB
        WnZrVUM1YlFmUkdGMlpTQkYKZUdGdGNHeGxJRHhrWVhabFFHVjRZVzF3YkdVdVkyOXRQb2lR
        QkJNV0NnQTRGaUVFdkRtT0RWUW1IS0JrTHBtdApScHlWd0FDMXl4SUZBbXE0b1RJQ0d3TUZD
        d2tJQndJR0ZRb0pDQXNDQkJZQ0F3RUNIZ0VDRjRBQUNna1FScHlWCndBQzF5eEp4TFFEK0k3
        TjVVOWViN0UyaU5ZU1RuSGtsUVJaQmpZejgzN2pnSWNpeEZPTit1T0FCQUkwYXpQcjEKeTNK
        N3B4Q2kySk15azVTWDBpa1NpaWFxcEhqMHV3VkJqNUVJbklzRWFyaWhOQklLS3dZQkJBR1hW
        UUVGQVFFSApRRm9Dayt0YStzNHIwNHNUcHEwcTFIK0VrQXFkWG5KWjJEenNXVmRkNFNNU0F3
        RUlCLzRIQXdMSi94NkZTR3ZuCnFmTUdFTFVrYTY5TFQzdEJJZDlkNU9NWWFSMWdOWDJDdU1h
        V1J3ZSt5c0lEaWl1VHR3WkRmUWQ3QSs2TmlSQ3AKSWpoU2VSRFdYelZ1OG4rUzhmeWVQaXlM
        R3VITnhMUEZpSGdFR0JZS0FDQVdJUVM4T1k0TlZDWWNvR1F1bWExRwpuSlhBQUxYTEVnVUNh
        cmloTkFJYkRBQUtDUkJHbkpYQUFMWExFdlM0QVFEUFQxRGc5MytwTFY4UklOZlNkd0g2Clox
        ZUNRTDNYQVp4UHEvNmJhUGxDQ3dEL1o1eGd4Q0NNclVmK2drQWk0L1lBUTZtM05ZbGZ4Z0VG
        K1g4SzdRK3cKTGdBPQo9VVF3egotLS0tLUVORCBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0t
        LQo=";

    /// "Dave, the passphrase opened it.", encrypted to Dave by GnuPG 2.4.9 on
    /// 2026-09-27, in a home directory holding only his locked key:
    ///
    /// ```text
    /// GNUPGHOME=/c/g171 gpg --batch --pinentry-mode loopback \
    ///     --passphrase 'correct horse' --import dave_locked.asc
    /// GNUPGHOME=/c/g171 gpg --batch --armor --trust-model always \
    ///     --encrypt -r dave@example.com -o to_dave.asc to_dave.txt
    /// ```
    ///
    /// Checked before it was written down: `gpg --decrypt` with the passphrase
    /// `wrong` answered "Bad passphrase", and with `correct horse` gave back
    /// the words exactly.
    pub(super) const TO_DAVE: &str = "
        LS0tLS1CRUdJTiBQR1AgTUVTU0FHRS0tLS0tCgpoRjREV3htNnBFOWVJOTRTQVFkQVVTNjZC
        U3Jrb0FEOXBObldoVm45YmtmUklsUndQRmt4eHdBbXlIVGFBbVl3CkVsQ09yRmR5VjlkbUVm
        UTBvcmg1bEVVYTFPaU1CQzZuRGJ1YStjYldMSmlaVDV6eGhrSFl5bFhGZUw0Y3EzU24KMG1V
        QmdKN0dCS0hPei9BemNuMU5PQ0pneEZMOGZNbFgzTWNWd0lvNk9DaG4yOFdmRUoxR2VZWWhp
        cTlCTHgxZwpWWU1BTVgvaUg1TXhuekQzMmdackNaQTZrcHF4bkNKck1CaFFFWWI3SzlpYmJy
        UWppVzg1YzhpdkdLaHZFdDV5CnpaeXU4SnJFemc9PQo9cHllQQotLS0tLUVORCBQR1AgTUVT
        U0FHRS0tLS0tCg==";

    /// "Carol, the key fits in one entry.", encrypted to Carol by GnuPG with
    /// `gpg --batch --armor --trust-model always --encrypt -r carol@example.com`,
    /// and checked to open under `gpg --decrypt` before it was written down.
    const TO_CAROL: &str = "
        LS0tLS1CRUdJTiBQR1AgTUVTU0FHRS0tLS0tCgpoRjREUUVOWDZPRTRVZUFTQVFkQTJxcjhj
        ZzZ4NVpyWmtCTmQxVklDaHZxK0IzUG56RDB5bU5kRTNnT2xKbG93Ckh2TDhTQi92KzVKWThI
        R3QrY29XSXcyek9FRTNlTTJtLzFWaC9mdks4TVNlWVc2Q1JUR0UzZm04b1lMOXJ1aTQKMG1R
        QnhJKzM2TEZRLzltZ2lNWlYzeWhzdFRrc1ZFWG9ibVZuRGJVUi9vRTlSRXorTVFGVmRRL1dP
        eVdvOXBmbQplamRxOThhZ3NuSHN4WHhqVnlPYjNuWWhkaGxVTWkzclZ3MDBkbVRzZjhmbmpV
        K1g3cUgwck5aMlhyWkFFbUh0CkJBL1FFYjNVCj1mOGlBCi0tLS0tRU5EIFBHUCBNRVNTQUdF
        LS0tLS0K";

    /// A whole PGP/MIME message to Alice, RFC 3156's `multipart/encrypted`,
    /// with every line ending in a carriage return and a line feed, as mail
    /// arrives.
    ///
    /// The encrypted part was made by GnuPG 2.4.9 on 2026-09-26, in a home
    /// directory short enough for `gpg-agent`'s socket, with Alice's keys
    /// imported from the two constants above:
    ///
    /// ```text
    /// GNUPGHOME=/c/g13 gpg --batch --import alice_public.asc
    /// GNUPGHOME=/c/g13 gpg --batch --armor --trust-model always \
    ///     --encrypt -r alice@example.com -o inner.asc inner.eml
    /// GNUPGHOME=/c/g13 gpg --batch --decrypt inner.asc | cmp - inner.eml
    /// ```
    ///
    /// The last line was checked before this was written down. `inner.eml` is
    /// a `multipart/mixed` holding a `multipart/alternative`, whose plain half
    /// says "The figures are in the minutes. See you Thursday." and whose HTML
    /// half says the same with an `img` pointing at
    /// `https://tracker.example.com/chart.png`, described and sized so a page
    /// in the clear would fetch it, and a file, `minutes.txt`,
    /// saying "Item one: the figures.". The armour was then put by hand in the
    /// second part of a `multipart/encrypted` whose first part is the
    /// `application/pgp-encrypted` control part saying `Version: 1`, the way
    /// Thunderbird lays one out, with `name="encrypted.asc"` on the second.
    pub(super) const PGP_MIME_TO_ALICE: &str = "
        RnJvbTogQm9iIEV4YW1wbGUgPGJvYkBleGFtcGxlLmNvbT4NClRvOiBBbGljZSBFeGFtcGxl
        IDxhbGljZUBleGFtcGxlLmNvbT4NClN1YmplY3Q6IFRoZSBmaWd1cmVzDQpEYXRlOiBUaHUs
        IDI0IFNlcCAyMDI2IDEwOjAwOjAwICswMDAwDQpNZXNzYWdlLUlEOiA8cGdwLW1pbWUtMTMt
        MTVAZXhhbXBsZS5jb20+DQpNSU1FLVZlcnNpb246IDEuMA0KQ29udGVudC1UeXBlOiBtdWx0
        aXBhcnQvZW5jcnlwdGVkOyBwcm90b2NvbD0iYXBwbGljYXRpb24vcGdwLWVuY3J5cHRlZCI7
        DQogYm91bmRhcnk9ImVuY3J5cHRlZC0xMy0xNSINCg0KVGhpcyBpcyBhbiBPcGVuUEdQL01J
        TUUgZW5jcnlwdGVkIG1lc3NhZ2UgKFJGQyA0ODgwIGFuZCAzMTU2KQ0KLS1lbmNyeXB0ZWQt
        MTMtMTUNCkNvbnRlbnQtVHlwZTogYXBwbGljYXRpb24vcGdwLWVuY3J5cHRlZA0KQ29udGVu
        dC1EZXNjcmlwdGlvbjogUEdQL01JTUUgdmVyc2lvbiBpZGVudGlmaWNhdGlvbg0KDQpWZXJz
        aW9uOiAxDQoNCi0tZW5jcnlwdGVkLTEzLTE1DQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9u
        L29jdGV0LXN0cmVhbTsgbmFtZT0iZW5jcnlwdGVkLmFzYyINCkNvbnRlbnQtRGVzY3JpcHRp
        b246IE9wZW5QR1AgZW5jcnlwdGVkIG1lc3NhZ2UNCkNvbnRlbnQtRGlzcG9zaXRpb246IGlu
        bGluZTsgZmlsZW5hbWU9ImVuY3J5cHRlZC5hc2MiDQoNCi0tLS0tQkVHSU4gUEdQIE1FU1NB
        R0UtLS0tLQ0KDQpoUUVNQTZsK2U3ZEJBZnMrQVFmL2NLL1EydGdmck9jUmNORnZramJJc0pB
        enVoUEJGZmxLUHpRUnN2cUcyUUMwDQoyYmVjbTEyRlNYK1lnaGlvMHVLck4rUHV1Zm4wczBw
        ZmVndk9NU2NxTWtWR3pwQ1JLcmNiUkNzZGE0ZkJyOVhvDQpQelhjUWZaL29LZ0VxK0N5WEdY
        Rk83MkEwUkNIaWxpcmV3Q2ZueHYydWt3ODJPVCtRVFZ4cVBqZ0cyMEdpL2tiDQp4THdubTRz
        NTZVRURXSVZ4ZzJkZ0IvQXo4R0x2M2xtRXNtam16SjRVdzJLcVRuU3N6RVJrdkxYNE1MbGVM
        TktWDQpQbTZBRTFtY0F3c24xeERpNThWWUEvSUZFQ2V2UCs4VnZmV0p1ZnltS3lScGkzR2Rv
        c2pXZ2lUWW1EdmhSc0pvDQppclVlajVGOGlCVThuTWZlRUlQL2ZzVmRlOUhmSUU0L012R0cy
        clhqWHRMQXBBSDhEQUZpM3VpZ3pEdUM3YXVBDQpTNmRVcDduMEp6bktKdmR4cWFtVisvWUMr
        TlBGSmlpdERqYlQxd0JubGhwNWplNGdRbFBqY0c1TUdwQXQzSVRvDQpTWlJuZGpmbkkyR0xW
        UVBpMU5FNDRMOWVLN3M0OEFEM1Y5Qm5IcXJlcUF5RXcwRTBra3lNTnpXK0FPR0tUN2ExDQox
        QXlCWGZFMG5zVDNZdWRIZDRadmhJRktmTXRXQzBJVEhpaEZFdkMyZit1VnF5MnNFZVFTK0Nl
        KzU2KzRyZmZNDQp6eTdZZHUrZlY1RXNLWUZMRHZhWUhPelAvSWZpUEJtOEtQVXk5Y3c4SWdO
        R2VuOXBjb3FsMzFkbXVFRDVJdUJaDQpTa2l6QTd1NUVwMk43SzJlY1BoeWgvcXZDcyt3QXNX
        dnI0VGpPVTl0Y01jNXdjMnIzK3lhVlM1dkdNcElGR2N5DQprWEFmVXNhbG1PSnZRMW1WeW5U
        V1RIUkZXRHlFUHhwenVFdlJOaDJRYzJ3bldLODBOYzZ5a2pOMXRER2JCQVB3DQplL0VGUElG
        aDM0dFRjcDB6ODFEMHJDWkJOZzdUWENZTXZuLzI4RnVRMEZMY0xBYWoyOEJzT09URTR0MU9W
        aEgyDQpESml5QzFuZw0KPVBNQ0ENCi0tLS0tRU5EIFBHUCBNRVNTQUdFLS0tLS0NCg0KLS1l
        bmNyeXB0ZWQtMTMtMTUtLQ0K";

    /// The bytes a fixture stands for.
    pub(super) fn armour(encoded: &str) -> String {
        let packed: String = encoded.split_whitespace().collect();
        String::from_utf8(STANDARD.decode(packed).expect("a fixture that decodes"))
            .expect("armour is text")
    }

    /// A store with no key in it, which is every fresh installation.
    ///
    /// The credential store is a thread-local map under test, so each test
    /// starts empty without doing anything, and nothing here ever writes into
    /// the credential store of whoever is running the tests.
    fn with_no_key() {
        secret_store::allow();
    }

    fn with_alices_key() {
        with_no_key();
        assert_eq!(
            import(&armour(ALICE_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );
    }

    #[test]
    fn test_a_message_encrypted_to_the_imported_key_opens_and_reads() {
        // The whole of what this task is for, and the fixtures are the reason
        // it is worth anything: the key and the message were both made by
        // GnuPG, so this is two implementations agreeing rather than one
        // agreeing with itself.
        with_alices_key();

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::Opened("The meeting moved to Thursday at ten.\n".to_string())
        );
    }

    #[test]
    fn test_an_ordinary_rsa_key_is_stored_and_opens_mail() {
        // Alice's key is RSA-2048 with no subkey, 1,836 characters armoured,
        // and Windows keeps 1,280 in one credential. Stored in one entry it
        // imported cleanly under test and failed on every real Windows
        // machine, so every entry written is held to what Windows keeps.
        with_no_key();
        let key = armour(ALICE_PRIVATE);
        assert!(
            key.encode_utf16().count() > 1_280,
            "the fixture no longer shows the case: it fits in one entry"
        );

        assert_eq!(import(&key), WhatImportingAKeyFound::Imported);

        for (user, secret) in secret_store::entries_under(KEYRING_SERVICE) {
            assert!(
                secret.encode_utf16().count() <= 1_280,
                "{user} holds {} UTF-16 units, more than Windows keeps in one entry",
                secret.encode_utf16().count()
            );
        }
        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::Opened("The meeting moved to Thursday at ten.\n".to_string())
        );
    }

    #[test]
    fn test_with_no_key_at_all_it_says_so_rather_than_failing() {
        // Every fresh installation. "There is no key here" is a different
        // thing to be told from "your key does not open this", and the
        // difference is what somebody does next.
        with_no_key();

        assert_eq!(open(&armour(TO_ALICE)), WhatOpeningItFound::NoKeyHere);
    }

    #[test]
    fn test_a_key_that_does_not_open_it_is_not_reported_as_no_key() {
        // The message was meant for somebody else, which is news about the
        // message rather than about this program's setup. Reported as "no key"
        // it would send somebody looking for a key they already have.
        with_no_key();
        assert_eq!(
            import(&armour(BOB_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::TheKeyHereDoesNotOpenIt
        );
    }

    #[test]
    fn test_damaged_armour_says_it_is_damaged_and_does_not_panic() {
        // A stranger's bytes into a new cryptographic parser. rPGP had two
        // CVEs of this shape in 2024, both fixed, which says what to test:
        // every prefix of a real message, the armour with its footer gone, and
        // something that was never armour at all.
        with_alices_key();
        let whole = armour(TO_ALICE);

        for cut in (0..whole.len()).step_by(11) {
            let truncated = &whole[..cut];
            assert!(
                matches!(
                    open(truncated),
                    WhatOpeningItFound::Damaged | WhatOpeningItFound::TheKeyHereDoesNotOpenIt
                ),
                "a {cut} byte prefix was read as something else"
            );
        }
        for rubbish in [
            "",
            "hello there",
            "-----BEGIN PGP MESSAGE-----\n\nnot base64\n",
        ] {
            assert_eq!(open(rubbish), WhatOpeningItFound::Damaged, "{rubbish:?}");
        }
    }

    #[test]
    fn test_a_message_whose_middle_was_corrupted_does_not_come_back_as_text() {
        // Not truncation: a message whose armour is whole and whose contents
        // are wrong. It must not open, and it must not panic.
        with_alices_key();
        let whole = armour(TO_ALICE);
        let corrupted = whole.replacen('h', "H", 1);

        assert_ne!(corrupted, whole, "the fixture did not change");
        assert!(
            !matches!(open(&corrupted), WhatOpeningItFound::Opened(_)),
            "a corrupted message opened"
        );
    }

    #[test]
    fn test_a_public_key_is_refused_by_name_and_nothing_is_stored() {
        // The mistake somebody makes at three in the morning. Stored under the
        // private key's name it would open nothing, for ever, and every
        // message would report the wrong reason.
        with_no_key();

        assert_eq!(
            import(&armour(ALICE_PUBLIC)),
            WhatImportingAKeyFound::NotAPrivateKey
        );
        assert!(!a_key_is_here(), "a public key was stored");
    }

    #[test]
    fn test_something_that_is_not_a_key_at_all_is_refused_separately() {
        // A different sentence from the one above, because a different thing
        // went wrong: the first is the right kind of file and the wrong half
        // of the key, the second is the wrong file.
        with_no_key();

        for not_a_key in ["", "hello", "-----BEGIN PGP MESSAGE-----\n\nx\n"] {
            assert_eq!(
                import(not_a_key),
                WhatImportingAKeyFound::NotAKey,
                "{not_a_key:?}"
            );
        }
        assert!(!a_key_is_here());
    }

    #[test]
    fn test_a_refusal_never_carries_anything_out_of_the_file() {
        // A key file's contents are the highest-value secret this program
        // handles and an error message is a log line nobody wrote. Checked
        // against the file's own bytes rather than against a shape, so a
        // refusal that quoted the key would be caught whatever it quoted.
        with_no_key();
        let key = armour(ALICE_PRIVATE);

        let refused = format!("{:?}", import(&key));

        for line in key.lines().filter(|line| line.len() > 20) {
            assert!(!refused.contains(line), "a refusal quoted the key file");
        }
    }

    #[test]
    fn test_every_entry_a_key_is_stored_under_is_one_uninstalling_erases() {
        // The names are permanent and the uninstaller names them without
        // reading anything. A part filed anywhere else is a piece of a
        // private key left on the machine after the program is gone.
        with_no_key();

        assert_eq!(
            import(&armour(ALICE_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );

        let written = secret_store::entries_under(KEYRING_SERVICE);
        assert!(!written.is_empty(), "nothing was written");
        let erased = super::super::keyring_entries();
        for (user, _) in &written {
            assert!(
                erased.contains(&(KEYRING_SERVICE.to_string(), user.clone())),
                "{user} was written and uninstalling does not name it"
            );
        }
        assert!(a_key_is_here());
    }

    #[test]
    fn test_a_key_an_older_build_kept_under_one_name_moves_into_parts_and_still_opens() {
        // Before keys were split, one key lived under `private-key`. On a
        // Windows machine only a key of 1,280 characters or fewer could have
        // been kept there, which is an elliptic-curve key like Carol's. It
        // has to go on opening mail, and it has to end up where the rest of
        // this module looks, with the old entry gone.
        with_no_key();
        let carol = armour(CAROL_PRIVATE);
        secret_store::write(KEYRING_SERVICE, KEYRING_PRIVATE_KEY, &carol)
            .expect("an older build's entry");

        assert_eq!(
            open(&armour(TO_CAROL)),
            WhatOpeningItFound::Opened("Carol, the key fits in one entry.\n".to_string())
        );
        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE),
            vec![("key-1-part-1".to_string(), carol)],
            "the older entry was not moved into the first slot"
        );
    }

    #[test]
    fn test_a_key_longer_than_its_parts_hold_is_refused_and_nothing_is_written() {
        // Cut to fit, a key would be stored as whole and never open anything,
        // and every message afterwards would report the wrong reason.
        with_no_key();
        let most = super::super::PARTS_PER_KEY * 1_280;

        assert_eq!(keep(&"k".repeat(most + 1)), Err(NotKept::TooLarge));
        assert_eq!(secret_store::entries_under(KEYRING_SERVICE), vec![]);

        assert_eq!(keep(&"k".repeat(most)), Ok(()), "the longest a key may be");
        assert_eq!(keys_here().expect("the store"), vec!["k".repeat(most)]);
    }

    #[test]
    fn test_a_part_left_over_past_a_keys_last_is_removed_when_its_slot_is_used() {
        // A removal the store stopped partway leaves parts behind a slot that
        // reads as free. A shorter key written there would read back with the
        // old key's tail joined to it.
        with_no_key();
        for part in ["key-1-part-2", "key-1-part-3"] {
            secret_store::write(KEYRING_SERVICE, part, "left over").expect("a stale part");
        }

        assert_eq!(keep("fresh"), Ok(()));

        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE),
            vec![("key-1-part-1".to_string(), "fresh".to_string())]
        );
    }

    #[test]
    fn test_a_second_key_goes_to_the_next_free_slot_and_leaves_the_first_alone() {
        with_no_key();

        assert_eq!(keep("first"), Ok(()));
        assert_eq!(keep("second"), Ok(()));

        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE),
            vec![
                ("key-1-part-1".to_string(), "first".to_string()),
                ("key-2-part-1".to_string(), "second".to_string()),
            ]
        );
    }

    #[test]
    fn test_with_every_slot_taken_another_key_is_refused_and_nothing_is_written() {
        with_no_key();
        for which in 1..=super::super::KEY_SLOTS {
            assert_eq!(keep(&format!("key {which}")), Ok(()), "slot {which}");
        }

        assert_eq!(keep("one more"), Err(NotKept::NoRoomLeft));
        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE).len(),
            super::super::KEY_SLOTS
        );
    }

    #[test]
    fn test_a_key_refused_for_its_length_or_for_room_says_which_in_words() {
        // These finish "Your key could not be saved: ..." on the File menu,
        // so each has to say what was wrong and nothing about the key.
        assert!(
            NotKept::TooLarge.reason().contains("10,240 characters"),
            "{}",
            NotKept::TooLarge.reason()
        );
        assert!(
            NotKept::NoRoomLeft.reason().contains("8 private keys"),
            "{}",
            NotKept::NoRoomLeft.reason()
        );
        assert_eq!(
            NotKept::Refused("the store is locked".to_string()).reason(),
            "the store is locked"
        );
    }

    #[test]
    fn test_a_store_that_will_not_take_it_says_so_and_says_nothing_about_the_key() {
        // What a locked or missing credential store looks like from here. It
        // is the one refusal carrying words, and they are the store's reason
        // rather than anything out of the file.
        with_no_key();
        secret_store::refuse("the credential store is not available");

        match import(&armour(ALICE_PRIVATE)) {
            WhatImportingAKeyFound::CouldNotBeStored { reason } => {
                assert!(reason.contains("credential store"), "{reason}");
            }
            other => panic!("a refused store came back as {other:?}"),
        }
    }

    #[test]
    fn test_a_stored_key_that_cannot_be_read_back_is_not_reported_as_no_key() {
        // Rare and worth its own answer. Import stores only what parsed, so a
        // stored key that will not parse means the store handed back something
        // else. Saying "there is no key here" would be a lie about the one
        // thing somebody has already done.
        with_no_key();
        secret_store::write(KEYRING_SERVICE, KEYRING_PRIVATE_KEY, "not a key any more")
            .expect("the store");

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::TheKeyHereCouldNotBeRead
        );
    }

    // ── Several keys, and public ones ────────────────────────────────────
    //
    // Every expected listing below is what `gpg --list-keys --with-colons`
    // said about the same key in the home directory that made or imported
    // it, on 2026-09-27, rather than anything this file or the crate
    // computed. Alice's key is `ecEC` there: it encrypts and certifies and
    // does not sign, which is the one field a listing that answered yes to
    // everything would get wrong.

    fn at(when: &str) -> chrono::DateTime<chrono::Utc> {
        when.parse().expect("a time written in the test")
    }

    fn alice_as_gnupg_lists_her(private: bool) -> KeyListing {
        KeyListing {
            user_ids: vec!["Alice Example <alice@example.com>".to_string()],
            key_id: "A97E7BB74101FB3E".to_string(),
            fingerprint: "6EFD87D1527731DE679B8E1BA97E7BB74101FB3E".to_string(),
            created: at("2026-09-06T16:14:44Z"),
            expires: None,
            private,
            locked: false,
            can_encrypt: true,
            can_sign: false,
        }
    }

    fn carol_as_gnupg_lists_her(private: bool) -> KeyListing {
        KeyListing {
            user_ids: vec!["Carol Example <carol@example.com>".to_string()],
            key_id: "52B5C043A2C64173".to_string(),
            fingerprint: "8DE4DEEC367D086637934A1C52B5C043A2C64173".to_string(),
            created: at("2026-09-27T04:17:40Z"),
            expires: None,
            private,
            locked: false,
            can_encrypt: true,
            can_sign: true,
        }
    }

    #[test]
    fn test_a_key_is_described_in_this_programs_words() {
        assert_eq!(
            describe(&armour(ALICE_PRIVATE)),
            vec![alice_as_gnupg_lists_her(true)]
        );
        assert_eq!(
            describe(&armour(CAROL_PUBLIC)),
            vec![carol_as_gnupg_lists_her(false)]
        );
        assert_eq!(describe("hello"), vec![]);
    }

    #[test]
    fn test_one_text_holding_a_private_and_a_public_key_answers_for_each() {
        // What a file exported from another program often is: more than one
        // key, of both halves. Each gets its own answer; the private one goes
        // to the credential store and the public one is handed back to keep.
        with_no_key();
        let text = format!("{}\n{}", armour(ALICE_PRIVATE), armour(CAROL_PUBLIC));

        let answers = import_keys(&text);

        assert_eq!(answers.len(), 2, "{answers:?}");
        assert_eq!(
            answers[0],
            WhatBecameOfAKey::Imported(alice_as_gnupg_lists_her(true))
        );
        match &answers[1] {
            WhatBecameOfAKey::PublicKeyToKeep { armour, listing } => {
                assert_eq!(listing, &carol_as_gnupg_lists_her(false));
                assert_eq!(describe(armour), vec![carol_as_gnupg_lists_her(false)]);
            }
            other => panic!("Carol's public key came back as {other:?}"),
        }
        assert_eq!(
            private_keys_here().expect("the store"),
            vec![alice_as_gnupg_lists_her(true)]
        );
    }

    #[test]
    fn test_each_private_key_here_opens_its_own_mail() {
        // Two keys, each the only one a message was encrypted to. Imported in
        // this order, Carol's is not the first key here, which is the case a
        // reading that tried only the first would get wrong.
        with_no_key();
        assert_eq!(
            import(&armour(ALICE_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );
        assert_eq!(
            import(&armour(CAROL_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::Opened("The meeting moved to Thursday at ten.\n".to_string())
        );
        assert_eq!(
            open(&armour(TO_CAROL)),
            WhatOpeningItFound::Opened("Carol, the key fits in one entry.\n".to_string())
        );
    }

    #[test]
    fn test_a_private_key_imported_twice_is_already_here_the_second_time() {
        with_no_key();
        assert_eq!(
            import_keys(&armour(CAROL_PRIVATE)),
            vec![WhatBecameOfAKey::Imported(carol_as_gnupg_lists_her(true))]
        );

        assert_eq!(
            import_keys(&armour(CAROL_PRIVATE)),
            vec![WhatBecameOfAKey::AlreadyHere(carol_as_gnupg_lists_her(
                true
            ))]
        );
        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE).len(),
            1,
            "the second import wrote a second copy"
        );
    }

    fn dave_as_gnupg_lists_him() -> KeyListing {
        KeyListing {
            user_ids: vec!["Dave Example <dave@example.com>".to_string()],
            key_id: "469C95C000B5CB12".to_string(),
            fingerprint: DAVES_FINGERPRINT.to_string(),
            created: at("2026-09-27T04:53:06Z"),
            expires: None,
            private: true,
            locked: true,
            can_encrypt: true,
            can_sign: true,
        }
    }

    fn with_daves_locked_key() {
        with_no_key();
        assert_eq!(
            import_keys(&armour(DAVE_LOCKED)),
            vec![WhatBecameOfAKey::Imported(dave_as_gnupg_lists_him())]
        );
    }

    #[test]
    fn test_a_locked_key_in_a_text_of_several_is_kept_locked_beside_the_others() {
        // Dave's key has its passphrase on it. It is kept as it came, still
        // locked, because the passphrase is asked for when a message needs it
        // (13-17.1), and the key after it in the same text comes in as well.
        with_no_key();
        let text = format!("{}\n{}", armour(DAVE_LOCKED), armour(CAROL_PRIVATE));

        let answers = import_keys(&text);

        assert_eq!(
            answers,
            vec![
                WhatBecameOfAKey::Imported(dave_as_gnupg_lists_him()),
                WhatBecameOfAKey::Imported(carol_as_gnupg_lists_her(true)),
            ]
        );
        assert_eq!(
            private_keys_here().expect("the store"),
            vec![dave_as_gnupg_lists_him(), carol_as_gnupg_lists_her(true)]
        );
        assert_eq!(
            keys_here().expect("the store")[0],
            armour(DAVE_LOCKED),
            "the locked key was not kept byte for byte as it came"
        );
    }

    #[test]
    fn test_a_message_to_a_locked_key_says_whose_key_it_is() {
        // Nothing typed yet, so the key is still shut. The answer names the
        // key from the key itself, which is the only name here a stranger's
        // message cannot choose.
        with_daves_locked_key();

        assert_eq!(
            open(&armour(TO_DAVE)),
            WhatOpeningItFound::TheKeyIsLocked(super::super::LockedKey {
                whose: "Dave Example <dave@example.com>".to_string(),
                fingerprint: DAVES_FINGERPRINT.to_string(),
            })
        );
    }

    #[test]
    fn test_a_wrong_passphrase_opens_nothing_and_nothing_of_it_is_kept() {
        with_daves_locked_key();

        assert_eq!(
            unlock(DAVES_FINGERPRINT, "wrong"),
            Unlocking::WrongPassphrase
        );

        assert!(
            !a_passphrase_is_held_for(DAVES_FINGERPRINT),
            "a passphrase that opens nothing was kept"
        );
        assert!(
            matches!(
                open(&armour(TO_DAVE)),
                WhatOpeningItFound::TheKeyIsLocked(_)
            ),
            "the message no longer says the key is locked"
        );
    }

    #[test]
    fn test_the_right_passphrase_opens_the_message_for_the_rest_of_the_session() {
        with_daves_locked_key();

        assert_eq!(
            unlock(DAVES_FINGERPRINT, "correct horse"),
            Unlocking::Unlocked
        );

        let words = WhatOpeningItFound::Opened("Dave, the passphrase opened it.\n".to_string());
        assert_eq!(open(&armour(TO_DAVE)), words);
        assert_eq!(open(&armour(TO_DAVE)), words, "it was not remembered");
        assert!(a_passphrase_is_held_for(DAVES_FINGERPRINT));
        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE),
            vec![("key-1-part-1".to_string(), armour(DAVE_LOCKED))],
            "the passphrase, or the key opened by it, was written to the store"
        );
    }

    #[test]
    fn test_a_key_with_no_passphrase_opens_as_before_beside_an_unlocked_one() {
        // The held passphrase is offered to locked keys only; a key with none
        // is opened with the empty one, as it always was.
        with_daves_locked_key();
        assert_eq!(
            import(&armour(ALICE_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );
        assert_eq!(
            unlock(DAVES_FINGERPRINT, "correct horse"),
            Unlocking::Unlocked
        );

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::Opened("The meeting moved to Thursday at ten.\n".to_string())
        );
        assert!(!a_passphrase_is_held_for(ALICES_FINGERPRINT));
    }

    #[test]
    fn test_a_message_to_somebody_else_is_not_reported_as_locked() {
        // Dave's locked key is here and the message is Alice's. Asking for
        // Dave's passphrase would open nothing.
        with_daves_locked_key();

        assert_eq!(
            open(&armour(TO_ALICE)),
            WhatOpeningItFound::TheKeyHereDoesNotOpenIt
        );
    }

    #[test]
    fn test_unlocking_a_key_that_is_not_here_says_so_and_holds_nothing() {
        with_daves_locked_key();

        assert_eq!(unlock(ALICES_FINGERPRINT, "anything"), Unlocking::NoSuchKey);
        assert!(!a_passphrase_is_held_for(ALICES_FINGERPRINT));
    }

    #[test]
    fn test_removing_a_locked_key_forgets_the_passphrase_typed_for_it() {
        // Imported again later, the key asks again: the passphrase went with
        // the key.
        with_daves_locked_key();
        assert_eq!(
            unlock(DAVES_FINGERPRINT, "correct horse"),
            Unlocking::Unlocked
        );

        assert!(remove_private_key(DAVES_FINGERPRINT).expect("the store"));

        assert!(!a_passphrase_is_held_for(DAVES_FINGERPRINT));
    }

    #[test]
    fn test_no_public_item_of_this_module_hands_out_a_passphrase() {
        // What is held is the crate's own passphrase type, and the only way
        // to it is through this module's functions, none of which gives it
        // back. A `pub` line naming the type would be a way out.
        fn public_lines_naming_the_type(source: &str) -> Vec<String> {
            source
                .lines()
                .map(str::trim_start)
                .filter(|line| line.starts_with("pub ") || line.starts_with("pub("))
                .filter(|line| line.contains("Password"))
                .map(str::to_string)
                .collect()
        }
        let mut found = Vec::new();
        for file in ["src/service/pgp/mod.rs", "src/service/pgp/keys.rs"] {
            let source = std::fs::read_to_string(file).expect("the module's source");
            found.extend(public_lines_naming_the_type(&source));
        }

        assert_eq!(found, Vec::<String>::new());
        // The companion: the reading sees such a line where there is one.
        assert_eq!(
            public_lines_naming_the_type("    pub fn held() -> Password {"),
            vec!["pub fn held() -> Password {".to_string()]
        );
    }

    #[test]
    fn test_removing_a_private_key_takes_every_part_of_it_and_leaves_the_others() {
        with_no_key();
        import(&armour(ALICE_PRIVATE));
        import(&armour(CAROL_PRIVATE));

        assert!(remove_private_key("6EFD87D1527731DE679B8E1BA97E7BB74101FB3E").expect("the store"));

        assert_eq!(
            private_keys_here().expect("the store"),
            vec![carol_as_gnupg_lists_her(true)]
        );
        assert_eq!(
            secret_store::entries_under(KEYRING_SERVICE),
            vec![("key-2-part-1".to_string(), armour(CAROL_PRIVATE))],
            "a part of Alice's key was left behind"
        );
        assert!(
            !remove_private_key("6EFD87D1527731DE679B8E1BA97E7BB74101FB3E").expect("the store"),
            "a key that is not here was reported removed"
        );
    }

    #[test]
    fn test_the_public_half_of_a_key_is_armour_that_describes_as_public() {
        with_no_key();
        import(&armour(ALICE_PRIVATE));

        let exported = public_half_of_a_key_here("6EFD87D1527731DE679B8E1BA97E7BB74101FB3E")
            .expect("the store")
            .expect("Alice's key is here");

        assert_eq!(describe(&exported), vec![alice_as_gnupg_lists_her(false)]);
        assert!(
            exported.contains("BEGIN PGP PUBLIC KEY BLOCK"),
            "{exported}"
        );
        assert!(
            !exported.contains("PRIVATE"),
            "the private half was handed out"
        );
        assert_eq!(
            public_half_of_a_key_here("8DE4DEEC367D086637934A1C52B5C043A2C64173")
                .expect("the store"),
            None,
            "a key that is not here had a public half"
        );
        let carols = public_half(&armour(CAROL_PUBLIC)).expect("Carol's public key");
        assert_eq!(describe(&carols), vec![carol_as_gnupg_lists_her(false)]);
    }
}
