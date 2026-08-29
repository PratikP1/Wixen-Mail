//! Reading mail that was signed or encrypted with S/MIME.
//!
//! # Where the line between portable and platform code falls, and why
//!
//! Almost none of this needs an operating system. Taking a signed message
//! apart, reading the certificate that came with it, checking that the
//! signature adds up against that certificate, and deciding what any of it
//! *means* are the same questions everywhere: they are arithmetic and a
//! document format, and neither changes when the window manager does. So all
//! of that is plain Rust with no `cfg` on it, and this project's later move to
//! macOS carries it across untouched.
//!
//! One thing is platform-bound: everything that needs the store the operating
//! system owns, holds the only keys to, and exposes through its own API. That
//! is [`CertificateStore`], and it is the whole of the boundary. A macOS port
//! writes one implementation of that trait against Keychain and Security
//! framework, and nothing else in this file changes.
//!
//! The three questions on that boundary are worth naming, because they are the
//! reason the line is where it is rather than a little further along:
//!
//! - **Does this computer trust whoever issued the certificate?** Trust is a
//!   list, the list belongs to the machine, and it is different on every
//!   machine. Nothing portable can answer it.
//! - **Has the certificate been withdrawn since it was issued?** The withdrawal
//!   lists this computer already holds are the operating system's, and so is
//!   the machinery for fetching a new one. See [`Withdrawal`] and [`Reach`].
//! - **Does this computer hold the private key a message was encrypted to?**
//!   The key is in the store and normally cannot be taken out of it, so the
//!   operation has to happen where the key is.
//!
//! # What this file will not claim
//!
//! A signature that adds up proves two things and no more: the content was not
//! changed after it was signed, and whoever signed it held the private key that
//! goes with the certificate attached. It does not prove that the sender is the
//! person the display name says, or that anybody checked who the certificate
//! was issued to.
//!
//! It does not say when the message was signed either, unless a timestamp
//! authority signed a statement of the moment and that statement checks out.
//! The sender's own claimed signing time is ignored on purpose: the sender
//! writes it.
//!
//! Whether the certificate has been withdrawn is a question this can now ask,
//! and the answer has five shapes rather than two, because "nobody asked" and
//! "somebody asked and could not find out" are not good news and must not be
//! worded as though they were.
//!
//! People read "signed" as "genuine", which is exactly the confusion somebody
//! forging mail is counting on, so every sentence this file produces says which
//! of the two it means. See [`SignatureReport::limits`], which is said every
//! time and not only when something is wrong.

use crate::common::{Error, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use x509_parser::prelude::*;

// ── What shape of S/MIME a part is in ────────────────────────────────────────

/// The shape of S/MIME a message part is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmimeLayout {
    /// The words are readable without any crypto and the signature sits beside
    /// them in a second part, named by the boundary that separates the two.
    SignatureBeside { boundary: String },
    /// The words are wrapped inside the signature, so nothing can be read until
    /// the signature is taken apart.
    SignatureAround,
    /// Encrypted. Nothing can be read without a private key.
    Encrypted,
}

/// Which layout a `Content-Type` header value describes, if any.
///
/// `None` for ordinary mail, which is nearly all of it, so this is the cheap
/// first question rather than something that has to take a message apart before
/// finding out it had nothing to do.
/// Whether a whole message claims a signature at all, from its headers alone.
///
/// The cheap first question, asked of the raw message rather than of a header
/// somebody has already pulled out. Nearly no mail is signed, so a reader
/// asking this on every message has to be able to say no without taking
/// anything apart.
///
/// It exists because the alternative was worse. Reading the content type out
/// of a message is already done here, behind functions this module keeps to
/// itself; a caller doing it a second time would be a second answer to one
/// question, and the pair in this program that did drift disagreed about a
/// quoted value and one of them was wrong. So the question is answered here,
/// once, and the header reading stays private.
pub fn claims_a_signature(raw: &[u8]) -> bool {
    let (headers, _) = split_headers_from_body(raw);
    // A signature and not merely S/MIME. `layout_of` also recognises encrypted
    // mail, and answering yes for that sends an encrypted message down the
    // signature path, where `take_apart` refuses it and every surface downstream
    // ends up saying "it says it is signed, but it carries no signature to
    // check" about a message that never said anything of the kind.
    matches!(
        header_value(headers, "content-type").and_then(|content_type| layout_of(&content_type)),
        Some(SmimeLayout::SignatureBeside { .. } | SmimeLayout::SignatureAround)
    )
}

pub fn layout_of(content_type: &str) -> Option<SmimeLayout> {
    let header = ContentType::read(content_type);
    match header.media_type.as_str() {
        // The protocol parameter is what says this multipart/signed is S/MIME
        // rather than PGP, which uses the same wrapper with a different
        // protocol. Getting that wrong means handing a PGP signature to a
        // reader that only knows about certificates.
        "multipart/signed" => {
            let protocol = header.parameter("protocol").unwrap_or_default();
            if !is_pkcs7(&protocol, "signature") {
                return None;
            }
            let boundary = header.parameter("boundary")?;
            Some(SmimeLayout::SignatureBeside { boundary })
        }
        // Folded to lower case before it is compared, the same as the media
        // type above it, the protocol inside `is_pkcs7` and the file suffix
        // below. This was the one value that was not, so a sender writing
        // `smime-type=Signed-Data` had their signed message read as not S/MIME
        // at all: the silent way to be wrong.
        media if is_pkcs7(media, "mime") => match header
            .parameter("smime-type")
            .map(|kind| kind.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("signed-data") => Some(SmimeLayout::SignatureAround),
            Some("enveloped-data") => Some(SmimeLayout::Encrypted),
            // Senders do leave smime-type off. The file name is the only other
            // thing that says which this is, and being wrong about it means
            // trying to read a signature as an envelope.
            _ => match header
                .parameter("name")
                .map(|name| file_suffix(&name))
                .as_deref()
            {
                Some("p7m") => Some(SmimeLayout::Encrypted),
                Some("p7s") => Some(SmimeLayout::SignatureAround),
                _ => None,
            },
        },
        _ => None,
    }
}

/// Whether a media type is one of the PKCS #7 kinds, under either spelling.
///
/// `application/x-pkcs7-signature` is the older spelling and OpenSSL still
/// writes it by default, so a reader that only knows the registered name
/// reports a large share of real signed mail as not signed at all.
fn is_pkcs7(media_type: &str, kind: &str) -> bool {
    let lowered = media_type.trim().to_ascii_lowercase();
    lowered
        .strip_prefix("application/x-pkcs7-")
        .or_else(|| lowered.strip_prefix("application/pkcs7-"))
        == Some(kind)
}

/// The part of a file name after the last dot, folded to lower case.
///
/// Case is folded because senders write both `smime.p7s` and `smime.P7S`, and a
/// comparison that misses one calls signed mail unsigned.
fn file_suffix(name: &str) -> String {
    name.rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// A `Content-Type` header value split into its media type and parameters.
///
/// Small and local on purpose. This has to read the header exactly as it
/// arrived, and a general MIME library would be a second opinion about bytes a
/// signature was computed over.
struct ContentType {
    media_type: String,
    parameters: Vec<(String, String)>,
}

impl ContentType {
    fn read(value: &str) -> Self {
        let mut pieces = split_outside_quotes(value, ';');
        let media_type = pieces
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let parameters = pieces
            .filter_map(|piece| {
                let (name, raw) = piece.split_once('=')?;
                Some((
                    name.trim().to_ascii_lowercase(),
                    unquote(raw.trim()).to_string(),
                ))
            })
            .collect();
        Self {
            media_type,
            parameters,
        }
    }

    fn parameter(&self, name: &str) -> Option<String> {
        self.parameters
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    }
}

/// Split on a separator, ignoring separators inside a quoted string.
///
/// A boundary may hold a semicolon, and splitting on it would cut the boundary
/// in half and leave nothing that matches the real delimiter in the body.
fn split_outside_quotes(value: &str, separator: char) -> impl Iterator<Item = &str> {
    let mut in_quotes = false;
    let mut pieces = Vec::new();
    let mut start = 0;
    for (index, character) in value.char_indices() {
        if character == '"' {
            in_quotes = !in_quotes;
        } else if character == separator && !in_quotes {
            pieces.push(&value[start..index]);
            start = index + character.len_utf8();
        }
    }
    pieces.push(&value[start..]);
    pieces.into_iter()
}

/// Take the quotes off a parameter value if it has any.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

// ── Taking a signed message apart ────────────────────────────────────────────

/// A signed message pulled into the two things a check needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedParts {
    /// Exactly the bytes the sender ran through the hash, or `None` when the
    /// content is wrapped inside the signature instead of sitting beside it.
    ///
    /// Byte for byte as it arrived, headers included. Anything that re-encodes
    /// this, even to tidy it, changes what the hash sees and turns a good
    /// signature into a bad one.
    pub content: Option<Vec<u8>>,
    /// The signature itself, as the DER of a PKCS #7 `ContentInfo`.
    pub signature: Vec<u8>,
}

/// The two things out of a whole raw message, whichever way it was signed.
pub fn take_apart(raw_message: &[u8]) -> Result<SignedParts> {
    let (headers, body) = split_headers_from_body(raw_message);
    let content_type = header_value(headers, "content-type")
        .ok_or_else(|| Error::Security("The message has no Content-Type header".to_string()))?;
    match layout_of(&content_type) {
        Some(SmimeLayout::SignatureBeside { boundary }) => signature_beside(body, &boundary),
        Some(SmimeLayout::SignatureAround) => Ok(SignedParts {
            content: None,
            signature: decode_body(headers, body)?,
        }),
        Some(SmimeLayout::Encrypted) => Err(Error::Security(
            "This message is encrypted, not signed".to_string(),
        )),
        None => Err(Error::Security(
            "This message is not signed with S/MIME".to_string(),
        )),
    }
}

/// Split the two parts of a `multipart/signed` body.
fn signature_beside(body: &[u8], boundary: &str) -> Result<SignedParts> {
    let parts = parts_between(body, boundary);
    let content = parts
        .first()
        .ok_or_else(|| Error::Security("The signed part of the message is missing".to_string()))?;
    let signature_part = parts.get(1).ok_or_else(|| {
        Error::Security("The signature part of the message is missing".to_string())
    })?;
    let (signature_headers, signature_body) = split_headers_from_body(signature_part);
    Ok(SignedParts {
        content: Some(content.to_vec()),
        signature: decode_body(signature_headers, signature_body)?,
    })
}

/// The raw bytes of each part of a multipart body, in order.
///
/// The line break in front of a delimiter belongs to the delimiter and not to
/// the part before it, which is the detail that decides whether a signature
/// checks out. Keeping one byte too many here fails every signature in exactly
/// the way a changed message would, and there would be nothing to tell the two
/// apart from the outside.
fn parts_between<'a>(body: &'a [u8], boundary: &str) -> Vec<&'a [u8]> {
    let opening = format!("--{boundary}");
    let mut parts = Vec::new();
    let mut start = None;
    let mut at = 0;
    while at < body.len() {
        let line_end = find(body, b"\r\n", at).unwrap_or(body.len());
        let line = &body[at..line_end];
        if line.starts_with(opening.as_bytes()) {
            if let Some(from) = start {
                // `at` is one past the CRLF that ends the part's last line, so
                // the part itself stops two bytes earlier.
                //
                // Unless the part is empty, which is what two delimiters in a
                // row means, and is legal. Then `at` is where the part began
                // and stepping back two bytes runs off the front of it. Without
                // the `max` that is a panic on a stranger's message, inside a
                // function whose caller promises never to fail.
                parts.push(&body[from..at.saturating_sub(2).max(from)]);
            }
            // A closing delimiter is the opening one with two more dashes.
            if line[opening.len()..].starts_with(b"--") {
                return parts;
            }
            start = Some((line_end + 2).min(body.len()));
        }
        at = line_end + 2;
    }
    if let Some(from) = start {
        parts.push(&body[from..]);
    }
    parts
}

/// Where `needle` next appears in `haystack` at or after `from`.
fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    haystack[from..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| from + offset)
}

/// Split a raw message or part into its headers and its body.
///
/// A part with no blank line in it is all headers and no body, which is what
/// the empty second half says.
fn split_headers_from_body(raw: &[u8]) -> (&[u8], &[u8]) {
    match find(raw, b"\r\n\r\n", 0) {
        Some(at) => (&raw[..at], &raw[at + 4..]),
        // Some senders and some stores use bare line feeds. Falling back rather
        // than refusing, because the alternative is telling somebody their mail
        // is unreadable over a line ending.
        None => match find(raw, b"\n\n", 0) {
            Some(at) => (&raw[..at], &raw[at + 2..]),
            None => (raw, &[]),
        },
    }
}

/// One header's value, with folded continuation lines joined back together.
fn header_value(headers: &[u8], wanted: &str) -> Option<String> {
    let text = String::from_utf8_lossy(headers);
    let mut collected: Option<String> = None;
    for line in text.split('\n') {
        let line = line.trim_end_matches('\r');
        // A line starting with white space continues the one before it.
        //
        // Skipped whether or not it continues the header being looked for. A
        // continuation of some earlier header, trimmed, looks exactly like a
        // header of its own: `Subject: hello` folded onto a second line reading
        // ` Content-Type: ...` would otherwise be read as this message's
        // Content-Type. The sender writes their own Subject, so that hands them
        // the answer to whether their message is signed, encrypted or neither.
        if line.starts_with([' ', '\t']) {
            if let Some(value) = collected.as_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }
        if collected.is_some() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.trim().eq_ignore_ascii_case(wanted)
        {
            collected = Some(value.trim().to_string());
        }
    }
    collected
}

/// A part's body, undone from whatever transfer encoding it carries.
fn decode_body(headers: &[u8], body: &[u8]) -> Result<Vec<u8>> {
    let encoding = header_value(headers, "content-transfer-encoding")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match encoding.as_str() {
        "base64" | "" => {
            // Base64 carrying a signature is always line-wrapped, and the line
            // breaks are not part of it.
            let packed: Vec<u8> = body
                .iter()
                .copied()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect();
            STANDARD
                .decode(&packed)
                .map_err(|e| Error::Security(format!("The signature is not valid base64: {e}")))
        }
        "binary" | "8bit" | "7bit" => Ok(body.to_vec()),
        other => Err(Error::Security(format!(
            "The signature arrived as {other}, which this cannot read"
        ))),
    }
}

// ── Reading ASN.1, longhand ──────────────────────────────────────────────────

/// Just enough DER to walk a PKCS #7 document.
///
/// Written out here rather than taken from a crate because the one crate that
/// does CMS has only ever published pre-release versions, and a pre-release is
/// not what should sit under a security answer people will act on. What is
/// needed is small, and it stays small by refusing everything a mail signature
/// has no business containing.
mod der {
    use crate::common::{Error, Result};

    pub const INTEGER: u8 = 0x02;
    pub const OCTET_STRING: u8 = 0x04;
    pub const OBJECT_IDENTIFIER: u8 = 0x06;
    pub const SEQUENCE: u8 = 0x30;
    pub const SET: u8 = 0x31;
    pub const GENERALIZED_TIME: u8 = 0x18;

    /// The tag for `[n]`, constructed and context specific.
    pub const fn context(number: u8) -> u8 {
        0xA0 | number
    }

    /// The tag for `[0]` holding a plain value rather than more elements.
    ///
    /// Named separately because it cannot come from [`context`]: a `const fn`
    /// call is not allowed where a pattern is, and this is matched on.
    pub const CONTEXT_0_PRIMITIVE: u8 = 0x80;

    /// One tag-length-value.
    #[derive(Debug, Clone, Copy)]
    pub struct Element<'a> {
        pub tag: u8,
        /// What is inside, without the tag and length.
        pub value: &'a [u8],
        /// The whole element, tag and length included.
        ///
        /// Kept because a signature is computed over exact bytes. Re-encoding an
        /// element to hand it on would be a different encoder's opinion of the
        /// same value, and any difference at all breaks the signature.
        pub encoded: &'a [u8],
    }

    /// Read one element off the front, and say what is left after it.
    pub fn take(input: &[u8]) -> Result<(Element<'_>, &[u8])> {
        let tag = *input
            .first()
            .ok_or_else(|| Error::Security("The signature ended early".to_string()))?;
        // High tag numbers, tag 31 and up, do not occur anywhere in PKCS #7.
        // Refusing them keeps this reader small rather than guessing.
        if tag & 0x1F == 0x1F {
            return Err(Error::Security(
                "The signature uses an ASN.1 tag this cannot read".to_string(),
            ));
        }
        let first_length_byte = *input
            .get(1)
            .ok_or_else(|| Error::Security("The signature ended early".to_string()))?;
        let (length, header_length) = if first_length_byte < 0x80 {
            (first_length_byte as usize, 2)
        } else if first_length_byte == 0x80 {
            // Indefinite length is BER, not DER. Some senders emit it. Saying so
            // is better than reading the rest of the document as if it were the
            // value, which is what a reader that ignores this would do.
            return Err(Error::Security(
                "The signature is in a form with open-ended lengths, which this cannot read"
                    .to_string(),
            ));
        } else {
            let count = (first_length_byte & 0x7F) as usize;
            // Four bytes is four gigabytes. Nothing in a mail signature is that
            // large, and a longer count is either a mistake or an attempt to
            // make this reader allocate.
            if count == 0 || count > 4 {
                return Err(Error::Security(
                    "The signature declares a length this cannot read".to_string(),
                ));
            }
            let bytes = input
                .get(2..2 + count)
                .ok_or_else(|| Error::Security("The signature ended early".to_string()))?;
            let length = bytes
                .iter()
                .fold(0usize, |total, byte| (total << 8) | *byte as usize);
            (length, 2 + count)
        };
        let end = header_length
            .checked_add(length)
            .filter(|end| *end <= input.len())
            .ok_or_else(|| {
                Error::Security("The signature says it is longer than it is".to_string())
            })?;
        Ok((
            Element {
                tag,
                value: &input[header_length..end],
                encoded: &input[..end],
            },
            &input[end..],
        ))
    }

    /// Every element inside a constructed one.
    pub fn children(value: &[u8]) -> Result<Vec<Element<'_>>> {
        let mut found = Vec::new();
        let mut rest = value;
        while !rest.is_empty() {
            let (element, remaining) = take(rest)?;
            found.push(element);
            rest = remaining;
        }
        Ok(found)
    }

    /// A dotted object identifier out of the bytes inside an OBJECT IDENTIFIER.
    pub fn oid(value: &[u8]) -> Result<String> {
        let first = *value
            .first()
            .ok_or_else(|| Error::Security("An empty identifier in the signature".to_string()))?;
        let mut parts = vec![(first / 40).to_string(), (first % 40).to_string()];
        let mut current: u64 = 0;
        let mut in_progress = false;
        for byte in &value[1..] {
            current = current
                .checked_shl(7)
                .and_then(|shifted| shifted.checked_add((byte & 0x7F) as u64))
                .ok_or_else(|| {
                    Error::Security(
                        "An identifier in the signature is too long to read".to_string(),
                    )
                })?;
            if byte & 0x80 == 0 {
                parts.push(current.to_string());
                current = 0;
                in_progress = false;
            } else {
                in_progress = true;
            }
        }
        if in_progress {
            return Err(Error::Security(
                "An identifier in the signature stops in the middle".to_string(),
            ));
        }
        Ok(parts.join("."))
    }
}

/// A named object identifier, so the code reads as the standard does.
mod oid {
    pub const SIGNED_DATA: &str = "1.2.840.113549.1.7.2";
    pub const ENVELOPED_DATA: &str = "1.2.840.113549.1.7.3";

    pub const ATTRIBUTE_CONTENT_TYPE: &str = "1.2.840.113549.1.9.3";
    pub const ATTRIBUTE_MESSAGE_DIGEST: &str = "1.2.840.113549.1.9.4";
    /// The unsigned attribute an RFC 3161 timestamp token travels in.
    pub const ATTRIBUTE_TIMESTAMP_TOKEN: &str = "1.2.840.113549.1.9.16.2.14";
    /// The content a timestamp token wraps: the authority's statement itself.
    pub const TIMESTAMP_STATEMENT: &str = "1.2.840.113549.1.9.16.1.4";

    pub const SHA1: &str = "1.3.14.3.2.26";
    pub const SHA256: &str = "2.16.840.1.101.3.4.2.1";
    pub const SHA384: &str = "2.16.840.1.101.3.4.2.2";
    pub const SHA512: &str = "2.16.840.1.101.3.4.2.3";

    pub const RSA_ENCRYPTION: &str = "1.2.840.113549.1.1.1";
    pub const RSA_PSS: &str = "1.2.840.113549.1.1.10";
    pub const SHA256_WITH_RSA: &str = "1.2.840.113549.1.1.11";
    pub const SHA384_WITH_RSA: &str = "1.2.840.113549.1.1.12";
    pub const SHA512_WITH_RSA: &str = "1.2.840.113549.1.1.13";
    pub const SHA1_WITH_RSA: &str = "1.2.840.113549.1.1.5";

    pub const EC_PUBLIC_KEY: &str = "1.2.840.10045.2.1";
    pub const ECDSA_WITH_SHA256: &str = "1.2.840.10045.4.3.2";
    pub const ECDSA_WITH_SHA384: &str = "1.2.840.10045.4.3.3";
    pub const CURVE_P256: &str = "1.2.840.10045.3.1.7";
    pub const CURVE_P384: &str = "1.3.132.0.34";
}

// ── The signature document ───────────────────────────────────────────────────

/// How the signer said who they are.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SignerNamed {
    /// The usual way: the name of whoever issued the certificate, and the
    /// number they gave it.
    IssuerAndSerial { issuer: Vec<u8>, serial: Vec<u8> },
    /// The newer way: a short identifier the certificate carries.
    KeyIdentifier(Vec<u8>),
}

/// One signer inside a signature, as it arrived.
#[derive(Debug, Clone)]
struct Signer {
    named: SignerNamed,
    digest_algorithm: String,
    signature_algorithm: String,
    /// The parameters that came with the signature algorithm, needed only for
    /// RSA-PSS, which carries its hash in there rather than in its identifier.
    signature_parameters: Option<Vec<u8>>,
    signature: Vec<u8>,
    /// The signed attributes, already re-tagged into the shape that was hashed.
    signed_attributes: Option<Vec<u8>>,
    /// The fingerprint of the content, out of the signed attributes.
    stated_digest: Option<Vec<u8>>,
    /// The kind of content the attributes say was signed.
    stated_content_type: Option<String>,
    /// An RFC 3161 timestamp token, when one came with the signature.
    ///
    /// It travels in the *unsigned* attributes, which is not a weakness: the
    /// token is itself a signed document covering this signature's bytes, so
    /// changing either one breaks it. It has to be unsigned because it cannot
    /// exist until after the signature it covers has been made.
    timestamp_token: Option<Vec<u8>>,
}

/// A PKCS #7 signature taken apart.
#[derive(Debug, Clone)]
struct Signature {
    /// Every certificate the sender sent along.
    certificates: Vec<Vec<u8>>,
    signers: Vec<Signer>,
    /// The content, when the sender wrapped it inside rather than beside.
    wrapped_content: Option<Vec<u8>>,
    /// What kind of content the signature says it covers.
    content_type: String,
}

impl Signature {
    /// Read a `ContentInfo` holding `SignedData`.
    fn read(der_bytes: &[u8]) -> Result<Self> {
        let (outer, _) = der::take(der_bytes)?;
        let outer = expect(outer, der::SEQUENCE, "The signature")?;
        let fields = der::children(outer.value)?;
        let content_type = read_oid(fields.first(), "The signature's kind")?;
        if content_type != oid::SIGNED_DATA {
            return Err(Error::Security(
                "This is a PKCS #7 document but not a signature".to_string(),
            ));
        }
        let wrapper = fields
            .get(1)
            .filter(|element| element.tag == der::context(0))
            .ok_or_else(|| Error::Security("The signature holds nothing".to_string()))?;
        let (signed_data, _) = der::take(wrapper.value)?;
        let signed_data = expect(signed_data, der::SEQUENCE, "The signed data")?;
        let parts = der::children(signed_data.value)?;

        // version, digestAlgorithms, encapContentInfo are fixed; certificates
        // and revocation lists are optional; signerInfos is last. Walking by
        // tag after the first three is how optional fields are meant to be
        // read, and is why a message with no certificates does not shift
        // everything after it by one.
        let encapsulated = parts
            .get(2)
            .copied()
            .ok_or_else(|| Error::Security("The signed data is incomplete".to_string()))?;
        let encapsulated = expect(encapsulated, der::SEQUENCE, "The signed content")?;
        let inside = der::children(encapsulated.value)?;
        let inner_content_type = read_oid(inside.first(), "The signed content's kind")?;
        let wrapped_content = inside
            .get(1)
            .filter(|element| element.tag == der::context(0))
            .map(|element| -> Result<Vec<u8>> {
                let (octets, _) = der::take(element.value)?;
                Ok(expect(octets, der::OCTET_STRING, "The wrapped content")?
                    .value
                    .to_vec())
            })
            .transpose()?;

        let certificates = parts
            .iter()
            .find(|element| element.tag == der::context(0))
            .map(|element| -> Result<Vec<Vec<u8>>> {
                Ok(der::children(element.value)?
                    .into_iter()
                    // Anything that is not a plain certificate is one of the
                    // other things a CertificateSet may hold, and none of them
                    // are usable here.
                    .filter(|candidate| candidate.tag == der::SEQUENCE)
                    .map(|candidate| candidate.encoded.to_vec())
                    .collect())
            })
            .transpose()?
            .unwrap_or_default();

        let signers = parts
            .iter()
            .skip(3)
            .find(|element| element.tag == der::SET)
            .map(|element| -> Result<Vec<Signer>> {
                der::children(element.value)?
                    .iter()
                    .map(Signer::read)
                    .collect()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            certificates,
            signers,
            wrapped_content,
            content_type: inner_content_type,
        })
    }
}

impl Signer {
    fn read(element: &der::Element<'_>) -> Result<Self> {
        let element = expect(*element, der::SEQUENCE, "A signer")?;
        let fields = der::children(element.value)?;
        let mut at = 1; // version comes first and says nothing this needs.

        let identity = fields
            .get(at)
            .copied()
            .ok_or_else(|| Error::Security("A signer names nobody".to_string()))?;
        let named = match identity.tag {
            der::SEQUENCE => {
                let pair = der::children(identity.value)?;
                let issuer = pair
                    .first()
                    .ok_or_else(|| Error::Security("A signer names no issuer".to_string()))?;
                let serial = pair
                    .get(1)
                    .filter(|element| element.tag == der::INTEGER)
                    .ok_or_else(|| Error::Security("A signer has no serial number".to_string()))?;
                SignerNamed::IssuerAndSerial {
                    // The issuer is compared as the exact bytes that arrived.
                    // Comparing the printed form instead would make two
                    // different names look the same whenever they print the
                    // same, which is the sort of thing worth forging.
                    issuer: issuer.encoded.to_vec(),
                    serial: serial.value.to_vec(),
                }
            }
            der::CONTEXT_0_PRIMITIVE => SignerNamed::KeyIdentifier(identity.value.to_vec()),
            _ => {
                return Err(Error::Security(
                    "A signer names itself in a way this cannot read".to_string(),
                ));
            }
        };
        at += 1;

        let digest_algorithm = algorithm_identifier(fields.get(at), "The digest")?.0;
        at += 1;

        let mut signed_attributes = None;
        let mut stated_digest = None;
        let mut stated_content_type = None;
        if let Some(attributes) = fields.get(at).filter(|e| e.tag == der::context(0)) {
            // What was signed is the DER of a SET OF Attribute. It travels
            // tagged as [0] to say it is the signed set rather than the
            // unsigned one, and the tag has to be put back to a plain SET
            // before hashing. Only the first byte differs, so the rest of the
            // bytes are exactly the ones the sender hashed. Missing this is the
            // classic way a checker reports every good signature as bad.
            let mut retagged = attributes.encoded.to_vec();
            if let Some(first) = retagged.first_mut() {
                *first = der::SET;
            }
            for attribute in der::children(attributes.value)? {
                let attribute = expect(attribute, der::SEQUENCE, "A signed attribute")?;
                let inside = der::children(attribute.value)?;
                let name = read_oid(inside.first(), "A signed attribute's name")?;
                let Some(values) = inside.get(1).filter(|e| e.tag == der::SET) else {
                    continue;
                };
                let first_value = der::children(values.value)?.into_iter().next();
                match name.as_str() {
                    oid::ATTRIBUTE_MESSAGE_DIGEST => {
                        stated_digest = first_value
                            .filter(|v| v.tag == der::OCTET_STRING)
                            .map(|v| v.value.to_vec());
                    }
                    oid::ATTRIBUTE_CONTENT_TYPE => {
                        stated_content_type = first_value
                            .filter(|v| v.tag == der::OBJECT_IDENTIFIER)
                            .map(|v| der::oid(v.value))
                            .transpose()?;
                    }
                    _ => {}
                }
            }
            signed_attributes = Some(retagged);
            at += 1;
        }

        let (signature_algorithm, signature_parameters) =
            algorithm_identifier(fields.get(at), "The signature")?;
        at += 1;

        let signature = fields
            .get(at)
            .filter(|element| element.tag == der::OCTET_STRING)
            .ok_or_else(|| Error::Security("A signer carries no signature".to_string()))?
            .value
            .to_vec();
        at += 1;

        let timestamp_token = fields
            .get(at)
            .filter(|element| element.tag == der::context(1))
            .and_then(|attributes| timestamp_among(attributes.value));

        Ok(Self {
            named,
            digest_algorithm,
            signature_algorithm,
            signature_parameters,
            signature,
            signed_attributes,
            stated_digest,
            stated_content_type,
            timestamp_token,
        })
    }
}

/// The timestamp token among a signer's unsigned attributes, if there is one.
///
/// Nothing here is a refusal, and that is the point: these attributes are the
/// part of a signature the sender can put anything into, because they are the
/// part no signature covers. Failing on rubbish found here would let a stranger
/// make a perfectly good signed message unreadable by appending nonsense to it.
/// So anything that will not read is passed over and the message is reported as
/// having no timestamp, which is true.
fn timestamp_among(attributes: &[u8]) -> Option<Vec<u8>> {
    for attribute in der::children(attributes).ok()? {
        if attribute.tag != der::SEQUENCE {
            continue;
        }
        let Ok(inside) = der::children(attribute.value) else {
            continue;
        };
        // Passed over, not given up on. Both of these used to end the whole
        // walk rather than this one attribute, which is the opposite of what
        // the paragraph above promises: one piece of nonsense appended in front
        // of a real timestamp hid the timestamp, and a message whose
        // certificate had expired since it was signed then read as signed with
        // an expired certificate.
        let Ok(name) = read_oid(inside.first(), "An unsigned attribute's name") else {
            continue;
        };
        if name != oid::ATTRIBUTE_TIMESTAMP_TOKEN {
            continue;
        }
        let Some(values) = inside.get(1).filter(|e| e.tag == der::SET) else {
            continue;
        };
        let Ok(held) = der::children(values.value) else {
            continue;
        };
        // The whole element, not its contents: the token is a document of its
        // own and every byte of it is covered by the authority's signature.
        if let Some(token) = held.first() {
            return Some(token.encoded.to_vec());
        }
    }
    None
}

/// An element, having checked it is the tag expected.
fn expect<'a>(element: der::Element<'a>, tag: u8, what: &str) -> Result<der::Element<'a>> {
    if element.tag == tag {
        Ok(element)
    } else {
        Err(Error::Security(format!("{what} is not the expected shape")))
    }
}

/// The dotted identifier out of an element that has to be an OBJECT IDENTIFIER.
fn read_oid(element: Option<&der::Element<'_>>, what: &str) -> Result<String> {
    let element = element
        .filter(|element| element.tag == der::OBJECT_IDENTIFIER)
        .ok_or_else(|| Error::Security(format!("{what} is missing")))?;
    der::oid(element.value)
}

/// The identifier and parameters out of an `AlgorithmIdentifier`.
fn algorithm_identifier(
    element: Option<&der::Element<'_>>,
    what: &str,
) -> Result<(String, Option<Vec<u8>>)> {
    let element = element
        .filter(|element| element.tag == der::SEQUENCE)
        .ok_or_else(|| Error::Security(format!("{what} algorithm is missing")))?;
    let inside = der::children(element.value)?;
    let name = read_oid(inside.first(), what)?;
    let parameters = inside.get(1).map(|element| element.encoded.to_vec());
    Ok((name, parameters))
}

// ── The certificate that came with the message ───────────────────────────────

/// What the certificate attached to a signed message says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerCertificate {
    /// Who it was issued to, printed the way the certificate spells it.
    pub subject: String,
    /// Who issued it.
    pub issuer: String,
    /// Every email address the certificate names, folded to lower case.
    pub email_addresses: Vec<String>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    /// The certificate names itself as its own issuer.
    ///
    /// That is a comparison of two names, not proof that it really signed
    /// itself. It is enough for what it is used for here: a certificate whose
    /// issuer is itself has had nobody else look at it, whatever else is true.
    pub self_issued: bool,
    pub serial_number: String,
    /// The bytes it arrived as, so the machine's own store can be asked about
    /// it without this having to hand over a parsed thing the store cannot use.
    pub der: Vec<u8>,
}

impl SignerCertificate {
    /// Read a certificate out of its DER.
    pub fn read(der_bytes: &[u8]) -> Result<Self> {
        let (_, certificate) = X509Certificate::from_der(der_bytes)
            .map_err(|e| Error::Security(format!("The certificate could not be read: {e}")))?;
        let valid_from = moment(certificate.validity().not_before.timestamp())?;
        let valid_until = moment(certificate.validity().not_after.timestamp())?;
        Ok(Self {
            subject: certificate.subject().to_string(),
            issuer: certificate.issuer().to_string(),
            email_addresses: email_addresses_in(&certificate),
            valid_from,
            valid_until,
            self_issued: certificate.subject().as_raw() == certificate.issuer().as_raw(),
            serial_number: certificate.raw_serial_as_string(),
            der: der_bytes.to_vec(),
        })
    }

    /// Whether an address is one this certificate names.
    ///
    /// Compared without regard to case. The part after the at sign is
    /// officially case-insensitive and the part before it officially is not,
    /// but no mail system in use treats them differently, and being strict here
    /// would tell somebody their own certificate is not theirs.
    pub fn names(&self, address: &str) -> bool {
        let wanted = address.trim().to_ascii_lowercase();
        self.email_addresses.contains(&wanted)
    }
}

/// Every email address a certificate names.
///
/// The subject alternative name is where a mail certificate is supposed to put
/// the address. The old `emailAddress` field in the subject is read as well,
/// because certificates that only have it are still in use and ignoring them
/// would report a correct certificate as naming nobody.
fn email_addresses_in(certificate: &X509Certificate<'_>) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    if let Ok(Some(alternative)) = certificate.subject_alternative_name() {
        for name in &alternative.value.general_names {
            if let GeneralName::RFC822Name(address) = name {
                found.push(address.to_ascii_lowercase());
            }
        }
    }
    for attribute in certificate.subject().iter_email() {
        if let Ok(address) = attribute.as_str() {
            found.push(address.to_ascii_lowercase());
        }
    }
    found.sort();
    found.dedup();
    found
}

/// A moment from the seconds count a certificate carries.
fn moment(seconds: i64) -> Result<DateTime<Utc>> {
    DateTime::from_timestamp(seconds, 0).ok_or_else(|| {
        Error::Security("The certificate carries a date outside the calendar".to_string())
    })
}

// ── What a check found ───────────────────────────────────────────────────────

/// One thing found while reading a signature. Each is a sentence somebody hears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// The content is bit for bit what was signed.
    ContentIsWhatWasSigned,
    /// The content is not what was signed.
    ContentIsNotWhatWasSigned,
    /// The signature was made by the key belonging to the attached certificate.
    SignedByThatCertificatesKey,
    /// It was not.
    NotSignedByThatCertificatesKey,
    /// The signature is of a kind this program cannot check.
    SignatureKindNotUnderstood { named: String },
    /// The signature was made with a fingerprint that can be forged.
    FingerprintTooWeakToTrust { named: String },
    /// The certificate's dates had run out at the moment asked about, and
    /// nothing says when the message was signed.
    CertificateHadExpired { expired_on: DateTime<Utc> },
    /// The certificate does not start until later.
    CertificateHasNotStarted { starts_on: DateTime<Utc> },
    /// The certificate was inside its dates.
    CertificateWasInDate,
    /// A timestamp says when this was signed, and the certificate was good
    /// then. `expired_since` is filled in when it has run out since.
    CertificateWasInDateWhenSigned {
        signed_on: DateTime<Utc>,
        expired_since: Option<DateTime<Utc>>,
    },
    /// A timestamp says when this was signed, and the certificate was not
    /// inside its dates at that moment.
    CertificateWasNotInDateWhenSigned {
        signed_on: DateTime<Utc>,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    },
    /// The certificate names the address the message says it came from.
    CertificateNamesTheSender { address: String },
    /// It names somebody else.
    CertificateNamesSomebodyElse {
        certificate_names: Vec<String>,
        message_says: String,
    },
    /// It names no address at all.
    CertificateNamesNobody,
    /// The certificate is its own issuer.
    CertificateIssuedItself,
    /// This computer's own list of authorities vouches for it.
    IssuerTrustedHere,
    /// It does not.
    IssuerNotTrustedHere { reason: String },
    /// Nobody asked, and why.
    IssuerNotChecked { reason: String },
    /// Whether the certificate has been withdrawn was not looked into.
    WithdrawalNotChecked,
    /// The certificate has been withdrawn. Whoever issued it has said it is no
    /// longer to be relied on, which is what is done after a key is stolen.
    CertificateWithdrawn,
    /// Something really looked, and the certificate has not been withdrawn.
    CertificateNotWithdrawn,
    /// The question has been asked and the answer has not come back yet.
    WithdrawalStillBeingLookedInto,
    /// Something looked and could not find out.
    WithdrawalCouldNotBeFoundOut { reason: String },
    /// A timestamp authority's own signed statement of when this was signed.
    SignedAtByATimestamp {
        moment: DateTime<Utc>,
        authority: String,
    },
    /// Nothing that can be relied on says when this was signed.
    WhenItWasSignedIsNotKnown,
    /// A timestamp came with the message and covers something other than this
    /// signature.
    TimestampIsForSomethingElse,
    /// A timestamp came with the message and could not be read or checked.
    TimestampCouldNotBeRead { reason: String },
    /// This computer's own list of authorities vouches for the timestamp
    /// authority.
    TimestampAuthorityTrustedHere,
    /// It does not.
    TimestampAuthorityNotTrustedHere { reason: String },
    /// Nobody asked whether the timestamp authority is one this computer
    /// trusts.
    TimestampAuthorityNotChecked { reason: String },
    /// The signature carries no signer.
    NoSignerInSignature,
    /// The signer's certificate did not travel with the message.
    SignersCertificateMissing,
    /// The signature says it covers a different kind of content.
    SignatureCoversSomethingElse,
    /// The signature could not be read at all.
    CouldNotBeRead { reason: String },
}

impl Finding {
    /// The sentence read out for this finding.
    pub fn spoken(&self) -> String {
        match self {
            Self::ContentIsWhatWasSigned => {
                "The message is exactly what was signed. Nothing has been changed since.".into()
            }
            Self::ContentIsNotWhatWasSigned => {
                "The message is not what was signed. Something has changed since it was signed."
                    .into()
            }
            Self::SignedByThatCertificatesKey => {
                "The signature was made by the private key belonging to the certificate that came \
                 with the message."
                    .into()
            }
            Self::NotSignedByThatCertificatesKey => {
                "The signature was not made by the key in the certificate that came with the \
                 message."
                    .into()
            }
            Self::SignatureKindNotUnderstood { named } => format!(
                "This is a {named} signature, which this program cannot check, so nothing here \
                 says whether it is genuine."
            ),
            Self::FingerprintTooWeakToTrust { named } => format!(
                "The signature was made using {named}. A second message can be built to have the \
                 same {named} fingerprint, so a signature made with it shows nothing."
            ),
            Self::CertificateHadExpired { expired_on } => format!(
                "The certificate ran out on {}. That does not on its own mean the signature is \
                 bad, because the message may have been signed while the certificate was still \
                 good, but nothing here says when it was signed.",
                day(*expired_on)
            ),
            Self::CertificateHasNotStarted { starts_on } => format!(
                "The certificate does not start until {}, which has not happened yet.",
                day(*starts_on)
            ),
            Self::CertificateWasInDate => {
                "The certificate was inside its dates at the moment asked about.".into()
            }
            Self::CertificateWasInDateWhenSigned {
                signed_on,
                expired_since: None,
            } => format!(
                "The certificate was inside its dates on {}, which a timestamp says is when this \
                 was signed.",
                day(*signed_on)
            ),
            Self::CertificateWasInDateWhenSigned {
                signed_on,
                expired_since: Some(expired_on),
            } => format!(
                "The certificate ran out on {}, but a timestamp says this was signed on {}, when \
                 it was still good. A certificate running out later does not make an older \
                 signature bad.",
                day(*expired_on),
                day(*signed_on)
            ),
            Self::CertificateWasNotInDateWhenSigned {
                signed_on,
                valid_from,
                valid_until,
            } => format!(
                "A timestamp says this was signed on {}, and the certificate was only good from {} \
                 to {}, so it was outside its dates at the moment it was used.",
                day(*signed_on),
                day(*valid_from),
                day(*valid_until)
            ),
            Self::CertificateNamesTheSender { address } => format!(
                "The certificate is for {address}, which is the address this message says it came \
                 from."
            ),
            Self::CertificateNamesSomebodyElse {
                certificate_names,
                message_says,
            } => format!(
                "The certificate is for {}. The message says it came from {message_says}. Those \
                 are not the same.",
                certificate_names.join(", ")
            ),
            Self::CertificateNamesNobody => {
                "The certificate names no email address, so there is nothing to check the sender \
                 against."
                    .into()
            }
            Self::CertificateIssuedItself => {
                "The certificate was issued by itself. Nobody else has checked who it belongs to, \
                 so anyone could have made it."
                    .into()
            }
            Self::IssuerTrustedHere => {
                "This computer's list of trusted authorities vouches for the certificate.".into()
            }
            Self::IssuerNotTrustedHere { reason } => {
                format!("This computer does not trust the certificate: {reason}.")
            }
            Self::IssuerNotChecked { reason } => {
                format!("Whether this computer trusts the certificate was not checked: {reason}.")
            }
            Self::WithdrawalNotChecked => {
                "Whether the certificate has been withdrawn since it was issued was not checked."
                    .into()
            }
            Self::CertificateWithdrawn => {
                "The certificate has been withdrawn. Whoever issued it has said it is no longer to \
                 be relied on, which is what is done after a private key is stolen. Read this \
                 message as though it were not signed at all."
                    .into()
            }
            Self::CertificateNotWithdrawn => {
                "The certificate has not been withdrawn. That was checked against a withdrawal \
                 list this computer already held."
                    .into()
            }
            Self::WithdrawalStillBeingLookedInto => {
                "Whether the certificate has been withdrawn is still being looked into. Until that \
                 comes back, treat it as unanswered rather than as good news."
                    .into()
            }
            Self::WithdrawalCouldNotBeFoundOut { reason } => format!(
                "Whether the certificate has been withdrawn could not be found out: {reason}. That \
                 is not the same as it being in good standing."
            ),
            Self::SignedAtByATimestamp { moment, authority } => format!(
                "A timestamp says this was signed on {}. The timestamp comes from an authority \
                 calling itself {authority}, not from the sender, so the sender could not choose \
                 the date.",
                day(*moment)
            ),
            Self::WhenItWasSignedIsNotKnown => {
                "Nothing here says when this was signed. The sender's own claim of a signing time \
                 travels with the message and is ignored, because the sender chooses it."
                    .into()
            }
            Self::TimestampIsForSomethingElse => {
                "A timestamp came with this message and it covers something other than this \
                 signature, so it says nothing about when this was signed."
                    .into()
            }
            Self::TimestampCouldNotBeRead { reason } => format!(
                "A timestamp came with this message and could not be checked: {reason}. Nothing \
                 here says when this was signed."
            ),
            Self::TimestampAuthorityTrustedHere => {
                "This computer's list of trusted authorities vouches for the timestamp authority."
                    .into()
            }
            Self::TimestampAuthorityNotTrustedHere { reason } => format!(
                "This computer does not trust the timestamp authority: {reason}. Anyone can run \
                 one, so a date from an authority nobody vouches for shows no more than the \
                 sender's own word."
            ),
            Self::TimestampAuthorityNotChecked { reason } => format!(
                "Whether this computer trusts the timestamp authority was not checked: {reason}."
            ),
            Self::NoSignerInSignature => {
                "The signature holds no signer, so there is nothing to check.".into()
            }
            Self::SignersCertificateMissing => {
                "The message names who signed it, but the certificate did not come with it, so \
                 there is no key here to check the signature against."
                    .into()
            }
            Self::SignatureCoversSomethingElse => {
                "The signature says it covers a different kind of content from the one it is \
                 attached to."
                    .into()
            }
            Self::CouldNotBeRead { reason } => {
                format!("The signature could not be read: {reason}.")
            }
        }
    }

    /// How much this matters, so the worst is said first.
    ///
    /// Ordering is by what somebody needs to hear soonest, not by how the
    /// checks happen to run.
    fn weight(&self) -> u8 {
        match self {
            Self::ContentIsNotWhatWasSigned
            | Self::NotSignedByThatCertificatesKey
            | Self::CertificateWithdrawn => 0,
            Self::FingerprintTooWeakToTrust { .. } | Self::SignatureCoversSomethingElse => 1,
            Self::CertificateNamesSomebodyElse { .. } => 2,
            Self::CouldNotBeRead { .. }
            | Self::NoSignerInSignature
            | Self::SignersCertificateMissing
            | Self::SignatureKindNotUnderstood { .. } => 3,
            Self::CertificateIssuedItself
            | Self::IssuerNotTrustedHere { .. }
            | Self::CertificateWasNotInDateWhenSigned { .. } => 4,
            Self::CertificateHadExpired { .. } | Self::CertificateHasNotStarted { .. } => 5,
            Self::CertificateNamesNobody | Self::TimestampAuthorityNotTrustedHere { .. } => 6,
            Self::IssuerNotChecked { .. }
            | Self::WithdrawalNotChecked
            | Self::WithdrawalStillBeingLookedInto
            | Self::WithdrawalCouldNotBeFoundOut { .. }
            | Self::WhenItWasSignedIsNotKnown
            | Self::TimestampIsForSomethingElse
            | Self::TimestampCouldNotBeRead { .. }
            | Self::TimestampAuthorityNotChecked { .. } => 7,
            Self::ContentIsWhatWasSigned
            | Self::SignedByThatCertificatesKey
            | Self::CertificateWasInDate
            | Self::CertificateWasInDateWhenSigned { .. }
            | Self::CertificateNamesTheSender { .. }
            | Self::IssuerTrustedHere
            | Self::CertificateNotWithdrawn
            | Self::SignedAtByATimestamp { .. }
            | Self::TimestampAuthorityTrustedHere => 8,
        }
    }
}

/// A date the way somebody would say it.
fn day(moment: DateTime<Utc>) -> String {
    moment.format("%-d %B %Y").to_string()
}

/// What checking a signature came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureOutcome {
    /// The arithmetic held: this content, that certificate's key.
    Matches,
    /// The arithmetic held but was done with a fingerprint that can be forged,
    /// so it is worth nothing.
    MatchesButWorthNothing,
    /// The arithmetic did not hold.
    DoesNotMatch,
    /// There is a signature and this computer could not check it.
    NotChecked,
    /// It said it was signed and there was no signature to check.
    NothingToCheck,
    /// A message signed more than once, whose signatures do not all come to the
    /// same answer.
    ///
    /// Its own outcome rather than the best or the worst of them, because
    /// neither of those is what happened. One signature that holds beside one
    /// that does not means somebody attached a signature that is not what it
    /// says, and reporting only the good one hides that; reporting only the bad
    /// one throws away a signature that really does show the content is
    /// unchanged.
    SignersDisagree,
}

/// What one signer's signature came to.
///
/// A message may be signed by more than one party. Each is a separate claim by
/// a separate key, so each gets its own answer rather than being folded into
/// one verdict that hides which of them held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerReport {
    pub outcome: SignatureOutcome,
    /// The certificate this signer named, when it travelled with the message.
    pub certificate: Option<SignerCertificate>,
    pub findings: Vec<Finding>,
    /// When a timestamp authority says this signature was made, and whose
    /// certificate said so, when a timestamp came with it and checked out.
    pub signed_at: Option<SigningTime>,
}

/// A timestamp authority's own signed statement of when something was signed.
///
/// Kept apart from the sender's claimed signing time, which is ignored on
/// purpose: the sender chooses that one and can write anything in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningTime {
    pub moment: DateTime<Utc>,
    /// The authority's certificate as it arrived inside the timestamp, so the
    /// machine's own store can be asked whether it trusts the authority.
    pub authority: SignerCertificate,
}

/// Everything reading a signed message found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureReport {
    pub outcome: SignatureOutcome,
    /// The certificate the first signer named, when one came with the message.
    ///
    /// The first, because nearly every signed message has exactly one signer.
    /// [`SignatureReport::signers`] is where the rest are, and is the field to
    /// read when a message may carry more than one.
    pub signer: Option<SignerCertificate>,
    /// Everything found, over every signer.
    ///
    /// Held flat as well as per signer because most callers have one signer and
    /// want one list. It is rebuilt from [`SignatureReport::signers`] whenever
    /// an answer is folded in, so the two cannot drift apart.
    pub findings: Vec<Finding>,
    /// The message text, when it was wrapped inside the signature and had to be
    /// unwrapped to be read at all.
    pub unwrapped_content: Option<Vec<u8>>,
    /// One entry per signer, in the order the message carries them.
    pub signers: Vec<SignerReport>,
}

impl SignatureReport {
    /// The one line said first.
    ///
    /// Never the word "signed" on its own. People read that as "genuine", and
    /// the whole point of this file is that those are different claims.
    pub fn headline(&self) -> String {
        // A message signed by more than one party has no single "the
        // certificate" and no single address, so a sentence in the shape of the
        // ones below would have to pick one signature and speak as though it
        // were the only one.
        if self.signers.len() > 1 {
            return self.headline_for_several_signatures();
        }
        match self.outcome {
            SignatureOutcome::NothingToCheck => {
                "This message says it is signed, but it carries no signature to check.".into()
            }
            SignatureOutcome::DoesNotMatch => {
                "This message does not match its signature. Read it as though it were not signed \
                 at all."
                    .into()
            }
            SignatureOutcome::NotChecked => {
                "This message is signed and this computer could not check the signature, so the \
                 signature shows nothing here."
                    .into()
            }
            // Why it is worth nothing, not merely that it is. There is more
            // than one way for a signature to add up and prove nothing, and
            // naming the wrong one is its own kind of wrong answer: somebody
            // told their correspondent's certificate was forgeable, when it
            // has in fact been withdrawn, has been told the sender chose a
            // weak setting rather than that a key has been stolen.
            SignatureOutcome::MatchesButWorthNothing => {
                if self
                    .findings
                    .iter()
                    .any(|finding| matches!(finding, Finding::CertificateWithdrawn))
                {
                    "The signature on this message adds up, and the certificate behind it has \
                     been withdrawn. Read it as though it were not signed at all."
                        .into()
                } else {
                    "The signature on this message adds up, but it was made in a way that can \
                     be forged. Read it as though it were not signed at all."
                        .into()
                }
            }
            SignatureOutcome::SignersDisagree => self.headline_for_several_signatures(),
            SignatureOutcome::Matches => self.headline_for_a_signature_that_held(),
        }
    }

    /// The first line for a message signed more than once.
    ///
    /// It counts them rather than picking one, because somebody hearing only
    /// the first sentence has to learn how many claims there are and whether
    /// they agree. Which signature said what is in the detail underneath, said
    /// one signature at a time.
    fn headline_for_several_signatures(&self) -> String {
        let count = |wanted: SignatureOutcome| {
            self.signers
                .iter()
                .filter(|signer| signer.outcome == wanted)
                .count()
        };
        let total = self.signers.len();
        let held = count(SignatureOutcome::Matches);
        let failed = count(SignatureOutcome::DoesNotMatch);
        let worthless = count(SignatureOutcome::MatchesButWorthNothing);
        let unchecked = total - held - failed - worthless;
        let mut said = vec![if self.outcome == SignatureOutcome::SignersDisagree {
            format!("This message carries {total} signatures and they do not agree.")
        } else {
            format!("This message carries {total} signatures.")
        }];
        if held > 0 {
            said.push(format!(
                "{held} of them {} up, {}.",
                one_or_more(held, "adds", "add"),
                one_or_more(
                    held,
                    "for its own certificate and its own address",
                    "each for its own certificate and its own address"
                )
            ));
        }
        if worthless > 0 {
            said.push(format!(
                "{worthless} {} up but {} made in a way that can be forged.",
                one_or_more(worthless, "adds", "add"),
                one_or_more(worthless, "was", "were")
            ));
        }
        if failed > 0 {
            said.push(format!(
                "{failed} {} not add up, so at least one signature on this message is not what it \
                 says it is. Read it as though it were not signed at all.",
                one_or_more(failed, "does", "do")
            ));
        }
        if unchecked > 0 {
            said.push(format!(
                "{unchecked} could not be checked here, so nothing {} carries is settled.",
                one_or_more(unchecked, "that one", "those ones")
            ));
        }
        said.join(" ")
    }

    fn headline_for_a_signature_that_held(&self) -> String {
        if self.findings.contains(&Finding::CertificateWithdrawn) {
            return "The signature on this message adds up, and the certificate it was made with \
                    has been withdrawn, which is what happens after a key is stolen. Read it as \
                    though it were not signed at all."
                .into();
        }
        let mismatch = self.findings.iter().find_map(|finding| match finding {
            Finding::CertificateNamesSomebodyElse {
                certificate_names,
                message_says,
            } => Some((certificate_names.join(", "), message_says.clone())),
            _ => None,
        });
        if let Some((certificate_names, message_says)) = mismatch {
            return format!(
                "Signed and unchanged since, but signed for {certificate_names}, not for \
                 {message_says}, which is the address this message says it came from."
            );
        }
        let address = self.findings.iter().find_map(|finding| match finding {
            Finding::CertificateNamesTheSender { address } => Some(address.clone()),
            _ => None,
        });
        match address {
            Some(address) => format!(
                "Signed for {address}, and not changed since. That shows the address, not the \
                 person."
            ),
            None => "Signed and not changed since, but the certificate names no email address, so \
                     there is nothing to check the sender against."
                .into(),
        }
    }

    /// What was found, worst first.
    ///
    /// With more than one signer the lines are grouped by signer and each group
    /// says which signature it is about. Run together they would read as one
    /// contradictory list: "the certificate is for alice" and "the certificate
    /// is for bob" are both true and neither is about the same signature.
    pub fn detail(&self) -> Vec<String> {
        if self.signers.len() < 2 {
            return worst_first(&self.findings);
        }
        let total = self.signers.len();
        let mut said = Vec::new();
        for (index, signer) in self.signers.iter().enumerate() {
            said.push(format!("Signature {} of {total}.", index + 1));
            said.extend(worst_first(&signer.findings));
        }
        said
    }

    /// What a signature does not show, said every time.
    ///
    /// Every time, and not only when something is wrong. A caveat that appears
    /// only on bad news teaches people that its absence is good news, and the
    /// gap between "the arithmetic held" and "this is who it says it is" is
    /// there on every message including the good ones.
    pub fn limits(&self) -> Vec<&'static str> {
        what_a_signature_is_worth()
    }

    /// The whole thing as one piece of speech.
    pub fn spoken(&self) -> String {
        let mut said = vec![self.headline()];
        said.extend(self.detail());
        said.extend(self.limits().iter().map(|line| line.to_string()));
        said.join(" ")
    }

    /// Fold in an answer only the machine's own store has, about the first
    /// signer.
    ///
    /// Separate from the rest so the reading and the deciding stay free of any
    /// platform code, and so a caller with no store to ask still gets a report
    /// that says the trust question was not asked rather than implying it was.
    pub fn with_issuer_trust(self, trust: IssuerTrust) -> Self {
        self.with_issuer_trust_for(0, trust)
    }

    /// The same, naming which signer the answer is about.
    ///
    /// Does nothing when there is no such signer, because a finding about a
    /// certificate that is not in the report is a sentence about nothing.
    pub fn with_issuer_trust_for(self, which: usize, trust: IssuerTrust) -> Self {
        self.replacing(
            which,
            |finding| matches!(finding, Finding::IssuerNotChecked { .. }),
            match trust {
                IssuerTrust::Trusted => Finding::IssuerTrustedHere,
                IssuerTrust::NotTrusted { reason } => Finding::IssuerNotTrustedHere { reason },
                IssuerTrust::NotChecked { reason } => Finding::IssuerNotChecked { reason },
            },
        )
    }

    /// Fold in whether the first signer's certificate has been withdrawn.
    ///
    /// Kept apart from [`SignatureReport::with_issuer_trust`] because they are
    /// different questions with different costs: whether this computer trusts
    /// the issuer is answered from a list already here, and whether the
    /// certificate has been withdrawn may need somebody to be asked. See
    /// [`Reach`].
    pub fn with_withdrawal(self, withdrawal: Withdrawal) -> Self {
        self.with_withdrawal_for(0, withdrawal)
    }

    /// The same, naming which signer the answer is about.
    pub fn with_withdrawal_for(self, which: usize, withdrawal: Withdrawal) -> Self {
        self.replacing(
            which,
            |finding| {
                matches!(
                    finding,
                    Finding::WithdrawalNotChecked
                        | Finding::CertificateWithdrawn
                        | Finding::CertificateNotWithdrawn
                        | Finding::WithdrawalStillBeingLookedInto
                        | Finding::WithdrawalCouldNotBeFoundOut { .. }
                )
            },
            match withdrawal {
                Withdrawal::Withdrawn => Finding::CertificateWithdrawn,
                Withdrawal::NotWithdrawn => Finding::CertificateNotWithdrawn,
                Withdrawal::StillBeingLookedInto => Finding::WithdrawalStillBeingLookedInto,
                Withdrawal::CouldNotFindOut { reason } => {
                    Finding::WithdrawalCouldNotBeFoundOut { reason }
                }
                Withdrawal::NotAsked => Finding::WithdrawalNotChecked,
            },
        )
    }

    /// Fold in whether this computer trusts the authority behind a signer's
    /// timestamp.
    ///
    /// Worth asking separately, because a timestamp from an authority nobody
    /// vouches for is worth no more than the sender's own claimed date: anyone
    /// can run a timestamp authority and sign any date they like.
    pub fn with_timestamp_authority_trust_for(self, which: usize, trust: IssuerTrust) -> Self {
        self.replacing(
            which,
            |finding| matches!(finding, Finding::TimestampAuthorityNotChecked { .. }),
            match trust {
                IssuerTrust::Trusted => Finding::TimestampAuthorityTrustedHere,
                IssuerTrust::NotTrusted { reason } => {
                    Finding::TimestampAuthorityNotTrustedHere { reason }
                }
                IssuerTrust::NotChecked { reason } => {
                    Finding::TimestampAuthorityNotChecked { reason }
                }
            },
        )
    }

    /// Swap one signer's open question for the answer, and rebuild the flat
    /// list so the two views cannot say different things.
    fn replacing(
        mut self,
        which: usize,
        was_open: impl Fn(&Finding) -> bool,
        answer: Finding,
    ) -> Self {
        let Some(signer) = self.signers.get_mut(which) else {
            return self;
        };
        signer.findings.retain(|finding| !was_open(finding));
        signer.findings.push(answer);
        // An answer that arrives later can change what the signature is worth,
        // and the word a caller branches on has to change with it. The
        // arithmetic of a message signed with a withdrawn certificate adds up
        // perfectly, which is the whole reason the arithmetic is not the
        // answer: withdrawing a certificate is what somebody does once their
        // key has been stolen. Left alone, anything picking a tick or a cross
        // from `outcome` would show the tick.
        signer.outcome = what_it_is_worth_now(signer.outcome, &signer.findings);
        self.findings = self
            .signers
            .iter()
            .flat_map(|signer| signer.findings.iter().cloned())
            .collect();
        self.outcome = agreed_outcome(&self.signers);
        self
    }
}

/// What a signature does not show, whatever any particular check found.
///
/// A free function as well as [`SignatureReport::limits`], because it is also
/// wanted where there is no report to ask: a message that says it is signed and
/// whose original form was not kept still tells somebody it is signed, and
/// "signed" is read as "genuine" whether or not anything was checked. Written
/// once here so the two surfaces cannot come to word it differently.
pub fn what_a_signature_is_worth() -> Vec<&'static str> {
    vec![
        "A signature shows two things: the message was not changed after it was signed, and \
         whoever signed it held the private key for the certificate attached.",
        "It does not show that the name shown as the sender is the person you have in mind.",
        "It does not show that anybody checked who the certificate was issued to.",
        "It does not show that the certificate has not been withdrawn since.",
        "It does not show when it was signed. The sender writes the date that travels with a \
         message, so only a timestamp from an authority settles that, and most messages carry \
         none.",
        "The subject line and the sender line travel outside the signature, so nothing here \
         covers them.",
    ]
}

/// What a signature is worth once a later answer has come back.
///
/// Only ever downgrades, and only from a signature that really does add up: a
/// finding cannot repair arithmetic that did not work, and one that says
/// nothing bad must not turn a failure into a pass.
///
/// Not yet known is not bad news and does not downgrade. Reporting every
/// message as worthless for as long as an answer took would train somebody to
/// ignore the word entirely, which is worse than not saying it.
fn what_it_is_worth_now(so_far: SignatureOutcome, findings: &[Finding]) -> SignatureOutcome {
    if so_far != SignatureOutcome::Matches {
        return so_far;
    }
    let worth_nothing = findings
        .iter()
        .any(|finding| matches!(finding, Finding::CertificateWithdrawn));
    if worth_nothing {
        return SignatureOutcome::MatchesButWorthNothing;
    }
    so_far
}

/// The wording that goes with a count.
///
/// Sentences here are read out loud, and "1 do not add up" is the sort of thing
/// that makes somebody stop and re-listen to work out what was meant.
fn one_or_more<'a>(count: usize, one: &'a str, more: &'a str) -> &'a str {
    if count == 1 { one } else { more }
}

/// Findings said in the order somebody needs to hear them.
fn worst_first(findings: &[Finding]) -> Vec<String> {
    let mut ordered: Vec<&Finding> = findings.iter().collect();
    ordered.sort_by_key(|finding| finding.weight());
    ordered.iter().map(|finding| finding.spoken()).collect()
}

/// What a whole message's signatures come to, given what each one came to.
///
/// Signatures that all say the same thing say that. Signatures that do not are
/// reported as disagreeing rather than as the best or the worst of them.
fn agreed_outcome(signers: &[SignerReport]) -> SignatureOutcome {
    let mut outcomes = signers.iter().map(|signer| signer.outcome);
    let Some(first) = outcomes.next() else {
        return SignatureOutcome::NothingToCheck;
    };
    if outcomes.all(|outcome| outcome == first) {
        first
    } else {
        SignatureOutcome::SignersDisagree
    }
}

// ── Checking a signature, with no operating system involved ──────────────────

/// Read a whole signed message and say what its signature is worth.
///
/// Never fails. A message that says it is signed and is not, or whose signature
/// will not parse, is the case this whole feature exists for, so it comes back
/// as a report saying so rather than as an error a caller might drop on the
/// floor and show nothing for.
///
/// `now` is passed in rather than read from the clock so that the answer to
/// "was the certificate in date" is about a moment the caller named, and so it
/// can be asked about a moment in a test.
pub fn examine_signed_message(
    raw_message: &[u8],
    sender_address: &str,
    now: DateTime<Utc>,
) -> SignatureReport {
    match take_apart(raw_message) {
        Ok(parts) => examine_signature(&parts, sender_address, now),
        Err(problem) => nothing_to_check(problem.to_string()),
    }
}

/// The same, for a caller that has already split the message.
pub fn examine_signature(
    parts: &SignedParts,
    sender_address: &str,
    now: DateTime<Utc>,
) -> SignatureReport {
    let signature = match Signature::read(&parts.signature) {
        Ok(signature) => signature,
        Err(problem) => return nothing_to_check(problem.to_string()),
    };
    if signature.signers.is_empty() {
        return SignatureReport {
            outcome: SignatureOutcome::NothingToCheck,
            signer: None,
            findings: vec![Finding::NoSignerInSignature],
            unwrapped_content: signature.wrapped_content,
            signers: Vec::new(),
        };
    }

    // The words the message shows, where it shows any, and only otherwise the
    // ones carried inside the signature.
    //
    // This order round, and it is the difference between a check and a
    // decoration. RFC 5652 carries the content outside the signature or inside
    // it and never both, so a message holding both is crafted, and the two ways
    // of resolving that are not equally wrong. Reading the inside one means
    // taking a real signature off somebody's message, wrapping it beside words
    // of your own, and having the arithmetic checked against the words that
    // came with the signature: the report then says "signed for
    // alice@example.com, and not changed since" over text Alice never wrote.
    // Reading the outside one checks the words that will be read, so a crafted
    // message fails the way a changed message fails, which is what it is.
    let content = parts
        .content
        .clone()
        .or_else(|| signature.wrapped_content.clone());

    // Every signer, not only the first. A message may be signed by more than
    // one party, and reading only the first means a second signature that does
    // not add up is never seen at all.
    let signers: Vec<SignerReport> = signature
        .signers
        .iter()
        .map(|signer| {
            examine_one_signer(&signature, signer, content.as_deref(), sender_address, now)
        })
        .collect();

    SignatureReport {
        outcome: agreed_outcome(&signers),
        signer: signers.first().and_then(|first| first.certificate.clone()),
        findings: signers
            .iter()
            .flat_map(|signer| signer.findings.iter().cloned())
            .collect(),
        unwrapped_content: signature.wrapped_content,
        signers,
    }
}

/// What one signer's signature comes to.
fn examine_one_signer(
    signature: &Signature,
    signer: &Signer,
    content: Option<&[u8]>,
    sender_address: &str,
    now: DateTime<Utc>,
) -> SignerReport {
    let mut findings = Vec::new();
    let Some(certificate) = certificate_for(signature, signer) else {
        findings.push(Finding::SignersCertificateMissing);
        findings.push(Finding::WithdrawalNotChecked);
        return SignerReport {
            outcome: SignatureOutcome::NotChecked,
            certificate: None,
            findings,
            signed_at: None,
        };
    };

    let Some(content) = content else {
        findings.push(Finding::CouldNotBeRead {
            reason: "the message it covers is not here".to_string(),
        });
        return SignerReport {
            outcome: SignatureOutcome::NotChecked,
            certificate: Some(certificate),
            findings,
            signed_at: None,
        };
    };

    let outcome = check_the_arithmetic(signature, signer, &certificate, content, &mut findings);
    let signed_at = fold_in_the_timestamp(signer, &mut findings);
    findings.extend(what_the_certificate_says(
        &certificate,
        sender_address,
        now,
        signed_at.as_ref().map(|stamped| stamped.moment),
    ));
    findings.push(Finding::IssuerNotChecked {
        reason: "this computer's own certificate store was not asked".to_string(),
    });
    findings.push(Finding::WithdrawalNotChecked);

    SignerReport {
        outcome,
        certificate: Some(certificate),
        findings,
        signed_at,
    }
}

/// Read the timestamp a signer carries, if it carries one, and say so either
/// way.
///
/// Said either way on purpose. "Nothing here says when this was signed" is a
/// fact somebody needs, and leaving it out on the messages that have no
/// timestamp would teach people that silence means a date was checked.
fn fold_in_the_timestamp(signer: &Signer, findings: &mut Vec<Finding>) -> Option<SigningTime> {
    let Some(token) = &signer.timestamp_token else {
        findings.push(Finding::WhenItWasSignedIsNotKnown);
        return None;
    };
    match read_timestamp(token, &signer.signature) {
        Ok(stamped) => {
            findings.push(Finding::SignedAtByATimestamp {
                moment: stamped.moment,
                authority: readable_name(&stamped.authority.subject),
            });
            findings.push(Finding::TimestampAuthorityNotChecked {
                reason: "this computer's own certificate store was not asked".to_string(),
            });
            Some(stamped)
        }
        Err(TimestampProblem::CoversSomethingElse) => {
            findings.push(Finding::TimestampIsForSomethingElse);
            findings.push(Finding::WhenItWasSignedIsNotKnown);
            None
        }
        Err(TimestampProblem::CouldNotBeRead(reason)) => {
            findings.push(Finding::TimestampCouldNotBeRead { reason });
            findings.push(Finding::WhenItWasSignedIsNotKnown);
            None
        }
    }
}

/// A report for a message whose signature could not be got at.
fn nothing_to_check(reason: String) -> SignatureReport {
    SignatureReport {
        outcome: SignatureOutcome::NothingToCheck,
        signer: None,
        findings: vec![Finding::CouldNotBeRead { reason }],
        unwrapped_content: None,
        signers: Vec::new(),
    }
}

/// The certificate belonging to this signer, out of the ones that came along.
fn certificate_for(signature: &Signature, signer: &Signer) -> Option<SignerCertificate> {
    signature
        .certificates
        .iter()
        .filter_map(|der| SignerCertificate::read(der).ok())
        .find(|candidate| certificate_belongs_to(&candidate.der, &signer.named))
}

/// Whether a certificate is the one a signer named.
fn certificate_belongs_to(der_bytes: &[u8], named: &SignerNamed) -> bool {
    let Ok((_, certificate)) = X509Certificate::from_der(der_bytes) else {
        return false;
    };
    match named {
        SignerNamed::IssuerAndSerial { issuer, serial } => {
            certificate.issuer().as_raw() == issuer.as_slice()
                && certificate.raw_serial() == serial.as_slice()
        }
        SignerNamed::KeyIdentifier(wanted) => {
            certificate
                .iter_extensions()
                .any(|extension| match extension.parsed_extension() {
                    ParsedExtension::SubjectKeyIdentifier(identifier) => identifier.0 == wanted,
                    _ => false,
                })
        }
    }
}

/// Do the sums, and add what they showed to the findings.
fn check_the_arithmetic(
    signature: &Signature,
    signer: &Signer,
    certificate: &SignerCertificate,
    content: &[u8],
    findings: &mut Vec<Finding>,
) -> SignatureOutcome {
    let digest = match digest_named(&signer.digest_algorithm) {
        Some(digest) => digest,
        None => {
            findings.push(Finding::SignatureKindNotUnderstood {
                named: format!("digest {}", signer.digest_algorithm),
            });
            return SignatureOutcome::NotChecked;
        }
    };

    // Where there are signed attributes, and there nearly always are, the
    // signature covers the attributes rather than the message, and one of the
    // attributes holds the message's fingerprint. So two things have to hold:
    // the fingerprint matches the message, and the signature matches the
    // attributes. Checking only the second is the mistake that lets somebody
    // swap the message and keep the signature.
    let signed_bytes = match &signer.signed_attributes {
        Some(attributes) => {
            let computed = ring::digest::digest(digest, content);
            match &signer.stated_digest {
                Some(stated) if stated.as_slice() == computed.as_ref() => {
                    findings.push(Finding::ContentIsWhatWasSigned);
                }
                Some(_) => {
                    findings.push(Finding::ContentIsNotWhatWasSigned);
                    return SignatureOutcome::DoesNotMatch;
                }
                None => {
                    findings.push(Finding::CouldNotBeRead {
                        reason: "the signature carries no fingerprint of the message".to_string(),
                    });
                    return SignatureOutcome::NotChecked;
                }
            }
            // RFC 5652 requires the attributes to name the same kind of content
            // the signature is attached to. A signature lifted off one document
            // and put on another can be caught here.
            if let Some(stated) = &signer.stated_content_type
                && *stated != signature.content_type
            {
                findings.push(Finding::SignatureCoversSomethingElse);
                return SignatureOutcome::DoesNotMatch;
            }
            attributes.clone()
        }
        // With no attributes the signature is over the message itself, so the
        // one check covers both questions at once.
        None => content.to_vec(),
    };

    let how = match verification_algorithm(certificate, signer) {
        Ok(how) => how,
        Err(named) => {
            findings.push(Finding::SignatureKindNotUnderstood { named });
            return SignatureOutcome::NotChecked;
        }
    };

    let key_bytes = match public_key_bytes(certificate) {
        Some(bytes) => bytes,
        None => {
            findings.push(Finding::CouldNotBeRead {
                reason: "the certificate's public key could not be read".to_string(),
            });
            return SignatureOutcome::NotChecked;
        }
    };

    let held = ring::signature::UnparsedPublicKey::new(how.algorithm, &key_bytes)
        .verify(&signed_bytes, &signer.signature)
        .is_ok();
    if !held {
        findings.push(Finding::NotSignedByThatCertificatesKey);
        return SignatureOutcome::DoesNotMatch;
    }
    findings.push(Finding::SignedByThatCertificatesKey);
    if signer.signed_attributes.is_none() {
        // Said out loud rather than left implied, because the sentence a person
        // hears should not depend on which of the two shapes the sender chose.
        findings.push(Finding::ContentIsWhatWasSigned);
    }

    // The fingerprint the arithmetic was actually done with, not the one the
    // signer declared it took of the content. Those are two fields and they
    // need not name the same hash: a signer declaring SHA-256 and signing with
    // sha1WithRSAEncryption used to verify through the SHA-1 verifier and come
    // back here as a signature worth trusting.
    if how.hash == oid::SHA1 {
        findings.push(Finding::FingerprintTooWeakToTrust {
            named: "SHA-1".to_string(),
        });
        return SignatureOutcome::MatchesButWorthNothing;
    }
    SignatureOutcome::Matches
}

/// The hash a digest identifier names.
fn digest_named(identifier: &str) -> Option<&'static ring::digest::Algorithm> {
    match identifier {
        oid::SHA1 => Some(&ring::digest::SHA1_FOR_LEGACY_USE_ONLY),
        oid::SHA256 => Some(&ring::digest::SHA256),
        oid::SHA384 => Some(&ring::digest::SHA384),
        oid::SHA512 => Some(&ring::digest::SHA512),
        _ => None,
    }
}

/// The bytes of a certificate's public key, in the shape a verifier wants.
///
/// For RSA that is the key's own structure and for an elliptic curve it is the
/// point, and in both cases those are exactly the bits inside the certificate's
/// key field, so nothing is re-encoded on the way.
fn public_key_bytes(certificate: &SignerCertificate) -> Option<Vec<u8>> {
    let (_, parsed) = X509Certificate::from_der(&certificate.der).ok()?;
    Some(parsed.public_key().subject_public_key.data.to_vec())
}

/// A verifier, and the fingerprint it is over.
///
/// The two travel together because two separate answers to one question is how
/// they came to disagree. Which verifier to use was read out of the signature
/// algorithm; whether the fingerprint can be forged was read out of the digest
/// the signer *says* it took of the content, which is a different field and
/// need not name the same hash. A signer declaring SHA-256 and signing with
/// sha1WithRSAEncryption was checked through the SHA-1 verifier and reported as
/// a signature that adds up, with nothing said about SHA-1 at all.
///
/// Returned as one value so the verdict cannot be about a fingerprint other
/// than the one the arithmetic was done with.
struct HowToVerify {
    algorithm: &'static dyn ring::signature::VerificationAlgorithm,
    /// The identifier of the hash `algorithm` is over.
    hash: String,
}

/// Which verifier suits this certificate and this signature, or the name of the
/// kind that is not handled.
fn verification_algorithm(
    certificate: &SignerCertificate,
    signer: &Signer,
) -> std::result::Result<HowToVerify, String> {
    use ring::signature;

    let hash = match signer.signature_algorithm.as_str() {
        // The plain RSA identifier says nothing about the hash, so the hash
        // comes from the signer's own digest algorithm. This is what OpenSSL
        // writes, so it is the common case rather than the odd one.
        oid::RSA_ENCRYPTION => signer.digest_algorithm.clone(),
        oid::SHA1_WITH_RSA => oid::SHA1.to_string(),
        oid::SHA256_WITH_RSA => oid::SHA256.to_string(),
        oid::SHA384_WITH_RSA => oid::SHA384.to_string(),
        oid::SHA512_WITH_RSA => oid::SHA512.to_string(),
        // RSA-PSS carries its hash inside its parameters rather than in its
        // name, so reading the parameters is the only way to know it.
        oid::RSA_PSS => {
            let hash = pss_hash(signer.signature_parameters.as_deref())
                .unwrap_or_else(|| oid::SHA1.to_string());
            return match hash.as_str() {
                oid::SHA256 => Ok(HowToVerify {
                    algorithm: &signature::RSA_PSS_2048_8192_SHA256,
                    hash,
                }),
                oid::SHA384 => Ok(HowToVerify {
                    algorithm: &signature::RSA_PSS_2048_8192_SHA384,
                    hash,
                }),
                oid::SHA512 => Ok(HowToVerify {
                    algorithm: &signature::RSA_PSS_2048_8192_SHA512,
                    hash,
                }),
                other => Err(format!("RSA-PSS over {}", readable_digest(other))),
            };
        }
        oid::ECDSA_WITH_SHA256 | oid::ECDSA_WITH_SHA384 => {
            return ecdsa_algorithm(certificate, &signer.signature_algorithm);
        }
        other => return Err(format!("signature algorithm {other}")),
    };

    let algorithm: &'static dyn ring::signature::VerificationAlgorithm = match hash.as_str() {
        oid::SHA1 => &signature::RSA_PKCS1_2048_8192_SHA1_FOR_LEGACY_USE_ONLY,
        oid::SHA256 => &signature::RSA_PKCS1_2048_8192_SHA256,
        oid::SHA384 => &signature::RSA_PKCS1_2048_8192_SHA384,
        oid::SHA512 => &signature::RSA_PKCS1_2048_8192_SHA512,
        other => return Err(format!("RSA over {}", readable_digest(other))),
    };
    Ok(HowToVerify { algorithm, hash })
}

/// Which curve verifier suits an elliptic curve certificate.
///
/// The curve comes from the certificate rather than from the signature,
/// because the signature only names the hash and a verifier has to be told
/// both. Picking the wrong curve reports a perfectly good signature as bad.
fn ecdsa_algorithm(
    certificate: &SignerCertificate,
    signature_algorithm: &str,
) -> std::result::Result<HowToVerify, String> {
    use ring::signature;

    let Ok((_, parsed)) = X509Certificate::from_der(&certificate.der) else {
        return Err("an elliptic curve signature whose certificate would not parse".to_string());
    };
    let key = parsed.public_key();
    if key.algorithm.algorithm.to_id_string() != oid::EC_PUBLIC_KEY {
        return Err("an elliptic curve signature made with a key that is not one".to_string());
    }
    let curve = key
        .algorithm
        .parameters
        .as_ref()
        .and_then(|parameters| parameters.as_oid().ok())
        .map(|curve| curve.to_id_string())
        .unwrap_or_default();
    match (curve.as_str(), signature_algorithm) {
        (oid::CURVE_P256, oid::ECDSA_WITH_SHA256) => Ok(HowToVerify {
            algorithm: &signature::ECDSA_P256_SHA256_ASN1,
            hash: oid::SHA256.to_string(),
        }),
        (oid::CURVE_P384, oid::ECDSA_WITH_SHA384) => Ok(HowToVerify {
            algorithm: &signature::ECDSA_P384_SHA384_ASN1,
            hash: oid::SHA384.to_string(),
        }),
        _ => Err(format!("an elliptic curve signature on curve {curve}")),
    }
}

/// The hash named inside RSA-PSS parameters, when they name one.
fn pss_hash(parameters: Option<&[u8]>) -> Option<String> {
    let (outer, _) = der::take(parameters?).ok()?;
    if outer.tag != der::SEQUENCE {
        return None;
    }
    let hash_field = der::children(outer.value)
        .ok()?
        .into_iter()
        .find(|field| field.tag == der::context(0))?;
    let (identifier, _) = der::take(hash_field.value).ok()?;
    let inside = der::children(identifier.value).ok()?;
    read_oid(inside.first(), "The PSS hash").ok()
}

/// A digest identifier the way somebody would say it.
fn readable_digest(identifier: &str) -> String {
    match identifier {
        oid::SHA1 => "SHA-1".to_string(),
        oid::SHA256 => "SHA-256".to_string(),
        oid::SHA384 => "SHA-384".to_string(),
        oid::SHA512 => "SHA-512".to_string(),
        other => other.to_string(),
    }
}

// ── When it was signed ───────────────────────────────────────────────────────

/// Why a timestamp came to nothing.
///
/// Two cases and not one, because they mean different things to whoever is
/// reading: a timestamp for another document is a mismatch worth saying out
/// loud, and a timestamp this program cannot read is this program's shortfall.
enum TimestampProblem {
    CoversSomethingElse,
    CouldNotBeRead(String),
}

/// Read an RFC 3161 timestamp token and check it really covers `signature`.
///
/// A timestamp is a second signed document, made by somebody who is not the
/// sender, saying that a particular run of bytes existed at a particular
/// moment. The bytes it covers are the signature value, so a sender cannot lift
/// a timestamp off one message and put it on another, and cannot choose the
/// date. The sender's own claimed signing time, which travels in the signed
/// attributes, is ignored on purpose and stays ignored: the sender writes it.
///
/// What this does not settle is whether the authority is one worth believing.
/// Anyone can run a timestamp authority. That question belongs to the machine's
/// own list of trusted authorities, which is why the authority's certificate
/// comes back in [`SigningTime`] for the caller to ask about.
fn read_timestamp(
    token: &[u8],
    signature: &[u8],
) -> std::result::Result<SigningTime, TimestampProblem> {
    let document = Signature::read(token)
        .map_err(|problem| TimestampProblem::CouldNotBeRead(problem.to_string()))?;
    check_timestamp(&document, signature)
}

/// The same, on a token already taken apart.
///
/// Split from the reading so each refusal can be reached from a test by
/// changing one field of a real token, rather than by needing a whole message
/// built to be wrong in that one way.
fn check_timestamp(
    document: &Signature,
    signature: &[u8],
) -> std::result::Result<SigningTime, TimestampProblem> {
    let unreadable = |reason: &str| TimestampProblem::CouldNotBeRead(reason.to_string());

    if document.content_type != oid::TIMESTAMP_STATEMENT {
        return Err(unreadable(
            "it does not hold a timestamp authority's statement",
        ));
    }
    let statement = document
        .wrapped_content
        .as_deref()
        .ok_or_else(|| unreadable("the authority's statement is not inside it"))?;
    let authority_signer = document
        .signers
        .first()
        .ok_or_else(|| unreadable("nobody signed it"))?;
    let authority = certificate_for(document, authority_signer)
        .ok_or_else(|| unreadable("the authority's certificate did not come with it"))?;

    // A timestamp made with a fingerprint that can be forged is worth as little
    // as a signature made with one, and here it would be worse: it would move a
    // certificate's dates.
    if authority_signer.digest_algorithm == oid::SHA1 {
        return Err(unreadable("it was made with SHA-1, which can be forged"));
    }
    let mut ignored = Vec::new();
    match check_the_arithmetic(
        document,
        authority_signer,
        &authority,
        statement,
        &mut ignored,
    ) {
        SignatureOutcome::Matches => {}
        _ => {
            return Err(unreadable(
                "the authority's own signature on it does not add up",
            ));
        }
    }

    let statement = TimestampStatement::read(statement)
        .map_err(|problem| TimestampProblem::CouldNotBeRead(problem.to_string()))?;
    if !statement.covers(signature) {
        return Err(TimestampProblem::CoversSomethingElse);
    }
    Ok(SigningTime {
        moment: statement.moment,
        authority,
    })
}

/// The authority's statement inside a timestamp token.
struct TimestampStatement {
    /// The hash the authority was asked about, and what made it.
    imprint_algorithm: String,
    imprint: Vec<u8>,
    moment: DateTime<Utc>,
}

impl TimestampStatement {
    /// Read a `TSTInfo`.
    ///
    /// Only the three fields that decide anything: what was stamped, and when.
    /// The policy, serial number and accuracy are the authority's own
    /// bookkeeping and change nothing a reader is told.
    fn read(der_bytes: &[u8]) -> Result<Self> {
        let (outer, _) = der::take(der_bytes)?;
        let outer = expect(outer, der::SEQUENCE, "The timestamp's statement")?;
        let fields = der::children(outer.value)?;
        // version, policy, messageImprint, serialNumber, genTime, then optional
        // fields nothing here reads.
        let imprint = fields
            .get(2)
            .filter(|element| element.tag == der::SEQUENCE)
            .ok_or_else(|| Error::Security("The timestamp names nothing it covers".to_string()))?;
        let inside = der::children(imprint.value)?;
        let imprint_algorithm = algorithm_identifier(inside.first(), "The timestamp's digest")?.0;
        let imprint = inside
            .get(1)
            .filter(|element| element.tag == der::OCTET_STRING)
            .ok_or_else(|| {
                Error::Security(
                    "The timestamp carries no fingerprint of what it covers".to_string(),
                )
            })?
            .value
            .to_vec();
        let moment = fields
            .get(4)
            .filter(|element| element.tag == der::GENERALIZED_TIME)
            .ok_or_else(|| Error::Security("The timestamp carries no moment".to_string()))
            .and_then(|element| generalized_time(element.value))?;
        Ok(Self {
            imprint_algorithm,
            imprint,
            moment,
        })
    }

    /// Whether this statement is about these exact bytes.
    fn covers(&self, bytes: &[u8]) -> bool {
        let Some(digest) = digest_named(&self.imprint_algorithm) else {
            return false;
        };
        ring::digest::digest(digest, bytes).as_ref() == self.imprint.as_slice()
    }
}

/// The part of a certificate's subject a person would call its name.
///
/// A subject is a run of `field=value` pairs, and read out whole it comes back
/// as "C N equals Wixen Test Timestamps comma O equals". The common name is the
/// piece that names the thing; the whole subject stays on the certificate for
/// anything that has to tell two of them apart.
fn readable_name(subject: &str) -> String {
    split_outside_quotes(subject, ',')
        .filter_map(|piece| piece.trim().strip_prefix("CN=").map(str::trim))
        .find(|name| !name.is_empty())
        .unwrap_or(subject)
        .to_string()
}

/// A moment out of an ASN.1 `GeneralizedTime`.
///
/// RFC 3161 requires the form `YYYYMMDDHHMMSS[.fff]Z`, so anything with an
/// offset or a missing zone is refused rather than guessed at. Guessing would
/// move a signing time by hours, which is the whole thing this reads.
fn generalized_time(value: &[u8]) -> Result<DateTime<Utc>> {
    let text = std::str::from_utf8(value)
        .map_err(|_| Error::Security("The timestamp's moment is not readable".to_string()))?;
    let unzoned = text
        .strip_suffix('Z')
        .ok_or_else(|| Error::Security("The timestamp's moment names no time zone".to_string()))?;
    let seconds = unzoned.split('.').next().unwrap_or_default();
    let moment = chrono::NaiveDateTime::parse_from_str(seconds, "%Y%m%d%H%M%S")
        .map_err(|_| Error::Security(format!("The timestamp's moment reads as {text}")))?;
    Ok(moment.and_utc())
}

/// What the certificate itself says, apart from whether the sums worked.
///
/// `signed_at` is the moment a timestamp authority says the signature was made,
/// when there is a timestamp and it checked out. It changes what the dates
/// mean: a certificate that has run out since says nothing bad about a
/// signature made while it was still good, and that is the difference a
/// timestamp exists to make.
fn what_the_certificate_says(
    certificate: &SignerCertificate,
    sender_address: &str,
    now: DateTime<Utc>,
    signed_at: Option<DateTime<Utc>>,
) -> Vec<Finding> {
    let mut findings = vec![what_the_dates_say(certificate, now, signed_at)];

    if certificate.email_addresses.is_empty() {
        findings.push(Finding::CertificateNamesNobody);
    } else if certificate.names(sender_address) {
        findings.push(Finding::CertificateNamesTheSender {
            address: sender_address.trim().to_ascii_lowercase(),
        });
    } else {
        findings.push(Finding::CertificateNamesSomebodyElse {
            certificate_names: certificate.email_addresses.clone(),
            message_says: sender_address.trim().to_ascii_lowercase(),
        });
    }

    if certificate.self_issued {
        findings.push(Finding::CertificateIssuedItself);
    }
    findings
}

/// What the certificate's dates come to, which depends on whether anything
/// says when the message was signed.
fn what_the_dates_say(
    certificate: &SignerCertificate,
    now: DateTime<Utc>,
    signed_at: Option<DateTime<Utc>>,
) -> Finding {
    let Some(signed_on) = signed_at else {
        // With nothing to say when it was signed, the only moment there is to
        // ask about is now, and the sentence for a certificate that has run out
        // says plainly that when it was signed is unknown.
        return if now > certificate.valid_until {
            Finding::CertificateHadExpired {
                expired_on: certificate.valid_until,
            }
        } else if now < certificate.valid_from {
            Finding::CertificateHasNotStarted {
                starts_on: certificate.valid_from,
            }
        } else {
            Finding::CertificateWasInDate
        };
    };
    if signed_on < certificate.valid_from || signed_on > certificate.valid_until {
        return Finding::CertificateWasNotInDateWhenSigned {
            signed_on,
            valid_from: certificate.valid_from,
            valid_until: certificate.valid_until,
        };
    }
    Finding::CertificateWasInDateWhenSigned {
        signed_on,
        expired_since: (now > certificate.valid_until).then_some(certificate.valid_until),
    }
}

// ── The one boundary: this computer's own certificate store ──────────────────

/// What this computer's own certificate store says about an issuer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssuerTrust {
    /// The machine's list of authorities vouches for the chain above it.
    Trusted,
    /// It does not, and here is why in words.
    NotTrusted { reason: String },
    /// Nobody could ask, and here is why.
    NotChecked { reason: String },
}

/// Whether a certificate has been withdrawn since it was issued.
///
/// Withdrawal is what somebody does after their private key is stolen, so it is
/// the one question that separates "this certificate was good" from "this
/// certificate is good". A reader that never asks reports a certificate
/// withdrawn a year ago exactly as it reports a sound one.
///
/// Five answers rather than two, because "nobody asked" and "somebody asked and
/// could not find out" and "the answer has not come back yet" are all different
/// from "it has not been withdrawn", and only the last of those is good news.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Withdrawal {
    /// Whoever issued it has said it is no longer to be relied on.
    Withdrawn,
    /// Something really looked at a withdrawal list and this certificate is not
    /// on it.
    NotWithdrawn,
    /// Somebody is looking and the answer is not back yet.
    ///
    /// The state that lets reading a message stay instant. A report can be put
    /// in front of somebody saying the question is open, and the answer folded
    /// in with [`SignatureReport::with_withdrawal_for`] when it arrives.
    StillBeingLookedInto,
    /// Something looked and could not find out, with the reason in words.
    CouldNotFindOut { reason: String },
    /// Nobody looked.
    NotAsked,
}

/// How far a question about withdrawal is allowed to go.
///
/// # Why this is a choice and not a setting buried in the code
///
/// Finding out whether a certificate has been withdrawn means reading a list
/// the authority publishes, and that list lives on the authority's own server.
/// Fetching it tells that server this computer is looking at that certificate,
/// now, from this address.
///
/// `application::pictures` refuses exactly that shape: a picture a message
/// points at is not fetched, because fetching it tells the sender the message
/// was opened. The two are not the same, and the difference is worth being
/// precise about, because it decides what is allowed here.
///
/// - A remote picture's address is **the sender's**. The sender picks the
///   server and can make the address different for every recipient, which turns
///   the fetch into a read receipt naming one person. That is what a tracking
///   pixel is.
/// - A withdrawal list's address is **the authority's**, written into the
///   certificate when it was issued and covered by the authority's signature
///   over it. The sender cannot vary it per recipient, and the authority is not
///   the sender. It is also fetched by everything else on the machine that
///   checks a certificate, and cached for days.
///
/// So it is a smaller leak, to a different party, and it does not report back
/// to the person whose honesty is in question. That is why asking is offered at
/// all rather than refused outright.
///
/// It is not a small enough leak to be the default, for two reasons. The first
/// is that it is still a request this computer makes because a stranger sent a
/// message, and this program has already answered that question the other way
/// once. The second is sharper: **for a certificate this computer does not
/// already trust, the address really is the sender's to choose.** Anyone can
/// make a certificate and write their own tracking address into it. So asking
/// is only ever done for a certificate whose chain this computer already
/// trusts, which is the condition that makes the first paragraph's reasoning
/// true. That condition is enforced rather than assumed: turning the setting on
/// does not let a stranger's own certificate name a server for this computer to
/// call.
///
/// [`Reach::WhatIsAlreadyHere`] is the default and is what reading a message
/// uses. It asks nobody anything, waits for nothing, and still answers whenever
/// this computer already holds a withdrawal list for the issuer, which is
/// common because other software fetches them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Only what this computer already holds. Contacts nobody, waits for
    /// nothing, and is safe to use while a message is being displayed.
    WhatIsAlreadyHere,
    /// Allowed to contact the authority named in the certificate, which tells
    /// that authority this computer is looking at that certificate now.
    ///
    /// Never on the path that displays a message: it waits on a network. A
    /// caller that wants this runs it after the report is already in front of
    /// somebody and folds the answer in when it comes back.
    AskTheAuthority,
}

impl Reach {
    /// What the setting means, read the way the setting is worded.
    ///
    /// The setting asks whether authorities may be contacted, and the safe
    /// answer is off, so somebody who does nothing tells nobody anything. This
    /// mirrors `application::pictures::Fetching::from_setting`, deliberately:
    /// the two settings are the same shape and should not read differently.
    pub fn from_setting(may_ask_authorities: bool) -> Self {
        if may_ask_authorities {
            Reach::AskTheAuthority
        } else {
            Reach::WhatIsAlreadyHere
        }
    }
}

/// Who a message was encrypted to, as the message names them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    /// The exact bytes of the issuer's name, as they arrived.
    pub issuer: Vec<u8>,
    /// The serial number that issuer gave the certificate.
    pub serial: Vec<u8>,
    /// The key the content key is wrapped with.
    pub key_wrapping_algorithm: String,
    /// The wrapped content key.
    pub wrapped_key: Vec<u8>,
}

/// The machine's own store of certificates and private keys.
///
/// **This trait is the entire platform boundary of S/MIME in this project.**
/// Everything else in this file is plain Rust that a macOS or Linux build
/// compiles unchanged. Porting means writing one more implementation of these
/// three methods, against Keychain and the Security framework on macOS, and
/// changing nothing else here.
///
/// The three are here and not elsewhere because each of them needs something
/// only the operating system has: a list of authorities the person running this
/// has agreed to trust, and a private key that in the ordinary case cannot be
/// taken out of the store at all, so the arithmetic has to happen where the key
/// lives.
pub trait CertificateStore {
    /// Whether this computer trusts whoever issued a certificate.
    ///
    /// Takes the certificate as the bytes it arrived in, because that is the
    /// one form every platform's store will accept, and because handing over a
    /// parsed object would make each port re-derive what it already has.
    fn issuer_trust(&self, certificate_der: &[u8], now: DateTime<Utc>) -> IssuerTrust;

    /// Whether a certificate has been withdrawn since it was issued.
    ///
    /// Here rather than in the portable half because the withdrawal lists this
    /// computer already holds are the operating system's, and so is the
    /// machinery that fetches a new one. `reach` says whether fetching is
    /// allowed at all; see [`Reach`] for why that is the caller's choice and
    /// what the default is.
    fn withdrawal(&self, certificate_der: &[u8], now: DateTime<Utc>, reach: Reach) -> Withdrawal;

    /// Whether this computer holds the private key for one of a message's
    /// recipients, and which one.
    fn which_recipient_is_us(&self, recipients: &[Recipient]) -> Result<Option<usize>>;

    /// Undo the wrapping on the key a message's content was encrypted with.
    fn unwrap_content_key(&self, recipient: &Recipient) -> Result<Vec<u8>>;
}

/// The store belonging to the computer this is running on.
pub fn this_computers_certificates() -> Box<dyn CertificateStore> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows_store::WindowsCertificateStore::the_persons_own())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(NoCertificateStore)
    }
}

/// The stand-in on a platform that has no implementation yet.
///
/// It refuses rather than guessing, and every refusal says which platform and
/// what is missing. A store that quietly answered "trusted" would turn the one
/// question this file cannot answer for itself into a fabricated yes, which is
/// worse than having no answer.
#[cfg(not(target_os = "windows"))]
pub struct NoCertificateStore;

#[cfg(not(target_os = "windows"))]
impl CertificateStore for NoCertificateStore {
    fn issuer_trust(&self, _certificate_der: &[u8], _now: DateTime<Utc>) -> IssuerTrust {
        IssuerTrust::NotChecked {
            reason: "Wixen Mail cannot read this operating system's certificate store yet"
                .to_string(),
        }
    }

    fn withdrawal(
        &self,
        _certificate_der: &[u8],
        _now: DateTime<Utc>,
        _reach: Reach,
    ) -> Withdrawal {
        Withdrawal::CouldNotFindOut {
            reason: "Wixen Mail cannot read this operating system's certificate store yet"
                .to_string(),
        }
    }

    fn which_recipient_is_us(&self, _recipients: &[Recipient]) -> Result<Option<usize>> {
        Err(Error::Security(
            "Wixen Mail cannot read this operating system's certificate store yet".to_string(),
        ))
    }

    fn unwrap_content_key(&self, _recipient: &Recipient) -> Result<Vec<u8>> {
        Err(Error::Security(
            "Wixen Mail cannot read this operating system's certificate store yet".to_string(),
        ))
    }
}

/// Which recipient of an encrypted message a certificate belongs to.
///
/// Split out of the platform code on purpose. Matching an issuer's name and a
/// serial number against a certificate is arithmetic on bytes and is the same
/// everywhere, so it is written and tested here, and the platform half is left
/// with only the part that really is the platform's: handing over the
/// certificates the machine holds.
fn recipient_matching(recipients: &[Recipient], certificate_der: &[u8]) -> Option<usize> {
    let (_, certificate) = X509Certificate::from_der(certificate_der).ok()?;
    recipients.iter().position(|recipient| {
        !recipient.issuer.is_empty()
            && certificate.issuer().as_raw() == recipient.issuer.as_slice()
            && certificate.raw_serial() == recipient.serial.as_slice()
    })
}

/// Windows' own certificate store, reached through Crypt32.
///
/// The only platform code in S/MIME, and it is here because a list of trusted
/// authorities and a private key that cannot leave the store are things only
/// Windows has. Flat calls behind a small `#[link]` block, which is how the
/// rest of this project talks to Windows outside the spell checker.
#[cfg(target_os = "windows")]
pub mod windows_store {
    use super::{CertificateStore, IssuerTrust, Reach, Recipient, Withdrawal, recipient_matching};
    use crate::common::{Error, Result};
    use chrono::{DateTime, Utc};
    use std::ffi::c_void;

    const X509_ASN_ENCODING: u32 = 0x0000_0001;
    const PKCS_7_ASN_ENCODING: u32 = 0x0001_0000;
    /// The certificate has key provider information attached, which is Windows'
    /// way of saying this machine holds the matching private key.
    const CERT_KEY_PROV_INFO_PROP_ID: u32 = 2;
    /// The other way Windows records a private key: a key handle attached to
    /// the certificate rather than a note of where the key is stored. A key
    /// imported into memory only has this and not the note, so a check that
    /// looks for the note alone says no to a key that is really there.
    const CERT_NCRYPT_KEY_HANDLE_PROP_ID: u32 = 78;
    /// Every named use has to be present. With no uses named this matches
    /// everything, which is what is wanted: what a certificate may be used for
    /// is a separate question from whether the machine trusts who issued it.
    const USAGE_MATCH_TYPE_AND: u32 = 0;

    /// Build the chain from what this computer already holds, fetching nothing.
    ///
    /// Without this Windows will download whatever the certificate's own
    /// extensions point at to complete a chain, from an address the sender
    /// chose if the sender made the certificate. That is a fetch caused by
    /// opening a message, going to a server named by whoever sent it, which is
    /// the shape `application::pictures` refuses for remote images. It also
    /// waits on a network while a message is being displayed.
    const CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL: u32 = 0x0000_0004;
    /// Ask about withdrawal for every certificate in the chain except the one
    /// at the top. The top is a root: nothing publishes a withdrawal list for
    /// it, and asking would make every answer "could not find out".
    const CERT_CHAIN_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT: u32 = 0x4000_0000;
    /// Answer the withdrawal question from lists already here and fetch none.
    const CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY: u32 = 0x8000_0000;
    /// Spend at most `url_retrieval_timeout` in total on fetching, rather than
    /// that long on each of however many addresses a chain names.
    const CERT_CHAIN_REVOCATION_ACCUMULATIVE_TIMEOUT: u32 = 0x0800_0000;

    /// The longest a withdrawal question may spend on the network, in
    /// milliseconds, and only ever off the path that displays a message.
    const LONGEST_WAIT_FOR_AN_AUTHORITY: u32 = 5_000;

    /// Windows' answers to "is this certificate withdrawn", as it numbers them.
    const REVOCATION_IS_FINE: u32 = 0;
    const CRYPT_E_REVOKED: u32 = 0x8009_2010;
    const CRYPT_E_NO_REVOCATION_DLL: u32 = 0x8009_2011;
    const CRYPT_E_NO_REVOCATION_CHECK: u32 = 0x8009_2012;
    const CRYPT_E_REVOCATION_OFFLINE: u32 = 0x8009_2013;

    /// Windows' bit for a certificate somewhere in the chain being withdrawn.
    const CERT_TRUST_IS_REVOKED: u32 = 0x0000_0004;

    /// Closing a store frees its certificates even if something still holds
    /// one. Used only for the in-memory store, which nothing outlives.
    #[cfg(test)]
    const CERT_CLOSE_STORE_FORCE_FLAG: u32 = 0x0000_0001;

    /// Keep an imported private key in this process and write nothing to disk.
    /// This is what makes importing a key for a test safe: the person's own
    /// certificate store is not touched and nothing survives the process.
    #[cfg(test)]
    const PKCS12_NO_PERSIST_KEY: u32 = 0x0000_8000;
    /// Required alongside the flag above: the key has to go to a CNG provider,
    /// because that is the only kind Windows can hold without persisting.
    #[cfg(test)]
    const PKCS12_ALWAYS_CNG_KSP: u32 = 0x0000_0200;

    /// Only the meanings worth putting into a sentence. Windows sets others
    /// that repeat what the portable checks already said.
    ///
    /// Each is the constant Windows names, spelled here as the number rather
    /// than the name because there is no header to include. Two of these used
    /// to say something the bit does not mean: `0x02` is about a certificate's
    /// dates not sitting inside its issuer's, not about anything being
    /// cancelled, and `0x0800` is about name constraints. Explicit distrust,
    /// which one of them claimed to be, is `0x0400_0000`.
    const TRUST_BITS: [(u32, &str); 9] = [
        (0x0000_0001, "the certificate is outside its dates"),
        (
            0x0000_0002,
            "the certificate's dates run outside those of whoever issued it",
        ),
        (0x0000_0004, "the certificate has been withdrawn"),
        (
            0x0000_0008,
            "the certificate's own signature does not check out",
        ),
        (0x0000_0010, "the certificate is not meant for this use"),
        (
            0x0000_0020,
            "the authority at the top of the chain is not one this computer trusts",
        ),
        (
            0x0000_0800,
            "the certificate names something whoever issued it is not allowed to vouch for",
        ),
        (
            0x0400_0000,
            "somebody has marked the certificate as not to be trusted on this computer",
        ),
        (
            0x0001_0000,
            "the chain of certificates above it is incomplete",
        ),
    ];

    #[repr(C)]
    struct CertContext {
        encoding_type: u32,
        encoded: *const u8,
        encoded_length: u32,
        info: *mut c_void,
        store: *mut c_void,
    }

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct EnhancedKeyUsage {
        count: u32,
        identifiers: *mut *mut i8,
    }

    #[repr(C)]
    struct UsageMatch {
        kind: u32,
        usage: EnhancedKeyUsage,
    }

    /// Windows' chain parameters, all of them.
    ///
    /// All of them and not just the first two, because `size` is how Windows
    /// decides which fields are present and the timeout is one of the later
    /// ones. Everything not filled in is left zero, which is the same as not
    /// asking for it.
    #[repr(C)]
    struct ChainParameters {
        size: u32,
        requested_usage: UsageMatch,
        requested_issuance_policy: UsageMatch,
        url_retrieval_timeout: u32,
        check_revocation_freshness_time: i32,
        revocation_freshness_time: u32,
        cache_resync: *mut c_void,
        strong_sign_parameters: *mut c_void,
        strong_sign_flags: u32,
    }

    /// Windows' two words about how much it likes something.
    #[repr(C)]
    struct TrustStatus {
        error_status: u32,
        info_status: u32,
    }

    /// The front of Windows' chain result.
    ///
    /// Windows' own structure continues past this. Only what is read is
    /// declared, and a field this never touches is a field this can get wrong.
    #[repr(C)]
    struct ChainContext {
        size: u32,
        trust: TrustStatus,
        chain_count: u32,
        chains: *const *const SimpleChain,
    }

    /// One chain of certificates, from the message's own up to a root.
    #[repr(C)]
    struct SimpleChain {
        size: u32,
        trust: TrustStatus,
        element_count: u32,
        elements: *const *const ChainElement,
    }

    /// One certificate's place in a chain, and what was found out about it.
    #[repr(C)]
    struct ChainElement {
        size: u32,
        certificate: *const CertContext,
        trust: TrustStatus,
        revocation: *const RevocationInfo,
    }

    /// What asking about withdrawal came to for one certificate.
    ///
    /// The field that matters is `result`, and the reason to read it rather
    /// than the chain's error bits is that it says whether anything was asked
    /// at all. A chain with no withdrawal errors may mean nothing was withdrawn
    /// or may mean nobody looked, and those must not be reported the same way.
    #[repr(C)]
    struct RevocationInfo {
        size: u32,
        result: u32,
    }

    /// A run of bytes, the way Windows takes one.
    #[cfg(test)]
    #[repr(C)]
    struct DataBlob {
        length: u32,
        data: *const u8,
    }

    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CertCreateCertificateContext(
            encoding: u32,
            encoded: *const u8,
            length: u32,
        ) -> *const CertContext;
        fn CertFreeCertificateContext(context: *const CertContext) -> i32;
        fn CertGetCertificateChain(
            engine: *mut c_void,
            certificate: *const CertContext,
            time: *const FileTime,
            additional_store: *mut c_void,
            parameters: *const ChainParameters,
            flags: u32,
            reserved: *mut c_void,
            chain: *mut *const ChainContext,
        ) -> i32;
        fn CertFreeCertificateChain(chain: *const ChainContext);
        fn CertOpenSystemStoreW(provider: usize, name: *const u16) -> *mut c_void;
        fn CertCloseStore(store: *mut c_void, flags: u32) -> i32;
        fn CertEnumCertificatesInStore(
            store: *mut c_void,
            previous: *const CertContext,
        ) -> *const CertContext;
        fn CertGetCertificateContextProperty(
            context: *const CertContext,
            property: u32,
            data: *mut c_void,
            size: *mut u32,
        ) -> i32;
        #[cfg(test)]
        fn PFXImportCertStore(
            pfx: *const DataBlob,
            password: *const u16,
            flags: u32,
        ) -> *mut c_void;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLastError() -> u32;
    }

    /// Where a store implementation looks for certificates this machine holds
    /// the private key for.
    enum Wherever {
        /// The store Windows keeps for the person signed in. This is what the
        /// running program uses, and nothing here ever writes to it.
        ThePersonsOwnStore,
        /// A store that exists only in this process. Nothing is written to disk
        /// and the person's own store is not touched. It exists so the private
        /// key question can be asked with a key that is really there, on a
        /// machine that has no S/MIME certificate installed.
        #[cfg(test)]
        OnlyInMemory(InMemoryStore),
    }

    /// A store handle that closes itself.
    #[cfg(test)]
    struct InMemoryStore(*mut c_void);

    #[cfg(test)]
    impl Drop for InMemoryStore {
        fn drop(&mut self) {
            // SAFETY: the handle came from PFXImportCertStore, is closed once,
            // and nothing outlives this value.
            unsafe {
                CertCloseStore(self.0, CERT_CLOSE_STORE_FORCE_FLAG);
            }
        }
    }

    /// Windows' own certificate store.
    pub struct WindowsCertificateStore {
        looking_in: Wherever,
    }

    impl WindowsCertificateStore {
        /// The store Windows keeps for the person signed in.
        pub fn the_persons_own() -> Self {
            Self {
                looking_in: Wherever::ThePersonsOwnStore,
            }
        }

        /// A store holding whatever is in some PKCS #12 bytes, in memory only.
        ///
        /// `PKCS12_NO_PERSIST_KEY` is what makes this safe to run on somebody's
        /// own machine: Windows keeps the private key inside this process and
        /// writes nothing to the person's certificate store, so nothing is left
        /// behind when the process ends.
        ///
        /// Only reachable from this crate. It exists so the question "does this
        /// machine hold the private key for that certificate" can be asked with
        /// a key that is really there, which is otherwise untestable on a
        /// machine with no S/MIME certificate installed.
        #[cfg(test)]
        pub(in crate::service) fn holding_only_in_memory(
            pkcs12: &[u8],
            password: &str,
        ) -> Result<Self> {
            let mut wide: Vec<u16> = password.encode_utf16().collect();
            wide.push(0);
            let blob = DataBlob {
                length: pkcs12.len() as u32,
                data: pkcs12.as_ptr(),
            };
            // SAFETY: the bytes and the password outlive the call, and the
            // handle that comes back is owned by the value being built.
            let handle = unsafe {
                PFXImportCertStore(
                    &blob,
                    wide.as_ptr(),
                    PKCS12_NO_PERSIST_KEY | PKCS12_ALWAYS_CNG_KSP,
                )
            };
            if handle.is_null() {
                return Err(Error::Security(format!(
                    "Windows would not read those PKCS #12 bytes ({:#010x})",
                    last_error()
                )));
            }
            Ok(Self {
                looking_in: Wherever::OnlyInMemory(InMemoryStore(handle)),
            })
        }

        /// Every certificate this store holds whose private key it also holds.
        pub(super) fn certificates_we_hold_keys_for(&self) -> Vec<Vec<u8>> {
            match &self.looking_in {
                Wherever::ThePersonsOwnStore => certificates_in("MY", true),
                // SAFETY: the handle is owned by this value and open for as
                // long as the borrow lasts.
                #[cfg(test)]
                Wherever::OnlyInMemory(store) => unsafe { certificates_held_by(store.0, true) },
            }
        }
    }

    impl CertificateStore for WindowsCertificateStore {
        fn issuer_trust(&self, certificate_der: &[u8], now: DateTime<Utc>) -> IssuerTrust {
            match build_chain(certificate_der, now, Asking::Nothing) {
                Err(reason) => IssuerTrust::NotChecked { reason },
                Ok(chain) => match chain.error_status {
                    0 => IssuerTrust::Trusted,
                    status => match describe(status) {
                        // Windows can set bits this does not put into words.
                        // Saying "not trusted, and I will not say why" is worse
                        // than saying it was not settled.
                        reasons if reasons.is_empty() => IssuerTrust::NotChecked {
                            reason: format!(
                                "Windows reported a state this does not know ({status:#010x})"
                            ),
                        },
                        reasons => IssuerTrust::NotTrusted {
                            reason: reasons.join(", and "),
                        },
                    },
                },
            }
        }

        fn withdrawal(
            &self,
            certificate_der: &[u8],
            now: DateTime<Utc>,
            reach: Reach,
        ) -> Withdrawal {
            // Whether this computer trusts the certificate decides whether the
            // address inside it is one worth connecting to. Asked only when the
            // answer could change anything, so the path that reads a message
            // does not build a second chain to learn something it will not use.
            let trusted = reach == Reach::AskTheAuthority
                && matches!(
                    self.issuer_trust(certificate_der, now),
                    IssuerTrust::Trusted
                );
            let asking = how_far_to_really_go(reach, trusted);
            match build_chain(certificate_der, now, asking) {
                Err(reason) => Withdrawal::CouldNotFindOut { reason },
                Ok(chain) => what_windows_found(&chain),
            }
        }

        fn which_recipient_is_us(&self, recipients: &[Recipient]) -> Result<Option<usize>> {
            for certificate in self.certificates_we_hold_keys_for() {
                if let Some(index) = recipient_matching(recipients, &certificate) {
                    return Ok(Some(index));
                }
            }
            Ok(None)
        }

        fn unwrap_content_key(&self, _recipient: &Recipient) -> Result<Vec<u8>> {
            // Deliberately refused rather than half-built. Undoing the wrapping
            // means driving Windows' key API with a key that never leaves the
            // store, and there is no way to test that here without a real
            // S/MIME certificate installed on the machine running the tests.
            // Code that looks like it decrypts and has never once decrypted
            // anything is the worst thing this file could contain.
            Err(Error::Security(
                "Wixen Mail cannot open encrypted mail yet. Reading the message needs a private \
                 key out of the Windows certificate store, and that part is not built."
                    .to_string(),
            ))
        }
    }

    /// Whether a chain may be asked about withdrawal, and how far that may go.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Asking {
        /// Do not ask about withdrawal at all. This is the trust question,
        /// which is answered from a list already on the machine.
        Nothing,
        /// Ask about withdrawal, using only lists this computer already holds.
        OnlyWhatIsHere,
        /// Ask about withdrawal, and fetch a list from the authority if there
        /// is not one here. This waits on a network.
        TheAuthorityToo,
    }

    impl Asking {
        /// The flags Windows wants for this, and how long it may spend.
        ///
        /// Every case turns off fetching a *certificate* from an address in the
        /// certificate itself, including the case that is allowed to fetch a
        /// withdrawal list. Those are different fetches: a withdrawal list is
        /// named by an authority whose chain this computer already trusts, and
        /// a missing issuer certificate is named by whoever made the
        /// certificate, which may be the sender.
        fn how(self) -> (u32, u32) {
            match self {
                Asking::Nothing => (CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL, 0),
                Asking::OnlyWhatIsHere => (
                    CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL
                        | CERT_CHAIN_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT
                        | CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY,
                    0,
                ),
                Asking::TheAuthorityToo => (
                    CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL
                        | CERT_CHAIN_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT
                        | CERT_CHAIN_REVOCATION_ACCUMULATIVE_TIMEOUT,
                    LONGEST_WAIT_FOR_AN_AUTHORITY,
                ),
            }
        }
    }

    /// How far a withdrawal question really goes.
    ///
    /// Two things decide it, and the second is the one that matters. A caller
    /// may allow the authority to be asked, but the address that would be
    /// connected to is written inside the certificate. For a certificate this
    /// computer already trusts, an authority it already trusts chose that
    /// address. For one it does not trust, whoever made the certificate chose
    /// it, and a stranger who made their own certificate can write an address
    /// that is different for every person they send to. That is a tracking
    /// pixel with a longer name, and `application::pictures` refuses it.
    ///
    /// So permission alone is not enough. Both have to be true.
    fn how_far_to_really_go(reach: Reach, trusted: bool) -> Asking {
        match (reach, trusted) {
            (Reach::WhatIsAlreadyHere, _) => Asking::OnlyWhatIsHere,
            (Reach::AskTheAuthority, true) => Asking::TheAuthorityToo,
            (Reach::AskTheAuthority, false) => Asking::OnlyWhatIsHere,
        }
    }

    /// What Windows made of a certificate, read out of the chain it built.
    ///
    /// A copy of the few numbers rather than the chain itself, so the chain is
    /// freed before anything here is looked at and no pointer outlives it.
    struct WhatWindowsMadeOfIt {
        /// The whole chain's error bits.
        error_status: u32,
        /// Whether anything asked about withdrawal for the certificate the
        /// message carried, and what came back.
        withdrawal_result: Option<u32>,
    }

    /// Build a chain for a certificate and read the few numbers that decide
    /// anything.
    fn build_chain(
        certificate_der: &[u8],
        now: DateTime<Utc>,
        asking: Asking,
    ) -> std::result::Result<WhatWindowsMadeOfIt, String> {
        if certificate_der.is_empty() {
            return Err("there is no certificate to ask about".to_string());
        }
        let when = as_file_time(now)?;
        let (flags, timeout) = asking.how();
        // SAFETY: the certificate bytes outlive the call, the context is freed
        // on every path out, the chain parameters are filled in fully with
        // their own declared size, and every pointer inside the chain is read
        // before the chain is freed.
        unsafe {
            let context = CertCreateCertificateContext(
                X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                certificate_der.as_ptr(),
                certificate_der.len() as u32,
            );
            if context.is_null() {
                return Err(format!(
                    "Windows would not read the certificate ({:#010x})",
                    last_error()
                ));
            }
            let parameters = ChainParameters {
                size: std::mem::size_of::<ChainParameters>() as u32,
                requested_usage: UsageMatch {
                    kind: USAGE_MATCH_TYPE_AND,
                    usage: EnhancedKeyUsage {
                        count: 0,
                        identifiers: std::ptr::null_mut(),
                    },
                },
                requested_issuance_policy: UsageMatch {
                    kind: USAGE_MATCH_TYPE_AND,
                    usage: EnhancedKeyUsage {
                        count: 0,
                        identifiers: std::ptr::null_mut(),
                    },
                },
                url_retrieval_timeout: timeout,
                check_revocation_freshness_time: 0,
                revocation_freshness_time: 0,
                cache_resync: std::ptr::null_mut(),
                strong_sign_parameters: std::ptr::null_mut(),
                strong_sign_flags: 0,
            };
            let mut chain: *const ChainContext = std::ptr::null();
            let built = CertGetCertificateChain(
                std::ptr::null_mut(),
                context,
                &when,
                std::ptr::null_mut(),
                &parameters,
                flags,
                std::ptr::null_mut(),
                &mut chain,
            );
            CertFreeCertificateContext(context);
            if built == 0 || chain.is_null() {
                return Err("Windows could not build a chain for the certificate".to_string());
            }
            let read = WhatWindowsMadeOfIt {
                error_status: (*chain).trust.error_status,
                withdrawal_result: match asking {
                    Asking::Nothing => None,
                    _ => withdrawal_result_of_the_first(chain),
                },
            };
            CertFreeCertificateChain(chain);
            Ok(read)
        }
    }

    /// What asking about withdrawal came to for the certificate the message
    /// carried, which is the first one in the first chain.
    ///
    /// `None` when nothing asked. That is the case that must not be read as
    /// good news, and reading it off the chain's error bits could not tell it
    /// apart from a certificate that really is in good standing.
    ///
    /// # Safety
    ///
    /// `chain` has to be a chain Windows handed out and has not been freed.
    unsafe fn withdrawal_result_of_the_first(chain: *const ChainContext) -> Option<u32> {
        unsafe {
            if (*chain).chain_count == 0 || (*chain).chains.is_null() {
                return None;
            }
            let first = *(*chain).chains;
            if first.is_null() || (*first).element_count == 0 || (*first).elements.is_null() {
                return None;
            }
            let element = *(*first).elements;
            if element.is_null() || (*element).revocation.is_null() {
                return None;
            }
            Some((*(*element).revocation).result)
        }
    }

    /// Windows' numbers turned into the one answer somebody hears.
    ///
    /// Written apart from the call that produces them so the deciding can be
    /// tested on its own. Windows only ever hands back a number here, and
    /// whether this file reads that number correctly is a separate question
    /// from whether Windows produced it.
    fn what_windows_found(chain: &WhatWindowsMadeOfIt) -> Withdrawal {
        // A withdrawn certificate anywhere in the chain settles it, whichever
        // of the two places Windows says so.
        if chain.error_status & CERT_TRUST_IS_REVOKED != 0 {
            return Withdrawal::Withdrawn;
        }
        match chain.withdrawal_result {
            None => Withdrawal::CouldNotFindOut {
                reason: "this computer holds no withdrawal list for whoever issued the certificate"
                    .to_string(),
            },
            Some(REVOCATION_IS_FINE) => Withdrawal::NotWithdrawn,
            Some(CRYPT_E_REVOKED) => Withdrawal::Withdrawn,
            Some(CRYPT_E_REVOCATION_OFFLINE) => Withdrawal::CouldNotFindOut {
                reason: "the authority that publishes the withdrawal list could not be reached"
                    .to_string(),
            },
            Some(CRYPT_E_NO_REVOCATION_CHECK) | Some(CRYPT_E_NO_REVOCATION_DLL) => {
                Withdrawal::CouldNotFindOut {
                    reason: "the certificate says nothing about where its withdrawal list is"
                        .to_string(),
                }
            }
            Some(other) => Withdrawal::CouldNotFindOut {
                reason: format!(
                    "Windows answered with something this does not know ({other:#010x})"
                ),
            },
        }
    }

    /// The last error Windows recorded on this thread.
    fn last_error() -> u32 {
        // SAFETY: reads a per-thread value and takes nothing.
        unsafe { GetLastError() }
    }

    /// Windows' error bits as sentences, dropping any this does not name.
    fn describe(status: u32) -> Vec<&'static str> {
        TRUST_BITS
            .iter()
            .filter(|(bit, _)| status & bit != 0)
            .map(|(_, said)| *said)
            .collect()
    }

    /// A moment as Windows counts them: hundreds of nanoseconds since 1601.
    fn as_file_time(moment: DateTime<Utc>) -> std::result::Result<FileTime, String> {
        /// Seconds between the start of 1601 and the start of 1970.
        const TO_UNIX_EPOCH: i64 = 11_644_473_600;
        let ticks = moment
            .timestamp()
            .checked_add(TO_UNIX_EPOCH)
            .and_then(|seconds| seconds.checked_mul(10_000_000))
            .filter(|ticks| *ticks >= 0)
            .ok_or_else(|| "that moment is outside the range Windows counts".to_string())?;
        Ok(FileTime {
            low: ticks as u32,
            high: (ticks >> 32) as u32,
        })
    }

    /// Every certificate in one of this machine's own stores.
    ///
    /// `only_with_private_keys` is what separates "certificates I could check a
    /// signature against" from "certificates I could decrypt with", and the
    /// second is a much shorter list.
    pub(super) fn certificates_in(store_name: &str, only_with_private_keys: bool) -> Vec<Vec<u8>> {
        let mut wide: Vec<u16> = store_name.encode_utf16().collect();
        wide.push(0);
        // SAFETY: the store is opened here and closed on every path out, and
        // the name outlives the call that reads it.
        unsafe {
            let store = CertOpenSystemStoreW(0, wide.as_ptr());
            if store.is_null() {
                return Vec::new();
            }
            let found = certificates_held_by(store, only_with_private_keys);
            CertCloseStore(store, 0);
            found
        }
    }

    /// Every certificate an already open store holds.
    ///
    /// Split from opening it so a store this process made and a store Windows
    /// keeps for the person are read by exactly the same code. That is what
    /// makes the private key question testable: the only difference between the
    /// two is where the handle came from.
    ///
    /// # Safety
    ///
    /// `store` has to be an open certificate store handle.
    unsafe fn certificates_held_by(
        store: *mut c_void,
        only_with_private_keys: bool,
    ) -> Vec<Vec<u8>> {
        let mut found = Vec::new();
        // SAFETY: each context comes from Windows and is handed straight back
        // to the enumerator, which frees the previous one, and the encoded
        // bytes are copied before the context goes.
        unsafe {
            let mut context: *const CertContext = std::ptr::null();
            loop {
                context = CertEnumCertificatesInStore(store, context);
                if context.is_null() {
                    break;
                }
                if only_with_private_keys && !has_private_key(context) {
                    continue;
                }
                let length = (*context).encoded_length as usize;
                if length > 0 && !(*context).encoded.is_null() {
                    found.push(std::slice::from_raw_parts((*context).encoded, length).to_vec());
                }
            }
        }
        found
    }

    /// Whether this machine holds the private key for a certificate it has.
    ///
    /// Asked by looking for the note Windows attaches saying where the key is,
    /// rather than by opening the key. Opening it can put a prompt on the
    /// screen, and this question gets asked while a message is being displayed.
    ///
    /// Two notes and not one. Windows writes the first for a key kept on disk
    /// and the second for a key held in this process only. Looking for the
    /// first alone answers no to a key that is really there, which is both
    /// wrong and the reason this could never be tested.
    ///
    /// # Safety
    ///
    /// `context` has to be a certificate context Windows handed out and has not
    /// been freed.
    unsafe fn has_private_key(context: *const CertContext) -> bool {
        [CERT_KEY_PROV_INFO_PROP_ID, CERT_NCRYPT_KEY_HANDLE_PROP_ID]
            .iter()
            .any(|property| {
                let mut size: u32 = 0;
                // SAFETY: asking for the size only, so nothing is written to
                // the null pointer.
                unsafe {
                    CertGetCertificateContextProperty(
                        context,
                        *property,
                        std::ptr::null_mut(),
                        &mut size,
                    ) != 0
                }
            })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_nothing_asked_about_withdrawal_is_not_the_same_as_nothing_wrong() {
            // The whole point of the state. A chain with no withdrawal errors
            // may mean nothing was withdrawn or may mean nobody looked, and
            // reading the error bits alone cannot tell those apart.
            let nobody_looked = WhatWindowsMadeOfIt {
                error_status: 0,
                withdrawal_result: None,
            };

            match what_windows_found(&nobody_looked) {
                Withdrawal::CouldNotFindOut { reason } => {
                    assert!(reason.contains("no withdrawal list"), "{reason}");
                }
                other => panic!("a certificate nobody asked about came back as {other:?}"),
            }
        }

        #[test]
        fn test_windows_saying_withdrawn_is_read_as_withdrawn_either_way_it_says_it() {
            // Windows says it in two places, and a reader that knew only one of
            // them would report a withdrawn certificate as in good standing.
            let by_the_element = WhatWindowsMadeOfIt {
                error_status: 0,
                withdrawal_result: Some(CRYPT_E_REVOKED),
            };
            let by_the_chain = WhatWindowsMadeOfIt {
                error_status: CERT_TRUST_IS_REVOKED,
                withdrawal_result: None,
            };

            assert_eq!(what_windows_found(&by_the_element), Withdrawal::Withdrawn);
            assert_eq!(what_windows_found(&by_the_chain), Withdrawal::Withdrawn);
        }

        #[test]
        fn test_a_certificate_really_checked_and_found_sound_says_so() {
            // Without this the reading above would pass just as well against a
            // function that can only ever say no, and a check that cannot come
            // back clean is not a check.
            let checked_and_fine = WhatWindowsMadeOfIt {
                error_status: 0,
                withdrawal_result: Some(REVOCATION_IS_FINE),
            };

            assert_eq!(
                what_windows_found(&checked_and_fine),
                Withdrawal::NotWithdrawn
            );
        }

        #[test]
        fn test_every_way_of_not_finding_out_says_which_one_it_was() {
            // These read alike from the outside and mean different things to
            // whoever is deciding what to do: an authority that could not be
            // reached may work in a minute, and a certificate that names no
            // withdrawal list never will.
            for (answer, expected) in [
                (CRYPT_E_REVOCATION_OFFLINE, "could not be reached"),
                (CRYPT_E_NO_REVOCATION_CHECK, "says nothing about where"),
                (CRYPT_E_NO_REVOCATION_DLL, "says nothing about where"),
                (0x8009_0000, "does not know"),
            ] {
                let found = what_windows_found(&WhatWindowsMadeOfIt {
                    error_status: 0,
                    withdrawal_result: Some(answer),
                });
                match found {
                    Withdrawal::CouldNotFindOut { reason } => {
                        assert!(reason.contains(expected), "{answer:#010x} said {reason}");
                    }
                    other => panic!("{answer:#010x} came back as {other:?}"),
                }
            }
        }

        #[test]
        fn test_permission_alone_does_not_make_a_strangers_address_worth_connecting_to() {
            // The condition the whole privacy argument rests on. Somebody who
            // turns the setting on is agreeing that authorities they already
            // trust may be contacted, not that anyone who makes a certificate
            // may name a server for this computer to call.
            assert_eq!(
                how_far_to_really_go(Reach::AskTheAuthority, false),
                Asking::OnlyWhatIsHere,
                "an untrusted certificate's own address was treated as an authority's"
            );
            assert_eq!(
                how_far_to_really_go(Reach::AskTheAuthority, true),
                Asking::TheAuthorityToo,
                "the setting does nothing at all, so it is not a setting"
            );
            for trusted in [true, false] {
                assert_eq!(
                    how_far_to_really_go(Reach::WhatIsAlreadyHere, trusted),
                    Asking::OnlyWhatIsHere,
                    "the default asked somebody something"
                );
            }
        }

        #[test]
        fn test_reading_a_message_never_asks_anybody_for_anything() {
            // The flags are the whole of the promise that reading a message
            // does not stall and tells nobody it happened. Both questions asked
            // while a message is on screen have to carry the two flags that
            // stop Windows fetching, and neither may carry a timeout, because a
            // timeout only means anything when something is being waited for.
            for asking in [Asking::Nothing, Asking::OnlyWhatIsHere] {
                let (flags, timeout) = asking.how();
                assert!(
                    flags & CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL != 0,
                    "{asking:?} would let Windows fetch a certificate"
                );
                assert_eq!(timeout, 0, "{asking:?} waits on something");
            }
            let (looking_here, _) = Asking::OnlyWhatIsHere.how();
            assert!(
                looking_here & CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY != 0,
                "the withdrawal question would go to the network"
            );

            // And the one that is allowed to ask still refuses to fetch a
            // certificate, because that address is whoever made the
            // certificate's to choose.
            let (asking_out, wait) = Asking::TheAuthorityToo.how();
            assert!(asking_out & CERT_CHAIN_REVOCATION_CHECK_CACHE_ONLY == 0);
            assert!(asking_out & CERT_CHAIN_CACHE_ONLY_URL_RETRIEVAL != 0);
            assert!(wait > 0, "asking with no limit on the wait");
        }
    }
}

// ── Encrypted mail ───────────────────────────────────────────────────────────

/// What an encrypted message says about itself from the outside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedMessage {
    /// Everyone it was encrypted to, as it names them.
    pub recipients: Vec<Recipient>,
    /// The cipher the content itself is under.
    pub content_algorithm: String,
}

impl EncryptedMessage {
    /// Read the outside of a PKCS #7 `EnvelopedData`.
    ///
    /// Reading who a message is addressed to needs no key, so it is portable
    /// and it is here. Opening the message needs a key that lives in the
    /// machine's store, so that part is behind [`CertificateStore`].
    pub fn read(der_bytes: &[u8]) -> Result<Self> {
        let (outer, _) = der::take(der_bytes)?;
        let outer = expect(outer, der::SEQUENCE, "The encrypted message")?;
        let fields = der::children(outer.value)?;
        let content_type = read_oid(fields.first(), "The message's kind")?;
        if content_type != oid::ENVELOPED_DATA {
            return Err(Error::Security(
                "This is a PKCS #7 document but not an encrypted message".to_string(),
            ));
        }
        let wrapper = fields
            .get(1)
            .filter(|element| element.tag == der::context(0))
            .ok_or_else(|| Error::Security("The encrypted message is empty".to_string()))?;
        let (enveloped, _) = der::take(wrapper.value)?;
        let enveloped = expect(enveloped, der::SEQUENCE, "The envelope")?;
        let parts = der::children(enveloped.value)?;

        let recipient_set = parts
            .iter()
            .find(|element| element.tag == der::SET)
            .ok_or_else(|| {
                Error::Security("The encrypted message names no recipients".to_string())
            })?;
        let recipients = der::children(recipient_set.value)?
            .iter()
            .map(Recipient::read)
            .collect::<Result<Vec<_>>>()?;

        let encrypted_content = parts
            .iter()
            .skip_while(|element| element.tag != der::SET)
            .find(|element| element.tag == der::SEQUENCE)
            .ok_or_else(|| Error::Security("The encrypted message has no content".to_string()))?;
        let inside = der::children(encrypted_content.value)?;
        let content_algorithm = algorithm_identifier(inside.get(1), "The content")?.0;

        Ok(Self {
            recipients,
            content_algorithm,
        })
    }

    /// What to say about an encrypted message before anything is opened.
    ///
    /// Honest about the state of this: nothing here can open one yet, and a
    /// person is better told that than shown an empty message body with no
    /// explanation.
    pub fn spoken(&self, addressed_to_us: Option<bool>) -> String {
        let who = match addressed_to_us {
            Some(true) => {
                "This computer holds a certificate this message was encrypted to.".to_string()
            }
            Some(false) => {
                "This message was not encrypted to any certificate on this computer.".to_string()
            }
            None => format!(
                "It is addressed to {} certificate{}.",
                self.recipients.len(),
                if self.recipients.len() == 1 { "" } else { "s" }
            ),
        };
        format!(
            "This message is encrypted. {who} Wixen Mail cannot open encrypted mail yet, so \
             nothing of it can be read here."
        )
    }
}

impl Recipient {
    fn read(element: &der::Element<'_>) -> Result<Self> {
        let element = expect(*element, der::SEQUENCE, "A recipient")?;
        let fields = der::children(element.value)?;
        let identity = fields
            .get(1)
            .copied()
            .ok_or_else(|| Error::Security("A recipient names nobody".to_string()))?;
        let (issuer, serial) = match identity.tag {
            der::SEQUENCE => {
                let pair = der::children(identity.value)?;
                let issuer = pair
                    .first()
                    .ok_or_else(|| Error::Security("A recipient names no issuer".to_string()))?;
                let serial = pair.get(1).ok_or_else(|| {
                    Error::Security("A recipient has no serial number".to_string())
                })?;
                (issuer.encoded.to_vec(), serial.value.to_vec())
            }
            // A recipient named by key identifier rather than by issuer. The
            // shape is read so the count of recipients is right, and the two
            // fields that identify one are left empty rather than filled with
            // something that would match the wrong certificate.
            _ => (Vec::new(), identity.value.to_vec()),
        };
        let key_wrapping_algorithm = algorithm_identifier(fields.get(2), "The key wrapping")?.0;
        let wrapped_key = fields
            .get(3)
            .filter(|element| element.tag == der::OCTET_STRING)
            .map(|element| element.value.to_vec())
            .unwrap_or_default();
        Ok(Self {
            issuer,
            serial,
            key_wrapping_algorithm,
            wrapped_key,
        })
    }
}

/// Whole signed messages for the tests of other modules.
///
/// The same bytes this file's own tests use, which are real OpenSSL output
/// rather than anything hand-built. A cache that keeps signed mail and a reader
/// that reports on it both have to be tested against a message that really is
/// signed: one written here to look signed would agree with whatever this
/// module happened to do and with nothing else.
#[cfg(test)]
pub(crate) mod for_tests {
    /// A message signed the usual way, with the words beside the signature.
    ///
    /// RSA with SHA-256, from a certificate issued to alice@example.com and
    /// good from 2020 to 2040.
    pub(crate) fn signed_beside() -> Vec<u8> {
        super::tests::message(super::tests::SIGNED_BESIDE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Messages to test against ─────────────────────────────────────────
    //
    // Real output from OpenSSL rather than anything hand-built here. A parser
    // checked only against fixtures written by the same person who wrote the
    // parser agrees with itself and nothing else, and every one of these
    // verifies under `openssl cms -verify` as well.
    //
    // Each is the whole message, base64 encoded, and decoded at the start of
    // the test that wants it. Encoded rather than written out as text on
    // purpose: the bytes are what a signature is over, and a source file that
    // held them as text would have its line endings rewritten by any tool that
    // touched it, which would break every signature here in a way that looks
    // exactly like a real failure.
    //
    // The signer certificates run from 2020 to 2040 and every test names the
    // moment it is asking about, so none of this starts failing on a date.

    /// A message signed the usual way, with the words beside the signature.
    /// RSA with SHA-256, from a certificate for alice@example.com.
    pub(super) const SIGNED_BESIDE: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlIjsgbWljYWxnPSJzaGEtMjU2IjsgYm91
        bmRhcnk9Ii0tLS0yQzhGQkVDMTg2RDlDNzM2RkY1OEVGQUFFNTdBMkJGQSINCg0KVGhpcyBpcyBh
        biBTL01JTUUgc2lnbmVkIG1lc3NhZ2UNCg0KLS0tLS0tMkM4RkJFQzE4NkQ5QzczNkZGNThFRkFB
        RTU3QTJCRkENCkNvbnRlbnQtVHlwZTogdGV4dC9wbGFpbg0KDQpUaGUgbWVldGluZyBtb3ZlZCB0
        byBUaHVyc2RheSBhdCB0ZW4uDQoNCi0tLS0tLTJDOEZCRUMxODZEOUM3MzZGRjU4RUZBQUU1N0Ey
        QkZBDQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlOyBuYW1lPSJz
        bWltZS5wN3MiDQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBiYXNlNjQNCkNvbnRlbnQtRGlz
        cG9zaXRpb246IGF0dGFjaG1lbnQ7IGZpbGVuYW1lPSJzbWltZS5wN3MiDQoNCk1JSUdNQVlKS29a
        SWh2Y05BUWNDb0lJR0lUQ0NCaDBDQVFFeER6QU5CZ2xnaGtnQlpRTUVBZ0VGQURBTEJna3ENCmhr
        aUc5dzBCQndHZ2dnT1FNSUlEakRDQ0FuU2dBd0lCQWdJVUpIT00rb01aNy9DOFR3My92UWd4dmxo
        bGd6d3cNCkRRWUpLb1pJaHZjTkFRRUxCUUF3T2pFZE1Cc0dBMVVFQXd3VVYybDRaVzRnVkdWemRD
        QkJkWFJvYjNKcGRIa3gNCkdUQVhCZ05WQkFvTUVGZHBlR1Z1SUUxaGFXd2dWR1Z6ZEhNd0hoY05N
        akF3TVRBeE1EQXdNREF3V2hjTk5EQXcNCk1UQXhNREF3TURBd1dqQXlNUTR3REFZRFZRUUREQVZo
        YkdsalpURWdNQjRHQ1NxR1NJYjNEUUVKQVJZUllXeHANClkyVkFaWGhoYlhCc1pTNWpiMjB3Z2dF
        aU1BMEdDU3FHU0liM0RRRUJBUVVBQTRJQkR3QXdnZ0VLQW9JQkFRQ3UNClpQcCtmWW9yMWErT3Yr
        YkJwZUNxKzRlVUtkcEdCaU9iREdLazFlcGJxSGZOZGhSWEMwcnY4MTE5UzFITzNETWINCllWRXJE
        cHVsUDBMZTFPOTM5cWlWdUNFeGtsZndkL2RLMVFrRWpIbzB2cWUwZE9YdzZFSnlhTlZHeUp0OEIv
        bXQNCmFmRVYzdG90ZlRDajQwRmpwU1BObmZsWlZ3dFkwdEE4b2lkNWNla2t2akJxSjlDOTZ5ZnZQ
        WkJSV2p5QnJtN2wNCnMyZGhRcTlBQ1JuTCtXZEZlNnR5QU9XSFA3ZytkWXNuZlBneFFyZE1WaXJv
        YnlQZ3pTVUpRR2Y5bGFtS3VTN2MNCmljOUg2enNsYkxTajF6M1BsU0cvVlZhM0lsdURmRmlqQzNI
        Tkg2dHgwUnRnSzFpeEQ1cTk3dzZqU1AvdzZ4cVUNCmlBUWtqVkJnQitLZ1dHVWh2NDFiQWdNQkFB
        R2pnWkV3Z1k0d0NRWURWUjBUQkFJd0FEQU9CZ05WSFE4QkFmOEUNCkJBTUNCYUF3RXdZRFZSMGxC
        QXd3Q2dZSUt3WUJCUVVIQXdRd0hBWURWUjBSQkJVd0U0RVJZV3hwWTJWQVpYaGgNCmJYQnNaUzVq
        YjIwd0hRWURWUjBPQkJZRUZBeHE5ZFFSU2JJcE1ibkx6NnBsa3R5clZVOC9NQjhHQTFVZEl3UVkN
        Ck1CYUFGUGFjR292SkxKQ29RWkJjZHl3S3FiSGhHb1QvTUEwR0NTcUdTSWIzRFFFQkN3VUFBNElC
        QVFCUENaaTANCjJSVS9wbEZEbXlNR2M1UlZWRnJhK08xUU14R0ZtTUgxakRZYjNmeTBienROQTlx
        T3hnWjVtZFYreHhFZDdYcEsNCjcrcFVVVyt3QjFwamt5Nm5TcGZnQnZqYjFQSmhJMlpPaWwrTE5p
        NWdHMTVOaW1CdjR6YkhWSGZlWlp2akxxSjcNCm1WWTNTZnlnT1hqVkY1YWZKL0ZTQTRiaEVEeXBM
        MW5pMTlnckUyY0svbnV1dTRqYzJvSWtXN21Zelhwdng0cTYNCnRnUGhIYysrYVlLV3VvSUxVT0VD
        ZTlPdmhrYTRZZXJ5OWljS0tka0ROQkJLL1JpU291TEovV2YyWHpzakh5YkMNCktybThJeGZhblRP
        enk2enloTTdONm5DcVNFcDVBZDJHVGlZUHhVWkxKZ3U4UGxzb3FRR0c3aThMdTc5bTJZVnQNCnZW
        cmFNc2dod3dnWUcybzJNWUlDWkRDQ0FtQUNBUUV3VWpBNk1SMHdHd1lEVlFRRERCUlhhWGhsYmlC
        VVpYTjANCklFRjFkR2h2Y21sMGVURVpNQmNHQTFVRUNnd1FWMmw0Wlc0Z1RXRnBiQ0JVWlhOMGN3
        SVVKSE9NK29NWjcvQzgNClR3My92UWd4dmxobGd6d3dEUVlKWUlaSUFXVURCQUlCQlFDZ2dlUXdH
        QVlKS29aSWh2Y05BUWtETVFzR0NTcUcNClNJYjNEUUVIQVRBY0Jna3Foa2lHOXcwQkNRVXhEeGNO
        TWpZd09ESTRNVFkxTnpRMldqQXZCZ2txaGtpRzl3MEINCkNRUXhJZ1FnZlpwY0lWRDE1NjlDNVBF
        MjQ3UXpXTFI3RHNGRndudFZXamV6Vll4K1NPQXdlUVlKS29aSWh2Y04NCkFRa1BNV3d3YWpBTEJn
        bGdoa2dCWlFNRUFTb3dDd1lKWUlaSUFXVURCQUVXTUFzR0NXQ0dTQUZsQXdRQkFqQUsNCkJnZ3Fo
        a2lHOXcwREJ6QU9CZ2dxaGtpRzl3MERBZ0lDQUlBd0RRWUlLb1pJaHZjTkF3SUNBVUF3QndZRkt3
        NEQNCkFnY3dEUVlJS29aSWh2Y05Bd0lDQVNnd0RRWUpLb1pJaHZjTkFRRUJCUUFFZ2dFQVl0K1dw
        K1ZzRS9DRHdiTmcNCm1Vdkt4eXVTcDYxWUc5WEp0TnBNWkRyQlVBSnRmUUM1QWRrQTd6ZHBmbmlL
        eVV6dCtOUmxMRXZHM1UwQVJEUkcNCjJGVWh5TFUzTVpaaWdTOXptUjFYb0ZLVWkvNlo0OHdDaC8v
        L1JrVFd6V3cxWnZRdTI5THkvdHp0QXBabVdPMVkNCkR2TGJ4bFRxU1RzVEtPcEpOdXNOYXU4NE9x
        Y3ZCMlhIaGpmT1BqTTBualFObmlVL21XQVJMVGtkdmoyRG9hNlcNCmJ5SGt1QjIyVWZzK0labWNC
        OS9iOHRXNmpUQ2lvY1ZKVzFnV0kvNU81T3pjSS9mOThkMUl4QWpITEpPS0cxMkUNCk04U0tqTEJ2
        NmhrWUJBSDRwQmcrZDhudDZFZFVVdW5KY2dzNzh1QzNWY2F2dDZyWWIwUEVKbFVrWDdmMWx0eC8N
        CkE1bitzQT09DQoNCi0tLS0tLTJDOEZCRUMxODZEOUM3MzZGRjU4RUZBQUU1N0EyQkZBLS0NCg0K";

    /// The same words, signed the other way, with the message wrapped inside
    /// the signature so nothing can be read until it is taken apart.
    pub(super) const SIGNED_AROUND: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtRGlzcG9zaXRpb246IGF0dGFjaG1lbnQ7IGZpbGVu
        YW1lPSJzbWltZS5wN20iDQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctbWltZTsg
        c21pbWUtdHlwZT1zaWduZWQtZGF0YTsgbmFtZT0ic21pbWUucDdtIg0KQ29udGVudC1UcmFuc2Zl
        ci1FbmNvZGluZzogYmFzZTY0DQoNCk1JSUdkd1lKS29aSWh2Y05BUWNDb0lJR2FEQ0NCbVFDQVFF
        eER6QU5CZ2xnaGtnQlpRTUVBZ0VGQURCU0Jna3ENCmhraUc5dzBCQndHZ1JRUkRRMjl1ZEdWdWRD
        MVVlWEJsT2lCMFpYaDBMM0JzWVdsdURRb05DbFJvWlNCdFpXVjANCmFXNW5JRzF2ZG1Wa0lIUnZJ
        RlJvZFhKelpHRjVJR0YwSUhSbGJpNE5DcUNDQTVBd2dnT01NSUlDZEtBREFnRUMNCkFoUWtjNHo2
        Z3hudjhMeFBEZis5Q0RHK1dHV0RQREFOQmdrcWhraUc5dzBCQVFzRkFEQTZNUjB3R3dZRFZRUUQN
        CkRCUlhhWGhsYmlCVVpYTjBJRUYxZEdodmNtbDBlVEVaTUJjR0ExVUVDZ3dRVjJsNFpXNGdUV0Zw
        YkNCVVpYTjANCmN6QWVGdzB5TURBeE1ERXdNREF3TURCYUZ3MDBNREF4TURFd01EQXdNREJhTURJ
        eERqQU1CZ05WQkFNTUJXRnMNCmFXTmxNU0F3SGdZSktvWklodmNOQVFrQkZoRmhiR2xqWlVCbGVH
        RnRjR3hsTG1OdmJUQ0NBU0l3RFFZSktvWkkNCmh2Y05BUUVCQlFBRGdnRVBBRENDQVFvQ2dnRUJB
        SzVrK241OWlpdlZyNDYvNXNHbDRLcjdoNVFwMmtZR0k1c00NCllxVFY2bHVvZDgxMkZGY0xTdS96
        WFgxTFVjN2NNeHRoVVNzT202VS9RdDdVNzNmMnFKVzRJVEdTVi9CMzkwclYNCkNRU01lalMrcDdS
        MDVmRG9RbkpvMVViSW0zd0grYTFwOFJYZTJpMTlNS1BqUVdPbEk4MmQrVmxYQzFqUzBEeWkNCkoz
        bHg2U1MrTUdvbjBMM3JKKzg5a0ZGYVBJR3VidVd6WjJGQ3IwQUpHY3Y1WjBWN3EzSUE1WWMvdUQ1
        MWl5ZDgNCitERkN0MHhXS3VodkkrRE5KUWxBWi8yVnFZcTVMdHlKejBmck95VnN0S1BYUGMrVkli
        OVZWcmNpVzROOFdLTUwNCmNjMGZxM0hSRzJBcldMRVBtcjN2RHFOSS8vRHJHcFNJQkNTTlVHQUg0
        cUJZWlNHL2pWc0NBd0VBQWFPQmtUQ0INCmpqQUpCZ05WSFJNRUFqQUFNQTRHQTFVZER3RUIvd1FF
        QXdJRm9EQVRCZ05WSFNVRUREQUtCZ2dyQmdFRkJRY0QNCkJEQWNCZ05WSFJFRUZUQVRnUkZoYkds
        alpVQmxlR0Z0Y0d4bExtTnZiVEFkQmdOVkhRNEVGZ1FVREdyMTFCRkoNCnNpa3h1Y3ZQcW1XUzNL
        dFZUejh3SHdZRFZSMGpCQmd3Rm9BVTlwd2FpOGtza0toQmtGeDNMQXFwc2VFYWhQOHcNCkRRWUpL
        b1pJaHZjTkFRRUxCUUFEZ2dFQkFFOEptTFRaRlQrbVVVT2JJd1p6bEZWVVd0cjQ3VkF6RVlXWXdm
        V00NCk5odmQvTFJ2TzAwRDJvN0dCbm1aMVg3SEVSM3Rla3J2NmxSUmI3QUhXbU9UTHFkS2wrQUcr
        TnZVOG1FalprNksNClg0czJMbUFiWGsyS1lHL2pOc2RVZDk1bG0rTXVvbnVaVmpkSi9LQTVlTlVY
        bHA4bjhWSURodUVRUEtrdldlTFgNCjJDc1Rad3IrZTY2N2lOemFnaVJidVpqTmVtL0hpcnEyQStF
        ZHo3NXBncGE2Z2d0UTRRSjcwNitHUnJoaDZ2TDINCkp3b3AyUU0wRUVyOUdKS2k0c245Wi9aZk95
        TWZKc0lxdWJ3akY5cWRNN1BMclBLRXpzM3FjS3BJU25rQjNZWk8NCkpnL0ZSa3NtQzd3K1d5aXBB
        WWJ1THd1N3YyYlpoVzI5V3RveXlDSERDQmdiYWpZeGdnSmtNSUlDWUFJQkFUQlMNCk1Eb3hIVEFi
        QmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFHOXlhWFI1TVJrd0Z3WURWUVFLREJCWGFYaGwN
        CmJpQk5ZV2xzSUZSbGMzUnpBaFFrYzR6Nmd4bnY4THhQRGYrOUNERytXR1dEUERBTkJnbGdoa2dC
        WlFNRUFnRUYNCkFLQ0I1REFZQmdrcWhraUc5dzBCQ1FNeEN3WUpLb1pJaHZjTkFRY0JNQndHQ1Nx
        R1NJYjNEUUVKQlRFUEZ3MHkNCk5qQTRNamd4TmpVM05EWmFNQzhHQ1NxR1NJYjNEUUVKQkRFaUJD
        QjltbHdoVVBYbnIwTGs4VGJqdEROWXRIc08NCndVWENlMVZhTjdOVmpINUk0REI1QmdrcWhraUc5
        dzBCQ1E4eGJEQnFNQXNHQ1dDR1NBRmxBd1FCS2pBTEJnbGcNCmhrZ0JaUU1FQVJZd0N3WUpZSVpJ
        QVdVREJBRUNNQW9HQ0NxR1NJYjNEUU1ITUE0R0NDcUdTSWIzRFFNQ0FnSUENCmdEQU5CZ2dxaGtp
        Rzl3MERBZ0lCUURBSEJnVXJEZ01DQnpBTkJnZ3Foa2lHOXcwREFnSUJLREFOQmdrcWhraUcNCjl3
        MEJBUUVGQUFTQ0FRQmkzNWFuNVd3VDhJUEJzMkNaUzhySEs1S25yVmdiMWNtMDJreGtPc0ZRQW0x
        OUFMa0INCjJRRHZOMmwrZUlySlRPMzQxR1VzUzhiZFRRQkVORWJZVlNISXRUY3hsbUtCTDNPWkhW
        ZWdVcFNML3BuanpBS0gNCi8vOUdSTmJOYkRWbTlDN2IwdkwrM08wQ2xtWlk3VmdPOHR2R1ZPcEpP
        eE1vNmtrMjZ3MXE3emc2cHk4SFpjZUcNCk44NCtNelNlTkEyZUpUK1pZQkV0T1IyK1BZT2hycFp2
        SWVTNEhiWlIrejRobVp3SDM5dnkxYnFOTUtLaHhVbGINCldCWWovazdrN053ajkvM3gzVWpFQ01j
        c2s0b2JYWVF6eElxTXNHL3FHUmdFQWZpa0dENTN5ZTNvUjFSUzZjbHkNCkN6dnk0TGRWeHErM3F0
        aHZROFFtVlNSZnQvV1czSDhEbWY2dw0KDQo=";

    /// The same words signed with an elliptic curve key rather than RSA, so the
    /// second arm of the algorithm choice has a message behind it too.
    const SIGNED_WITH_A_CURVE: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlIjsgbWljYWxnPSJzaGEtMjU2IjsgYm91
        bmRhcnk9Ii0tLS02MTA1RDJDREI3M0QxNkE5OTIxRDVBQTU5QTNGM0ZFRiINCg0KVGhpcyBpcyBh
        biBTL01JTUUgc2lnbmVkIG1lc3NhZ2UNCg0KLS0tLS0tNjEwNUQyQ0RCNzNEMTZBOTkyMUQ1QUE1
        OUEzRjNGRUYNCkNvbnRlbnQtVHlwZTogdGV4dC9wbGFpbg0KDQpUaGUgbWVldGluZyBtb3ZlZCB0
        byBUaHVyc2RheSBhdCB0ZW4uDQoNCi0tLS0tLTYxMDVEMkNEQjczRDE2QTk5MjFENUFBNTlBM0Yz
        RkVGDQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlOyBuYW1lPSJz
        bWltZS5wN3MiDQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBiYXNlNjQNCkNvbnRlbnQtRGlz
        cG9zaXRpb246IGF0dGFjaG1lbnQ7IGZpbGVuYW1lPSJzbWltZS5wN3MiDQoNCk1JSUVwZ1lKS29a
        SWh2Y05BUWNDb0lJRWx6Q0NCSk1DQVFFeER6QU5CZ2xnaGtnQlpRTUVBZ0VGQURBTEJna3ENCmhr
        aUc5dzBCQndHZ2dnTEZNSUlDd1RDQ0FhbWdBd0lCQWdJVUpIT00rb01aNy9DOFR3My92UWd4dmxo
        bGd6OHcNCkRRWUpLb1pJaHZjTkFRRUxCUUF3T2pFZE1Cc0dBMVVFQXd3VVYybDRaVzRnVkdWemRD
        QkJkWFJvYjNKcGRIa3gNCkdUQVhCZ05WQkFvTUVGZHBlR1Z1SUUxaGFXd2dWR1Z6ZEhNd0hoY05N
        akF3TVRBeE1EQXdNREF3V2hjTk5EQXcNCk1UQXhNREF3TURBd1dqQXlNUTR3REFZRFZRUUREQVZq
        WVhKdmJERWdNQjRHQ1NxR1NJYjNEUUVKQVJZUlkyRnkNCmIyeEFaWGhoYlhCc1pTNWpiMjB3V1RB
        VEJnY3Foa2pPUFFJQkJnZ3Foa2pPUFFNQkJ3TkNBQVNkeGlpUHRDN3oNCjhJNXQvUkZ2TUVodUsx
        QkdIKzlCQ2dxekJZOVFWNXN4Z1Z5dDBkMkVXYkpVWHM2MnF1UHhIdHN5bEJhSUN4ekUNCnUzWGp5
        K2wvN05rbW80R1JNSUdPTUFrR0ExVWRFd1FDTUFBd0RnWURWUjBQQVFIL0JBUURBZ2VBTUJNR0Ex
        VWQNCkpRUU1NQW9HQ0NzR0FRVUZCd01FTUJ3R0ExVWRFUVFWTUJPQkVXTmhjbTlzUUdWNFlXMXdi
        R1V1WTI5dE1CMEcNCkExVWREZ1FXQkJUVnBZWHBjeVZLWHhYUDN0WU5rWWZhckM5WGNUQWZCZ05W
        SFNNRUdEQVdnQlQybkJxTHlTeVENCnFFR1FYSGNzQ3FteDRScUUvekFOQmdrcWhraUc5dzBCQVFz
        RkFBT0NBUUVBWmFNVThkU0I3cHF4RzNPUUhMR3kNCjNRbnhwU2M1aFdwZ09SU1NpLysxVlRrN0NK
        WDRQS0hGbXZRcTJRdmxMVkkwejhOQnZiQkZnZFM5aC82S1NZeHYNCjk2WDVlQUdhb0J4dkJuT3Fu
        R3Q5Z1hVcXhWYVFjTkptOXhTS0pGNG11QlBlNm1rdW05YmF2VHBIeUF3VGUxTXYNClkwZU5JVGlz
        eTFEQzVlOXRLaEdDNUdTUklYMWxQeE9UR1cyblZNL1FMM3BaWWVyd2dPVVFLeGVqaE9SaEY1OVgN
        Cm1SUHRONHBsRFRMN1Z2YWZyWlhRQmtCRFFSU0hQbDR0TGhmeTBETG1uczgweHdxRXdRNHZGZzEx
        Zlo5YmZyWk0NCkhyVHVacEsrVTc5NGs4czNoblJoOVpIQ1liSjRyeVVmOTVjak1aVTdvRC8xWjFG
        a3dNUXFjN1FlQWJqQW5iQmcNCkR6R0NBYVV3Z2dHaEFnRUJNRkl3T2pFZE1Cc0dBMVVFQXd3VVYy
        bDRaVzRnVkdWemRDQkJkWFJvYjNKcGRIa3gNCkdUQVhCZ05WQkFvTUVGZHBlR1Z1SUUxaGFXd2dW
        R1Z6ZEhNQ0ZDUnpqUHFER2Uvd3ZFOE4vNzBJTWI1WVpZTS8NCk1BMEdDV0NHU0FGbEF3UUNBUVVB
        b0lIa01CZ0dDU3FHU0liM0RRRUpBekVMQmdrcWhraUc5dzBCQndFd0hBWUoNCktvWklodmNOQVFr
        Rk1ROFhEVEkyTURneU9ERTJOVGMwTmxvd0x3WUpLb1pJaHZjTkFRa0VNU0lFSUgyYVhDRlENCjll
        ZXZRdVR4TnVPME0xaTBldzdCUmNKN1ZWbzNzMVdNZmtqZ01Ia0dDU3FHU0liM0RRRUpEekZzTUdv
        d0N3WUoNCllJWklBV1VEQkFFcU1Bc0dDV0NHU0FGbEF3UUJGakFMQmdsZ2hrZ0JaUU1FQVFJd0Nn
        WUlLb1pJaHZjTkF3Y3cNCkRnWUlLb1pJaHZjTkF3SUNBZ0NBTUEwR0NDcUdTSWIzRFFNQ0FnRkFN
        QWNHQlNzT0F3SUhNQTBHQ0NxR1NJYjMNCkRRTUNBZ0VvTUFvR0NDcUdTTTQ5QkFNQ0JFWXdSQUln
        WXNFTkwzVnZCWm8zeHQweDFXUUtPRkpvY2F5VjkrSDENCmNlS21KbjZ0NlVJQ0lHd0dYNGk4aWZ3
        K2lxUW9vQTJ0MEpRS09zQkdGZVBwS055NFVJY3UyU0VxDQoNCi0tLS0tLTYxMDVEMkNEQjczRDE2
        QTk5MjFENUFBNTlBM0YzRkVGLS0NCg0K";

    /// The same words again, signed with RSA-PSS, which keeps the name of its
    /// fingerprint inside its parameters instead of in its own identifier.
    const SIGNED_WITH_PSS: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3BrY3M3LXNpZ25hdHVyZSI7IG1pY2FsZz0ic2hhLTI1NiI7IGJvdW5k
        YXJ5PSItLS0tRUU4MkJGRkJCOEU4MjcwOEYzMzY4OEQ4NzU5QzI3OTAiDQoNClRoaXMgaXMgYW4g
        Uy9NSU1FIHNpZ25lZCBtZXNzYWdlDQoNCi0tLS0tLUVFODJCRkZCQjhFODI3MDhGMzM2ODhEODc1
        OUMyNzkwDQpDb250ZW50LVR5cGU6IHRleHQvcGxhaW4NCg0KVGhlIG1lZXRpbmcgbW92ZWQgdG8g
        VGh1cnNkYXkgYXQgdGVuLg0KDQotLS0tLS1FRTgyQkZGQkI4RTgyNzA4RjMzNjg4RDg3NTlDMjc5
        MA0KQ29udGVudC1UeXBlOiBhcHBsaWNhdGlvbi9wa2NzNy1zaWduYXR1cmU7IG5hbWU9InNtaW1l
        LnA3cyINCkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6IGJhc2U2NA0KQ29udGVudC1EaXNwb3Np
        dGlvbjogYXR0YWNobWVudDsgZmlsZW5hbWU9InNtaW1lLnA3cyINCg0KTUlJR1lBWUpLb1pJaHZj
        TkFRY0NvSUlHVVRDQ0JrMENBUUV4RFRBTEJnbGdoa2dCWlFNRUFnRXdDd1lKS29aSQ0KaHZjTkFR
        Y0JvSUlEa0RDQ0E0d3dnZ0owb0FNQ0FRSUNGQ1J6alBxREdlL3d2RThOLzcwSU1iNVlaWU04TUEw
        Rw0KQ1NxR1NJYjNEUUVCQ3dVQU1Eb3hIVEFiQmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFH
        OXlhWFI1TVJrdw0KRndZRFZRUUtEQkJYYVhobGJpQk5ZV2xzSUZSbGMzUnpNQjRYRFRJd01ERXdN
        VEF3TURBd01Gb1hEVFF3TURFdw0KTVRBd01EQXdNRm93TWpFT01Bd0dBMVVFQXd3RllXeHBZMlV4
        SURBZUJna3Foa2lHOXcwQkNRRVdFV0ZzYVdObA0KUUdWNFlXMXdiR1V1WTI5dE1JSUJJakFOQmdr
        cWhraUc5dzBCQVFFRkFBT0NBUThBTUlJQkNnS0NBUUVBcm1UNg0KZm4yS0s5V3Zqci9td2FYZ3F2
        dUhsQ25hUmdZam13eGlwTlhxVzZoM3pYWVVWd3RLNy9OZGZVdFJ6dHd6RzJGUg0KS3c2YnBUOUMz
        dFR2ZC9hb2xiZ2hNWkpYOEhmM1N0VUpCSXg2Tkw2bnRIVGw4T2hDY21qVlJzaWJmQWY1cldueA0K
        RmQ3YUxYMHdvK05CWTZVanpaMzVXVmNMV05MUVBLSW5lWEhwSkw0d2FpZlF2ZXNuN3oyUVVWbzhn
        YTV1NWJObg0KWVVLdlFBa1p5L2xuUlh1cmNnRGxoeis0UG5XTEozejRNVUszVEZZcTZHOGo0TTBs
        Q1VCbi9aV3Bpcmt1M0luUA0KUitzN0pXeTBvOWM5ejVVaHYxVld0eUpiZzN4WW93dHh6UityY2RF
        YllDdFlzUSthdmU4T28wai84T3NhbElnRQ0KSkkxUVlBZmlvRmhsSWIrTld3SURBUUFCbzRHUk1J
        R09NQWtHQTFVZEV3UUNNQUF3RGdZRFZSMFBBUUgvQkFRRA0KQWdXZ01CTUdBMVVkSlFRTU1Bb0dD
        Q3NHQVFVRkJ3TUVNQndHQTFVZEVRUVZNQk9CRVdGc2FXTmxRR1Y0WVcxdw0KYkdVdVkyOXRNQjBH
        QTFVZERnUVdCQlFNYXZYVUVVbXlLVEc1eTgrcVpaTGNxMVZQUHpBZkJnTlZIU01FR0RBVw0KZ0JU
        Mm5CcUx5U3lRcUVHUVhIY3NDcW14NFJxRS96QU5CZ2txaGtpRzl3MEJBUXNGQUFPQ0FRRUFUd21Z
        dE5rVg0KUDZaUlE1c2pCbk9VVlZSYTJ2anRVRE1SaFpqQjlZdzJHOTM4dEc4N1RRUGFqc1lHZVpu
        VmZzY1JIZTE2U3UvcQ0KVkZGdnNBZGFZNU11cDBxWDRBYjQyOVR5WVNObVRvcGZpell1WUJ0ZVRZ
        cGdiK00yeDFSMzNtV2I0eTZpZTVsVw0KTjBuOG9EbDQxUmVXbnlmeFVnT0c0UkE4cVM5WjR0ZllL
        eE5uQ3Y1N3JydUkzTnFDSkZ1NW1NMTZiOGVLdXJZRA0KNFIzUHZtbUNscnFDQzFEaEFudlRyNFpH
        dUdIcTh2WW5DaW5aQXpRUVN2MFlrcUxpeWYxbjlsODdJeDhtd2lxNQ0KdkNNWDJwMHpzOHVzOG9U
        T3plcHdxa2hLZVFIZGhrNG1EOFZHU3lZTHZENWJLS2tCaHU0dkM3dS9adG1GYmIxYQ0KMmpMSUlj
        TUlHQnRxTmpHQ0FwWXdnZ0tTQWdFQk1GSXdPakVkTUJzR0ExVUVBd3dVVjJsNFpXNGdWR1Z6ZENC
        Qg0KZFhSb2IzSnBkSGt4R1RBWEJnTlZCQW9NRUZkcGVHVnVJRTFoYVd3Z1ZHVnpkSE1DRkNSempQ
        cURHZS93dkU4Tg0KLzcwSU1iNVlaWU04TUFzR0NXQ0dTQUZsQXdRQ0FhQ0I1REFZQmdrcWhraUc5
        dzBCQ1FNeEN3WUpLb1pJaHZjTg0KQVFjQk1Cd0dDU3FHU0liM0RRRUpCVEVQRncweU5qQTRNamd4
        TmpVM05EWmFNQzhHQ1NxR1NJYjNEUUVKQkRFaQ0KQkNCOW1sd2hVUFhucjBMazhUYmp0RE5ZdEhz
        T3dVWENlMVZhTjdOVmpINUk0REI1QmdrcWhraUc5dzBCQ1E4eA0KYkRCcU1Bc0dDV0NHU0FGbEF3
        UUJLakFMQmdsZ2hrZ0JaUU1FQVJZd0N3WUpZSVpJQVdVREJBRUNNQW9HQ0NxRw0KU0liM0RRTUhN
        QTRHQ0NxR1NJYjNEUU1DQWdJQWdEQU5CZ2dxaGtpRzl3MERBZ0lCUURBSEJnVXJEZ01DQnpBTg0K
        QmdncWhraUc5dzBEQWdJQktEQkJCZ2txaGtpRzl3MEJBUW93TktBUE1BMEdDV0NHU0FGbEF3UUNB
        UVVBb1J3dw0KR2dZSktvWklodmNOQVFFSU1BMEdDV0NHU0FGbEF3UUNBUVVBb2dNQ0FTQUVnZ0VB
        ZVhyYks1QUFzMzdSeUVhRA0KbkYyVWM1V2VnOTVqdjdpY3AvZXRCSjdWWUVSYmU5c0xkRWVWajdV
        cmNudktBMmVhcXNHSHR0aHFoTnlBL2dWbw0KMmdPZWZCeG5LVFZHUlVPOEFtUzNMNUFzcXRtd0lz
        M21aWTVBempqSkNkYnpYNDVxY1JTak8wV1R0Y1ZCSUp5ag0KcE9TWkxMeG5QdjEyUkc4Z2VsT2oz
        VERrU2Q0L0EvbjVoRjBiUXVWTU9CTDFiU2QxVkVjWmNVb1pVdVhiSFFnQw0KV1FjQTFKaU83ZUw1
        QlRZZTltTGs1UVNVaXlROGNTcGdLSkpicGZ5OHh2eXRoWjNwVEFHS1VmbVlvb1ZZd0ZJWA0KbFMx
        eVE0V0hYY3lOYVJpdHF6M2VxS2xQV05GaGFPOG9QNVJIQW45RDJUNnh2TlNTYVVHU00zV0F4N25n
        azduag0KQ0JiQ1hRPT0NCg0KLS0tLS0tRUU4MkJGRkJCOEU4MjcwOEYzMzY4OEQ4NzU5QzI3OTAt
        LQ0KDQo=";

    /// The same words encrypted to the certificate for alice@example.com.
    const ENCRYPTED_TO_ALICE: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtRGlzcG9zaXRpb246IGF0dGFjaG1lbnQ7IGZpbGVu
        YW1lPSJzbWltZS5wN20iDQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctbWltZTsg
        c21pbWUtdHlwZT1lbnZlbG9wZWQtZGF0YTsgbmFtZT0ic21pbWUucDdtIg0KQ29udGVudC1UcmFu
        c2Zlci1FbmNvZGluZzogYmFzZTY0DQoNCk1JSUNCZ1lKS29aSWh2Y05BUWNEb0lJQjl6Q0NBZk1D
        QVFBeGdnRnVNSUlCYWdJQkFEQlNNRG94SFRBYkJnTlYNCkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhW
        MGFHOXlhWFI1TVJrd0Z3WURWUVFLREJCWGFYaGxiaUJOWVdsc0lGUmwNCmMzUnpBaFFrYzR6Nmd4
        bnY4THhQRGYrOUNERytXR1dEUERBTkJna3Foa2lHOXcwQkFRRUZBQVNDQVFBbzltbEENCnZKNVdQ
        Uldrd3VOWDArb0Q5TXBwU2RWbVM1K3cxUFg3akN5ZjJKMFU5c0RIeEVtVmdzV2FSR0VQNFJqeXkw
        K2ENCjNIcDBwU2dJTUVYREl0K0tUUEZWOVNyVFcxTFlFSGpCRjVSQndKZEF1d0lMa3EvNEhlT3NC
        amdGbWJkR2I0OWcNCktpTGRXYkVzb0JFZnRTNXF5blpFb1BwQmpQMitZVmpQQTRmQnBlN0w4Wk9w
        NW1BZk9QTVF1eC9xREJBY1g5eEgNCmNLU1ZiOVNxVS9ySlZwUE92dFYyNmhPQ0piR3dpWDVmWlJW
        azc1UVQwS2MwZ21ZVWhKUGQvNVBOZVlZQ3U1dFkNCjNPRFFYVnVDRERBMjB2WmIxWEdreVFtdThM
        K1Q1U0NrK3k4dC9abjlVUi9odUc2d2trL3ZvZkRlWkg1amcwd24NCnNhcDNMcFB2UHg5clVkamVN
        SHdHQ1NxR1NJYjNEUUVIQVRBZEJnbGdoa2dCWlFNRUFTb0VFR0RkeksrTUVqZXMNCjRVUUVPdVpY
        ZG95QVVCNndnMjRFUGp0d3JKazNPL1pwNjcwcm9nbzl0Q2M3cTZBdHcwTnI4M3kvRHl5d1RpV2YN
        Ck5RaUd3YWVNaUtxbmw5OVZlbjljZGNGV29HQkFSTC9qbElWU0pJb21hWU5JeTFZekVlMFI2RlJz
        DQoNCg==";

    /// The same words signed with SHA-1, which can be forged. The arithmetic
    /// holds; that is the point. What has to be said is that it proves nothing.
    const SIGNED_WITH_A_WEAK_FINGERPRINT: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlIjsgbWljYWxnPSJzaGExIjsgYm91bmRh
        cnk9Ii0tLS1BMjdFNDAyQzQyNTQ1QkE1MDNBRjlEMUY3MTcwREYyQSINCg0KVGhpcyBpcyBhbiBT
        L01JTUUgc2lnbmVkIG1lc3NhZ2UNCg0KLS0tLS0tQTI3RTQwMkM0MjU0NUJBNTAzQUY5RDFGNzE3
        MERGMkENCkNvbnRlbnQtVHlwZTogdGV4dC9wbGFpbg0KDQpUaGUgbWVldGluZyBtb3ZlZCB0byBU
        aHVyc2RheSBhdCB0ZW4uDQoNCi0tLS0tLUEyN0U0MDJDNDI1NDVCQTUwM0FGOUQxRjcxNzBERjJB
        DQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlOyBuYW1lPSJzbWlt
        ZS5wN3MiDQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBiYXNlNjQNCkNvbnRlbnQtRGlzcG9z
        aXRpb246IGF0dGFjaG1lbnQ7IGZpbGVuYW1lPSJzbWltZS5wN3MiDQoNCk1JSUdIQVlKS29aSWh2
        Y05BUWNDb0lJR0RUQ0NCZ2tDQVFFeEN6QUpCZ1VyRGdNQ0dnVUFNQXNHQ1NxR1NJYjMNCkRRRUhB
        YUNDQTVBd2dnT01NSUlDZEtBREFnRUNBaFFrYzR6Nmd4bnY4THhQRGYrOUNERytXR1dEUERBTkJn
        a3ENCmhraUc5dzBCQVFzRkFEQTZNUjB3R3dZRFZRUUREQlJYYVhobGJpQlVaWE4wSUVGMWRHaHZj
        bWwwZVRFWk1CY0cNCkExVUVDZ3dRVjJsNFpXNGdUV0ZwYkNCVVpYTjBjekFlRncweU1EQXhNREV3
        TURBd01EQmFGdzAwTURBeE1ERXcNCk1EQXdNREJhTURJeERqQU1CZ05WQkFNTUJXRnNhV05sTVNB
        d0hnWUpLb1pJaHZjTkFRa0JGaEZoYkdsalpVQmwNCmVHRnRjR3hsTG1OdmJUQ0NBU0l3RFFZSktv
        WklodmNOQVFFQkJRQURnZ0VQQURDQ0FRb0NnZ0VCQUs1aytuNTkNCmlpdlZyNDYvNXNHbDRLcjdo
        NVFwMmtZR0k1c01ZcVRWNmx1b2Q4MTJGRmNMU3UvelhYMUxVYzdjTXh0aFVTc08NCm02VS9RdDdV
        NzNmMnFKVzRJVEdTVi9CMzkwclZDUVNNZWpTK3A3UjA1ZkRvUW5KbzFVYkltM3dIK2ExcDhSWGUN
        CjJpMTlNS1BqUVdPbEk4MmQrVmxYQzFqUzBEeWlKM2x4NlNTK01Hb24wTDNySis4OWtGRmFQSUd1
        YnVXeloyRkMNCnIwQUpHY3Y1WjBWN3EzSUE1WWMvdUQ1MWl5ZDgrREZDdDB4V0t1aHZJK0ROSlFs
        QVovMlZxWXE1THR5SnowZnINCk95VnN0S1BYUGMrVkliOVZWcmNpVzROOFdLTUxjYzBmcTNIUkcy
        QXJXTEVQbXIzdkRxTkkvL0RyR3BTSUJDU04NClVHQUg0cUJZWlNHL2pWc0NBd0VBQWFPQmtUQ0Jq
        akFKQmdOVkhSTUVBakFBTUE0R0ExVWREd0VCL3dRRUF3SUYNCm9EQVRCZ05WSFNVRUREQUtCZ2dy
        QmdFRkJRY0RCREFjQmdOVkhSRUVGVEFUZ1JGaGJHbGpaVUJsZUdGdGNHeGwNCkxtTnZiVEFkQmdO
        VkhRNEVGZ1FVREdyMTFCRkpzaWt4dWN2UHFtV1MzS3RWVHo4d0h3WURWUjBqQkJnd0ZvQVUNCjlw
        d2FpOGtza0toQmtGeDNMQXFwc2VFYWhQOHdEUVlKS29aSWh2Y05BUUVMQlFBRGdnRUJBRThKbUxU
        WkZUK20NClVVT2JJd1p6bEZWVVd0cjQ3VkF6RVlXWXdmV01OaHZkL0xSdk8wMEQybzdHQm5tWjFY
        N0hFUjN0ZWtydjZsUlINCmI3QUhXbU9UTHFkS2wrQUcrTnZVOG1FalprNktYNHMyTG1BYlhrMktZ
        Ry9qTnNkVWQ5NWxtK011b251WlZqZEoNCi9LQTVlTlVYbHA4bjhWSURodUVRUEtrdldlTFgyQ3NU
        WndyK2U2NjdpTnphZ2lSYnVaak5lbS9IaXJxMkErRWQNCno3NXBncGE2Z2d0UTRRSjcwNitHUnJo
        aDZ2TDJKd29wMlFNMEVFcjlHSktpNHNuOVovWmZPeU1mSnNJcXVid2oNCkY5cWRNN1BMclBLRXpz
        M3FjS3BJU25rQjNZWk9KZy9GUmtzbUM3dytXeWlwQVlidUx3dTd2MmJaaFcyOVd0b3kNCnlDSERD
        QmdiYWpZeGdnSlVNSUlDVUFJQkFUQlNNRG94SFRBYkJnTlZCQU1NRkZkcGVHVnVJRlJsYzNRZ1FY
        VjANCmFHOXlhWFI1TVJrd0Z3WURWUVFLREJCWGFYaGxiaUJOWVdsc0lGUmxjM1J6QWhRa2M0ejZn
        eG52OEx4UERmKzkNCkNERytXR1dEUERBSkJnVXJEZ01DR2dVQW9JSFlNQmdHQ1NxR1NJYjNEUUVK
        QXpFTEJna3Foa2lHOXcwQkJ3RXcNCkhBWUpLb1pJaHZjTkFRa0ZNUThYRFRJMk1EZ3lPREUzTkRF
        ek5sb3dJd1lKS29aSWh2Y05BUWtFTVJZRUZOMGINClpmNFVBUnd1MEJDUnBvMFBhc3pOT1hET01I
        a0dDU3FHU0liM0RRRUpEekZzTUdvd0N3WUpZSVpJQVdVREJBRXENCk1Bc0dDV0NHU0FGbEF3UUJG
        akFMQmdsZ2hrZ0JaUU1FQVFJd0NnWUlLb1pJaHZjTkF3Y3dEZ1lJS29aSWh2Y04NCkF3SUNBZ0NB
        TUEwR0NDcUdTSWIzRFFNQ0FnRkFNQWNHQlNzT0F3SUhNQTBHQ0NxR1NJYjNEUU1DQWdFb01BMEcN
        CkNTcUdTSWIzRFFFQkFRVUFCSUlCQUlEY3RHMHcrNElYRnU4dUhFL1FnNlozQkR1RWx1MVJPNjky
        SXg3SnZtSFUNCkM3eEt1Tzh6dmRSK2VpK0h4V1I2N3o4bWR6T3I5YmR5K3JkNzQzL0tnTGdXc0Vt
        TUprOU41T0FjV2xuWDJPbnMNCk54emx1elRBa2tvVE5vbmY3bFFOTVpsakh2MnR5RzJMSmpJMDQy
        MXZ2Y1d4QUZubFE3bU9HVXRESUNSYzF0alANCnI1K3VTd20zQ1FpTmo3N3NzNkdzWmlENklSb0M0
        eTYrTzRzZHo5TWljU0NuZDR6ME5COTVpbGlWbVRWc3FISFYNCks1V3R3QU4rdEppSis4TmNIN2g1
        VFdrMzFoaEgvUkFiWEw0STYwalprNnM5L25yNHdKbFdVcTZaN2R1YUk5RU8NCkczMnVub0xpYUlV
        Z1kxTTI0eklwK3pEUWIxTkxuRjNSRWZZZzZsV3BDQ289DQoNCi0tLS0tLUEyN0U0MDJDNDI1NDVC
        QTUwM0FGOUQxRjcxNzBERjJBLS0NCg0K";

    /// A signature whose certificate did not travel with it. Some senders leave
    /// it out, and then there is no key here to check the signature against.
    const SIGNED_WITH_NO_CERTIFICATE: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlIjsgbWljYWxnPSJzaGEtMjU2IjsgYm91
        bmRhcnk9Ii0tLS04NUNGNkZFRjk1MUM3NkVDMDY0QjU2RDQ4MDRGRTNEOSINCg0KVGhpcyBpcyBh
        biBTL01JTUUgc2lnbmVkIG1lc3NhZ2UNCg0KLS0tLS0tODVDRjZGRUY5NTFDNzZFQzA2NEI1NkQ0
        ODA0RkUzRDkNCkNvbnRlbnQtVHlwZTogdGV4dC9wbGFpbg0KDQpUaGUgbWVldGluZyBtb3ZlZCB0
        byBUaHVyc2RheSBhdCB0ZW4uDQoNCi0tLS0tLTg1Q0Y2RkVGOTUxQzc2RUMwNjRCNTZENDgwNEZF
        M0Q5DQpDb250ZW50LVR5cGU6IGFwcGxpY2F0aW9uL3gtcGtjczctc2lnbmF0dXJlOyBuYW1lPSJz
        bWltZS5wN3MiDQpDb250ZW50LVRyYW5zZmVyLUVuY29kaW5nOiBiYXNlNjQNCkNvbnRlbnQtRGlz
        cG9zaXRpb246IGF0dGFjaG1lbnQ7IGZpbGVuYW1lPSJzbWltZS5wN3MiDQoNCk1JSUNuQVlKS29a
        SWh2Y05BUWNDb0lJQ2pUQ0NBb2tDQVFFeER6QU5CZ2xnaGtnQlpRTUVBZ0VGQURBTEJna3ENCmhr
        aUc5dzBCQndFeGdnSmtNSUlDWUFJQkFUQlNNRG94SFRBYkJnTlZCQU1NRkZkcGVHVnVJRlJsYzNR
        Z1FYVjANCmFHOXlhWFI1TVJrd0Z3WURWUVFLREJCWGFYaGxiaUJOWVdsc0lGUmxjM1J6QWhRa2M0
        ejZneG52OEx4UERmKzkNCkNERytXR1dEUERBTkJnbGdoa2dCWlFNRUFnRUZBS0NCNURBWUJna3Fo
        a2lHOXcwQkNRTXhDd1lKS29aSWh2Y04NCkFRY0JNQndHQ1NxR1NJYjNEUUVKQlRFUEZ3MHlOakE0
        TWpneE56UXhNelphTUM4R0NTcUdTSWIzRFFFSkJERWkNCkJDQjltbHdoVVBYbnIwTGs4VGJqdERO
        WXRIc093VVhDZTFWYU43TlZqSDVJNERCNUJna3Foa2lHOXcwQkNROHgNCmJEQnFNQXNHQ1dDR1NB
        RmxBd1FCS2pBTEJnbGdoa2dCWlFNRUFSWXdDd1lKWUlaSUFXVURCQUVDTUFvR0NDcUcNClNJYjNE
        UU1ITUE0R0NDcUdTSWIzRFFNQ0FnSUFnREFOQmdncWhraUc5dzBEQWdJQlFEQUhCZ1VyRGdNQ0J6
        QU4NCkJnZ3Foa2lHOXcwREFnSUJLREFOQmdrcWhraUc5dzBCQVFFRkFBU0NBUUJEbS9OVWo0dEVC
        anV2TVRkOTVqV2UNCndxaEZ1bzF3dXpsaUZvdlRwVGJmVEIzVytCTVU0V3FDdEs3cm5ZMk0vdmlW
        dHRxYlVFMkJkOGh6cnBUa2xraGcNCkRncXBVWEpJSk01a0NPSXE5bjFFL0czRTNYZVRUSFdYUmpC
        S1RteDc1bGFPazlhM2NwNG1wNWNsajVCbW5ONmENCmRybG8zTUozU0JBV2FOczZoOEI1NTNTQkZL
        THVoMS85Sk5qdGhoQlU2ZXNVWGN2aHlJK2gwallPa0NRcjRicVENCnYxZFZMaEFnUzJreWk2amhJ
        bW54VUJUcFhydWtRS0FvNS9xUTRhS3N2YzluNlBNS2dMRktWN0lUWFYvVzJRanoNCm1kalhtakxD
        cExUQjE2Skw5VUp0WU1yM2daNmlxRFE4R2tzM3FnQXJzNUxlTjViOXhnak9JbGV6RnN1cUlxZkoN
        Cg0KLS0tLS0tODVDRjZGRUY5NTFDNzZFQzA2NEI1NkQ0ODA0RkUzRDktLQ0KDQo=";

    /// The same words again, from a sender that names itself by the short
    /// identifier its certificate carries rather than by issuer and serial
    /// number. Both ways are in use and the certificate has to be found either
    /// way, or a perfectly good signature has nothing to check it against.
    const SIGNED_NAMING_A_KEY: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3BrY3M3LXNpZ25hdHVyZSI7IG1pY2FsZz0ic2hhLTI1NiI7IGJvdW5k
        YXJ5PSItLS0tRTIyODY0NTJFRUZDRTA3QjI5MjhBMUVDRTYwQTI2QUIiDQoNClRoaXMgaXMgYW4g
        Uy9NSU1FIHNpZ25lZCBtZXNzYWdlDQoNCi0tLS0tLUUyMjg2NDUyRUVGQ0UwN0IyOTI4QTFFQ0U2
        MEEyNkFCDQpDb250ZW50LVR5cGU6IHRleHQvcGxhaW4NCg0KVGhlIG1lZXRpbmcgbW92ZWQgdG8g
        VGh1cnNkYXkgYXQgdGVuLg0KDQotLS0tLS1FMjI4NjQ1MkVFRkNFMDdCMjkyOEExRUNFNjBBMjZB
        Qg0KQ29udGVudC1UeXBlOiBhcHBsaWNhdGlvbi9wa2NzNy1zaWduYXR1cmU7IG5hbWU9InNtaW1l
        LnA3cyINCkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6IGJhc2U2NA0KQ29udGVudC1EaXNwb3Np
        dGlvbjogYXR0YWNobWVudDsgZmlsZW5hbWU9InNtaW1lLnA3cyINCg0KTUlJRjdnWUpLb1pJaHZj
        TkFRY0NvSUlGM3pDQ0Jkc0NBUU14RFRBTEJnbGdoa2dCWlFNRUFnRXdDd1lKS29aSQ0KaHZjTkFR
        Y0JvSUlEa0RDQ0E0d3dnZ0owb0FNQ0FRSUNGQ1J6alBxREdlL3d2RThOLzcwSU1iNVlaWU04TUEw
        Rw0KQ1NxR1NJYjNEUUVCQ3dVQU1Eb3hIVEFiQmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFH
        OXlhWFI1TVJrdw0KRndZRFZRUUtEQkJYYVhobGJpQk5ZV2xzSUZSbGMzUnpNQjRYRFRJd01ERXdN
        VEF3TURBd01Gb1hEVFF3TURFdw0KTVRBd01EQXdNRm93TWpFT01Bd0dBMVVFQXd3RllXeHBZMlV4
        SURBZUJna3Foa2lHOXcwQkNRRVdFV0ZzYVdObA0KUUdWNFlXMXdiR1V1WTI5dE1JSUJJakFOQmdr
        cWhraUc5dzBCQVFFRkFBT0NBUThBTUlJQkNnS0NBUUVBcm1UNg0KZm4yS0s5V3Zqci9td2FYZ3F2
        dUhsQ25hUmdZam13eGlwTlhxVzZoM3pYWVVWd3RLNy9OZGZVdFJ6dHd6RzJGUg0KS3c2YnBUOUMz
        dFR2ZC9hb2xiZ2hNWkpYOEhmM1N0VUpCSXg2Tkw2bnRIVGw4T2hDY21qVlJzaWJmQWY1cldueA0K
        RmQ3YUxYMHdvK05CWTZVanpaMzVXVmNMV05MUVBLSW5lWEhwSkw0d2FpZlF2ZXNuN3oyUVVWbzhn
        YTV1NWJObg0KWVVLdlFBa1p5L2xuUlh1cmNnRGxoeis0UG5XTEozejRNVUszVEZZcTZHOGo0TTBs
        Q1VCbi9aV3Bpcmt1M0luUA0KUitzN0pXeTBvOWM5ejVVaHYxVld0eUpiZzN4WW93dHh6UityY2RF
        YllDdFlzUSthdmU4T28wai84T3NhbElnRQ0KSkkxUVlBZmlvRmhsSWIrTld3SURBUUFCbzRHUk1J
        R09NQWtHQTFVZEV3UUNNQUF3RGdZRFZSMFBBUUgvQkFRRA0KQWdXZ01CTUdBMVVkSlFRTU1Bb0dD
        Q3NHQVFVRkJ3TUVNQndHQTFVZEVRUVZNQk9CRVdGc2FXTmxRR1Y0WVcxdw0KYkdVdVkyOXRNQjBH
        QTFVZERnUVdCQlFNYXZYVUVVbXlLVEc1eTgrcVpaTGNxMVZQUHpBZkJnTlZIU01FR0RBVw0KZ0JU
        Mm5CcUx5U3lRcUVHUVhIY3NDcW14NFJxRS96QU5CZ2txaGtpRzl3MEJBUXNGQUFPQ0FRRUFUd21Z
        dE5rVg0KUDZaUlE1c2pCbk9VVlZSYTJ2anRVRE1SaFpqQjlZdzJHOTM4dEc4N1RRUGFqc1lHZVpu
        VmZzY1JIZTE2U3UvcQ0KVkZGdnNBZGFZNU11cDBxWDRBYjQyOVR5WVNObVRvcGZpell1WUJ0ZVRZ
        cGdiK00yeDFSMzNtV2I0eTZpZTVsVw0KTjBuOG9EbDQxUmVXbnlmeFVnT0c0UkE4cVM5WjR0ZllL
        eE5uQ3Y1N3JydUkzTnFDSkZ1NW1NMTZiOGVLdXJZRA0KNFIzUHZtbUNscnFDQzFEaEFudlRyNFpH
        dUdIcTh2WW5DaW5aQXpRUVN2MFlrcUxpeWYxbjlsODdJeDhtd2lxNQ0KdkNNWDJwMHpzOHVzOG9U
        T3plcHdxa2hLZVFIZGhrNG1EOFZHU3lZTHZENWJLS2tCaHU0dkM3dS9adG1GYmIxYQ0KMmpMSUlj
        TUlHQnRxTmpHQ0FpUXdnZ0lnQWdFRGdCUU1hdlhVRVVteUtURzV5OCtxWlpMY3ExVlBQekFMQmds
        Zw0KaGtnQlpRTUVBZ0dnZ2VRd0dBWUpLb1pJaHZjTkFRa0RNUXNHQ1NxR1NJYjNEUUVIQVRBY0Jn
        a3Foa2lHOXcwQg0KQ1FVeER4Y05Nall3T0RJNE1UYzFNelUyV2pBdkJna3Foa2lHOXcwQkNRUXhJ
        Z1FnZlpwY0lWRDE1NjlDNVBFMg0KNDdReldMUjdEc0ZGd250VldqZXpWWXgrU09Bd2VRWUpLb1pJ
        aHZjTkFRa1BNV3d3YWpBTEJnbGdoa2dCWlFNRQ0KQVNvd0N3WUpZSVpJQVdVREJBRVdNQXNHQ1dD
        R1NBRmxBd1FCQWpBS0JnZ3Foa2lHOXcwREJ6QU9CZ2dxaGtpRw0KOXcwREFnSUNBSUF3RFFZSUtv
        WklodmNOQXdJQ0FVQXdCd1lGS3c0REFnY3dEUVlJS29aSWh2Y05Bd0lDQVNndw0KRFFZSktvWklo
        dmNOQVFFQkJRQUVnZ0VBb3hvVkNQclhITTRMekp1am5NRTNqT0hqRFZxak5BSjRkaWE1VGZMKw0K
        U1U4QXpranZKZnB1OFBMZUZMbGhGdEMzdEVoM0NuVVo4Ri8zRTIzbVlhc2ZlQ3VZeVhUNjJuTjA4
        cCtIYjJDbQ0KM2NsbmtNQ1RFN1c0clBEVEc1Z24wUGR2eTdZaE4vVDkwbk1LVnkvRWE4TFVQMnox
        UjdEdlRZODlaU0NLS1hSKw0KZ0o1d3RhOFkwUzduVkVEU3luZnpNdVRyUmdIT3EzbytjSmdoTU1C
        RFJqQW5rTFg2TmowMGQyMHFqSlZ2MGVJOA0KVHRRb2cweEJ2bHQ5di85VFowOGJnRDhnQUhGa012
        STFuR2lHOGRqTklaZTJ3cUFpcGQwOE55MitNSlFackxlYw0KekF2czVOZ21oWUZrM1JYQTU0SFln
        bzJxam04UVhGa3hObUkwK3NOVXk0bWtodz09DQoNCi0tLS0tLUUyMjg2NDUyRUVGQ0UwN0IyOTI4
        QTFFQ0U2MEEyNkFCLS0NCg0K";

    /// A certificate that issued itself: the small authority that issued the
    /// signer certificates above.
    const SELF_ISSUED_CERTIFICATE: &str = "
        MIIDRjCCAi6gAwIBAgIUEBrh72dKgw0FY3Gb6/1i+Dv4pnMwDQYJKoZIhvcNAQELBQAwOjEdMBsG
        A1UEAwwUV2l4ZW4gVGVzdCBBdXRob3JpdHkxGTAXBgNVBAoMEFdpeGVuIE1haWwgVGVzdHMwIBcN
        MjYwODI4MTYxMTQyWhgPMjEyNjA4MDQxNjExNDJaMDoxHTAbBgNVBAMMFFdpeGVuIFRlc3QgQXV0
        aG9yaXR5MRkwFwYDVQQKDBBXaXhlbiBNYWlsIFRlc3RzMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A
        MIIBCgKCAQEAt3+6AVVQpLNxxI50OZabZGlpe+UB4pOF7VhdevQfb0oCH3QhoMOULgjZmYI/lRsd
        +PS2RZauzC3GEa8VTvC9SLKSPQnP4ztLt8N9Y6NpSKT5DrXUxfmeFw2H1fa6/xVCK3jAhb7sHsPc
        Ovm4EOXudA4PvCBdXYGmugaPQ8pGgc5TYuoizdS0fizGSuI19jMvumGr50mpIld81tO6y9TnCwWy
        +dl9q+MJuIORgfFltjgbjt0FFNrKL1ZtwQTFtNLGNpubyBknYXa27wzh/4+pzAxu2W/RBfzrSXsD
        6q5B2GsxCKOdo0cuFLDHuXyBQ+PZoyNDIJeVdkEX5AAVSra/XwIDAQABo0IwQDAPBgNVHRMBAf8E
        BTADAQH/MA4GA1UdDwEB/wQEAwIBBjAdBgNVHQ4EFgQU9pwai8kskKhBkFx3LAqpseEahP8wDQYJ
        KoZIhvcNAQELBQADggEBAK9Rvz9my82DTXSdibZym7mW7P3Y2eIuMzQ9pSCgwp3EPnZNQSU4zL6s
        YzDAe//VwghuR6CcJ6prUBMi6+pCZMJG/ZkFrx5sbj7FB9Ex+1etm4XNOWDjrSOLbIjrFhSQeUVs
        5mNdL0pdA9bU2GeEcmOl6UCoIFgaJGg3OQWVX3r0sB2LdWYcHAfMUPErtrxdHsPnAOvm+0JRCBTh
        zWWa8X6ZvPgiHIDbdUiENSVDfPMZB0/xfDUqO9RDjdvcBsKrchRFLxRYGn5t7qlrHm5TGsTvXZHe
        0Vun3SYgIfpE2oUV08IOXoliXu7vA4y87oDtGKzjI/j++OINhCiOz1S0dh0=";

    /// The same words signed by two people at once, alice first and bob second.
    ///
    /// A message may carry more than one signature and a reader that stops at
    /// the first never sees the second at all. Both of these hold as they
    /// stand, so a test that wants a disagreement makes one by changing the
    /// last byte of the document, which is the last byte of bob's signature.
    const SIGNED_BY_TWO: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3BrY3M3LXNpZ25hdHVyZSI7IG1pY2FsZz0ic2hhLTI1NiI7IGJvdW5k
        YXJ5PSItLS0tM0RCRjkzOEIzMTJFMEExQUVEQUVERjcyMUJCQTgzNDUiDQoNClRoaXMgaXMgYW4g
        Uy9NSU1FIHNpZ25lZCBtZXNzYWdlDQoNCi0tLS0tLTNEQkY5MzhCMzEyRTBBMUFFREFFREY3MjFC
        QkE4MzQ1DQpDb250ZW50LVR5cGU6IHRleHQvcGxhaW4NCg0KVGhlIG1lZXRpbmcgbW92ZWQgdG8g
        VGh1cnNkYXkgYXQgdGVuLg0KDQotLS0tLS0zREJGOTM4QjMxMkUwQTFBRURBRURGNzIxQkJBODM0
        NQ0KQ29udGVudC1UeXBlOiBhcHBsaWNhdGlvbi9wa2NzNy1zaWduYXR1cmU7IG5hbWU9InNtaW1l
        LnA3cyINCkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6IGJhc2U2NA0KQ29udGVudC1EaXNwb3Np
        dGlvbjogYXR0YWNobWVudDsgZmlsZW5hbWU9InNtaW1lLnA3cyINCg0KTUlJTUdBWUpLb1pJaHZj
        TkFRY0NvSUlNQ1RDQ0RBVUNBUUV4RFRBTEJnbGdoa2dCWlFNRUFnRXdDd1lKS29aSQ0KaHZjTkFR
        Y0JvSUlIR2pDQ0E0WXdnZ0p1b0FNQ0FRSUNGRkxBUTB6cWhyOXZvT2pycWQ2aG9yM2tibTJOTUEw
        Rw0KQ1NxR1NJYjNEUUVCQ3dVQU1Eb3hIVEFiQmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFH
        OXlhWFI1TVJrdw0KRndZRFZRUUtEQkJYYVhobGJpQk5ZV2xzSUZSbGMzUnpNQjRYRFRJd01ERXdN
        VEF3TURBd01Gb1hEVFF3TURFdw0KTVRBd01EQXdNRm93TGpFTU1Bb0dBMVVFQXd3RFltOWlNUjR3
        SEFZSktvWklodmNOQVFrQkZnOWliMkpBWlhoaA0KYlhCc1pTNWpiMjB3Z2dFaU1BMEdDU3FHU0li
        M0RRRUJBUVVBQTRJQkR3QXdnZ0VLQW9JQkFRRFFwVndxZUJTaw0Kck9DMXh1UDEwNWlZS1YwcTlw
        cEgwMjc2azdvQUI4RThEb1EwUmdOUWJrS0dxMm5MejJ1ZzJrT3h5ZjhadkZ5NA0KUFBSUFcya1dr
        eDh0L09NZDhHM3FLQmlHN2l6eFlIZGlDbTZxYmQ5REkxRThRREFXZ0xGblBhVFQvcEdMSkc3MA0K
        WGxpKzNtNTk5aWhEYnhIUngvV3ZPNU1GNFp0VTFGQjVJUytYUkc3eDF0cEFqcVhFVi9RK0QxTEJr
        NTgyWXdicg0KM09rSjBVUXpLS1JkUE9RdDdZNmtJQnhuVkc2bzFhcGpJeUw1amtsSm5heVB0V0tu
        VEV2Nk4yTGdEODFid0tObQ0KZngvMjZvUm11SDVzcjR5K3J2ZnpVVmh6ZnZRZHRMR3BDMlFXNGJ1
        VlozWUxkY2IzTlh0UHc4SG4zczc2MC8yWA0KdzcyRHZOeWExa2p6QWdNQkFBR2pnWTh3Z1l3d0NR
        WURWUjBUQkFJd0FEQU9CZ05WSFE4QkFmOEVCQU1DQmFBdw0KRXdZRFZSMGxCQXd3Q2dZSUt3WUJC
        UVVIQXdRd0dnWURWUjBSQkJNd0VZRVBZbTlpUUdWNFlXMXdiR1V1WTI5dA0KTUIwR0ExVWREZ1FX
        QkJSc216eWo3UTdOek1DT3lUQS8rNVJqTi9PeEd6QWZCZ05WSFNNRUdEQVdnQlNNZWN5Rw0KM0to
        dnA2QW9XYnF1dmhmSEZ6dEdNVEFOQmdrcWhraUc5dzBCQVFzRkFBT0NBUUVBTjdlNXE5eDVkbUUy
        ckxwZg0KRUV6TW53amhRaDRXZklqOVhTWVVXV0ppSXp6VnR6dUVLbmJVZlZyY09TMzFrcVUvWGdD
        Y0tmMUErbDhvRC9aUg0KanFVRWVxbFF3ZXQ5azhiMndVUWZLVzZwV082NDNpSTJOVXNNMGJlVGlH
        OWlWUHJteGVHenczamN4NFRIRWJ5dQ0KaUlKeHV6b3BlbDFCM25xZkkrd1lqdGJxM0VUMkVtRGdh
        YkhmTU4vdjRhaVlWVjYrZVM4ZHpQeklJV0hISnZFTQ0KUTdZRnNLMko3MER1MWFMUVZBeFJkR1d3
        cjF2MjFDM1NFdkRlVXd1NTB2bXhQWXFIOU5FZjVmUDJORzhBS2N5Sg0KcmJFUkpCZkNHMDNnc1RU
        S3UvWXhGdDY1MWJDSzZDYjdNVXZBUGl4enIzZTlhRTV0Y2x0bjJScW1tdzdqY3Rlcg0KZU5WSWhU
        Q0NBNHd3Z2dKMG9BTUNBUUlDRkZMQVEwenFocjl2b09qcnFkNmhvcjNrYm0yTU1BMEdDU3FHU0li
        Mw0KRFFFQkN3VUFNRG94SFRBYkJnTlZCQU1NRkZkcGVHVnVJRlJsYzNRZ1FYVjBhRzl5YVhSNU1S
        a3dGd1lEVlFRSw0KREJCWGFYaGxiaUJOWVdsc0lGUmxjM1J6TUI0WERUSXdNREV3TVRBd01EQXdN
        Rm9YRFRRd01ERXdNVEF3TURBdw0KTUZvd01qRU9NQXdHQTFVRUF3d0ZZV3hwWTJVeElEQWVCZ2tx
        aGtpRzl3MEJDUUVXRVdGc2FXTmxRR1Y0WVcxdw0KYkdVdVkyOXRNSUlCSWpBTkJna3Foa2lHOXcw
        QkFRRUZBQU9DQVE4QU1JSUJDZ0tDQVFFQXdYdU5YdVIyZWhWRg0Kbmdwb0lPWlJqZFFRY2NTcUNR
        WHpZTEtRU2tVY2ZzVHVONHBkQUc1NG1JcUhwUWJQYks1bFFiMDJLUmhPMEUzTA0KcGZScm45VitM
        Mzg2aFNqc3NNVmNzZmY3SFBMdVYvRnJBaERRWktXbzltYUhyL1JOR0hPU1NpYml2V1QzZFhsbA0K
        QkdLS2V1dXNHTXo4N0djb3Z5eURxSVByb2JkTE5STTFRNjRQSTRIeFcweXFVbUx0blVSb0lXMEZJ
        YWlxZjg1ZA0Kbkwzdzc3YzhtWlRhWGlJNzhFSENwbUpBNG9ESjQyMEpQNVFMeXIvc3NJTlM3ditm
        MUlKS2l2QjZ1bm01ODRZUQ0KN3FNM21KMmNhakN1NnhOMEFIemZKMWhlZkV6RFdIN1BSVHNmTm41
        U0F6SHdJWmFETlRaOGNhRktNRHhRNEJkNQ0KMFQ0dnNwd3M0d0lEQVFBQm80R1JNSUdPTUFrR0Ex
        VWRFd1FDTUFBd0RnWURWUjBQQVFIL0JBUURBZ1dnTUJNRw0KQTFVZEpRUU1NQW9HQ0NzR0FRVUZC
        d01FTUJ3R0ExVWRFUVFWTUJPQkVXRnNhV05sUUdWNFlXMXdiR1V1WTI5dA0KTUIwR0ExVWREZ1FX
        QkJUWUZRMmdxdyszTERYNmZmcVFCVEwvS3BhNy9UQWZCZ05WSFNNRUdEQVdnQlNNZWN5Rw0KM0to
        dnA2QW9XYnF1dmhmSEZ6dEdNVEFOQmdrcWhraUc5dzBCQVFzRkFBT0NBUUVBREY3U3dNd04vZ3ho
        K0RjYw0Kc3Q2cXMwcU15WVAxMkcveUlxOWZwQUtMRFdmeUhKQkxQRDdFYk5yQzM1SEkrcmtlcTk0
        eEtNTFFrTXE2RHhNZg0KeTczOUtXY3gwS3hhRzFpY2FLNk5GczY4S2hkTGhFRkt3TFhlRklWQzFS
        TWMxalovNEl2QmhnTTBKUlVNS1dhLw0KR0ZCbW9OOHR3UmYzcjRPNFpGNzNRSjJ4WU1KS0l4ZmJy
        K1Z5WFBsN2tTbEpOYTJvY3lzZEJVM1NnbTNsVGE2ZQ0KU1BtRXl0V2xHZGkzVmc5QmVGejJORDdF
        clFJUUR3bEZkYVZnV044OGJXTUpnZ2NUTnBnK3N3SlVKUGxhZFRlKw0KaW5qMUZjUXM2cXdva1Vv
        R1pHVTZFMlBwYnZucjh3cUx1b0svcXI0SlZQU0lxVDNRUEVkWmhnR2kxckhtZTZSTA0KN0FOTHFE
        R0NCTVF3Z2dKZUFnRUJNRkl3T2pFZE1Cc0dBMVVFQXd3VVYybDRaVzRnVkdWemRDQkJkWFJvYjNK
        cA0KZEhreEdUQVhCZ05WQkFvTUVGZHBlR1Z1SUUxaGFXd2dWR1Z6ZEhNQ0ZGTEFRMHpxaHI5dm9P
        anJxZDZob3Izaw0KYm0yTU1Bc0dDV0NHU0FGbEF3UUNBYUNCNURBWUJna3Foa2lHOXcwQkNRTXhD
        d1lKS29aSWh2Y05BUWNCTUJ3Rw0KQ1NxR1NJYjNEUUVKQlRFUEZ3MHlOakE0TWpneE9EVTFNakph
        TUM4R0NTcUdTSWIzRFFFSkJERWlCQ0I5bWx3aA0KVVBYbnIwTGs4VGJqdEROWXRIc093VVhDZTFW
        YU43TlZqSDVJNERCNUJna3Foa2lHOXcwQkNROHhiREJxTUFzRw0KQ1dDR1NBRmxBd1FCS2pBTEJn
        bGdoa2dCWlFNRUFSWXdDd1lKWUlaSUFXVURCQUVDTUFvR0NDcUdTSWIzRFFNSA0KTUE0R0NDcUdT
        SWIzRFFNQ0FnSUFnREFOQmdncWhraUc5dzBEQWdJQlFEQUhCZ1VyRGdNQ0J6QU5CZ2dxaGtpRw0K
        OXcwREFnSUJLREFOQmdrcWhraUc5dzBCQVFFRkFBU0NBUUJJYnlVWlNpRnBuUTFoQ2xIRzJ1bkFC
        TlVmR2RUbw0KZExEOS9WbVdrNDZNeFBaMmhXRlc2OUwzbWNKcS9icEhwZzZod0EzaTA4MUlSQUhW
        clUzRWlYYllJRXM5cG9Yag0KYTJyaTgwNHUyb0ZsbGxNYUkwUTlJRXBESWRNQlJrekNxYk9uUUEw
        L2YycXV0K0EyNGZtY2NmUllzNW1TRXBHeg0KbnBmRlpZRjVLNmY0ZmtXdU93TUZHVk5HRUhzUHV4
        cmFiMlBNendoTS9BS2p4QzNsUDJ0NHBtTnNpcHNtVHV2Sg0KdlRyZTNnaXovbXRLS3BibmZ3VXhy
        NnVBU05rSTJZdnVzeUU1Q0ZGaEcyandmSStXYmZTYk1Td0NEZ0t0MlkwSA0KZlVHK0QvMER3M0pK
        UVhlakhXQktuTkE1YnlDbW9aalBSUWVUQ3RxMStXNkJqSWNjMUhxUzl1RytNSUlDWGdJQg0KQVRC
        U01Eb3hIVEFiQmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFHOXlhWFI1TVJrd0Z3WURWUVFL
        REJCWA0KYVhobGJpQk5ZV2xzSUZSbGMzUnpBaFJTd0VOTTZvYS9iNkRvNjZuZW9hSzk1RzV0alRB
        TEJnbGdoa2dCWlFNRQ0KQWdHZ2dlUXdHQVlKS29aSWh2Y05BUWtETVFzR0NTcUdTSWIzRFFFSEFU
        QWNCZ2txaGtpRzl3MEJDUVV4RHhjTg0KTWpZd09ESTRNVGcxTlRJeVdqQXZCZ2txaGtpRzl3MEJD
        UVF4SWdRZ2ZacGNJVkQxNTY5QzVQRTI0N1F6V0xSNw0KRHNGRndudFZXamV6Vll4K1NPQXdlUVlK
        S29aSWh2Y05BUWtQTVd3d2FqQUxCZ2xnaGtnQlpRTUVBU293Q3dZSg0KWUlaSUFXVURCQUVXTUFz
        R0NXQ0dTQUZsQXdRQkFqQUtCZ2dxaGtpRzl3MERCekFPQmdncWhraUc5dzBEQWdJQw0KQUlBd0RR
        WUlLb1pJaHZjTkF3SUNBVUF3QndZRkt3NERBZ2N3RFFZSUtvWklodmNOQXdJQ0FTZ3dEUVlKS29a
        SQ0KaHZjTkFRRUJCUUFFZ2dFQVVHcHZvMC9RdDhGbUZxSGUrdExjb0cwamYrRGhLWkVCMUlFalQ0
        cENlc3ZmSyt2OA0Kb09RTnM3cWViRllqZ29NYitwaXRQdkZMSUtRNXJPd2crZFFjeW0vd05halJk
        akkrZzE4NWozMFFCNUUxTmoveA0Kd3gwSEs0NEFnZFhjRm82cVNlc3E0bStmVWRtbmVCYmxvOTNt
        SGxBZnFsODIrWncwZ3YvWUxIejJMM2liSnVDcA0KeHY4SkJTek0yZ1dIbFhtcGNsWUhlbUNNRmND
        cjVmQWtiYUViZTV3MlZDa2txZ29SMFE3WkhzMjRGeGt1VElXSg0KRXM0N2U0WGttdEFRYVVhSEtG
        N2FZMHgrd1VpSGh3QVJnTjExY2xYWnRWRytvbDAxNk1QbExtei95bDN5a3hjNw0KRVFWLzMrKzNs
        TlVmRkNOUWtOU012Qlp6Qnl3bDBvTmFDZ0x3YlE9PQ0KDQotLS0tLS0zREJGOTM4QjMxMkUwQTFB
        RURBRURGNzIxQkJBODM0NS0tDQoNCg==";

    /// The same words again, with an RFC 3161 timestamp on the signature.
    ///
    /// Real output from OpenSSL throughout: the message from `cms -sign`, the
    /// timestamp from `ts -reply` acting as an authority over the exact bytes
    /// of the signature, and the token put into the signer's unsigned
    /// attributes where the standard says it goes. The authority says the
    /// signing moment was 28 August 2026, inside the signer certificate's dates
    /// and long before they run out in 2040.
    const SIGNED_AND_TIMESTAMPED: &str = "
        TUlNRS1WZXJzaW9uOiAxLjANCkNvbnRlbnQtVHlwZTogbXVsdGlwYXJ0L3NpZ25lZDsgcHJvdG9j
        b2w9ImFwcGxpY2F0aW9uL3BrY3M3LXNpZ25hdHVyZSI7IG1pY2FsZz0ic2hhLTI1NiI7IGJvdW5k
        YXJ5PSItLS0tRkQ0MDU4MzQ0NENFREVERjAyNjY1NTZDNzFGRDFCQ0YiDQoNClRoaXMgaXMgYW4g
        Uy9NSU1FIHNpZ25lZCBtZXNzYWdlDQoNCi0tLS0tLUZENDA1ODM0NDRDRURFREYwMjY2NTU2Qzcx
        RkQxQkNGDQpDb250ZW50LVR5cGU6IHRleHQvcGxhaW4NCg0KVGhlIG1lZXRpbmcgbW92ZWQgdG8g
        VGh1cnNkYXkgYXQgdGVuLg0KDQotLS0tLS1GRDQwNTgzNDQ0Q0VERURGMDI2NjU1NkM3MUZEMUJD
        Rg0KQ29udGVudC1UeXBlOiBhcHBsaWNhdGlvbi9wa2NzNy1zaWduYXR1cmU7IG5hbWU9InNtaW1l
        LnA3cyINCkNvbnRlbnQtVHJhbnNmZXItRW5jb2Rpbmc6IGJhc2U2NA0KQ29udGVudC1EaXNwb3Np
        dGlvbjogYXR0YWNobWVudDsgZmlsZW5hbWU9InNtaW1lLnA3cyINCg0KTUlJTTJnWUpLb1pJaHZj
        TkFRY0NvSUlNeXpDQ0RNY0NBUUV4RFRBTEJnbGdoa2dCWlFNRUFnRXdDd1lKS29aSQ0KaHZjTkFR
        Y0JvSUlEa0RDQ0E0d3dnZ0owb0FNQ0FRSUNGRkxBUTB6cWhyOXZvT2pycWQ2aG9yM2tibTJNTUEw
        Rw0KQ1NxR1NJYjNEUUVCQ3dVQU1Eb3hIVEFiQmdOVkJBTU1GRmRwZUdWdUlGUmxjM1FnUVhWMGFH
        OXlhWFI1TVJrdw0KRndZRFZRUUtEQkJYYVhobGJpQk5ZV2xzSUZSbGMzUnpNQjRYRFRJd01ERXdN
        VEF3TURBd01Gb1hEVFF3TURFdw0KTVRBd01EQXdNRm93TWpFT01Bd0dBMVVFQXd3RllXeHBZMlV4
        SURBZUJna3Foa2lHOXcwQkNRRVdFV0ZzYVdObA0KUUdWNFlXMXdiR1V1WTI5dE1JSUJJakFOQmdr
        cWhraUc5dzBCQVFFRkFBT0NBUThBTUlJQkNnS0NBUUVBd1h1Tg0KWHVSMmVoVkZuZ3BvSU9aUmpk
        UVFjY1NxQ1FYellMS1FTa1VjZnNUdU40cGRBRzU0bUlxSHBRYlBiSzVsUWIwMg0KS1JoTzBFM0xw
        ZlJybjlWK0wzODZoU2pzc01WY3NmZjdIUEx1Vi9GckFoRFFaS1dvOW1hSHIvUk5HSE9TU2liaQ0K
        dldUM2RYbGxCR0tLZXV1c0dNejg3R2Nvdnl5RHFJUHJvYmRMTlJNMVE2NFBJNEh4VzB5cVVtTHRu
        VVJvSVcwRg0KSWFpcWY4NWRuTDN3NzdjOG1aVGFYaUk3OEVIQ3BtSkE0b0RKNDIwSlA1UUx5ci9z
        c0lOUzd2K2YxSUpLaXZCNg0KdW5tNTg0WVE3cU0zbUoyY2FqQ3U2eE4wQUh6ZkoxaGVmRXpEV0g3
        UFJUc2ZObjVTQXpId0laYUROVFo4Y2FGSw0KTUR4UTRCZDUwVDR2c3B3czR3SURBUUFCbzRHUk1J
        R09NQWtHQTFVZEV3UUNNQUF3RGdZRFZSMFBBUUgvQkFRRA0KQWdXZ01CTUdBMVVkSlFRTU1Bb0dD
        Q3NHQVFVRkJ3TUVNQndHQTFVZEVRUVZNQk9CRVdGc2FXTmxRR1Y0WVcxdw0KYkdVdVkyOXRNQjBH
        QTFVZERnUVdCQlRZRlEyZ3F3KzNMRFg2ZmZxUUJUTC9LcGE3L1RBZkJnTlZIU01FR0RBVw0KZ0JT
        TWVjeUczS2h2cDZBb1dicXV2aGZIRnp0R01UQU5CZ2txaGtpRzl3MEJBUXNGQUFPQ0FRRUFERjdT
        d013Tg0KL2d4aCtEY2NzdDZxczBxTXlZUDEyRy95SXE5ZnBBS0xEV2Z5SEpCTFBEN0ViTnJDMzVI
        SStya2VxOTR4S01MUQ0Ka01xNkR4TWZ5NzM5S1djeDBLeGFHMWljYUs2TkZzNjhLaGRMaEVGS3dM
        WGVGSVZDMVJNYzFqWi80SXZCaGdNMA0KSlJVTUtXYS9HRkJtb044dHdSZjNyNE80WkY3M1FKMnhZ
        TUpLSXhmYnIrVnlYUGw3a1NsSk5hMm9jeXNkQlUzUw0KZ20zbFRhNmVTUG1FeXRXbEdkaTNWZzlC
        ZUZ6Mk5EN0VyUUlRRHdsRmRhVmdXTjg4YldNSmdnY1ROcGcrc3dKVQ0KSlBsYWRUZStpbmoxRmNR
        czZxd29rVW9HWkdVNkUyUHBidm5yOHdxTHVvSy9xcjRKVlBTSXFUM1FQRWRaaGdHaQ0KMXJIbWU2
        Ukw3QU5McURHQ0NSQXdnZ2tNQWdFQk1GSXdPakVkTUJzR0ExVUVBd3dVVjJsNFpXNGdWR1Z6ZENC
        Qg0KZFhSb2IzSnBkSGt4R1RBWEJnTlZCQW9NRUZkcGVHVnVJRTFoYVd3Z1ZHVnpkSE1DRkZMQVEw
        enFocjl2b09qcg0KcWQ2aG9yM2tibTJNTUFzR0NXQ0dTQUZsQXdRQ0FhQ0I1REFZQmdrcWhraUc5
        dzBCQ1FNeEN3WUpLb1pJaHZjTg0KQVFjQk1Cd0dDU3FHU0liM0RRRUpCVEVQRncweU5qQTRNamd4
        T1RBd05UbGFNQzhHQ1NxR1NJYjNEUUVKQkRFaQ0KQkNCOW1sd2hVUFhucjBMazhUYmp0RE5ZdEhz
        T3dVWENlMVZhTjdOVmpINUk0REI1QmdrcWhraUc5dzBCQ1E4eA0KYkRCcU1Bc0dDV0NHU0FGbEF3
        UUJLakFMQmdsZ2hrZ0JaUU1FQVJZd0N3WUpZSVpJQVdVREJBRUNNQW9HQ0NxRw0KU0liM0RRTUhN
        QTRHQ0NxR1NJYjNEUU1DQWdJQWdEQU5CZ2dxaGtpRzl3MERBZ0lCUURBSEJnVXJEZ01DQnpBTg0K
        QmdncWhraUc5dzBEQWdJQktEQU5CZ2txaGtpRzl3MEJBUUVGQUFTQ0FRQjNpZUZORkRIMVNsZTEw
        dVd5RkFkMw0KWGhsUUFTN3hpYW9GWHM0M09tSE9uV2EzZmVxV0l6bFJsNjBLMXNVU0pFOGdPRmxF
        VHRjdDlwU3BLVGhjQ3YwNA0KUzhjMnZONmJoVkRLNlZsYmt1bnpXeTRHU0Jab0xOQ2l4SVJ6ODE2
        cEZDUUdEWVFFMXVYWm13MWh1b2wybklxRw0KSWwrNkZoY2pZQ1ZxclY1WCt2ZWVPVnZ0UEU0aTBu
        MzRSYXBZRkFWbklwTTNmaEg3MmpnY3BIRHoraURuWm92Sg0KWWFjSUlZK0drTE92ZUFhUHdLQUdK
        ZEhGcHB6NzZRVTI0a1huSjBnUysySWRCejlPbHB0Q1Q3dWszTCtScVVtTA0KNVhtbit1UUFBNzZk
        eW5xdmI5R1Z6WkNCQnhXN28rNjNsK1BPMkRuU3VpdnBOY3hHOG5SUUFCS1g2cTdrUmd1aA0Kb1lJ
        R3FqQ0NCcVlHQ3lxR1NJYjNEUUVKRUFJT01ZSUdsVENDQnBFR0NTcUdTSWIzRFFFSEFxQ0NCb0l3
        Z2daKw0KQWdFRE1ROHdEUVlKWUlaSUFXVURCQUlCQlFBd2diOEdDeXFHU0liM0RRRUpFQUVFb0lH
        dkJJR3NNSUdwQWdFQg0KQmdrckJnRUVBWWFOSHdFd01UQU5CZ2xnaGtnQlpRTUVBZ0VGQUFRZ1Bj
        L1h5Tk5NQ3ZpWnZzeXVIdEpCNEZxWg0KRkdxZnFwYVIzSkRkcjRldlpaY0NBUVFZRHpJd01qWXdP
        REk0TVRrd01qQTBXakFEQWdFQkFRSC9BZ2tBaW4rVg0KSUM5WW5BYWdQNlE5TURzeEhqQWNCZ05W
        QkFNTUZWZHBlR1Z1SUZSbGMzUWdWR2x0WlhOMFlXMXdjekVaTUJjRw0KQTFVRUNnd1FWMmw0Wlc0
        Z1RXRnBiQ0JVWlhOMGM2Q0NBM3d3Z2dONE1JSUNZS0FEQWdFQ0FoUlN3RU5NNm9hLw0KYjZEbzY2
        bmVvYUs5NUc1dGpqQU5CZ2txaGtpRzl3MEJBUXNGQURBNk1SMHdHd1lEVlFRRERCUlhhWGhsYmlC
        VQ0KWlhOMElFRjFkR2h2Y21sMGVURVpNQmNHQTFVRUNnd1FWMmw0Wlc0Z1RXRnBiQ0JVWlhOMGN6
        QWVGdzB5TURBeA0KTURFd01EQXdNREJhRncwME1EQXhNREV3TURBd01EQmFNRHN4SGpBY0JnTlZC
        QU1NRlZkcGVHVnVJRlJsYzNRZw0KVkdsdFpYTjBZVzF3Y3pFWk1CY0dBMVVFQ2d3UVYybDRaVzRn
        VFdGcGJDQlVaWE4wY3pDQ0FTSXdEUVlKS29aSQ0KaHZjTkFRRUJCUUFEZ2dFUEFEQ0NBUW9DZ2dF
        QkFMWGlMdWF6S0JJSTZ3VW83dmlBaXAzdnEzdG1jS0pRbFVjbQ0KY2lueWNZM2VTcUJkTXRoZTV0
        M0JtcmJkd2hRV09OOHpGcWx3c0I0Q3N5ZFpoemgzSkFMSFFNNjR0d2MyMUlHVg0Kd2F0Q1FiZ2t5
        Wnl1OFlNK0Q5TlJ5QTdjVytNaExVRTlmK0dqd0pVUmVDUklhOXNXSkVaWlBsVkNaaWh5dmxKUg0K
        V3FKYUp0UjRqVnQ3MHYxYU5aZG5SS2VDN3M0UW5peFREVFJmVlZNc0tKc2hQVkoyWEFaUnJoT2JO
        R0IyY211Sg0KWk1PL1FOTUtUZmxma0xwYmFPZlQwU2gwb05EenJKeCt5RXVFYXNLRmkrMldqTEdm
        NmFOMDVrVEV2dml6NnVTcg0KcTREVVV0T1BzM3RvemQzYStsZ2hkaEc0dGtzZXNDM2tLWEx1Mzdy
        N09BRjNVYmdiT1hzQ0F3RUFBYU4xTUhNdw0KQ1FZRFZSMFRCQUl3QURBT0JnTlZIUThCQWY4RUJB
        TUNCNEF3RmdZRFZSMGxBUUgvQkF3d0NnWUlLd1lCQlFVSA0KQXdnd0hRWURWUjBPQkJZRUZFRERN
        NmN5bU9wUVkyYmlyamRndDUvZ3FPbUZNQjhHQTFVZEl3UVlNQmFBRkl4NQ0KekliY3FHK25vQ2ha
        dXE2K0Y4Y1hPMFl4TUEwR0NTcUdTSWIzRFFFQkN3VUFBNElCQVFDejU1NW5GakR4dFBqZg0Kd3JL
        dWxuT1h4dWg1K3hUbnNKNENPbFBWbUxFeG5UKzdFeEpsVlh0VG8rSklVNEF1czN4VnJzTzd2ZHRG
        ZmJHaw0Kb3RkNFp2b2xaTHNhOEtFMlFISWYxamtodzA4ZDBseHI2S2syN2l2ZVlITWFOY3UrSEEw
        VmRzSUlpYkhpRnJ4Ng0KZkp4djFTZnh4UEloZW40ZERBUUEwYU96Z2xuREtkTEpyb1JpV0ExcXVo
        YXNNd2F6SGpSWnQyUVVISHA5OXBwTw0KU3NqOXhEVExvM0VCWXBCR1pGUjlxZmxNWlVvZ1dZcUwx
        UmlmZ29sSDlDWkFlelFyVVRhSUpiN2VpSitXVGhpaw0KRHhEZmU4bXhxNDNBMWtYOE5PYUxtNTcy
        S3hKUVJ2dVJHd2pmbVB3SjlrdkZjb25WZUVvMFNXdUR2eERoUkNCTg0KaElsQkFIVWtNWUlDSkRD
        Q0FpQUNBUUV3VWpBNk1SMHdHd1lEVlFRRERCUlhhWGhsYmlCVVpYTjBJRUYxZEdodg0KY21sMGVU
        RVpNQmNHQTFVRUNnd1FWMmw0Wlc0Z1RXRnBiQ0JVWlhOMGN3SVVVc0JEVE9xR3YyK2c2T3VwM3FH
        aQ0KdmVSdWJZNHdEUVlKWUlaSUFXVURCQUlCQlFDZ2dhUXdHZ1lKS29aSWh2Y05BUWtETVEwR0N5
        cUdTSWIzRFFFSg0KRUFFRU1Cd0dDU3FHU0liM0RRRUpCVEVQRncweU5qQTRNamd4T1RBeU1EUmFN
        QzhHQ1NxR1NJYjNEUUVKQkRFaQ0KQkNEQkMyVGFBQWR5c0EwY25ZbEVCT04xVlQ3aURyS2h0RSty
        WFR3UEREd1o0akEzQmdzcWhraUc5dzBCQ1JBQw0KTHpFb01DWXdKREFpQkNCYmlQU0YxY0RpNWEr
        VzIwT1pkTUF0UWN3RGFsS0V2REZ4SHJPM1BidERGakFOQmdrcQ0KaGtpRzl3MEJBUUVGQUFTQ0FR
        Q1orckIzMXFKYzRHc1Nrd3NENTRxUHV0dDl1MVBSYWF3QXA4Nnc0SUZXYmdHVQ0KU1JvOThRQzRG
        NnlHQ0MxTnVsQnRidWE2TWp6OTd6b0VBVkxsdnZFeUFTMHhkU3VtckRaNGtrWTMzcm5LMlZvdA0K
        MUg0VHV4dG5pSHF1WmxtUGNZRXJhSVN5dXFqallCR3doNWRzc2Fxd3ZqSTVNcDhib2luUmkvTFA5
        L2VlN0ozNg0KVUlodnk3Q0tRZmJIdmc2MHJVeVdIYTJsd3Y1SExyclg0S05Zb1IrYkpQSDUzYWsy
        ZmlwZTdMZDRRbklrcGtFdg0KRktuWUF2NGpiM3I0WEwxOCsvUlZSTExKZmYya1pLaERFMmExWmNm
        ZUlRVUNrOHVCUWxlQXBhSkRYc0tOMjdJZQ0KdmEvRWQzdVNZL3hXUGh4M0RtVHN3VnNjZnBxaHVh
        VUN2MXlUU0Znag0KLS0tLS0tRkQ0MDU4MzQ0NENFREVERjAyNjY1NTZDNzFGRDFCQ0YtLQ0KDQo=";

    /// A certificate and the private key that goes with it, as PKCS #12.
    ///
    /// The only way to ask "does this machine hold the private key" and get
    /// yes. Windows can take these bytes into a store held in memory only, so
    /// the answer comes from a key that is really there and nothing is written
    /// to the person's own certificate store.
    const A_KEY_AND_ITS_CERTIFICATE: &str = "
        MIIKSAIBAzCCCg4GCSqGSIb3DQEHAaCCCf8Eggn7MIIJ9zCCBHcGCSqGSIb3DQEHBqCCBGgwggRk
        AgEAMIIEXQYJKoZIhvcNAQcBMBwGCiqGSIb3DQEMAQMwDgQIPxQuNSKvkAcCAggAgIIEMAQYHhcC
        w9rEAIEATQFzWmxjqr0Ivm0DCVhVw8FFd3ipDMiWWOrw+xGLGTB0V0y37Fl5/n+3DdeRuk7XMzbq
        3VWuHAvitGEUwpTOCNRnRYkseazOx+U8XymR/w3ztX7LHL5Y/CuYiyVRL9xSVG/IkomsgTTIFcbM
        apoFCHVJarNISuYp0jmVjY65zawbcv64A4m5+AhD+CIViapQelbuP4KN+fyMB5KZqLrFmfPFT+pP
        oNR6/hpxjBun2WiiW0u9vxAFgMc5G99zCeMgf8Os6JH64dGXlusShbbj0xcGFNto36Pej5uArhlX
        4pVK/UWjnC9OrnVWnrjCG/yZHjXAK+5g1HqIuY/O9FPrj5HYCOx1ZsRUFpsGDxJ7bZLIM0oSMMQt
        kZZd0GkitknE7YCtihsQhWE7HRsEgQXNumMvPlpMlJtHZJZ3kQicgq/NqBPZAjPiPXh0txc5N6ES
        2OhW3BXSMnSd6c+mwjT0lorodADHkEJIDh0/ycLthIDejT/DppvYoTOazuMdzTR5lzCjzJMdKLm+
        KFg2o8Rw6QzsSorcOVSazrUjZupU0ducMEhJj6GEMY29XnIM8xVZgQyYEypY9WuupbcFFeE15nQb
        W37pBh2MJBtfp6olAg6F4sAY6yJaLllxT9am7WhwB14UymzhR4r2lmussunj5YU3DlcILAy6VYuu
        kqqFV7XK4jh3wmc7v1M850tF28amakD0BiW7fFqS5CZ0RHKuKsSic+g4cKw4AB7X0HIpVxycsbCB
        B6JW3ejXoZSbzcOL7jEO0ifrfqoRi6pWsAnxE4m6AhuyaKq9Wcn6LinzMaD/QUTgzxZApCoLFpE7
        zNkzlk2rYjY91DHX6V905+VXb+T7OaF1EQtItsxnqUo6tZrGFvt/SjNkpOw0yGv4bn3/s4fcWKcB
        wYNqelJZKVdJTs9H6BQJvQaxtwxrCeQ9mVdzha2ipe5CT/CBXCuB6bxor2FFwLGNqAyFWgV6Iar+
        d56weWRNlvEIbrDhvvPe0vdEb3inx2+ShcXXwvdTGRTyF3+4TPUXss6YiThqS9ixy7dBP4zS+gsb
        T3diMWFM0eQyFvtSsx/lngzlJq2vaklgH8VR8tLuAzB2CdQxtjAtir759jocG7pubamDsDTy86kt
        eqZf5lbJyC6DOTAbctOONLv119EdencmZkvsJ3ADZYXvqp7xH2VakZyK40P4570QIgNuYIBMyvhT
        HBP/mkgcn6Wwh+eLI+7YH7Wi4F8Y0vmHyiE91fCkauWNnXFPueBxKVC8GfKXri4KXDhFskZh6CWp
        G2u/xK4Sk3BgCcxb8NC1QxSsezxfHCsg+k5bSrtaoHXkxNAQqHWRI4FlmoCfHeizEasCrWghTUoD
        yMHs05/TvIzmyDR2U1/lts2ghUXp6AIi3XYSq5OEYL7pjwqWOi+t8Y4wggV4BgkqhkiG9w0BBwGg
        ggVpBIIFZTCCBWEwggVdBgsqhkiG9w0BDAoBAqCCBO4wggTqMBwGCiqGSIb3DQEMAQMwDgQI5C2c
        GPw0TowCAggABIIEyPew7nWrjuJN0/0miPCozjIjeYIG+hxNQsnbMIuhic5WnSKLP27NtnC/qH+Y
        7ilJbFprufkEYC8t3bPCSdpfRDl40zdXHpYmfvcG8ni6nQEZ6886XospzJeQwkxuM7+RXi0xvSzk
        UXNqTLMKoympOdJ/zY6yRSZXTVEIy5SyKeEVcsBRWIAmEGAQ0R1JVkDUyGHpLjwXhri9cmrZI88D
        RfuZWqOt/RHTTZa7j2EF84+IlBHmrvItpMj8ADviuyZl4dasAa6GO3TICE6BKBo5t5JHnXwNVSwS
        YO2sfm+lWv6oGDJ18A8vqDUWgqnAHAXWogiumtQb+kvgb/KIAPYMWuQu85LcF1/oWow5YTrD8jY4
        zVFOJJFHCcDPTUcSZfvT0ASuhmwo37+Nq6M+62UIho50kuYJ7HxzkoXRWKubb1tk01Xjrlj+3K1B
        jAkZrHM7vBLUuAApTkG53xihsj8SS4WybdpUdjsqJFTXnGl7TycNs9z9f6VklsGILezaeS7hK57g
        Ij2u+Ecz89IYLgxX2pWbrEDnqnQQs+kGShTx1/KI79qE0WozKqRLbzSr2+oYzajgA0bQBoRUFYWV
        x+w4lKrU2P4J9sc5S4AGhR7qY8zs8ddJmdJtUzXHnagquC0WqDqb8Afwg9PcjDLlbHbBU4LNSy2h
        N6tOL8pYdzSe8tT13UTJAgdKVNZ8mK7BGQbsgowbBf5eKs8yjCzikbtxSgLrpW1G8e7YrHAKsLSP
        jcaPnAVtYpbjvAFqErYiVSPlOSgi8nNca8Fgd1xpfFqYP6tw0NSLp/LxLyESwDVYxjN5lQnwqZod
        leBg9F+gHwxM3LROeX2PcxPs/aX57Z0q3Sy9BGvPrLPGFbEEk2v1XO1tYX3wh+k1WF3yEbxwhgtH
        jGMdigomznqS4Pq5Wez8NN2S5MyK7oJL1DN70GQD/DCMlHScjTvq4cuZnM1BbH/fv42WW+PNsah8
        1MgDdw99RWYLNJnF8+qMq6Cy+wIGnaiy3+1Sa19jKBPtM5NqAnFgPHQlwvn4I4vJYX5Hy/SScUDC
        hm+ftjezUNoxu3/0xbshn16MBxT4MNlHLMpgVEfAz/cASazu4YbSU+8AZTFs0o/tLEMXcWzaypiw
        nS/4w0GN3ul/0WgfYoyZD0tO9+L+IS5bk0VDK9UR38KfYk7tFkcYukE11l54iqgkwTtTu5SydTWR
        I4cvDwVWEcjOmB7mmZ3Zcy5/o17Mht9jnVafxR3wbFK7za+5znEanUBSizy9jbYOxBwJEyjUnl/j
        20sQ/bqic1hsBQtIVuHJn7e9y/9L7l725+KvTAWBNdMta1sM3Za9e8wV5kPhT9vt14XJckMtQ8No
        v/zHP8luRNpQZurvRfSHGhwOFYROKkxshOCuSpxx7n3jjMH/X1Lnxdyije6WzCCBbrmsFsWyi20Z
        oTBn5bG01Q8VFFLRB3zAldk1qy0VDq70lpP86Qu9gcDQbjoO4+PlA7bSB6IzDE8sEQbgjRVgT/UP
        o/7RpWaP+iH1gzluy1GLfVWfDICU3NqaJaOTbUO/9e1l8hM94K2fUD0yY0Cy+78tlNAaIViYN1+H
        Zjc7+eHiMMNtMUVyREwcHnhLWdjFXgV6udHaz+qL29U72ZEduoVDgjFcMCMGCSqGSIb3DQEJFTEW
        BBQdbLqAQVzEYW2PP0SWxOK/A/3cHzA1BgkqhkiG9w0BCRQxKB4mAFcAaQB4AGUAbgAgAE0AYQBp
        AGwAIAB0AGUAcwB0ACAAawBlAHkwMTAhMAkGBSsOAwIaBQAEFKCscDQZE919zk+OVnwJ0AEDgz5z
        BAjWCPFo/vSFogICCAA=";

    /// The password the fixture above is locked with.
    ///
    /// In the source in plain sight, which is right: it locks a key made for
    /// this test and nothing else, and a password nobody can read is a fixture
    /// nobody can regenerate.
    const THE_TEST_KEYS_PASSWORD: &str = "wixen-test";

    /// The bytes a fixture stands for.
    pub(super) fn message(encoded: &str) -> Vec<u8> {
        let packed: String = encoded.split_whitespace().collect();
        STANDARD.decode(packed).expect("a fixture that decodes")
    }

    /// The envelope a whole encrypted message fixture carries.
    fn envelope_of(encoded: &str) -> EncryptedMessage {
        let raw = message(encoded);
        let (headers, body) = split_headers_from_body(&raw);
        EncryptedMessage::read(&decode_body(headers, body).expect("base64"))
            .expect("an envelope that reads")
    }

    /// A moment inside the signer certificates' dates.
    pub(super) fn while_the_certificate_was_good() -> DateTime<Utc> {
        at("2026-08-28T12:00:00Z")
    }

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().expect("a moment that reads")
    }

    // ── What shape of S/MIME a part is in ────────────────────────────────

    #[test]
    fn test_the_two_ways_of_signing_and_the_one_way_of_encrypting_are_told_apart() {
        assert_eq!(
            layout_of(
                r#"multipart/signed; protocol="application/pkcs7-signature"; micalg="sha-256"; boundary="abc""#
            ),
            Some(SmimeLayout::SignatureBeside {
                boundary: "abc".to_string()
            })
        );
        assert_eq!(
            layout_of(r#"application/pkcs7-mime; smime-type=signed-data; name="smime.p7m""#),
            Some(SmimeLayout::SignatureAround)
        );
        assert_eq!(
            layout_of(r#"application/pkcs7-mime; smime-type=enveloped-data"#),
            Some(SmimeLayout::Encrypted)
        );
        assert_eq!(layout_of("text/plain; charset=utf-8"), None);
    }

    #[test]
    fn test_the_kind_of_smime_is_read_however_it_is_capitalised() {
        // Every neighbouring value here is folded to lower case before it is
        // compared: the media type, the protocol, the file suffix. This one was
        // not, and the comment beside the file suffix says exactly what that
        // costs, because a comparison that misses one spelling calls signed
        // mail unsigned, which is the silent way to be wrong.
        assert_eq!(
            layout_of("application/pkcs7-mime; smime-type=Signed-Data"),
            Some(SmimeLayout::SignatureAround)
        );
        assert_eq!(
            layout_of("application/pkcs7-mime; smime-type=ENVELOPED-DATA"),
            Some(SmimeLayout::Encrypted)
        );
    }

    #[test]
    fn test_the_older_spelling_of_the_media_type_is_still_read() {
        // OpenSSL writes application/x-pkcs7-signature by default and a great
        // deal of real signed mail carries it. A reader that only knows the
        // registered name reports all of that as not signed at all, which is
        // the worst way to be wrong: silent.
        assert_eq!(
            layout_of(
                r#"multipart/signed; protocol="application/x-pkcs7-signature"; boundary="xyz""#
            ),
            Some(SmimeLayout::SignatureBeside {
                boundary: "xyz".to_string()
            })
        );
        assert_eq!(
            layout_of("application/x-pkcs7-mime; smime-type=enveloped-data"),
            Some(SmimeLayout::Encrypted)
        );
    }

    #[test]
    fn test_a_pgp_signature_in_the_same_wrapper_is_not_taken_for_smime() {
        // multipart/signed is shared with PGP and only the protocol parameter
        // tells them apart. Handing a PGP signature to a reader that only knows
        // certificates ends in "could not be checked" on mail that is perfectly
        // well signed.
        assert_eq!(
            layout_of(r#"multipart/signed; protocol="application/pgp-signature"; boundary="xyz""#),
            None
        );
    }

    #[test]
    fn test_a_signed_part_with_no_boundary_is_not_claimed_to_be_readable() {
        // Without a boundary there is no way to find where the signed content
        // ends, so there is nothing to check and saying so is the only honest
        // answer.
        assert_eq!(
            layout_of(r#"multipart/signed; protocol="application/pkcs7-signature""#),
            None
        );
    }

    #[test]
    fn test_the_file_name_decides_when_the_sender_left_the_type_off() {
        // Senders do leave smime-type off. Guessing wrong means trying to read
        // a signature as an envelope and telling somebody their readable mail
        // is encrypted.
        assert_eq!(
            layout_of(r#"application/pkcs7-mime; name="smime.p7m""#),
            Some(SmimeLayout::Encrypted)
        );
        assert_eq!(
            layout_of(r#"application/pkcs7-mime; name="smime.P7S""#),
            Some(SmimeLayout::SignatureAround)
        );
        assert_eq!(layout_of("application/pkcs7-mime"), None);
    }

    #[test]
    fn test_a_semicolon_inside_a_boundary_does_not_cut_it_in_half() {
        // Half a boundary matches nothing in the body, so the signed content
        // would be reported as missing on a message that is perfectly fine.
        assert_eq!(
            layout_of(
                r#"multipart/signed; protocol="application/pkcs7-signature"; boundary="a;b""#
            ),
            Some(SmimeLayout::SignatureBeside {
                boundary: "a;b".to_string()
            })
        );
    }

    // ── Taking a message apart ───────────────────────────────────────────

    #[test]
    fn test_the_line_break_in_front_of_a_boundary_is_not_part_of_the_content() {
        // One byte too many here fails every signature in exactly the way a
        // changed message would, and from the outside there would be nothing
        // to tell the two apart.
        let body =
            b"--edge\r\nContent-Type: text/plain\r\n\r\nhello\r\n--edge\r\nsecond\r\n--edge--\r\n";

        let parts = parts_between(body, "edge");

        assert_eq!(parts.len(), 2, "{parts:?}");
        assert_eq!(parts[0], b"Content-Type: text/plain\r\n\r\nhello");
        assert_eq!(parts[1], b"second");
    }

    #[test]
    fn test_a_signed_message_gives_back_the_words_and_the_signature_separately() {
        let parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");

        let content = parts.content.expect("the words are beside the signature");
        assert!(
            String::from_utf8_lossy(&content).contains("The meeting moved to Thursday"),
            "{}",
            String::from_utf8_lossy(&content)
        );
        assert!(
            !content.ends_with(b"\r\n--"),
            "the boundary was taken into the content"
        );
        // A signature is a DER SEQUENCE, so it always starts with 0x30.
        assert_eq!(parts.signature.first(), Some(&0x30));
    }

    #[test]
    fn test_a_message_whose_signature_wraps_the_words_has_nothing_beside_it() {
        let parts = take_apart(&message(SIGNED_AROUND)).expect("a signed message");

        assert_eq!(parts.content, None);
        assert_eq!(parts.signature.first(), Some(&0x30));
    }

    #[test]
    fn test_an_encrypted_message_is_not_handed_back_as_a_signed_one() {
        // Reading an envelope as a signature would end in "could not check the
        // signature" on a message that has no signature to check and a real
        // reason nothing can be read.
        let refused = take_apart(&message(ENCRYPTED_TO_ALICE)).expect_err("not a signature");

        assert!(refused.to_string().contains("encrypted"), "{refused}");
    }

    #[test]
    fn test_ordinary_mail_is_refused_rather_than_half_read() {
        let plain = b"From: someone@example.com\r\nContent-Type: text/plain\r\n\r\nhello";

        let refused = take_apart(plain).expect_err("not signed at all");

        assert!(refused.to_string().contains("not signed"), "{refused}");
    }

    #[test]
    fn test_a_folded_content_type_header_is_read_as_one_line() {
        // Senders fold long headers, and a boundary parameter is exactly the
        // sort of long value that gets folded onto the next line.
        let headers = b"Subject: hello\r\nContent-Type: multipart/signed;\r\n \
                        protocol=\"application/pkcs7-signature\";\r\n boundary=\"abc\"\r\n";

        let value = header_value(headers, "content-type").expect("a content type");

        assert_eq!(
            layout_of(&value),
            Some(SmimeLayout::SignatureBeside {
                boundary: "abc".to_string()
            }),
            "{value}"
        );
    }

    // ── Reading ASN.1 ────────────────────────────────────────────────────

    #[test]
    fn test_a_length_that_runs_past_the_end_is_refused() {
        // Believing a declared length over the bytes really there is how a
        // parser reads memory it was not given.
        let claims_five_holds_two = [0x04, 0x05, 0xAA, 0xBB];

        let refused = der::take(&claims_five_holds_two).expect_err("a lie about length");

        assert!(
            refused.to_string().contains("longer than it is"),
            "{refused}"
        );
    }

    #[test]
    fn test_an_open_ended_length_is_refused_by_name() {
        // Some senders write this. Ignoring the marker and reading on would
        // take the rest of the document as the value of this one element.
        let open_ended = [0x30, 0x80, 0x04, 0x00, 0x00, 0x00];

        let refused = der::take(&open_ended).expect_err("an open-ended length");

        assert!(refused.to_string().contains("open-ended"), "{refused}");
    }

    #[test]
    fn test_a_length_count_no_message_could_carry_is_refused() {
        // Eight length bytes describes something larger than any computer's
        // memory. The only reason to send it is to see what this does.
        let absurd = [0x04, 0x88, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

        let refused = der::take(&absurd).expect_err("an absurd length");

        assert!(refused.to_string().contains("length"), "{refused}");
    }

    #[test]
    fn test_an_identifier_reads_back_as_the_number_the_standard_prints() {
        // Getting one of these wrong means checking a SHA-256 signature with
        // SHA-512, which fails on mail that is perfectly good.
        assert_eq!(
            der::oid(&[0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x02]).expect("an oid"),
            oid::SIGNED_DATA
        );
        assert_eq!(
            der::oid(&[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01]).expect("an oid"),
            oid::SHA256
        );
        assert_eq!(
            der::oid(&[0x2B, 0x0E, 0x03, 0x02, 0x1A]).expect("an oid"),
            oid::SHA1
        );
    }

    #[test]
    fn test_an_identifier_that_stops_in_the_middle_is_refused() {
        // The high bit says another byte follows. With nothing after it, a
        // reader that ignored this would quietly drop the last number.
        let unfinished = [0x2A, 0x86];

        assert!(der::oid(&unfinished).is_err());
    }

    // ── The signature that holds ─────────────────────────────────────────

    #[test]
    fn test_a_real_signed_message_adds_up_and_names_who_signed_it() {
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        assert!(report.findings.contains(&Finding::ContentIsWhatWasSigned));
        assert!(
            report
                .findings
                .contains(&Finding::SignedByThatCertificatesKey)
        );
        assert!(
            report
                .findings
                .contains(&Finding::CertificateNamesTheSender {
                    address: "alice@example.com".to_string()
                })
        );
        let signer = report.signer.expect("a certificate came with it");
        assert_eq!(
            signer.email_addresses,
            vec!["alice@example.com".to_string()]
        );
        assert!(signer.subject.contains("alice"), "{}", signer.subject);
        assert!(
            !signer.self_issued,
            "the signer certificate is not its own issuer"
        );
    }

    #[test]
    fn test_a_signature_made_with_an_elliptic_curve_key_is_checked_too() {
        // The choice of verifier switches on the algorithm, and an arm with no
        // message behind it is an arm that could be deleted without a test
        // noticing.
        let report = examine_signed_message(
            &message(SIGNED_WITH_A_CURVE),
            "carol@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_a_signature_made_with_pss_padding_is_checked_too() {
        // PSS keeps the name of its fingerprint inside its parameters. Reading
        // it wrong means falling back to SHA-1 and calling a good signature bad.
        let report = examine_signed_message(
            &message(SIGNED_WITH_PSS),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_a_wrapped_signature_hands_back_the_words_it_was_hiding() {
        // Without this the message cannot be read at all, so getting it wrong
        // shows somebody an empty message rather than a wrong answer.
        let report = examine_signed_message(
            &message(SIGNED_AROUND),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        let words = report.unwrapped_content.expect("the words were inside");
        assert!(
            String::from_utf8_lossy(&words).contains("The meeting moved to Thursday"),
            "{}",
            String::from_utf8_lossy(&words)
        );
    }

    // ── The cases this feature exists for ────────────────────────────────

    #[test]
    fn test_changing_one_letter_of_a_signed_message_is_noticed() {
        // This is the whole reason for the feature. If it passes silently,
        // everything else here is decoration.
        let mut parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");
        let content = parts.content.as_mut().expect("words beside the signature");
        let at = content
            .windows(8)
            .position(|window| window == b"Thursday")
            .expect("the word to change");
        content[at] = b'W';

        let report = examine_signature(
            &parts,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(report.outcome, SignatureOutcome::DoesNotMatch);
        assert!(
            report
                .findings
                .contains(&Finding::ContentIsNotWhatWasSigned)
        );
        assert!(
            report.headline().contains("not signed at all"),
            "{}",
            report.headline()
        );
    }

    #[test]
    fn test_a_signature_that_does_not_belong_to_the_message_is_noticed() {
        // The last byte of the document is the last byte of the signature
        // itself, so changing it leaves every length intact and every part
        // still readable. Only the arithmetic fails, which is exactly the case
        // a checker that only reads structure would wave through.
        let mut parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");
        let last = parts.signature.len() - 1;
        parts.signature[last] ^= 0xFF;

        let report = examine_signature(
            &parts,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::DoesNotMatch,
            "{:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .contains(&Finding::NotSignedByThatCertificatesKey)
        );
    }

    #[test]
    fn test_a_sender_that_names_its_key_rather_than_its_issuer_is_still_matched() {
        // The two ways of naming a signer are a choice in the standard and both
        // are in use. Handling only the common one means a perfectly good
        // signature comes back with no certificate to check it against.
        let report = examine_signed_message(
            &message(SIGNED_NAMING_A_KEY),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        assert!(
            !report
                .findings
                .contains(&Finding::SignersCertificateMissing)
        );
    }

    #[test]
    fn test_a_signature_made_with_a_forgeable_fingerprint_is_not_called_proof() {
        // The arithmetic on this one holds. That is exactly what makes it
        // dangerous: a checker that reports "the signature is valid" and stops
        // is telling somebody a SHA-1 signature means what a SHA-256 one means,
        // and a second document can be built to have the same SHA-1
        // fingerprint as the first.
        let report = examine_signed_message(
            &message(SIGNED_WITH_A_WEAK_FINGERPRINT),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::MatchesButWorthNothing,
            "{:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .contains(&Finding::FingerprintTooWeakToTrust {
                    named: "SHA-1".to_string()
                })
        );
        assert!(
            report.headline().contains("can be forged")
                && report.headline().contains("not signed at all"),
            "{}",
            report.headline()
        );
    }

    #[test]
    fn test_a_signature_whose_certificate_did_not_arrive_is_not_checked() {
        // Real senders leave the certificate out. There is then no key here to
        // check anything against, and the one thing this must not do is treat
        // "nothing to check it with" as "nothing wrong with it".
        let report = examine_signed_message(
            &message(SIGNED_WITH_NO_CERTIFICATE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::NotChecked,
            "{:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .contains(&Finding::SignersCertificateMissing)
        );
        assert_eq!(report.signer, None);
        assert!(
            report.headline().contains("could not check the signature"),
            "{}",
            report.headline()
        );
    }

    #[test]
    fn test_a_message_that_says_it_is_signed_and_carries_rubbish_says_so() {
        let parts = SignedParts {
            content: Some(b"hello".to_vec()),
            signature: b"this is not a signature at all".to_vec(),
        };

        let report = examine_signature(
            &parts,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(report.outcome, SignatureOutcome::NothingToCheck);
        assert!(
            report.headline().contains("no signature to check"),
            "{}",
            report.headline()
        );
    }

    #[test]
    fn test_the_words_of_a_signed_message_are_never_read_from_the_wrong_place() {
        // Pairing one message's words with another's signature has to fail.
        // Both fixtures sign the same sentence, so the fingerprint of the words
        // matches either way and only the check against the key can catch it.
        let beside = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");
        let curve = take_apart(&message(SIGNED_WITH_A_CURVE)).expect("a signed message");
        let crossed = SignedParts {
            content: beside.content,
            signature: curve.signature,
        };

        let report = examine_signature(
            &crossed,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        // Carol really did sign these words, so the arithmetic holds. What has
        // to be caught is that the certificate is not the sender's, and it has
        // to be said in the headline rather than buried.
        assert!(
            report.headline().contains("carol@example.com")
                && report.headline().contains("alice@example.com"),
            "{}",
            report.headline()
        );
    }

    // ── What the certificate says ────────────────────────────────────────

    #[test]
    fn test_an_expired_certificate_is_said_to_have_expired_and_not_called_a_bad_signature() {
        // These are different facts and running them together is how somebody
        // is told a genuine message is a forgery. The signature still adds up.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            at("2050-01-01T00:00:00Z"),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        let expiry = report
            .findings
            .iter()
            .find(|finding| matches!(finding, Finding::CertificateHadExpired { .. }))
            .expect("the expiry was noticed");
        assert!(expiry.spoken().contains("2040"), "{}", expiry.spoken());
        assert!(
            expiry
                .spoken()
                .contains("does not on its own mean the signature is bad"),
            "{}",
            expiry.spoken()
        );
    }

    #[test]
    fn test_a_certificate_that_has_not_started_yet_is_said_to_have_not_started() {
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            at("2019-01-01T00:00:00Z"),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::CertificateHasNotStarted { .. })),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_a_certificate_for_somebody_other_than_the_sender_is_said_so_first() {
        // A signature that adds up on a certificate for another address is the
        // most convincing shape a forgery takes, so it belongs in the first
        // sentence and not in a list further down.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "eve@elsewhere.example",
            while_the_certificate_was_good(),
        );

        assert_eq!(report.outcome, SignatureOutcome::Matches);
        assert!(
            report
                .findings
                .contains(&Finding::CertificateNamesSomebodyElse {
                    certificate_names: vec!["alice@example.com".to_string()],
                    message_says: "eve@elsewhere.example".to_string(),
                })
        );
        assert!(
            report.headline().contains("eve@elsewhere.example"),
            "{}",
            report.headline()
        );
        assert_eq!(
            report
                .detail()
                .first()
                .map(String::as_str)
                .unwrap_or_default(),
            Finding::CertificateNamesSomebodyElse {
                certificate_names: vec!["alice@example.com".to_string()],
                message_says: "eve@elsewhere.example".to_string(),
            }
            .spoken(),
            "the mismatch was not said first"
        );
    }

    #[test]
    fn test_a_sender_address_is_matched_whatever_case_it_is_written_in() {
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "Alice@Example.COM",
            while_the_certificate_was_good(),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::CertificateNamesTheSender { .. })),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_a_certificate_that_issued_itself_is_told_apart_from_one_that_did_not() {
        let its_own = SignerCertificate::read(&message(SELF_ISSUED_CERTIFICATE))
            .expect("a certificate that reads");

        assert!(its_own.self_issued);
        assert!(
            its_own.subject.contains("Wixen Test Authority"),
            "{}",
            its_own.subject
        );
        assert_eq!(
            its_own.email_addresses,
            Vec::<String>::new(),
            "an authority certificate names no mailbox"
        );
    }

    #[test]
    fn test_a_certificate_naming_nobody_is_not_reported_as_naming_the_sender() {
        // Answering "yes, it is for you" from an empty list is how a
        // certificate that says nothing about an address gets read as agreeing.
        let nobody = SignerCertificate::read(&message(SELF_ISSUED_CERTIFICATE))
            .expect("a certificate that reads");

        assert!(!nobody.names("anyone@example.com"));
        let findings = what_the_certificate_says(
            &nobody,
            "anyone@example.com",
            while_the_certificate_was_good(),
            None,
        );
        assert!(
            findings.contains(&Finding::CertificateNamesNobody),
            "{findings:?}"
        );
    }

    // ── The sentences ────────────────────────────────────────────────────

    /// One of every finding, so no arm can be added without a sentence.
    fn one_of_each_finding() -> Vec<Finding> {
        let a_moment = at("2026-01-01T00:00:00Z");
        vec![
            Finding::ContentIsWhatWasSigned,
            Finding::ContentIsNotWhatWasSigned,
            Finding::SignedByThatCertificatesKey,
            Finding::NotSignedByThatCertificatesKey,
            Finding::SignatureKindNotUnderstood {
                named: "something".to_string(),
            },
            Finding::FingerprintTooWeakToTrust {
                named: "SHA-1".to_string(),
            },
            Finding::CertificateHadExpired {
                expired_on: a_moment,
            },
            Finding::CertificateHasNotStarted {
                starts_on: a_moment,
            },
            Finding::CertificateWasInDate,
            Finding::CertificateNamesTheSender {
                address: "alice@example.com".to_string(),
            },
            Finding::CertificateNamesSomebodyElse {
                certificate_names: vec!["bob@other.example".to_string()],
                message_says: "alice@example.com".to_string(),
            },
            Finding::CertificateNamesNobody,
            Finding::CertificateIssuedItself,
            Finding::IssuerTrustedHere,
            Finding::IssuerNotTrustedHere {
                reason: "nobody vouches for it".to_string(),
            },
            Finding::IssuerNotChecked {
                reason: "nobody asked".to_string(),
            },
            Finding::WithdrawalNotChecked,
            Finding::NoSignerInSignature,
            Finding::SignersCertificateMissing,
            Finding::SignatureCoversSomethingElse,
            Finding::CouldNotBeRead {
                reason: "it stopped early".to_string(),
            },
        ]
    }

    #[test]
    fn test_every_finding_has_a_sentence_somebody_could_act_on() {
        // A finding with nothing to say is a row in a list read out as silence.
        for finding in one_of_each_finding() {
            let said = finding.spoken();
            assert!(said.len() > 20, "{finding:?} said only {said:?}");
            assert!(said.ends_with('.'), "{finding:?} said {said:?}");
            assert!(
                !said.contains('{') && !said.contains("Certificate"),
                "{finding:?} leaked its own name or a placeholder: {said:?}"
            );
        }
    }

    #[test]
    fn test_the_word_signed_never_stands_on_its_own_in_a_headline() {
        // People read "signed" as "genuine". Every headline for a signature
        // that held has to carry the difference in the same breath.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        let said = report.headline();
        assert!(said.contains("not changed since"), "{said}");
        assert!(said.contains("the address, not the person"), "{said}");
    }

    #[test]
    fn test_what_a_signature_does_not_show_is_said_on_a_good_message_too() {
        // A caveat that only turns up on bad news teaches people that its
        // absence is good news, which is the opposite of what it is for.
        let good = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );
        let mut broken_parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");
        broken_parts.content = Some(b"nothing like the real message".to_vec());
        let broken = examine_signature(
            &broken_parts,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(good.limits(), broken.limits());
        let all = good.limits().join(" ");
        assert!(all.contains("withdrawn"), "{all}");
        assert!(all.contains("does not show when it was signed"), "{all}");
        assert!(all.contains("subject line"), "{all}");
        assert!(all.contains("is the person you have in mind"), "{all}");
    }

    #[test]
    fn test_the_worst_thing_found_is_the_first_thing_said() {
        // Somebody working by ear hears these in order and may stop after the
        // first. Burying "this has been changed" under "the dates are fine"
        // is the failure this ordering exists to prevent.
        let report = SignatureReport {
            outcome: SignatureOutcome::DoesNotMatch,
            signer: None,
            findings: vec![
                Finding::CertificateWasInDate,
                Finding::WithdrawalNotChecked,
                Finding::ContentIsNotWhatWasSigned,
            ],
            unwrapped_content: None,
            signers: Vec::new(),
        };

        let detail = report.detail();

        assert_eq!(detail[0], Finding::ContentIsNotWhatWasSigned.spoken());
    }

    #[test]
    fn test_everything_a_report_holds_ends_up_in_what_is_spoken() {
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        let spoken = report.spoken();

        assert!(spoken.starts_with(&report.headline()), "{spoken}");
        for line in report.detail() {
            assert!(
                spoken.contains(&line),
                "missing from what is spoken: {line}"
            );
        }
        for limit in report.limits() {
            assert!(
                spoken.contains(limit),
                "missing from what is spoken: {limit}"
            );
        }
    }

    // ── The boundary ─────────────────────────────────────────────────────

    #[test]
    fn test_a_report_says_the_trust_question_was_not_asked_until_it_is() {
        // Leaving it out entirely would read as though it had been asked and
        // come back fine.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::IssuerNotChecked { .. })),
            "{:?}",
            report.findings
        );
        assert!(report.findings.contains(&Finding::WithdrawalNotChecked));
    }

    #[test]
    fn test_an_answer_from_the_store_replaces_the_note_saying_nobody_asked() {
        // Both at once would have the report say the question was open and
        // settled in the same breath.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_issuer_trust(IssuerTrust::NotTrusted {
            reason: "nobody here vouches for it".to_string(),
        });

        assert!(
            !report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::IssuerNotChecked { .. })),
            "{:?}",
            report.findings
        );
        assert!(report.findings.contains(&Finding::IssuerNotTrustedHere {
            reason: "nobody here vouches for it".to_string()
        }));
        // Whether it was withdrawn is a different question and still open.
        assert!(report.findings.contains(&Finding::WithdrawalNotChecked));
    }

    #[test]
    fn test_a_store_that_could_not_answer_still_leaves_the_question_open() {
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_issuer_trust(IssuerTrust::NotChecked {
            reason: "the store would not open".to_string(),
        });

        assert!(report.findings.contains(&Finding::IssuerNotChecked {
            reason: "the store would not open".to_string()
        }));
    }

    // ── Encrypted mail ───────────────────────────────────────────────────

    #[test]
    fn test_an_encrypted_message_says_who_it_was_locked_for_and_how() {
        let envelope = envelope_of(ENCRYPTED_TO_ALICE);

        assert_eq!(envelope.recipients.len(), 1);
        assert!(!envelope.recipients[0].issuer.is_empty());
        assert!(!envelope.recipients[0].wrapped_key.is_empty());
        // AES-256 in CBC mode, which is what nearly all S/MIME uses.
        assert_eq!(envelope.content_algorithm, "2.16.840.1.101.3.4.1.42");
    }

    #[test]
    fn test_a_signature_is_not_read_as_an_envelope() {
        let parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");

        let refused = EncryptedMessage::read(&parts.signature).expect_err("not an envelope");

        assert!(
            refused.to_string().contains("not an encrypted"),
            "{refused}"
        );
    }

    #[test]
    fn test_the_certificate_a_recipient_names_is_the_one_found() {
        // The half of "can this computer open it" that is arithmetic rather
        // than an operating system, so it is tested here rather than left to a
        // machine that happens to have the right certificate installed.
        let envelope = envelope_of(ENCRYPTED_TO_ALICE);
        let signed = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );
        let alice = signed.signer.expect("alice's certificate").der;

        assert_eq!(recipient_matching(&envelope.recipients, &alice), Some(0));
        assert_eq!(
            recipient_matching(&envelope.recipients, &message(SELF_ISSUED_CERTIFICATE)),
            None,
            "a certificate the message was not encrypted to was taken for one that it was"
        );
    }

    #[test]
    fn test_an_encrypted_message_says_plainly_that_it_cannot_be_opened() {
        // Showing an empty message body with no explanation is the failure this
        // sentence exists to prevent.
        let envelope = envelope_of(ENCRYPTED_TO_ALICE);

        for asked in [Some(true), Some(false), None] {
            let said = envelope.spoken(asked);
            assert!(said.contains("encrypted"), "{said}");
            assert!(said.contains("cannot open encrypted mail yet"), "{said}");
        }
        assert!(envelope.spoken(Some(true)).contains("holds a certificate"));
        assert!(
            envelope
                .spoken(Some(false))
                .contains("not encrypted to any")
        );
        assert!(envelope.spoken(None).contains("1 certificate"));
    }

    #[test]
    fn test_opening_an_encrypted_message_is_refused_in_words_rather_than_pretended() {
        // The one thing here that is not built. It has to fail loudly, because
        // a silent empty answer looks exactly like an empty message.
        let store = this_computers_certificates();
        let recipient = Recipient {
            issuer: Vec::new(),
            serial: Vec::new(),
            key_wrapping_algorithm: oid::RSA_ENCRYPTION.to_string(),
            wrapped_key: vec![1, 2, 3],
        };

        let refused = store
            .unwrap_content_key(&recipient)
            .expect_err("nothing here can open an encrypted message");

        assert!(
            refused
                .to_string()
                .contains("cannot open encrypted mail yet"),
            "{refused}"
        );
    }

    // ── This computer's own certificate store ────────────────────────────

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_certificate_that_issued_itself_is_not_trusted_by_this_computer() {
        // The one that matters: an authority nobody has heard of, made five
        // minutes ago, must not come back trusted.
        let store = this_computers_certificates();

        let trust = store.issuer_trust(
            &message(SELF_ISSUED_CERTIFICATE),
            while_the_certificate_was_good(),
        );

        match trust {
            IssuerTrust::NotTrusted { reason } => {
                assert!(reason.contains("not one this computer trusts"), "{reason}");
            }
            other => panic!("a certificate nobody has ever seen came back as {other:?}"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_something_this_computer_really_does_trust_comes_back_trusted() {
        // Without this, the test above would pass just as well against a
        // function that never says yes to anything, and a check that can only
        // ever say no is not a check.
        let store = this_computers_certificates();
        let roots = windows_store::certificates_in("ROOT", false);
        assert!(
            !roots.is_empty(),
            "this machine's list of trusted authorities is empty, which cannot be right"
        );

        let trusted = roots.iter().filter(|root| {
            matches!(
                store.issuer_trust(root, while_the_certificate_was_good()),
                IssuerTrust::Trusted
            )
        });

        assert!(
            trusted.count() > 0,
            "not one of the {} authorities this computer trusts came back trusted",
            roots.len()
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_rubbish_in_place_of_a_certificate_is_not_checked_rather_than_trusted() {
        let store = this_computers_certificates();

        for nonsense in [Vec::new(), b"not a certificate".to_vec()] {
            match store.issuer_trust(&nonsense, while_the_certificate_was_good()) {
                IssuerTrust::NotChecked { .. } => {}
                other => panic!("{nonsense:?} came back as {other:?}"),
            }
        }
    }

    // ── More than one signer ─────────────────────────────────────────────

    #[test]
    fn test_every_signature_on_a_message_is_examined_and_not_only_the_first() {
        // A message may be signed by more than one party. Stopping at the first
        // means a second signature that does not add up is never looked at, and
        // the sentence a person hears is about one claim out of two.
        let report = examine_signed_message(
            &message(SIGNED_BY_TWO),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(report.signers.len(), 2, "{:?}", report.findings);
        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        let named: Vec<Vec<String>> = report
            .signers
            .iter()
            .map(|signer| {
                signer
                    .certificate
                    .as_ref()
                    .map(|certificate| certificate.email_addresses.clone())
                    .unwrap_or_default()
            })
            .collect();
        assert_eq!(
            named,
            vec![
                vec!["alice@example.com".to_string()],
                vec!["bob@example.com".to_string()]
            ],
            "each signature has to be matched to its own certificate"
        );
    }

    #[test]
    fn test_one_signature_that_holds_beside_one_that_does_not_is_called_neither() {
        // The case the whole of this exists for. Reporting only the good one
        // hides that somebody attached a signature that is not what it says;
        // reporting only the bad one throws away a signature that really does
        // show the words are unchanged.
        //
        // The last byte of the document is the last byte of the second signer's
        // signature, so changing it leaves every length intact and every other
        // signature untouched.
        let mut parts = take_apart(&message(SIGNED_BY_TWO)).expect("a signed message");
        let last = parts.signature.len() - 1;
        parts.signature[last] ^= 0xFF;

        let report = examine_signature(
            &parts,
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::SignersDisagree,
            "{:?}",
            report.findings
        );
        assert_eq!(report.signers[0].outcome, SignatureOutcome::Matches);
        assert_eq!(report.signers[1].outcome, SignatureOutcome::DoesNotMatch);
        let said = report.headline();
        assert!(said.contains("do not agree"), "{said}");
        assert!(said.contains("1 of them adds up"), "{said}");
        assert!(said.contains("1 does not add up"), "{said}");
        assert!(said.contains("not signed at all"), "{said}");
    }

    #[test]
    fn test_what_is_said_about_two_signatures_says_which_one_it_is_about() {
        // Run together the two lists contradict each other: "the certificate is
        // for alice" and "the certificate is for bob" are both true and neither
        // is about the same signature. Somebody working by ear needs to be told
        // where one stops and the next starts.
        let report = examine_signed_message(
            &message(SIGNED_BY_TWO),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        let detail = report.detail();

        assert_eq!(detail[0], "Signature 1 of 2.", "{detail:?}");
        let second = detail
            .iter()
            .position(|line| line == "Signature 2 of 2.")
            .expect("the second signature is named");
        let about_alice = detail[..second].join(" ");
        let about_bob = detail[second..].join(" ");
        assert!(about_alice.contains("alice@example.com"), "{about_alice}");
        assert!(about_bob.contains("bob@example.com"), "{about_bob}");
        assert!(
            !about_alice.contains("bob@example.com"),
            "bob turned up under alice's signature: {about_alice}"
        );
    }

    #[test]
    fn test_a_message_signed_once_is_not_made_to_sound_like_a_list() {
        // Nearly every signed message has one signer. Numbering that one
        // "signature 1 of 1" would put a sentence in front of somebody that
        // exists only because the code can count.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert!(
            !report
                .detail()
                .iter()
                .any(|line| line.starts_with("Signature")),
            "{:?}",
            report.detail()
        );
    }

    #[test]
    fn test_a_message_signed_twice_is_not_described_as_though_it_had_one_signature() {
        // The single-signature wording says "signed for alice@example.com".
        // Used on a message bob also signed, that sentence is true about one of
        // the two claims and silently drops the other.
        let report = examine_signed_message(
            &message(SIGNED_BY_TWO),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        let said = report.headline();

        assert!(said.contains("carries 2 signatures"), "{said}");
        assert!(said.contains("2 of them add up"), "{said}");
        assert!(
            !said.contains("Signed for alice@example.com"),
            "one signature was spoken for as though it were the whole message: {said}"
        );
        // And everything found still reaches what is read out.
        let spoken = report.spoken();
        for line in report.detail() {
            assert!(
                spoken.contains(&line),
                "missing from what is spoken: {line}"
            );
        }
    }

    #[test]
    fn test_an_answer_about_one_signature_does_not_land_on_the_other() {
        // The store is asked about one certificate at a time. Folding the
        // answer into the wrong signature would say a sound certificate had
        // been withdrawn, or worse, the other way round.
        let report = examine_signed_message(
            &message(SIGNED_BY_TWO),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_withdrawal_for(1, Withdrawal::Withdrawn);

        assert!(
            report.signers[0]
                .findings
                .contains(&Finding::WithdrawalNotChecked),
            "{:?}",
            report.signers[0].findings
        );
        assert!(
            report.signers[1]
                .findings
                .contains(&Finding::CertificateWithdrawn),
            "{:?}",
            report.signers[1].findings
        );
        assert!(
            report.findings.contains(&Finding::CertificateWithdrawn),
            "the flat list fell behind the per-signer one"
        );
    }

    #[test]
    fn test_an_answer_about_a_signature_that_is_not_there_changes_nothing() {
        // A finding about a certificate the report does not hold is a sentence
        // about nothing, and it would be read out as though it were about the
        // message.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        let unchanged = report
            .clone()
            .with_withdrawal_for(7, Withdrawal::Withdrawn)
            .with_issuer_trust_for(7, IssuerTrust::Trusted);

        assert_eq!(unchanged, report);
    }

    #[test]
    fn test_signatures_that_all_say_the_same_thing_say_it_and_the_rest_disagree() {
        // The rule in one place, so the arms that decide it are each reached by
        // something rather than by whichever fixtures happen to exist.
        let of = |outcomes: &[SignatureOutcome]| {
            let signers: Vec<SignerReport> = outcomes
                .iter()
                .map(|outcome| SignerReport {
                    outcome: *outcome,
                    certificate: None,
                    findings: Vec::new(),
                    signed_at: None,
                })
                .collect();
            agreed_outcome(&signers)
        };

        assert_eq!(of(&[]), SignatureOutcome::NothingToCheck);
        assert_eq!(
            of(&[SignatureOutcome::Matches]),
            SignatureOutcome::Matches,
            "one signature is its own answer"
        );
        assert_eq!(
            of(&[
                SignatureOutcome::DoesNotMatch,
                SignatureOutcome::DoesNotMatch
            ]),
            SignatureOutcome::DoesNotMatch,
            "two that agree are not a disagreement"
        );
        for mixed in [
            [SignatureOutcome::Matches, SignatureOutcome::DoesNotMatch],
            [SignatureOutcome::Matches, SignatureOutcome::NotChecked],
            [
                SignatureOutcome::Matches,
                SignatureOutcome::MatchesButWorthNothing,
            ],
        ] {
            assert_eq!(
                of(&mixed),
                SignatureOutcome::SignersDisagree,
                "{mixed:?} was folded into one answer"
            );
        }
    }

    // ── When it was signed ───────────────────────────────────────────────

    #[test]
    fn test_a_timestamp_says_when_a_message_was_signed_and_who_says_so() {
        let report = examine_signed_message(
            &message(SIGNED_AND_TIMESTAMPED),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        let stamped = report.signers[0]
            .signed_at
            .as_ref()
            .expect("a timestamp came with it");
        assert_eq!(stamped.moment, at("2026-08-28T19:02:04Z"));
        assert!(
            stamped.authority.subject.contains("Wixen Test Timestamps"),
            "{}",
            stamped.authority.subject
        );
        let said = report.detail().join(" ");
        assert!(said.contains("28 August 2026"), "{said}");
        assert!(
            said.contains("calling itself Wixen Test Timestamps,"),
            "the authority was named the way a certificate spells it, not the way somebody says \
             it: {said}"
        );
        assert!(
            said.contains("not from the sender"),
            "the sentence has to say why a timestamp is worth more than a claim: {said}"
        );
    }

    #[test]
    fn test_a_certificate_that_ran_out_after_a_timestamped_signature_is_not_called_bad() {
        // The whole payoff. Without a timestamp an expired certificate and a
        // certificate that expired years after the message was sent read the
        // same, and one of those is ordinary and the other is not.
        let report = examine_signed_message(
            &message(SIGNED_AND_TIMESTAMPED),
            "alice@example.com",
            at("2050-01-01T00:00:00Z"),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        let dates = report
            .findings
            .iter()
            .find(|finding| matches!(finding, Finding::CertificateWasInDateWhenSigned { .. }))
            .expect("the timestamp was used to read the dates");
        assert!(dates.spoken().contains("still good"), "{}", dates.spoken());
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::CertificateHadExpired { .. })),
            "the sentence saying nothing knows when it was signed is no longer true: {:?}",
            report.findings
        );
    }

    #[test]
    fn test_a_message_with_no_timestamp_says_so_rather_than_staying_quiet() {
        // Silence would teach people that a date had been checked. This fixture
        // carries the sender's own claimed signing time, which is ignored on
        // purpose, so "not known" has to be said even though something in the
        // message does claim a date.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert!(
            report
                .findings
                .contains(&Finding::WhenItWasSignedIsNotKnown),
            "{:?}",
            report.findings
        );
        assert_eq!(report.signers[0].signed_at, None);
        let said = Finding::WhenItWasSignedIsNotKnown.spoken();
        assert!(said.contains("the sender chooses it"), "{said}");
    }

    #[test]
    fn test_a_timestamp_made_for_another_signature_is_not_taken_as_this_ones_date() {
        // A timestamp is bytes a sender can copy from anywhere. What stops it
        // being lifted off another message is that it names the exact signature
        // it covers, and that has to be checked rather than assumed.
        let parts = take_apart(&message(SIGNED_AND_TIMESTAMPED)).expect("a signed message");
        let signature = Signature::read(&parts.signature).expect("a signature that reads");
        let token = signature.signers[0]
            .timestamp_token
            .clone()
            .expect("a timestamp came with it");

        let refused = read_timestamp(&token, b"a signature it was never made for");

        assert!(
            matches!(refused, Err(TimestampProblem::CoversSomethingElse)),
            "a timestamp for other bytes was taken as this signature's date"
        );
        // And the same token over the bytes it really covers still reads, so
        // the check above is not passing because nothing works.
        assert!(read_timestamp(&token, &signature.signers[0].signature).is_ok());
    }

    #[test]
    fn test_whether_the_timestamp_authority_is_trusted_is_left_open_until_asked() {
        // Anyone can run a timestamp authority. A date from one nobody vouches
        // for is worth no more than the sender's own word, so the report must
        // not imply the authority was checked.
        let report = examine_signed_message(
            &message(SIGNED_AND_TIMESTAMPED),
            "alice@example.com",
            while_the_certificate_was_good(),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::TimestampAuthorityNotChecked { .. })),
            "{:?}",
            report.findings
        );

        let answered = report.with_timestamp_authority_trust_for(
            0,
            IssuerTrust::NotTrusted {
                reason: "nobody here vouches for it".to_string(),
            },
        );

        assert!(
            !answered
                .findings
                .iter()
                .any(|finding| matches!(finding, Finding::TimestampAuthorityNotChecked { .. })),
            "{:?}",
            answered.findings
        );
        let said = answered
            .findings
            .iter()
            .find(|finding| matches!(finding, Finding::TimestampAuthorityNotTrustedHere { .. }))
            .expect("the answer was folded in")
            .spoken();
        assert!(said.contains("Anyone can run one"), "{said}");
    }

    #[test]
    fn test_a_timestamp_that_will_not_read_leaves_the_date_unknown_rather_than_guessed() {
        // A sender writes the unsigned attributes, so anything at all can turn
        // up in them. What must not happen is a half-read timestamp producing
        // half a date.
        let parts = take_apart(&message(SIGNED_AND_TIMESTAMPED)).expect("a signed message");
        let signature = Signature::read(&parts.signature).expect("a signature that reads");
        let mut signer = signature.signers[0].clone();
        signer.timestamp_token = Some(b"this is not a timestamp".to_vec());
        let mut findings = Vec::new();

        let stamped = fold_in_the_timestamp(&signer, &mut findings);

        assert_eq!(stamped, None);
        assert!(
            findings.contains(&Finding::WhenItWasSignedIsNotKnown),
            "{findings:?}"
        );
        let said = findings
            .iter()
            .find(|finding| matches!(finding, Finding::TimestampCouldNotBeRead { .. }))
            .expect("the timestamp that would not read was said")
            .spoken();
        assert!(said.contains("could not be checked"), "{said}");
    }

    #[test]
    fn test_rubbish_where_a_timestamp_would_go_does_not_make_a_message_unreadable() {
        // The unsigned attributes are the part of a signature no signature
        // covers, so a stranger can put anything there. Refusing the whole
        // document over it would hand anybody a way to make somebody else's
        // signed mail unreadable.
        assert_eq!(timestamp_among(b"this is not an attribute at all"), None);
        assert_eq!(
            timestamp_among(&[0x30, 0x03, 0x02, 0x01, 0x01]),
            None,
            "an attribute whose name is a number, not an identifier"
        );
        assert_eq!(timestamp_among(&[]), None);
    }

    #[test]
    fn test_rubbish_in_front_of_a_timestamp_does_not_hide_the_timestamp() {
        // The other half of the rule above, and the half that was missing. Two
        // of the ways an attribute can be unreadable gave up on the whole list
        // rather than passing over the one attribute, so anybody could append
        // one piece of nonsense in front of a real timestamp and make a
        // properly timestamped message report as having none. That is not a
        // small loss: without a timestamp, a certificate that had expired by
        // the time somebody reads the message is reported as expired rather
        // than as having been in date when it was used.
        //
        // The attributes are built here rather than taken from a fixture
        // because what is under test is the walk over the list, and a real
        // token would only make it harder to see what the list holds.
        let name_is_a_number = [0x30, 0x03, 0x02, 0x01, 0x01];
        let token = [0x30, 0x03, 0x02, 0x01, 0x01];
        let real_timestamp = [
            0x30, 0x14, // SEQUENCE, 20 bytes
            0x06, 0x0B, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x09, 0x10, 0x02,
            0x0E, // the timestamp token attribute's name
            0x31, 0x05, 0x30, 0x03, 0x02, 0x01, 0x01, // SET holding the token
        ];
        let mut attributes = name_is_a_number.to_vec();
        attributes.extend_from_slice(&real_timestamp);

        // On its own it is found, so the fixture is right and the test below is
        // about the rubbish rather than about the token.
        assert_eq!(timestamp_among(&real_timestamp), Some(token.to_vec()));
        assert_eq!(timestamp_among(&attributes), Some(token.to_vec()));
    }

    #[test]
    fn test_the_fingerprint_judged_is_the_one_the_signature_was_verified_with() {
        // A signer names two things that need not agree: the digest it says it
        // took of the content, and the algorithm the signature itself is under.
        // The verifier goes by the second. The verdict on whether the
        // fingerprint can be forged went by the first, and nothing made them
        // agree, so a signer declaring SHA-256 and signing with
        // sha1WithRSAEncryption was checked through the SHA-1 verifier and came
        // back as a signature that adds up with nothing said about it.
        //
        // The two are one answer now, so this asks the one question: which
        // fingerprint is this signature really made with.
        let parts = take_apart(&message(SIGNED_BESIDE)).expect("a signed message");
        let signature = Signature::read(&parts.signature).expect("a signature that reads");
        let mut signer = signature.signers[0].clone();
        let certificate =
            certificate_for(&signature, &signer).expect("the certificate came with it");
        signer.digest_algorithm = oid::SHA256.to_string();
        signer.signature_algorithm = oid::SHA1_WITH_RSA.to_string();

        let how = verification_algorithm(&certificate, &signer).expect("a verifier");

        assert_eq!(how.hash, oid::SHA1);
    }

    #[test]
    fn test_a_timestamp_made_with_a_forgeable_fingerprint_is_refused() {
        // A timestamp moves a certificate's dates, so a forgeable one is worse
        // than a forgeable signature: somebody could build a second statement
        // with the same fingerprint and a date of their choosing.
        let parts = take_apart(&message(SIGNED_AND_TIMESTAMPED)).expect("a signed message");
        let signature = Signature::read(&parts.signature).expect("a signature that reads");
        let over = signature.signers[0].signature.clone();
        let token = signature.signers[0]
            .timestamp_token
            .clone()
            .expect("a timestamp came with it");
        let mut document = Signature::read(&token).expect("a token that reads");

        // As it arrived it is a SHA-256 timestamp and it reads, so the refusal
        // below is about SHA-1 and not about the fixture.
        assert_eq!(document.signers[0].digest_algorithm, oid::SHA256);
        assert!(check_timestamp(&document, &over).is_ok());

        document.signers[0].digest_algorithm = oid::SHA1.to_string();
        let refused = check_timestamp(&document, &over);

        match refused {
            Err(TimestampProblem::CouldNotBeRead(reason)) => {
                assert!(reason.contains("SHA-1"), "{reason}");
            }
            _ => panic!("a timestamp made with a forgeable fingerprint was believed"),
        }
    }

    #[test]
    fn test_a_timestamp_whose_authority_did_not_send_its_certificate_is_not_believed() {
        // With no certificate there is no key to check the authority's own
        // signature against, so the moment is somebody's unsupported word.
        let parts = take_apart(&message(SIGNED_AND_TIMESTAMPED)).expect("a signed message");
        let signature = Signature::read(&parts.signature).expect("a signature that reads");
        let over = signature.signers[0].signature.clone();
        let token = signature.signers[0]
            .timestamp_token
            .clone()
            .expect("a timestamp came with it");
        let mut document = Signature::read(&token).expect("a token that reads");
        document.certificates.clear();

        let refused = check_timestamp(&document, &over);

        match refused {
            Err(TimestampProblem::CouldNotBeRead(reason)) => {
                assert!(
                    reason.contains("certificate did not come with it"),
                    "{reason}"
                );
            }
            _ => panic!("a timestamp with no certificate behind it was believed"),
        }
    }

    #[test]
    fn test_an_authority_is_named_the_way_somebody_would_say_it() {
        // Read out whole, a subject comes back as "C N equals Wixen Test
        // Timestamps comma O equals Wixen Mail Tests", which is a machine's
        // spelling of a name in a sentence a person hears.
        assert_eq!(
            readable_name("CN=Wixen Test Timestamps, O=Wixen Mail Tests"),
            "Wixen Test Timestamps"
        );
        assert_eq!(
            readable_name("O=Somebody, CN=A Name Later On"),
            "A Name Later On"
        );
        // A comma inside a quoted value does not cut the name in half.
        assert_eq!(
            readable_name(r#"CN="Timestamps, Limited", O=Somebody"#),
            r#""Timestamps, Limited""#
        );
        // With no common name at all, the whole subject is better than nothing.
        assert_eq!(readable_name("O=Somebody"), "O=Somebody");
        assert_eq!(readable_name("CN=, O=Somebody"), "CN=, O=Somebody");
    }

    #[test]
    fn test_a_moment_is_read_the_way_the_standard_writes_one() {
        // Getting this wrong moves a signing time by hours or years, and the
        // whole point of a timestamp is the exact moment.
        assert_eq!(
            generalized_time(b"20260828190204Z").expect("a moment"),
            at("2026-08-28T19:02:04Z")
        );
        assert_eq!(
            generalized_time(b"20260828190204.500Z").expect("a moment with a fraction"),
            at("2026-08-28T19:02:04Z"),
            "the fraction is dropped rather than making the whole moment unreadable"
        );
        // A moment with no zone would be read as UTC when it is not, so it is
        // refused rather than assumed.
        assert!(generalized_time(b"20260828190204").is_err());
        assert!(generalized_time(b"20260828190204+0100").is_err());
        assert!(generalized_time(b"not a moment at all").is_err());
    }

    #[test]
    fn test_a_signature_made_outside_its_certificates_dates_is_said_to_have_been() {
        // The other side of the timestamp. A date the certificate was not good
        // on is worse news than no date at all, and must not be reported with
        // the sentence for a signature made while it was still good.
        // Alice's certificate, which runs from 2020 to 2040.
        let certificate = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .signer
        .expect("a certificate came with it");
        let now = while_the_certificate_was_good();

        let too_late = what_the_dates_say(&certificate, now, Some(at("2041-01-01T00:00:00Z")));
        let too_early = what_the_dates_say(&certificate, now, Some(at("2019-01-01T00:00:00Z")));
        let inside = what_the_dates_say(&certificate, now, Some(now));

        for outside in [&too_late, &too_early] {
            assert!(
                matches!(outside, Finding::CertificateWasNotInDateWhenSigned { .. }),
                "{outside:?}"
            );
            assert!(
                outside.spoken().contains("outside its dates at the moment"),
                "{}",
                outside.spoken()
            );
        }
        assert_eq!(
            inside,
            Finding::CertificateWasInDateWhenSigned {
                signed_on: now,
                expired_since: None,
            }
        );
    }

    // ── Whether the certificate has been withdrawn ───────────────────────

    #[test]
    fn test_the_withdrawal_question_starts_open_and_every_answer_closes_it() {
        // Two of these at once would have the report say the question was open
        // and settled in the same breath, and the open one is the sentence
        // somebody would act on.
        let asked = |withdrawal: Withdrawal| {
            examine_signed_message(
                &message(SIGNED_BESIDE),
                "alice@example.com",
                while_the_certificate_was_good(),
            )
            .with_withdrawal(withdrawal)
        };

        for withdrawal in [
            Withdrawal::Withdrawn,
            Withdrawal::NotWithdrawn,
            Withdrawal::StillBeingLookedInto,
            Withdrawal::CouldNotFindOut {
                reason: "nothing here to look in".to_string(),
            },
            Withdrawal::NotAsked,
        ] {
            let report = asked(withdrawal.clone());
            let about_withdrawal: Vec<&Finding> = report
                .findings
                .iter()
                .filter(|finding| {
                    matches!(
                        finding,
                        Finding::WithdrawalNotChecked
                            | Finding::CertificateWithdrawn
                            | Finding::CertificateNotWithdrawn
                            | Finding::WithdrawalStillBeingLookedInto
                            | Finding::WithdrawalCouldNotBeFoundOut { .. }
                    )
                })
                .collect();
            assert_eq!(
                about_withdrawal.len(),
                1,
                "{withdrawal:?} left {about_withdrawal:?}"
            );
        }
    }

    #[test]
    fn test_an_answer_that_has_not_come_back_is_not_read_as_good_news() {
        // The state that lets reading a message stay instant. It has to sound
        // like an open question and not like a clean bill of health, or the
        // whole arrangement is worse than not asking.
        let said = Finding::WithdrawalStillBeingLookedInto.spoken();

        assert!(said.contains("still being looked into"), "{said}");
        assert!(said.contains("rather than as good news"), "{said}");
        assert!(
            !said.contains("has not been withdrawn"),
            "an unanswered question read as an answer: {said}"
        );
    }

    #[test]
    fn test_a_withdrawn_certificate_is_said_in_the_first_sentence() {
        // The signature still adds up. That is what makes this dangerous: a
        // reader that leads with "signed and not changed since" is describing a
        // stolen key as though nothing were wrong.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_withdrawal(Withdrawal::Withdrawn);

        let said = report.headline();

        assert!(said.contains("has been withdrawn"), "{said}");
        assert!(said.contains("not signed at all"), "{said}");
        assert_eq!(
            report
                .detail()
                .first()
                .map(String::as_str)
                .unwrap_or_default(),
            Finding::CertificateWithdrawn.spoken(),
            "the withdrawal was not said first"
        );
    }

    #[test]
    fn test_finding_out_nothing_is_never_worded_as_finding_out_nothing_is_wrong() {
        // The one wording that would undo the whole thing.
        for said in [
            Finding::WithdrawalNotChecked.spoken(),
            Finding::WithdrawalStillBeingLookedInto.spoken(),
            Finding::WithdrawalCouldNotBeFoundOut {
                reason: "there is no list here".to_string(),
            }
            .spoken(),
        ] {
            assert!(
                !said.contains("has not been withdrawn"),
                "not knowing was worded as knowing: {said}"
            );
        }
        assert!(
            Finding::CertificateNotWithdrawn
                .spoken()
                .contains("has not been withdrawn"),
            "and the one that did check has to say so plainly"
        );
    }

    #[test]
    fn test_the_setting_that_lets_authorities_be_asked_is_off_unless_turned_on() {
        // Somebody who does nothing tells nobody anything. This is the same
        // shape as the setting for pictures a message points at, and the two
        // must not read differently.
        assert_eq!(Reach::from_setting(false), Reach::WhatIsAlreadyHere);
        assert_eq!(Reach::from_setting(true), Reach::AskTheAuthority);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_asking_this_computer_about_withdrawal_answers_without_asking_anybody() {
        // A certificate nobody has ever published a withdrawal list for. The
        // answer has to be that nothing could be found out, never that it is in
        // good standing, and it has to come back rather than waiting on a
        // network.
        let store = this_computers_certificates();

        let found = store.withdrawal(
            &message(SELF_ISSUED_CERTIFICATE),
            while_the_certificate_was_good(),
            Reach::WhatIsAlreadyHere,
        );

        match found {
            Withdrawal::CouldNotFindOut { .. } => {}
            other => panic!("a certificate with no withdrawal list came back as {other:?}"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_strangers_own_certificate_is_never_asked_about_over_the_network() {
        // Turning the setting on does not put a stranger's chosen address in
        // reach. This certificate issued itself, so nobody vouches for it, and
        // the answer has to come back from what is already here.
        //
        // What this does not show, and nothing in this suite shows, is that a
        // real fetch to a real authority works. That path has never been run
        // from this machine and is not run by any test.
        let store = this_computers_certificates();

        let found = store.withdrawal(
            &message(SELF_ISSUED_CERTIFICATE),
            while_the_certificate_was_good(),
            Reach::AskTheAuthority,
        );

        match found {
            Withdrawal::CouldNotFindOut { .. } => {}
            other => panic!("a certificate nobody vouches for came back as {other:?}"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_none_of_this_computers_own_authorities_is_reported_as_withdrawn() {
        // A weaker claim than it looks, and worth making: it says the call runs
        // against real certificates and does not manufacture bad news. What it
        // cannot show is a real withdrawal, which needs a certificate somebody
        // has really withdrawn and a list this machine holds.
        let store = this_computers_certificates();
        let roots = windows_store::certificates_in("ROOT", false);
        assert!(!roots.is_empty(), "this machine trusts no authorities");

        for root in roots.iter().take(20) {
            let found = store.withdrawal(
                root,
                while_the_certificate_was_good(),
                Reach::WhatIsAlreadyHere,
            );
            assert_ne!(found, Withdrawal::Withdrawn);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_the_withdrawal_question_really_reaches_windows_own_answer() {
        // Without this the two tests above would pass against a function that
        // can only ever say "nothing was found out", which is what a wrong walk
        // through Windows' chain structures would produce: every pointer read
        // as null, every answer honest and useless.
        //
        // The intermediate authorities this machine holds are the certificates
        // that really carry a withdrawal address, so at least one of them has
        // to come back with something Windows decided rather than with nothing
        // at all. Measured on the machine this was written on: ten answered
        // "not withdrawn" from a list already here, one answered "withdrawn",
        // and six said the authority could not be reached.
        let store = this_computers_certificates();
        let authorities = windows_store::certificates_in("CA", false);
        assert!(
            !authorities.is_empty(),
            "this machine holds no intermediate authorities to ask about"
        );

        // Only the two answers that Windows can give per certificate and this
        // code can only reach by walking into the chain it built. A certificate
        // marked withdrawn by the chain's own error bits does not count here,
        // because that one is readable without the walk.
        let answered = authorities.iter().filter(|certificate| {
            match store.withdrawal(
                certificate,
                while_the_certificate_was_good(),
                Reach::WhatIsAlreadyHere,
            ) {
                Withdrawal::NotWithdrawn => true,
                Withdrawal::CouldNotFindOut { reason } => reason.contains("could not be reached"),
                _ => false,
            }
        });

        assert!(
            answered.count() > 0,
            "not one of the {} authorities this machine holds got a per-certificate answer out of \
             Windows, so nothing here is reading its verdict",
            authorities.len()
        );
    }

    // ── The private key this machine holds ───────────────────────────────

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_private_key_this_machine_really_holds_is_answered_yes() {
        // Until this existed the answer had only ever been no, because no
        // machine any of this is developed on has an S/MIME certificate
        // installed. Windows can take a key into a store held in this process
        // only, so the question gets a real key to answer about and the
        // person's own certificate store is never touched.
        let store = windows_store::WindowsCertificateStore::holding_only_in_memory(
            &message(A_KEY_AND_ITS_CERTIFICATE),
            THE_TEST_KEYS_PASSWORD,
        )
        .expect("a key and a certificate that Windows will take");
        let held = store.certificates_we_hold_keys_for();
        assert_eq!(
            held.len(),
            1,
            "the private key did not come in with the certificate"
        );
        let somebody_else = recipient_naming(&message(SELF_ISSUED_CERTIFICATE));
        let us = recipient_naming(&held[0]);

        let ours = store
            .which_recipient_is_us(&[somebody_else.clone(), us])
            .expect("asking is not itself a failure");

        assert_eq!(
            ours,
            Some(1),
            "the machine did not find the key it is holding"
        );
        assert_eq!(
            store
                .which_recipient_is_us(&[somebody_else])
                .expect("asking is not itself a failure"),
            None,
            "a certificate whose key is not here was claimed as ours"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_importing_a_key_for_a_test_writes_nothing_to_the_persons_own_store() {
        // The condition the test above is only allowed to exist under. If this
        // ever fails, running the tests has left a key behind on somebody's
        // machine.
        let store = windows_store::WindowsCertificateStore::holding_only_in_memory(
            &message(A_KEY_AND_ITS_CERTIFICATE),
            THE_TEST_KEYS_PASSWORD,
        )
        .expect("a key and a certificate that Windows will take");
        let imported = store.certificates_we_hold_keys_for();

        let the_persons_own = windows_store::certificates_in("MY", false);

        for certificate in &imported {
            assert!(
                !the_persons_own.contains(certificate),
                "the test key was written to the person's own certificate store"
            );
        }
        assert_eq!(
            this_computers_certificates()
                .which_recipient_is_us(&[recipient_naming(&imported[0])])
                .expect("asking is not itself a failure"),
            None,
            "the person's own store now answers yes for a key made in a test"
        );
    }

    /// A recipient naming a certificate the way an encrypted message would.
    #[cfg(target_os = "windows")]
    fn recipient_naming(certificate_der: &[u8]) -> Recipient {
        let (_, certificate) =
            X509Certificate::from_der(certificate_der).expect("a certificate that reads");
        Recipient {
            issuer: certificate.issuer().as_raw().to_vec(),
            serial: certificate.raw_serial().to_vec(),
            key_wrapping_algorithm: oid::RSA_ENCRYPTION.to_string(),
            wrapped_key: vec![1, 2, 3],
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_message_encrypted_to_nobody_on_this_machine_is_said_to_be() {
        // A machine with no S/MIME certificate installed, which is every
        // machine this is developed on, has to answer "not you" rather than
        // failing.
        let envelope = envelope_of(ENCRYPTED_TO_ALICE);
        let store = this_computers_certificates();

        let ours = store
            .which_recipient_is_us(&envelope.recipients)
            .expect("asking is not itself a failure");

        assert_eq!(
            ours, None,
            "this machine claimed to hold the private key for a certificate made in a test"
        );
    }
}

#[cfg(test)]
mod a_withdrawn_certificate_is_not_a_good_one {
    use super::tests::{SIGNED_BESIDE, message, while_the_certificate_was_good};
    use super::*;

    #[test]
    fn test_learning_a_certificate_was_withdrawn_stops_it_reading_as_a_match() {
        // The arithmetic still adds up when a certificate has been withdrawn,
        // and that is exactly why the arithmetic is not the answer: withdrawing
        // a certificate is what somebody does once their key has been stolen,
        // so a message signed with it adds up perfectly and proves nothing.
        //
        // Everything a person hears already leads with the withdrawal. This is
        // about the one word a caller is most likely to branch on: something
        // choosing a tick or a cross from `outcome` alone must not choose the
        // tick.
        let report = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        );
        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "the fixture no longer adds up, so this test is about nothing"
        );

        let once_known = report.with_withdrawal(Withdrawal::Withdrawn);

        assert_ne!(
            once_known.outcome,
            SignatureOutcome::Matches,
            "a withdrawn certificate still reports as a match, so anything \
             reading the outcome alone shows a message signed with a stolen \
             key as though it were sound"
        );
        assert_eq!(once_known.outcome, SignatureOutcome::MatchesButWorthNothing);
    }

    #[test]
    fn test_learning_it_was_not_withdrawn_leaves_a_match_alone() {
        // The other half, so the downgrade cannot be written as one that
        // fires whenever anybody asks.
        let once_known = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_withdrawal(Withdrawal::NotWithdrawn);

        assert_eq!(once_known.outcome, SignatureOutcome::Matches);
    }

    #[test]
    fn test_an_answer_still_being_looked_into_leaves_a_match_alone() {
        // Not yet known is not bad news. Downgrading here would report every
        // message as worthless for as long as the asking took.
        let once_known = examine_signed_message(
            &message(SIGNED_BESIDE),
            "alice@example.com",
            while_the_certificate_was_good(),
        )
        .with_withdrawal(Withdrawal::StillBeingLookedInto);

        assert_eq!(once_known.outcome, SignatureOutcome::Matches);
    }
}

#[cfg(test)]
mod the_cheap_first_question {
    use super::tests::{SIGNED_BESIDE, message};
    use super::*;

    #[test]
    fn test_a_signed_message_says_it_claims_a_signature() {
        assert!(claims_a_signature(&message(SIGNED_BESIDE)));
    }

    #[test]
    fn test_ordinary_mail_says_nothing_of_the_kind() {
        // Nearly every message. This has to answer no without taking anything
        // apart, because a reader asks it on every message it opens.
        let plain =
            b"From: a@example.com\r\nSubject: Hello\r\nContent-Type: text/plain\r\n\r\nBody\r\n";

        assert!(!claims_a_signature(plain));
    }

    #[test]
    fn test_a_message_with_no_content_type_at_all_is_not_signed() {
        // Allowed, and common in old mail: a message with no content type is
        // plain text by default, and plain text is not a signature.
        assert!(!claims_a_signature(b"From: a@example.com\r\n\r\nBody\r\n"));
    }

    #[test]
    fn test_the_header_is_found_however_it_is_capitalised() {
        // Header names have no case, and a reader that only knew one spelling
        // would call a signed message ordinary, which is the failure that
        // shows nothing rather than showing something wrong.
        let shouted = b"FROM: a@example.com\r\nCONTENT-TYPE: application/pkcs7-mime; smime-type=signed-data\r\n\r\nx\r\n";

        assert!(claims_a_signature(shouted));
    }

    #[test]
    fn test_an_encrypted_message_does_not_claim_a_signature() {
        // Encrypted is not signed, and this question decides which sentence a
        // reader says. Answering yes sends an encrypted message down the
        // signature path, where `take_apart` refuses it and the reader ends up
        // saying "it says it is signed, but it carries no signature to check"
        // about a message that never said any such thing.
        let encrypted = b"From: a@example.com\r\nContent-Type: application/pkcs7-mime; \
                          smime-type=enveloped-data; name=\"smime.p7m\"\r\n\r\nx\r\n";

        assert!(!claims_a_signature(encrypted));
    }

    #[test]
    fn test_a_folded_line_before_the_wanted_header_is_not_read_as_a_header() {
        // A folded continuation belongs to the header above it. Read as a
        // header of its own, it lets a sender put whatever they like in their
        // own Subject line and have it answered as this message's Content-Type,
        // which decides whether the message is treated as signed at all.
        let headers = b"Subject: hello\r\n Content-Type: application/pkcs7-mime; \
                        smime-type=enveloped-data\r\n\
                        Content-Type: multipart/signed; \
                        protocol=\"application/pkcs7-signature\"; boundary=\"b\"";

        assert_eq!(
            layout_of(&header_value(headers, "content-type").expect("a content type")),
            Some(SmimeLayout::SignatureBeside {
                boundary: "b".to_string()
            })
        );
    }

    #[test]
    fn test_a_multipart_with_an_empty_part_does_not_bring_the_reader_down() {
        // Two delimiters in a row is a legal multipart with an empty part, and
        // it is also what somebody sends on purpose to see what happens.
        // `examine_signed_message` promises never to fail, so this has to come
        // back as a report saying there is nothing to check.
        let raw = b"Content-Type: multipart/signed; \
                    protocol=\"application/pkcs7-signature\"; boundary=\"b\"\r\n\
                    \r\n--b\r\n--b--\r\n";

        let report = examine_signed_message(raw, "a@example.com", a_moment_in_2026());

        assert_eq!(report.outcome, SignatureOutcome::NothingToCheck);
    }

    /// A moment the fixtures' certificates are good at.
    fn a_moment_in_2026() -> DateTime<Utc> {
        "2026-08-28T00:00:00Z".parse().expect("a fixed moment")
    }
}

#[cfg(test)]
mod the_words_that_were_signed {
    use super::tests::{SIGNED_AROUND, message};
    use super::*;

    /// A moment the fixtures' certificates are good at.
    fn a_moment_in_2026() -> DateTime<Utc> {
        "2026-08-28T00:00:00Z".parse().expect("a fixed moment")
    }

    #[test]
    fn test_words_beside_a_signature_are_checked_against_those_words() {
        // The forgery this closes. Take a real signature off a message whose
        // words are wrapped inside it, put it beside words of your own in a
        // `multipart/signed` wrapper, and send it on. If the check reads the
        // words out of the signature rather than the ones the message shows,
        // it reports the sender's own certificate over text they never wrote.
        //
        // RFC 5652 says the content is carried outside the signature or inside
        // it and never both, so a message carrying both is not something to
        // choose between: the words shown are the ones the reader will read,
        // so they are the ones the arithmetic has to be about.
        let real = take_apart(&message(SIGNED_AROUND)).expect("a signed message");
        let rewrapped = SignedParts {
            content: Some(
                b"Content-Type: text/plain\r\n\r\nSend the money to me instead.".to_vec(),
            ),
            signature: real.signature.clone(),
        };

        let report = examine_signature(&rewrapped, "alice@example.com", a_moment_in_2026());

        assert_ne!(
            report.outcome,
            SignatureOutcome::Matches,
            "words the sender never signed were reported as signed: {:?}",
            report.findings
        );
    }

    #[test]
    fn test_words_wrapped_inside_a_signature_are_still_read_from_inside_it() {
        // The other half, so the fix above cannot be a check that simply
        // stopped reading the wrapped content. Nothing carries the words of a
        // signed-around message except the signature itself.
        let report = examine_signed_message(
            &message(SIGNED_AROUND),
            "alice@example.com",
            a_moment_in_2026(),
        );

        assert_eq!(
            report.outcome,
            SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
        assert!(report.unwrapped_content.is_some());
    }
}
