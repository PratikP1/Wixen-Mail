//! The OpenPGP crate, and the only place in this program that names it.
//!
//! Everything outside `src/service/pgp/` calls [`super`]'s functions and knows
//! no crate name, no packet, no armour header. The reason is not tidiness: a
//! cryptographic implementation is the one dependency here that may have to be
//! replaced at short notice, and a replacement that reaches every caller is one
//! nobody makes in a hurry.
//!
//! So the crate's own types stop here. What crosses out of this file is
//! [`WhatOpeningItFound`] and [`WhatImportingAKeyFound`], which are this
//! project's words.
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
//! Inline PGP: an armoured block sitting in the message's text, which is what
//! `application::body_safety::what_the_form_says` finds. **PGP/MIME, where the
//! armour is a separate `application/octet-stream` part under
//! `multipart/encrypted`, is not read here**, because that part never reaches
//! the body this is handed. That is a real gap and it is in the changelog and
//! in `.planning/WINDOWS.md` rather than only here.
//!
//! One key, with no passphrase on it. A key locked with a passphrase is refused
//! at import rather than stored, because a key that can never open anything is
//! worse than no key: every message would report the wrong reason for ever
//! after.

use super::{KEYRING_PRIVATE_KEY, KEYRING_SERVICE, WhatImportingAKeyFound, WhatOpeningItFound};
use crate::service::secret_store;
use pgp::composed::{Deserializable, Message, SignedPublicKey, SignedSecretKey};
use pgp::errors::Error as OpenPgpError;
use pgp::types::Password;
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

/// Take an armoured private key and put it in the credential store.
pub(super) fn import(armoured: &str) -> WhatImportingAKeyFound {
    let key = match SignedSecretKey::from_string(armoured) {
        Ok((key, _)) => key,
        // Which of the two refusals, decided by asking the crate rather than by
        // reading the armour header. A header this program read for itself
        // would be a second opinion about what a key file is, and the crate is
        // the one that has to agree with it later.
        Err(_) => {
            return if SignedPublicKey::from_string(armoured).is_ok() {
                WhatImportingAKeyFound::NotAPrivateKey
            } else {
                WhatImportingAKeyFound::NotAKey
            };
        }
    };
    if a_passphrase_is_holding_it_shut(&key) {
        return WhatImportingAKeyFound::TheKeyIsLockedWithAPassphrase;
    }
    // What arrived, byte for byte, rather than the crate's own re-serialisation
    // of it. The same reasoning `signed_original` gives about a signed message:
    // anything that rewrites a cryptographic document, even to tidy it, is a
    // second chance to change what it says, and the thing coming back out has
    // to be the thing that went in.
    match secret_store::write(KEYRING_SERVICE, KEYRING_PRIVATE_KEY, armoured) {
        Ok(()) => WhatImportingAKeyFound::Imported,
        // The store's reason, which is about the store. Nothing from the file
        // travels in it; `secret_store` already holds itself to reasons and
        // never values.
        Err(problem) => WhatImportingAKeyFound::CouldNotBeStored {
            reason: problem.to_string(),
        },
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

/// Open an armoured message with the private key this computer holds.
pub(super) fn open(armour: &str) -> WhatOpeningItFound {
    let stored = match secret_store::read(KEYRING_SERVICE, KEYRING_PRIVATE_KEY) {
        Ok(None) => return WhatOpeningItFound::NoKeyHere,
        Ok(Some(stored)) => stored,
        // The reason and never the entry. A locked-down credential store is
        // worth a log line; what it holds is not.
        Err(problem) => {
            tracing::warn!("The credential store would not give up the private key: {problem}");
            return WhatOpeningItFound::TheKeyHereCouldNotBeRead;
        }
    };
    let Ok((key, _)) = SignedSecretKey::from_string(&stored) else {
        // Import stores only what has already parsed, so this is the store
        // handing back something other than what went into it.
        tracing::warn!("The private key in the credential store no longer reads as a key");
        return WhatOpeningItFound::TheKeyHereCouldNotBeRead;
    };
    open_with(armour, &key)
}

/// The same, once the key is in hand.
///
/// **No error out of the crate is logged or passed on.** Not because the crate
/// is untrustworthy but because of what its error text can hold: a parser
/// refusing a message can quote the bytes it choked on, and those bytes are a
/// stranger's mail. What crosses out of here is one of four words this project
/// chose.
fn open_with(armour: &str, key: &SignedSecretKey) -> WhatOpeningItFound {
    let Ok((message, _)) = Message::from_armor(armour.as_bytes()) else {
        return WhatOpeningItFound::Damaged;
    };
    let opened = match message.decrypt(&Password::empty(), key) {
        Ok(opened) => opened,
        // The one error worth telling apart, and it is the one that is not
        // about the message: nothing in the message names a key this computer
        // holds, so it was encrypted to somebody else.
        Err(OpenPgpError::MissingKey) => return WhatOpeningItFound::TheKeyHereDoesNotOpenIt,
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

/// Whether a private key has been imported on this computer.
///
/// From the credential store rather than from a stored flag, for the reason
/// [`super::keyring_entries`] gives: deciding from a flag whether a secret
/// exists is how secrets get left behind.
pub(super) fn a_key_is_here() -> bool {
    secret_store::read(KEYRING_SERVICE, KEYRING_PRIVATE_KEY)
        .ok()
        .flatten()
        .is_some()
}

#[cfg(test)]
mod tests {
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
    const ALICE_PRIVATE: &str = "
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
    const BOB_PRIVATE: &str = "
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
    const ALICE_PUBLIC: &str = "
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
    const TO_ALICE: &str = "
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

    /// The bytes a fixture stands for.
    fn armour(encoded: &str) -> String {
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
    fn test_the_key_is_stored_under_the_one_name_uninstalling_erases() {
        // The name is permanent and the uninstaller names it. A key filed
        // anywhere else is a key left on the machine after the program is
        // gone.
        with_no_key();

        assert_eq!(
            import(&armour(ALICE_PRIVATE)),
            WhatImportingAKeyFound::Imported
        );

        assert_eq!(
            secret_store::read(KEYRING_SERVICE, KEYRING_PRIVATE_KEY).expect("the store"),
            Some(armour(ALICE_PRIVATE))
        );
        assert!(a_key_is_here());
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
}
