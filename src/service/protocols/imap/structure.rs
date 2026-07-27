//! Reading a message's shape without downloading it.
//!
//! A mailbox listing shows whether a message has an attachment. Finding that
//! out by fetching every message is not an option at the sizes this client is
//! built for, so the server is asked for BODYSTRUCTURE instead: the MIME tree,
//! with types and dispositions, and none of the content.
//!
//! The judgement here is the same one `service::mime` makes once a message is
//! open, and it has to agree with it. A newsletter's spacer images are not
//! attachments. If they were, the column would be true for nearly every row,
//! and a column true for nearly every row costs a reader a moment on each one
//! while telling them nothing.

use async_imap::imap_proto::{BodyStructure, ContentDisposition};

/// Whether this message has something the reader would call an attachment.
pub fn has_attachments(structure: &BodyStructure<'_>) -> bool {
    match structure {
        BodyStructure::Multipart { bodies, .. } => bodies.iter().any(has_attachments),
        BodyStructure::Basic { common, .. } => is_attachment(
            common.disposition.as_ref(),
            &common.ty.ty,
            &common.ty.subtype,
        ),
        BodyStructure::Text { common, .. } => {
            // A text part is the body unless the sender said otherwise, which
            // is how a .txt or .csv file arrives.
            disposition_is(common.disposition.as_ref(), "attachment")
        }
        BodyStructure::Message { .. } => {
            // A forwarded message carried as message/rfc822. The reader gets a
            // file they can open, so it counts.
            true
        }
    }
}

/// Whether one non-text part is a file the reader has, rather than furniture.
fn is_attachment(
    disposition: Option<&ContentDisposition<'_>>,
    content_type: &str,
    subtype: &str,
) -> bool {
    if disposition_is(disposition, "attachment") {
        return true;
    }
    if disposition_is(disposition, "inline") {
        // Inline and named is a photograph sent in the body: something the
        // sender meant the reader to have. Inline and unnamed is a spacer.
        return filename_of(disposition).is_some();
    }
    // No disposition at all. Anything that is not text or a multipart wrapper
    // is a file, which is what most senders of a bare application/pdf mean.
    !content_type.eq_ignore_ascii_case("text") && !content_type.eq_ignore_ascii_case("multipart")
        || subtype.eq_ignore_ascii_case("octet-stream")
}

/// Whether the disposition is the one named, ignoring case.
fn disposition_is(disposition: Option<&ContentDisposition<'_>>, name: &str) -> bool {
    disposition.is_some_and(|d| d.ty.eq_ignore_ascii_case(name))
}

/// The filename a disposition carries, if it carries one.
fn filename_of<'a>(disposition: Option<&'a ContentDisposition<'a>>) -> Option<&'a str> {
    disposition?
        .params
        .as_ref()?
        .iter()
        .find_map(|(key, value)| {
            (key.eq_ignore_ascii_case("filename") && !value.trim().is_empty())
                .then_some(value.as_ref())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_imap::imap_proto::{
        BodyContentCommon, BodyContentSinglePart, ContentEncoding, ContentType,
    };
    use std::borrow::Cow;

    fn content_type(ty: &'static str, subtype: &'static str) -> ContentType<'static> {
        ContentType {
            ty: Cow::Borrowed(ty),
            subtype: Cow::Borrowed(subtype),
            params: None,
        }
    }

    fn disposition(
        ty: &'static str,
        filename: Option<&'static str>,
    ) -> ContentDisposition<'static> {
        ContentDisposition {
            ty: Cow::Borrowed(ty),
            params: filename.map(|name| vec![(Cow::Borrowed("filename"), Cow::Borrowed(name))]),
        }
    }

    fn single_part() -> BodyContentSinglePart<'static> {
        BodyContentSinglePart {
            id: None,
            md5: None,
            description: None,
            transfer_encoding: ContentEncoding::Binary,
            octets: 128,
        }
    }

    fn text(disp: Option<ContentDisposition<'static>>) -> BodyStructure<'static> {
        BodyStructure::Text {
            common: BodyContentCommon {
                ty: content_type("text", "plain"),
                disposition: disp,
                language: None,
                location: None,
            },
            other: single_part(),
            lines: 4,
            extension: None,
        }
    }

    fn basic(
        ty: &'static str,
        subtype: &'static str,
        disp: Option<ContentDisposition<'static>>,
    ) -> BodyStructure<'static> {
        BodyStructure::Basic {
            common: BodyContentCommon {
                ty: content_type(ty, subtype),
                disposition: disp,
                language: None,
                location: None,
            },
            other: single_part(),
            extension: None,
        }
    }

    fn multipart(bodies: Vec<BodyStructure<'static>>) -> BodyStructure<'static> {
        BodyStructure::Multipart {
            common: BodyContentCommon {
                ty: content_type("multipart", "mixed"),
                disposition: None,
                language: None,
                location: None,
            },
            bodies,
            extension: None,
        }
    }

    #[test]
    fn test_a_plain_message_has_no_attachment() {
        assert!(!has_attachments(&text(None)));
    }

    #[test]
    fn test_an_alternative_message_has_no_attachment() {
        // Plain and HTML versions of the same thing.
        let message = multipart(vec![text(None), basic("text", "html", None)]);
        assert!(!has_attachments(&message));
    }

    #[test]
    fn test_a_pdf_marked_as_an_attachment_counts() {
        let message = multipart(vec![
            text(None),
            basic(
                "application",
                "pdf",
                Some(disposition("attachment", Some("report.pdf"))),
            ),
        ]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_a_pdf_with_no_disposition_at_all_still_counts() {
        // Plenty of senders omit it, and a reader with a PDF sitting in their
        // message would not accept "the header was missing" as a reason for an
        // empty column.
        let message = multipart(vec![text(None), basic("application", "pdf", None)]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_a_newsletters_spacer_image_does_not_count() {
        // The whole reason this function is not just "is there a non-text
        // part". A column true for nearly every row tells the reader nothing.
        let message = multipart(vec![
            basic("text", "html", None),
            basic("image", "gif", Some(disposition("inline", None))),
        ]);
        assert!(!has_attachments(&message));
    }

    #[test]
    fn test_a_named_inline_photograph_does_count() {
        let message = multipart(vec![
            basic("text", "html", None),
            basic(
                "image",
                "jpeg",
                Some(disposition("inline", Some("beach.jpg"))),
            ),
        ]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_an_inline_part_named_with_only_spaces_is_still_furniture() {
        let message = multipart(vec![
            basic("text", "html", None),
            basic("image", "gif", Some(disposition("inline", Some("   ")))),
        ]);
        assert!(!has_attachments(&message));
    }

    #[test]
    fn test_a_text_file_sent_as_an_attachment_counts() {
        // text/csv and text/plain arrive as attachments all the time, and the
        // part type alone cannot tell them from the body.
        let message = multipart(vec![
            text(None),
            text(Some(disposition("attachment", Some("data.csv")))),
        ]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_a_forwarded_message_counts() {
        // message/rfc822: the reader gets something they can open.
        let forwarded = BodyStructure::Message {
            common: BodyContentCommon {
                ty: content_type("message", "rfc822"),
                disposition: None,
                language: None,
                location: None,
            },
            other: single_part(),
            envelope: async_imap::imap_proto::Envelope {
                date: None,
                subject: None,
                from: None,
                sender: None,
                reply_to: None,
                to: None,
                cc: None,
                bcc: None,
                in_reply_to: None,
                message_id: None,
            },
            body: Box::new(text(None)),
            lines: 10,
            extension: None,
        };
        assert!(has_attachments(&multipart(vec![text(None), forwarded])));
    }

    #[test]
    fn test_an_attachment_nested_two_levels_down_is_still_found() {
        // multipart/mixed wrapping multipart/alternative plus the file is the
        // shape almost every mail client sends.
        let inner = multipart(vec![text(None), basic("text", "html", None)]);
        let message = multipart(vec![
            inner,
            basic(
                "application",
                "zip",
                Some(disposition("attachment", Some("photos.zip"))),
            ),
        ]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_the_disposition_is_read_whatever_its_case() {
        let message = multipart(vec![
            text(None),
            basic("application", "pdf", Some(disposition("ATTACHMENT", None))),
        ]);
        assert!(has_attachments(&message));
    }

    #[test]
    fn test_an_empty_multipart_has_no_attachment() {
        assert!(!has_attachments(&multipart(vec![])));
    }
}
