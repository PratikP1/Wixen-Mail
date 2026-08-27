//! Which pictures in a message are shown, and which would report you.
//!
//! # The two kinds, and why they are not the same
//!
//! A picture in a message arrives one of two ways. It is **carried**, as a part
//! of the message itself, referred to by `cid:`; or it is **pointed at**, as an
//! address somewhere on the internet.
//!
//! A carried picture costs nothing to show. It is already on this computer, and
//! showing it tells nobody anything.
//!
//! A pointed-at picture has to be fetched, and fetching it tells the server it
//! came from that this message was opened, by this computer, at this moment. A
//! single invisible one-pixel image is the whole of how mail tracking works.
//! Every marketing message carries one, and so does a good deal of worse than
//! marketing.
//!
//! So the two get opposite defaults: carried pictures are shown, pointed-at
//! ones are not until somebody asks.
//!
//! # What was happening before this
//!
//! Exactly backwards. The sanitizer stripped `cid:` addresses, because they are
//! not a scheme it recognises, so the safe pictures did not appear at all. And
//! it kept the remote ones, which the browser then fetched, so every tracking
//! pixel in every message did its job. Nobody had chosen either of those.

/// Whether pictures that have to be fetched may be fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetching {
    /// Leave them alone. Nothing is asked for, nobody is told.
    Blocked,
    /// Fetch them, which tells each sender their message was opened.
    Allowed,
}

impl Fetching {
    /// What the setting means, read the way the setting is worded.
    ///
    /// The setting asks whether to block, and blocking is what is on by
    /// default, so the ordinary answer is the safe one and somebody who does
    /// nothing is not tracked.
    pub fn from_setting(blocked: bool) -> Self {
        if blocked {
            Fetching::Blocked
        } else {
            Fetching::Allowed
        }
    }
}

/// The picture kinds a message may carry inline.
///
/// Raster formats only, and this is a security decision rather than a
/// convenience. An SVG is a document: it can carry a script, and a picture that
/// can run code is not something to inline into a page showing a stranger's
/// mail. A sender who wants a vector drawing seen can attach it.
pub const KINDS_WORTH_CARRYING: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/// The most one carried picture may be before it is left where it is.
///
/// Carrying it means keeping it in the message cache, which has a budget of its
/// own that evicts. Two megabytes is larger than any logo or signature picture
/// and smaller than a photograph somebody meant to send as a file.
pub const MOST_ONE_PICTURE_MAY_BE: usize = 2 * 1024 * 1024;

/// The most all of one message's carried pictures may come to.
///
/// A newsletter can carry dozens. Past this the rest are left alone rather than
/// turning one message into a large fraction of the cache.
pub const MOST_ONE_MESSAGE_MAY_CARRY: usize = 8 * 1024 * 1024;

/// One picture a message carries, ready to be put into the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carried {
    /// The `Content-ID`, without the angle brackets a header wraps it in.
    pub named: String,
    pub kind: String,
    pub bytes: Vec<u8>,
}

/// Whether a carried picture is one worth putting into the body.
pub fn worth_carrying(kind: &str, how_big: usize) -> bool {
    let kind = kind.trim().to_ascii_lowercase();
    KINDS_WORTH_CARRYING.iter().any(|known| *known == kind)
        && how_big > 0
        && how_big <= MOST_ONE_PICTURE_MAY_BE
}

/// A `Content-ID` as it appears in a `cid:` address.
///
/// Headers write it wrapped in angle brackets and the address does not, so one
/// of the two has to be trimmed or nothing ever matches. Compared without case,
/// the way addresses are.
pub fn plain_content_id(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim()
        .to_ascii_lowercase()
}

/// Put the pictures a message carries into its markup, so they can be shown
/// without asking anybody for anything.
///
/// The `cid:` addresses a message uses mean nothing to a browser, so the parts
/// are written into the body itself. Done once, where the message is parsed,
/// rather than every time it is shown.
///
/// The `data:` addresses a sender wrote are taken out first, so far as a
/// pattern can, which keeps them out of the cache. It is not what makes
/// admitting `data:` pictures safe: that is the renderer's own filter, and
/// [`strip_data_addresses`] says why.
pub fn carry_the_pictures(html: &str, carried: &[Carried]) -> String {
    use base64::Engine as _;

    let without_the_senders = strip_data_addresses(html);
    let mut room_left = MOST_ONE_MESSAGE_MAY_CARRY;
    let mut out = without_the_senders;
    for picture in carried {
        if !worth_carrying(&picture.kind, picture.bytes.len()) || picture.bytes.len() > room_left {
            continue;
        }
        room_left -= picture.bytes.len();
        let inline = format!(
            "data:{};base64,{}",
            picture.kind.trim().to_ascii_lowercase(),
            base64::engine::general_purpose::STANDARD.encode(&picture.bytes)
        );
        // Both spellings of the address, since a message may write either.
        for address in [
            format!("cid:{}", picture.named),
            format!("cid:<{}>", picture.named),
        ] {
            out = replace_without_case(&out, &address, &inline);
        }
    }
    out
}

/// Take out the `data:` addresses a sender wrote, so far as a pattern can.
///
/// This catches the ordinary double-quoted spelling and not the others, which
/// is worth saying plainly because an earlier version of this comment claimed
/// it was what made admitting `data:` pictures safe. It is not. What makes that
/// safe is the renderer's own filter, which admits only the exact shape written
/// below, on an `img`, and only in raster kinds. A raster picture cannot run
/// code whatever its bytes turn out to be; an SVG can, and is never admitted.
///
/// So this is one layer of two, and the one that can be fooled. It stays
/// because taking a sender's own out early keeps them out of the cache too.
fn strip_data_addresses(html: &str) -> String {
    replace_without_case(html, "src=\"data:", "src=\"removed-data:")
}

/// Replace every occurrence, comparing without case.
///
/// Addresses in mail are written every way a mail program can think of, and a
/// `CID:` that did not match a `cid:` would leave the picture out for no reason
/// anybody could see.
fn replace_without_case(haystack: &str, needle: &str, with: &str) -> String {
    let lower_hay = haystack.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(haystack.len());
    let mut at = 0;
    while let Some(found) = lower_hay[at..].find(&lower_needle) {
        let start = at + found;
        out.push_str(&haystack[at..start]);
        out.push_str(with);
        at = start + needle.len();
    }
    out.push_str(&haystack[at..]);
    out
}

/// Take the carried pictures back out, leaving what they were described as.
///
/// For quoting. A reply or a forward puts the whole of the original body into
/// the new message, and once pictures are carried in the body that means
/// sending every one of them back to the person who sent them. Measured on
/// three ordinary banners: a message of a hundred and thirty five bytes quoted
/// as two point nine megabytes, and the limit above would allow ten. Many
/// servers refuse a message that size.
///
/// The description is kept, so the quote still reads as something rather than
/// going silently blank.
pub fn without_carried_pictures(html: &str) -> String {
    // The whole tag first, then its attributes read out of it. Trying to
    // capture the description inside one pattern does not work: the
    // description is optional and comes after the address, so a lazy run
    // before an optional group matches the empty string every time and every
    // picture came back as "[Picture]" with the words thrown away.
    an_image_tag_re()
        .replace_all(html, |caught: &regex::Captures<'_>| {
            let tag = &caught[0];
            let address = attribute_of(tag, "src").unwrap_or_default();
            if !is_a_picture_we_carried(&address) {
                return tag.to_string();
            }
            match attribute_of(tag, "alt")
                .map(|alt| alt.trim().to_string())
                .filter(|alt| !alt.is_empty())
            {
                Some(alt) => format!("[Picture: {alt}]"),
                None => "[Picture]".to_string(),
            }
        })
        .into_owned()
}

/// A whole `img` tag.
fn an_image_tag_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r#"(?is)<img\b[^>]*>"#).expect("valid image tag regex"))
}

/// One attribute's value out of a tag.
fn attribute_of(tag: &str, name: &str) -> Option<String> {
    let looking_for = format!(r#"(?i)\b{name}="([^"]*)""#);
    regex::Regex::new(&looking_for)
        .ok()?
        .captures(tag)?
        .get(1)
        .map(|found| found.as_str().to_string())
}

/// Whether an address is a picture this application wrote into the body itself.
///
/// The renderer admits these and nothing else beginning `data:`, which is what
/// keeps a sender's own `data:` picture, and the script an SVG could carry,
/// out of the page.
pub fn is_a_picture_we_carried(address: &str) -> bool {
    KINDS_WORTH_CARRYING.iter().any(|kind| {
        address
            .to_ascii_lowercase()
            .starts_with(&format!("data:{kind};base64,"))
    })
}

/// A picture lifted out of a body so it can be sent as a part of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToSend {
    /// What the body's `cid:` reference names, with no angle brackets.
    pub content_id: String,
    /// The media type, for the part's `Content-Type`.
    pub kind: String,
    /// The picture itself, decoded.
    pub bytes: Vec<u8>,
    /// What it was described as, carried along for the plain half.
    pub described: String,
}

/// Take the pictures out of a body about to be sent, leaving `cid:` behind.
///
/// # Why a body cannot go out with its pictures written into it
///
/// A picture written into the body is a `data:` address, which is how this
/// program shows one and is not how mail carries one. Gmail and Outlook both
/// drop `data:` pictures out of a message they receive, so a body sent as it
/// is displayed looks right to the person who wrote it and arrives with
/// nothing where each picture was. Nobody sending it would find out.
///
/// So the pictures come back out on the way to the server and go as their own
/// parts, which is what `multipart/related` is for and what the same pictures
/// arrived as before [`carry_the_pictures`] put them in. This is that function
/// read backwards, and the pair has to stay a pair.
pub fn pictures_out_of(html: &str) -> (String, Vec<ToSend>) {
    let mut taken: Vec<ToSend> = Vec::new();
    let rewritten = an_image_tag_re()
        .replace_all(html, |caught: &regex::Captures<'_>| {
            let tag = &caught[0];
            let address = attribute_of(tag, "src").unwrap_or_default();
            let described = attribute_of(tag, "alt").unwrap_or_default();
            let Some(picture) = a_carried_picture(&address, &described, taken.len()) else {
                return tag.to_string();
            };
            let referring = format!("cid:{}", picture.content_id);
            taken.push(picture);
            // Exactly, not [`replace_without_case`]: the address came out of
            // this tag a line ago, so it matches as it stands, and comparing
            // without case would lowercase a copy of the whole address to
            // learn nothing. That address is the picture, megabytes of it.
            tag.replace(&address, &referring)
        })
        .into_owned();
    (rewritten, taken)
}

/// One carried picture decoded, or nothing when the address is not one.
///
/// Numbered by where it sits in the message rather than by anything from the
/// picture itself. A name taken from the bytes would tell a recipient that two
/// messages carried the same picture, and a name taken from the clock would
/// say when it was written.
fn a_carried_picture(address: &str, described: &str, already_taken: usize) -> Option<ToSend> {
    use base64::Engine as _;

    if !is_a_picture_we_carried(address) {
        return None;
    }
    let (declared, encoded) = address.split_once(";base64,")?;
    let kind = declared.strip_prefix("data:")?.to_ascii_lowercase();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .ok()?;
    Some(ToSend {
        content_id: format!("picture-{}@wixen-mail.invalid", already_taken + 1),
        kind,
        bytes,
        described: html_escape::decode_html_entities(described).into_owned(),
    })
}

/// What the plain half of a message should add about the pictures it carries.
///
/// Empty when it carries none, which is almost every message.
///
/// A page reads an `img` as no text whatever, and the plain half of an outgoing
/// message is that reading. So without this a message with a picture in it goes
/// out with a silent hole in its plain half where the picture was. The
/// description is the one thing this module will not let anybody skip, and
/// losing it here would lose it for exactly the people the rule is for: the
/// ones reading in plain text.
///
/// At the end rather than in place, because the plain half is what the page
/// rendered and there is no honest way to know where in it each picture fell.
pub fn what_the_plain_text_should_say(pictures: &[ToSend]) -> String {
    let described: Vec<&str> = pictures
        .iter()
        .map(|picture| picture.described.trim())
        .filter(|described| !described.is_empty())
        .collect();
    if described.is_empty() {
        return String::new();
    }
    format!("\n\nPictures in this message:\n{}", described.join("\n"))
}

/// A picture somebody is putting into a message they are writing.
///
/// Carried, the same way a picture a message arrives with is carried, so the
/// person receiving it sees it without fetching anything and without this
/// application having to keep a file anywhere.
///
/// The description is not optional and this is deliberate. Everything else in
/// this module exists because a picture nobody described cannot be read out to
/// somebody who cannot see it. Writing one that way, in an application built
/// for exactly those people, would be the one place it could still be done.
pub fn a_picture_to_send(kind: &str, bytes: &[u8], described: &str) -> Result<String, String> {
    use base64::Engine as _;

    if described.trim().is_empty() {
        return Err(
            "A picture needs a description, so somebody who cannot see it \
                    still knows what you sent."
                .to_string(),
        );
    }
    if !worth_carrying(kind, bytes.len()) {
        return Err(format!(
            "{kind} pictures cannot be put in a message here, or this one is \
             larger than {} megabytes. Attach it as a file instead.",
            MOST_ONE_PICTURE_MAY_BE / (1024 * 1024)
        ));
    }
    Ok(format!(
        r#"<img src="data:{};base64,{}" alt="{}">"#,
        kind.trim().to_ascii_lowercase(),
        base64::engine::general_purpose::STANDARD.encode(bytes),
        html_escape::encode_double_quoted_attribute(described.trim())
    ))
}

/// The kind of picture a file holds, from its name.
///
/// By extension rather than by reading the file, because the answer is only
/// used to decide whether to offer to carry it, and a file that is not really
/// what it is named is refused later by the size and kind check anyway.
pub fn kind_of_picture_file(name: &str) -> Option<&'static str> {
    let name = name.to_ascii_lowercase();
    [
        (".png", "image/png"),
        (".jpg", "image/jpeg"),
        (".jpeg", "image/jpeg"),
        (".gif", "image/gif"),
        (".webp", "image/webp"),
    ]
    .into_iter()
    .find(|(ending, _)| name.ends_with(ending))
    .map(|(_, kind)| kind)
}

/// What to do about one picture's address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// Show it. It is already here.
    ItIsCarried,
    /// Show it, and something on the internet learns this message was opened.
    ItWillBeFetched,
    /// Do not show it, and say so where it would have been.
    HeldBack,
}

/// What to do with the address a picture points at.
///
/// An address that is neither carried nor fetchable is held back too: an
/// address nobody recognises is not one to hand to a browser.
pub fn what_to_do_about(address: &str, fetching: Fetching) -> Showing {
    let address = address.trim();
    if address.to_ascii_lowercase().starts_with("cid:") || is_a_picture_we_carried(address) {
        return Showing::ItIsCarried;
    }
    let reaches_out = ["http://", "https://", "//"]
        .iter()
        .any(|start| address.to_ascii_lowercase().starts_with(start));
    match (reaches_out, fetching) {
        (true, Fetching::Allowed) => Showing::ItWillBeFetched,
        (true, Fetching::Blocked) => Showing::HeldBack,
        // A `data:` address carries its own picture and asks nobody for
        // anything, but this application does not pass a sender's along: a
        // `data:` picture can be an SVG, and an SVG can carry a script.
        (false, _) => Showing::HeldBack,
    }
}

/// What to say where a held-back picture would have been.
///
/// The sender's own description when they gave one, so somebody knows what
/// they are choosing whether to fetch, and a plain statement when they did not.
/// Either way it says the picture was held back rather than leaving a gap that
/// reads as a message with nothing in it.
pub fn what_stands_in_for_it(described: &str) -> String {
    let described = described.trim();
    if described.is_empty() {
        "[Picture not shown, and the sender did not describe it]".to_string()
    } else {
        format!("[Picture not shown: {described}]")
    }
}

/// What to say about a whole message once its pictures have been counted.
///
/// Empty when nothing was held back, so an ordinary message says nothing. The
/// count matters more than it looks: one held-back picture in a message from a
/// person is usually their signature, and thirty is a mailing.
pub fn what_was_held_back(held_back: usize) -> String {
    match held_back {
        0 => String::new(),
        1 => "1 picture was not shown, because fetching it would have told the \
              sender you opened this. Settings, Reading has the switch."
            .to_string(),
        many => format!(
            "{many} pictures were not shown, because fetching them would have told \
             the senders you opened this. Settings, Reading has the switch."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_picture_being_sent_must_be_described() {
        // The one place somebody using this application could still make a
        // picture nobody can read out. Everything else in this module exists
        // because that is what a picture without a description costs.
        let refused = a_picture_to_send("image/png", &a_tiny_png(), "   ");

        assert!(refused.is_err());
        assert!(refused.unwrap_err().contains("needs a description"));
    }

    #[test]
    fn test_a_described_picture_is_carried_into_the_message() {
        // Carried rather than pointed at, so the person receiving it sees it
        // without fetching anything from anywhere.
        let markup = a_picture_to_send("image/png", &a_tiny_png(), "A chart of sales")
            .expect("a described picture");

        assert!(markup.contains("data:image/png;base64,"), "{markup}");
        assert!(markup.contains(r#"alt="A chart of sales""#), "{markup}");
    }

    #[test]
    fn test_a_description_with_a_quote_in_it_cannot_break_out_of_the_attribute() {
        // Somebody's own words, going straight into markup. A bare quote would
        // close the attribute and start writing tags.
        let markup = a_picture_to_send("image/png", &a_tiny_png(), r#"The "big" chart"#)
            .expect("a described picture");

        assert!(!markup.contains(r#"alt="The "big""#), "{markup}");
        assert!(markup.contains("&quot;"), "{markup}");
    }

    #[test]
    fn test_a_kind_that_cannot_be_carried_says_to_attach_it_instead() {
        let refused = a_picture_to_send("image/svg+xml", &a_tiny_png(), "A drawing");

        assert!(refused.unwrap_err().contains("Attach it as a file"));
    }

    #[test]
    fn test_a_picture_file_is_recognised_by_its_name() {
        assert_eq!(kind_of_picture_file("holiday.JPG"), Some("image/jpeg"));
        assert_eq!(kind_of_picture_file("logo.png"), Some("image/png"));
        assert_eq!(kind_of_picture_file("notes.txt"), None);
        assert_eq!(kind_of_picture_file("drawing.svg"), None);
    }

    fn a_tiny_png() -> Vec<u8> {
        // Not a real PNG. Nothing here decodes it; what matters is that it is
        // bytes of a kind worth carrying and of a size under the limit.
        vec![0x89, b'P', b'N', b'G', 1, 2, 3, 4]
    }

    #[test]
    fn test_a_picture_leaves_the_body_before_it_is_sent() {
        // The defect this exists for. A body sent as it is displayed carries
        // its pictures as `data:` addresses, which Gmail and Outlook both drop
        // out of a message they receive, so it arrives blank where every
        // picture was and the person who sent it is never told.
        let written = a_picture_to_send("image/png", &a_tiny_png(), "A cat").expect("a picture");

        let (rewritten, parts) = pictures_out_of(&written);

        assert_eq!(
            parts.len(),
            1,
            "the picture stayed in the body: {rewritten}"
        );
        assert!(
            !rewritten.contains("data:"),
            "a data address went out to the server: {rewritten}"
        );
    }

    #[test]
    fn test_the_body_points_at_the_part_that_was_taken_out_of_it() {
        // The two halves have to name the same thing. A body referring to one
        // name and a part carrying another is a message whose pictures are all
        // present and none of them shown, which is the same symptom as not
        // having done any of this.
        let written = a_picture_to_send("image/png", &a_tiny_png(), "A cat").expect("a picture");

        let (rewritten, parts) = pictures_out_of(&written);

        assert!(
            rewritten.contains(&format!("cid:{}", parts[0].content_id)),
            "the body does not refer to the part: {rewritten}"
        );
    }

    #[test]
    fn test_the_picture_is_carried_out_whole_and_still_described() {
        // Decoded back to what went in, because the part carries bytes rather
        // than the text they were written as. The description stays on the tag:
        // it is what a screen reader reads at the other end, and losing it here
        // would undo the one thing this module refuses to let anybody skip.
        let written = a_picture_to_send("image/png", &a_tiny_png(), "A cat").expect("a picture");

        let (rewritten, parts) = pictures_out_of(&written);

        assert_eq!(
            parts[0].bytes,
            a_tiny_png(),
            "the picture came back changed"
        );
        assert_eq!(parts[0].kind, "image/png");
        assert!(rewritten.contains(r#"alt="A cat""#), "{rewritten}");
    }

    #[test]
    fn test_every_picture_in_a_message_gets_a_name_of_its_own() {
        // Two pictures sharing one name is a message where the second replaces
        // the first everywhere it is shown. Nothing about one picture would
        // look wrong, so this is only ever found with more than one.
        let one = a_picture_to_send("image/png", &a_tiny_png(), "First").expect("a picture");
        let two = a_picture_to_send("image/jpeg", &a_tiny_png(), "Second").expect("a picture");

        let (_, parts) = pictures_out_of(&format!("<p>{one}</p><p>{two}</p>"));

        assert_eq!(parts.len(), 2);
        assert_ne!(
            parts[0].content_id, parts[1].content_id,
            "two pictures were given the same name"
        );
        assert_eq!(parts[1].kind, "image/jpeg", "the kinds were mixed up");
    }

    #[test]
    fn test_the_plain_half_still_says_what_the_pictures_were() {
        // A page reads an `img` as no text at all, so the plain half of a
        // message goes out with a silent hole where each picture was. The
        // description is the one thing this module refuses to let anybody
        // skip, and dropping it here drops it for the people that rule is for.
        let written = a_picture_to_send("image/png", &a_tiny_png(), "A cat").expect("a picture");
        let (_, parts) = pictures_out_of(&written);

        let said = what_the_plain_text_should_say(&parts);

        assert!(said.contains("A cat"), "the description was lost: {said:?}");
    }

    #[test]
    fn test_a_description_written_with_a_quote_in_it_reads_as_it_was_typed() {
        // It goes into the markup escaped, so reading it straight back out
        // would put `&quot;` in front of somebody in the plain half.
        let written =
            a_picture_to_send("image/png", &a_tiny_png(), r#"Ada's "best" cat"#).expect("one");
        let (_, parts) = pictures_out_of(&written);

        assert_eq!(parts[0].described, r#"Ada's "best" cat"#);
    }

    #[test]
    fn test_a_message_with_no_pictures_adds_nothing_to_the_plain_half() {
        // Almost every message. Anything added here would be added to all mail
        // rather than the little of it that carries a picture.
        assert!(what_the_plain_text_should_say(&[]).is_empty());
    }

    #[test]
    fn test_a_body_with_no_pictures_is_left_exactly_as_it_was() {
        // Every message goes through this, and almost none of them carry a
        // picture. Rewriting an ordinary body would put this in the path of
        // all mail rather than the little of it this is about.
        let ordinary = r#"<p>Hello</p><img src="https://example.com/tracker.gif">"#;

        let (rewritten, parts) = pictures_out_of(ordinary);

        assert_eq!(rewritten, ordinary);
        assert!(parts.is_empty(), "something was taken out of a plain body");
    }

    #[test]
    fn test_a_carried_picture_is_written_into_the_body_so_it_can_be_shown() {
        // A `cid:` address means nothing to a browser. Without this the picture
        // the message already holds cannot be drawn at all.
        let out = carry_the_pictures(
            r#"<img src="cid:logo@example" alt="Logo">"#,
            &[Carried {
                named: "logo@example".to_string(),
                kind: "image/png".to_string(),
                bytes: a_tiny_png(),
            }],
        );

        assert!(out.contains("data:image/png;base64,"), "{out}");
        assert!(!out.contains("cid:"), "the address was left behind: {out}");
        assert!(out.contains(r#"alt="Logo""#), "the description went: {out}");
    }

    #[test]
    fn test_an_address_written_with_brackets_is_matched_too() {
        // Some mail programs write the angle brackets into the address itself.
        let out = carry_the_pictures(
            r#"<img src="CID:<Logo@Example>">"#,
            &[Carried {
                named: "logo@example".to_string(),
                kind: "image/png".to_string(),
                bytes: a_tiny_png(),
            }],
        );

        assert!(out.contains("data:image/png;base64,"), "{out}");
    }

    #[test]
    fn test_a_drawing_that_could_carry_a_script_is_not_carried() {
        // An SVG is a document and can run code. A picture that can run code is
        // not something to write into a page showing a stranger's mail.
        assert!(!worth_carrying("image/svg+xml", 100));
        assert!(worth_carrying("image/png", 100));
    }

    #[test]
    fn test_a_picture_too_large_to_be_worth_keeping_is_left_alone() {
        // Carrying it means keeping it in the message cache for as long as the
        // message is kept.
        assert!(!worth_carrying("image/png", MOST_ONE_PICTURE_MAY_BE + 1));
        assert!(worth_carrying("image/png", MOST_ONE_PICTURE_MAY_BE));
        assert!(!worth_carrying("image/png", 0));
    }

    #[test]
    fn test_a_message_carrying_more_than_its_share_stops_at_the_limit() {
        // A newsletter can carry dozens. The rest are left as they were rather
        // than turning one message into a large part of the cache.
        let big = Carried {
            named: "a".to_string(),
            kind: "image/png".to_string(),
            bytes: vec![7; MOST_ONE_PICTURE_MAY_BE],
        };
        let many: Vec<Carried> = (0..8)
            .map(|n| Carried {
                named: format!("p{n}"),
                ..big.clone()
            })
            .collect();
        let html: String = (0..8).map(|n| format!(r#"<img src="cid:p{n}">"#)).collect();

        let out = carry_the_pictures(&html, &many);

        let carried = out.matches("data:image/png;base64,").count();
        assert!(carried > 0, "nothing was carried at all");
        assert!(
            carried < 8,
            "every one was carried, so the limit does nothing: {carried}"
        );
    }

    #[test]
    fn test_a_picture_the_sender_wrote_in_themselves_is_taken_out() {
        // This is what lets the renderer admit `data:` pictures at all: after
        // this, the only ones left are the ones written here. A sender's own
        // could be an SVG carrying a script.
        let out = carry_the_pictures(
            r#"<img src="data:image/svg+xml,<svg onload=alert(1)>">"#,
            &[],
        );

        assert!(!out.contains(r#"src="data:"#), "{out}");
    }

    #[test]
    fn test_only_what_this_wrote_is_recognised_as_carried() {
        assert!(is_a_picture_we_carried("data:image/png;base64,AAAA"));
        assert!(is_a_picture_we_carried("DATA:IMAGE/JPEG;BASE64,AAAA"));
        assert!(!is_a_picture_we_carried("data:image/svg+xml;base64,AAAA"));
        assert!(!is_a_picture_we_carried("data:text/html;base64,AAAA"));
        assert!(!is_a_picture_we_carried("https://example.com/x.png"));
    }

    #[test]
    fn test_quoting_a_message_does_not_send_its_pictures_back() {
        // A body carries its pictures inline now, so quoting one whole means
        // replying with every picture the sender sent. Measured before this
        // was written: three ordinary banners turned a message of 135 bytes
        // into 2.9 megabytes, and the per-message limit allows ten. Servers
        // refuse messages that size.
        let carried = carry_the_pictures(
            r#"<p>Hello</p><img src="cid:p0" alt="Spring banner">"#,
            &[Carried {
                named: "p0".to_string(),
                kind: "image/png".to_string(),
                bytes: vec![7; 64 * 1024],
            }],
        );
        assert!(carried.len() > 64 * 1024, "the picture was not carried");

        let quoted = without_carried_pictures(&carried);

        assert!(
            quoted.len() < 200,
            "the quote still carries the picture: {} bytes",
            quoted.len()
        );
        assert!(quoted.contains("[Picture: Spring banner]"), "{quoted}");
        assert!(
            quoted.contains("Hello"),
            "the message itself went: {quoted}"
        );
    }

    #[test]
    fn test_a_quoted_picture_nobody_described_still_leaves_a_mark() {
        let carried = carry_the_pictures(
            r#"<img src="cid:p0">"#,
            &[Carried {
                named: "p0".to_string(),
                kind: "image/png".to_string(),
                bytes: a_tiny_png(),
            }],
        );

        assert_eq!(without_carried_pictures(&carried), "[Picture]");
    }

    #[test]
    fn test_quoting_leaves_a_picture_that_was_only_pointed_at_alone() {
        // It costs nothing to quote: it is an address, not the picture, and
        // taking it out would change what the original said.
        let pointed_at = r#"<img src="https://example.com/x.png" alt="A chart">"#;

        assert_eq!(without_carried_pictures(pointed_at), pointed_at);
    }

    #[test]
    fn test_a_content_id_is_matched_however_the_header_wrapped_it() {
        assert_eq!(plain_content_id("<Logo@Example>"), "logo@example");
        assert_eq!(plain_content_id("  logo@example  "), "logo@example");
    }

    #[test]
    fn test_a_picture_the_message_carries_is_shown_whatever_the_setting_says() {
        // It is already on this computer. Showing it asks nobody for anything,
        // so there is nothing for the setting to protect against.
        for fetching in [Fetching::Blocked, Fetching::Allowed] {
            assert_eq!(
                what_to_do_about("cid:part1@example.com", fetching),
                Showing::ItIsCarried,
                "a carried picture was held back with fetching {fetching:?}"
            );
        }
    }

    #[test]
    fn test_a_carried_picture_is_recognised_however_it_is_capitalised() {
        // Mail headers and addresses come from every mail program ever
        // written, and the scheme is compared without case everywhere else.
        assert_eq!(
            what_to_do_about("CID:Part1@example.com", Fetching::Blocked),
            Showing::ItIsCarried
        );
    }

    #[test]
    fn test_a_picture_somewhere_on_the_internet_is_held_back_by_default() {
        // The whole point. One invisible pixel is how mail tracking works.
        assert_eq!(
            what_to_do_about("https://tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
        assert_eq!(
            what_to_do_about("http://tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
    }

    #[test]
    fn test_an_address_with_no_scheme_of_its_own_still_reaches_out() {
        // `//host/x.gif` takes the scheme of the page around it, which for a
        // message shown in a browser is a real fetch. Missing this would leave
        // the commonest way of writing a tracking pixel working.
        assert_eq!(
            what_to_do_about("//tracker.example.com/pixel.gif", Fetching::Blocked),
            Showing::HeldBack
        );
    }

    #[test]
    fn test_somebody_who_asks_for_them_gets_them_and_is_told_what_that_means() {
        assert_eq!(
            what_to_do_about("https://example.com/photo.jpg", Fetching::Allowed),
            Showing::ItWillBeFetched
        );
    }

    #[test]
    fn test_an_address_this_does_not_understand_is_held_back() {
        // Including `data:`, which carries its own picture and asks nobody for
        // anything, and is still not passed along: it can be an SVG and an SVG
        // can carry a script.
        for address in [
            "data:image/svg+xml,<svg onload=alert(1)>",
            "javascript:alert(1)",
            "ftp://example.com/x.gif",
            "",
        ] {
            assert_eq!(
                what_to_do_about(address, Fetching::Allowed),
                Showing::HeldBack,
                "{address:?} was not held back"
            );
        }
    }

    #[test]
    fn test_the_setting_reads_the_way_it_is_worded() {
        // The switch asks whether to block, and blocking is what is on by
        // default, so somebody who changes nothing is not tracked.
        assert_eq!(Fetching::from_setting(true), Fetching::Blocked);
        assert_eq!(Fetching::from_setting(false), Fetching::Allowed);
    }

    #[test]
    fn test_a_held_back_picture_leaves_the_senders_description_in_its_place() {
        // So somebody knows what they would be fetching before they decide to.
        let said = what_stands_in_for_it("Our new spring range");

        assert!(said.contains("Our new spring range"), "{said}");
        assert!(said.contains("not shown"), "{said}");
    }

    #[test]
    fn test_a_held_back_picture_nobody_described_still_leaves_a_mark() {
        // A gap where a picture was reads as a message with a hole in it. The
        // sender's missing description is said rather than hidden.
        let said = what_stands_in_for_it("   ");

        assert!(said.contains("did not describe"), "{said}");
    }

    #[test]
    fn test_a_message_with_nothing_held_back_says_nothing() {
        assert!(what_was_held_back(0).is_empty());
    }

    #[test]
    fn test_one_held_back_picture_is_not_reported_in_the_plural() {
        let said = what_was_held_back(1);

        assert!(said.starts_with("1 picture was"), "{said}");
        assert!(!said.contains("pictures"), "{said}");
    }

    #[test]
    fn test_the_report_says_how_many_and_where_to_change_it() {
        // The count tells a signature apart from a mailing, and naming the
        // setting is the difference between a warning and a dead end.
        let said = what_was_held_back(30);

        assert!(said.starts_with("30 pictures"), "{said}");
        assert!(said.contains("Settings, Reading"), "{said}");
    }
}
