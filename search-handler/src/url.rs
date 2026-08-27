//! The URL scheme the indexer uses to name one thing in the store.

/// The name of our scheme, and the name the handler is registered under.
pub const SCHEME: &str = "wixen-mail";

/// The host part every one of our URLs carries.
const HOST: &str = "localhost";

/// Why a URL could not be read.
///
/// One flat reason rather than a string, so nothing that came out of somebody's
/// mailbox can be carried into a message. A folder name is in the URL, and a
/// folder name can be private.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlError {
    /// The scheme was missing or belonged to somebody else.
    NotOurScheme,
    /// The host was missing or named another machine.
    NotThisMachine,
    /// There were more path parts than the store has levels, or an empty one.
    Malformed,
    /// The last part should have been a message number and was not.
    BadUid,
}

/// One place in the store, named by a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemUrl {
    /// Which signed-in user the indexer should read this as, when it says.
    ///
    /// Microsoft's URL shape puts an optional user security identifier ahead of
    /// the host so the indexer knows whose data it is looking at. It is kept
    /// rather than dropped so a URL survives a round trip unchanged.
    pub user: Option<String>,
    /// Which level of the store the URL points at.
    pub place: Place,
}

/// How deep into the store a URL reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// Every account. The indexer starts here.
    Root,
    /// Every folder in one account.
    Account { account: String },
    /// Every message in one folder.
    Folder { account: String, folder: String },
    /// One message.
    Message {
        account: String,
        folder: String,
        uid: u32,
    },
}

impl ItemUrl {
    /// Read a URL the indexer handed us.
    pub fn parse(text: &str) -> Result<Self, UrlError> {
        let rest = text
            .strip_prefix(SCHEME)
            .and_then(|rest| rest.strip_prefix("://"))
            .ok_or(UrlError::NotOurScheme)?;

        let mut parts = rest.split('/');
        let first = parts.next().unwrap_or_default();
        let (user, host) = match first.starts_with('{') {
            true => (
                Some(checked_user(first)?),
                parts.next().ok_or(UrlError::NotThisMachine)?,
            ),
            false => (None, first),
        };
        if !host.eq_ignore_ascii_case(HOST) {
            return Err(UrlError::NotThisMachine);
        }

        // The indexer trims the final slash, so a URL can arrive with or
        // without one and both mean the same place. Only the last part may be
        // empty; an empty one anywhere else is a missing name, not a tidy end.
        let mut segments: Vec<&str> = parts.collect();
        if segments.last() == Some(&"") {
            segments.pop();
        }
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(UrlError::Malformed);
        }

        let place = match segments.as_slice() {
            [] => Place::Root,
            [account] => Place::Account {
                account: unescape(account)?,
            },
            [account, folder] => Place::Folder {
                account: unescape(account)?,
                folder: unescape(folder)?,
            },
            [account, folder, uid] => Place::Message {
                account: unescape(account)?,
                folder: unescape(folder)?,
                uid: uid.parse().map_err(|_| UrlError::BadUid)?,
            },
            _ => return Err(UrlError::Malformed),
        };

        Ok(Self { user, place })
    }
}

impl std::fmt::Display for ItemUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SCHEME}://")?;
        if let Some(user) = &self.user {
            write!(f, "{user}/")?;
        }
        write!(f, "{HOST}")?;

        // Every name here came from a mail server, so every name here is
        // escaped. The uid is a number this code produced and needs nothing.
        match &self.place {
            Place::Root => Ok(()),
            Place::Account { account } => write!(f, "/{}", escape(account)),
            Place::Folder { account, folder } => {
                write!(f, "/{}/{}", escape(account), escape(folder))
            }
            Place::Message {
                account,
                folder,
                uid,
            } => write!(f, "/{}/{}/{uid}", escape(account), escape(folder)),
        }
    }
}

/// Accept a user identifier only if it could be one.
///
/// The braces are the whole signal that separates a user from an account name,
/// so what is inside them has to be checked. A security identifier is the
/// letter S, digits and hyphens, and nothing here needs to understand more of
/// its shape than that.
fn checked_user(text: &str) -> Result<String, UrlError> {
    let inside = text
        .strip_prefix('{')
        .and_then(|text| text.strip_suffix('}'))
        .ok_or(UrlError::Malformed)?;

    let looks_like_one = !inside.is_empty()
        && inside
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-');

    match looks_like_one {
        true => Ok(text.to_string()),
        false => Err(UrlError::Malformed),
    }
}

/// Characters that may stand for themselves in a path part.
///
/// The unreserved set from RFC 3986. Everything else is escaped, which is
/// wider than strictly necessary and is the safe direction: a folder name
/// comes from a mail server, and the cost of escaping a character that did not
/// need it is nothing, while the cost of missing one is a URL that means a
/// different place.
fn stands_for_itself(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Write one name so it can only ever be one path part.
fn escape(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for byte in name.bytes() {
        match stands_for_itself(byte) {
            true => out.push(byte as char),
            false => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Read one path part back into the name it came from.
///
/// A damaged escape is an error rather than a character left as it was.
/// Half-decoding produces a name that looks plausible and matches no row, and
/// that failure is much harder to recognise than a refusal.
fn unescape(part: &str) -> Result<String, UrlError> {
    let bytes = part.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            out.push(bytes[index]);
            index += 1;
            continue;
        }
        let digits = part.get(index + 1..index + 3).ok_or(UrlError::Malformed)?;
        out.push(u8::from_str_radix(digits, 16).map_err(|_| UrlError::Malformed)?);
        index += 3;
    }

    String::from_utf8(out).map_err(|_| UrlError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_message_url_names_the_account_the_folder_and_the_uid() {
        // The whole scheme exists to turn one opaque string from the indexer
        // back into a row nobody else can find. If this loses a part, the
        // handler either reads the wrong message or reads none.
        let parsed = ItemUrl::parse("wixen-mail://localhost/work/INBOX/4211")
            .expect("a plain message url should parse");

        assert_eq!(parsed.user, None);
        assert_eq!(
            parsed.place,
            Place::Message {
                account: "work".to_string(),
                folder: "INBOX".to_string(),
                uid: 4211,
            }
        );
    }

    #[test]
    fn test_the_four_levels_of_the_store_are_told_apart_by_how_deep_the_url_goes() {
        // The indexer walks down from the root, asking at each level for the
        // children below it. Reading a folder url as an account, or the other
        // way round, would either enumerate the wrong list or enumerate
        // nothing, and enumerating nothing looks exactly like an empty mailbox.
        let cases = [
            ("wixen-mail://localhost", Place::Root),
            (
                "wixen-mail://localhost/work",
                Place::Account {
                    account: "work".to_string(),
                },
            ),
            (
                "wixen-mail://localhost/work/INBOX",
                Place::Folder {
                    account: "work".to_string(),
                    folder: "INBOX".to_string(),
                },
            ),
            (
                "wixen-mail://localhost/work/INBOX/7",
                Place::Message {
                    account: "work".to_string(),
                    folder: "INBOX".to_string(),
                    uid: 7,
                },
            ),
        ];

        for (text, expected) in cases {
            let parsed = ItemUrl::parse(text).unwrap_or_else(|e| panic!("{text} gave {e:?}"));
            assert_eq!(parsed.place, expected, "{text}");
        }
    }

    #[test]
    fn test_a_user_identifier_ahead_of_the_host_is_kept_rather_than_dropped() {
        // Microsoft's url shape allows a security identifier before the host so
        // the indexer knows whose mail it is reading. Dropping it would hand
        // back a url that no longer names the same thing, and the indexer
        // compares urls to decide what it has already seen.
        let parsed = ItemUrl::parse("wixen-mail://{S-1-5-21-99-1001}/localhost/work/INBOX/7")
            .expect("a url with a user should parse");

        assert_eq!(parsed.user.as_deref(), Some("{S-1-5-21-99-1001}"));
        assert_eq!(
            parsed.place,
            Place::Message {
                account: "work".to_string(),
                folder: "INBOX".to_string(),
                uid: 7,
            }
        );
    }

    #[test]
    fn test_a_trailing_slash_is_ignored_because_the_indexer_strips_one_anyway() {
        // Microsoft's page says the indexer trims the final slash, so a
        // handler cannot use one to tell a folder from an item. Treating the
        // empty tail as a real path part would turn every account url into a
        // malformed folder url.
        assert_eq!(
            ItemUrl::parse("wixen-mail://localhost/work/")
                .expect("trailing slash")
                .place,
            Place::Account {
                account: "work".to_string()
            }
        );
        assert_eq!(
            ItemUrl::parse("wixen-mail://localhost/")
                .expect("trailing slash")
                .place,
            Place::Root
        );
    }

    #[test]
    fn test_a_folder_whose_name_contains_a_slash_cannot_forge_a_deeper_url() {
        // Gmail really does call a folder "[Gmail]/All Mail", and folder names
        // arrive from a server we do not control. If the slash were written
        // out raw, that one folder would read back as an account called
        // "[Gmail]" holding a folder called "All Mail", and a message in it
        // would be looked up in a folder that does not exist. Worse, a server
        // could name a folder to point the handler at a different account.
        let awkward = ItemUrl {
            user: None,
            place: Place::Folder {
                account: "work".to_string(),
                folder: "[Gmail]/All Mail".to_string(),
            },
        };

        let written = awkward.to_string();
        assert!(!written.ends_with("All Mail"), "{written}");
        assert_eq!(ItemUrl::parse(&written).expect("round trip"), awkward);
    }

    #[test]
    fn test_every_level_survives_being_written_out_and_read_back() {
        // The handler writes urls when it enumerates children and reads them
        // when the indexer comes back for one. Those two have to agree, and
        // they are far enough apart in the code to drift.
        let places = [
            Place::Root,
            Place::Account {
                account: "work account".to_string(),
            },
            Place::Folder {
                account: "work account".to_string(),
                folder: "Wichtige E-Mails".to_string(),
            },
            Place::Message {
                account: "work account".to_string(),
                folder: "Wichtige E-Mails".to_string(),
                uid: u32::MAX,
            },
        ];

        for user in [None, Some("{S-1-5-21-99-1001}".to_string())] {
            for place in places.clone() {
                let original = ItemUrl {
                    user: user.clone(),
                    place,
                };
                let written = original.to_string();
                assert_eq!(
                    ItemUrl::parse(&written).unwrap_or_else(|e| panic!("{written} gave {e:?}")),
                    original,
                    "{written}"
                );
            }
        }
    }

    #[test]
    fn test_a_message_number_that_is_not_a_number_is_refused() {
        // A uid is a column the lookup goes straight into. Anything that is
        // not a plain number is somebody else's url or a mangled one, and
        // guessing at it would query for a message that was never asked for.
        for text in [
            "wixen-mail://localhost/work/INBOX/not-a-number",
            "wixen-mail://localhost/work/INBOX/-1",
            "wixen-mail://localhost/work/INBOX/4294967296",
            "wixen-mail://localhost/work/INBOX/ 7",
        ] {
            assert_eq!(ItemUrl::parse(text), Err(UrlError::BadUid), "{text}");
        }
    }

    #[test]
    fn test_a_url_deeper_than_the_store_goes_is_refused() {
        // There is no level below a message. A url with a fifth part is not
        // ours however much of it looks familiar, and answering it would mean
        // inventing a place.
        assert_eq!(
            ItemUrl::parse("wixen-mail://localhost/work/INBOX/7/attachment"),
            Err(UrlError::Malformed)
        );
    }

    #[test]
    fn test_an_empty_part_in_the_middle_is_refused_rather_than_collapsed() {
        // An empty account or folder name matches no row. Collapsing it would
        // silently shift every part along by one and look up a real message
        // under the wrong name.
        for text in [
            "wixen-mail://localhost//INBOX/7",
            "wixen-mail://localhost/work//7",
        ] {
            assert_eq!(ItemUrl::parse(text), Err(UrlError::Malformed), "{text}");
        }
    }

    #[test]
    fn test_a_broken_escape_is_refused_rather_than_half_decoded() {
        // Percent decoding is the one place a url turns back into text. A
        // truncated or non-hexadecimal escape means the url was damaged, and
        // half decoding it produces a name that looks plausible and matches
        // nothing.
        for text in [
            "wixen-mail://localhost/work/%",
            "wixen-mail://localhost/work/%2",
            "wixen-mail://localhost/work/%zz",
            // Valid escapes that do not spell valid text once joined up.
            "wixen-mail://localhost/work/%FF%FE",
        ] {
            assert_eq!(ItemUrl::parse(text), Err(UrlError::Malformed), "{text}");
        }
    }

    #[test]
    fn test_a_user_identifier_can_only_look_like_one() {
        // The braces are what tells a user identifier apart from an account
        // name, so anything inside them that is not a security identifier is a
        // url built by something other than this handler.
        for text in [
            "wixen-mail://{not a sid}/localhost/work",
            "wixen-mail://{unclosed/localhost/work",
            "wixen-mail://{}/localhost/work",
        ] {
            assert_eq!(ItemUrl::parse(text), Err(UrlError::Malformed), "{text}");
        }
    }

    #[test]
    fn test_the_scheme_is_the_one_name_registration_and_parsing_share() {
        // The name written into the registry and the name accepted here have
        // to be the same word. They are read by different code at different
        // times, and a mismatch shows up as an indexer that never calls this
        // handler at all, which looks identical to it not being installed.
        assert_eq!(SCHEME, "wixen-mail");
        assert!(
            ItemUrl::parse(&format!("{SCHEME}://localhost")).is_ok(),
            "the scheme constant did not parse as our own scheme"
        );
    }

    #[test]
    fn test_a_url_that_is_not_ours_is_refused_rather_than_guessed_at() {
        // The indexer hands a handler only its own scheme, but this is the
        // outer boundary of a DLL loaded into somebody else's process, so it
        // checks rather than assuming.
        assert!(ItemUrl::parse("file:///C:/mail/INBOX/4211").is_err());
        assert!(ItemUrl::parse("mapi://localhost/work/INBOX/4211").is_err());
        assert!(ItemUrl::parse("wixen-mail:/localhost/work").is_err());
        assert!(ItemUrl::parse("").is_err());
    }
}
