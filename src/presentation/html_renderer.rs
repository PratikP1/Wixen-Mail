//! HTML Rendering with Sanitization and Accessibility
//!
//! Renders HTML email content with security (XSS protection) and accessibility features.

use crate::common::types::MessageBody;
use ammonia::clean;
use std::sync::OnceLock;

// The patterns below are literals compiled once. An `expect` here can only fire
// if one of them is edited into something invalid, which every test in this
// module would catch on the first run.

const SAFE_URL_SCHEMES: [&str; 3] = ["http://", "https://", "mailto:"];

/// The language a message document asks to be read in.
///
/// This is the language the person reading is likely to be reading in, not the
/// language the message was written in. Nothing here knows the second one: no
/// message carries a Content-Language header through this application, and a
/// sender's own `<html lang="de">` is dropped on the way in because the
/// sanitiser keeps no `html` element. So the honest source is the machine.
///
/// Read once. It is a call into the operating system and there is one of these
/// per message opened.
fn document_language() -> Option<String> {
    static LANGUAGE: OnceLock<Option<String>> = OnceLock::new();
    LANGUAGE
        .get_or_init(crate::service::spellcheck::system_language)
        .clone()
}

/// The `lang` attribute for the opening tag, or nothing at all.
///
/// `lang="en"` used to be written into every document, which told a reader that
/// German mail on a German machine was English and had it pronounce a whole
/// message with English rules. That is worse than saying nothing, because a
/// reader given nothing carries on in the voice its owner chose.
///
/// So when the machine will not say, no attribute is written. That is a known
/// gap against WCAG 3.1.1 Language of Page and it is a deliberate one: an
/// absent attribute is "not known", and a wrong attribute is a claim a reader
/// acts on. Do not fill it back in with a default.
///
/// The tag is cut down to what a language tag can contain before it goes near
/// the document, because on Windows it comes from a locale name rather than
/// from anything written here.
fn language_attribute(machine: Option<&str>) -> String {
    let Some(tag) = machine else {
        return String::new();
    };
    let cleaned: String = tag
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(35)
        .collect();
    if cleaned.is_empty() {
        return String::new();
    }
    format!(" lang=\"{cleaned}\"")
}

fn html_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"<[^>]*>").expect("valid html tag regex"))
}

fn newline_compact_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\n\s*\n\s*\n+").expect("valid newline compact regex"))
}

pub fn image_alt_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(?is)<img[^>]*?alt=(?:"([^"]*)"|'([^']*)')[^>]*?>"#)
            .expect("valid image alt regex")
    })
}

pub fn link_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(?is)<a[^>]*?href=(?:"([^"]*)"|'([^']*)')[^>]*?>([\s\S]*?)</a>"#)
            .expect("valid link regex")
    })
}

pub fn img_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<img\b").expect("valid image tag regex"))
}

pub fn anchor_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<a\b").expect("valid anchor tag regex"))
}

pub fn script_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<\s*script\b").expect("valid script tag regex"))
}

/// HTML renderer with sanitization
pub struct HtmlRenderer {
    /// Whether to strip all HTML and return plain text
    plain_text_only: bool,
}

/// One message in a combined conversation document.
///
/// The body is untrusted and is sanitized on the way in, like every other
/// body. Being part of a thread does not make a stranger's HTML safer.
pub struct ThreadPart {
    pub sender: String,
    pub date: String,
    pub subject: String,
    /// The body, still carrying whether it is text or markup.
    ///
    /// Not a `String`. This used to be one, and the kind was worked out here by
    /// looking for angle brackets, which reads "write to <ada@example.com>" as
    /// a tag and hands it to the sanitiser, and the sanitiser deletes anything
    /// tag-shaped. [`HtmlRenderer::wrap_body`] had the same bug and had it
    /// taken out; this path kept it until every message started opening
    /// through here.
    pub body: MessageBody,
    pub depth: usize,
}

impl HtmlRenderer {
    /// Create a new HTML renderer
    pub fn new() -> Self {
        Self {
            plain_text_only: false,
        }
    }

    /// Create a renderer that returns plain text only
    pub fn plain_text_only() -> Self {
        Self {
            plain_text_only: true,
        }
    }

    /// Sanitize HTML content for safe display.
    ///
    /// Removes dangerous markup while preserving the formatting and structure
    /// a screen reader navigates by: headings, lists, tables, link text, and
    /// alt text all survive.
    ///
    /// The result is always safe to embed in a document. That matters in plain
    /// text mode, where `html_to_plain_text` strips tags and *then* decodes
    /// entities, so a body containing `&lt;script&gt;` comes back out as live
    /// markup. Correct as plain text, and an injection the moment it is placed
    /// in the WebView, so it is escaped here.
    /// Public again, and for a real reason: the message editor is a live DOM
    /// in a browser engine, and a reply quotes a stranger. #39 narrowed this
    /// when nothing outside the file used it; something does now.
    pub fn sanitize_html(&self, html: &str) -> String {
        if self.plain_text_only {
            return html_escape::encode_text(&self.html_to_plain_text(html)).to_string();
        }

        clean(html)
    }

    /// Convert HTML to accessible plain text
    ///
    /// This is useful for screen readers and text-only displays, and for
    /// copying a message into a task or a note, where a description full of
    /// markup is worse than no description.
    pub fn html_to_plain_text(&self, html: &str) -> String {
        // Basic HTML to text conversion
        let mut text = html.to_string();

        // Replace common tags with plain text equivalents
        text = text.replace("<br>", "\n");
        text = text.replace("<br/>", "\n");
        text = text.replace("<br />", "\n");
        text = text.replace("</p>", "\n\n");
        text = text.replace("</div>", "\n");
        text = text.replace("</h1>", "\n\n");
        text = text.replace("</h2>", "\n\n");
        text = text.replace("</h3>", "\n\n");
        text = text.replace("</h4>", "\n\n");
        text = text.replace("</h5>", "\n\n");
        text = text.replace("</h6>", "\n\n");
        text = text.replace("</li>", "\n");

        // Remove all remaining HTML tags
        text = html_tag_re().replace_all(&text, "").to_string();

        // Decode HTML entities
        text = html_escape::decode_html_entities(&text).to_string();

        // Clean up whitespace
        text = newline_compact_re().replace_all(&text, "\n\n").to_string();

        text.trim().to_string()
    }

    /// Extract alt text from images for accessibility
    pub fn extract_image_alt_texts(&self, html: &str) -> Vec<String> {
        let mut alt_texts = Vec::new();

        for cap in image_alt_re().captures_iter(html) {
            // Group 1 captures double-quoted alt text, group 2 captures single-quoted alt text.
            if let Some(alt) = cap.get(1).or_else(|| cap.get(2)) {
                alt_texts.push(alt.as_str().to_string());
            }
        }

        alt_texts
    }

    /// Extract link texts for accessibility
    pub fn extract_link_texts(&self, html: &str) -> Vec<LinkInfo> {
        let mut links = Vec::new();

        for cap in link_re().captures_iter(html) {
            if let (Some(href), Some(text)) = (cap.get(1).or_else(|| cap.get(2)), cap.get(3))
                && let Some(safe_url) = Self::safe_external_url(href.as_str())
            {
                links.push(LinkInfo {
                    url: safe_url,
                    text: self.html_to_plain_text(text.as_str()),
                });
            }
        }

        links
    }

    /// Wrap a message body in a full document for WebView display.
    ///
    /// Applies readable typography, dark-mode support, image containment,
    /// and blockquote styling. Text is escaped and wrapped in `<pre>` so its
    /// line breaks survive; markup is sanitised.
    ///
    /// The kind is taken rather than worked out. This used to test the string
    /// for angle brackets, which reads "if a < b and c > d" as markup and hands
    /// it to the sanitiser, and the sanitiser deletes anything tag-shaped: a
    /// bare address in a plain-text message disappeared out of the middle of
    /// the sentence with nothing said.
    pub fn wrap_body(&self, body: &MessageBody) -> String {
        let content = match body {
            MessageBody::Html(html) | MessageBody::Multipart { html, .. } => {
                self.sanitize_html(html)
            }
            MessageBody::Plain(text) => format!(
                "<pre style=\"white-space:pre-wrap;font-family:inherit\">{}</pre>",
                html_escape::encode_text(text)
            ),
        };
        self.wrap_prepared(&content, "Back to message list (Escape)")
    }

    /// Put the document shell around markup that is already safe.
    ///
    /// For content this application assembled itself out of already-sanitised
    /// pieces, where sanitising a second time would be wrong rather than
    /// merely wasteful: under the plain-text setting `sanitize_html` escapes
    /// its input, which would turn a conversation's own headings into visible
    /// angle brackets and take away the structure the surface exists for.
    ///
    /// Everything that comes from a message body has to have been through
    /// [`Self::sanitize_html`] before it reaches here.
    /// `way_out` names what the Back button does on this surface. The preview
    /// sits beside the message list and goes back to it; the reading window is
    /// a window and closes. One label for both told half of everybody something
    /// that was not true about the button they were on.
    ///
    /// The document's language is fetched here and is not a parameter, so a
    /// caller cannot leave it out. It was a parameter for one release and the
    /// reading window was handed `None` by hand; see
    /// [`Self::wrap_prepared_in_language`].
    fn wrap_prepared(&self, content: &str, way_out: &str) -> String {
        self.wrap_prepared_in_language(content, way_out, document_language().as_deref())
    }

    /// The same shell, built in a language the caller names.
    ///
    /// This exists for tests. A test that renders in a language it chose can
    /// read the tag that came out; while the shell fetched its own language,
    /// the only assertion available called the same two functions the document
    /// did, which moves both sides together and can never fail.
    ///
    /// Nothing outside tests calls it, and that is the point. When every call
    /// site had to hand the language in, one of the two was read by a test and
    /// the other was not, and the conversation document went out with no `lang`
    /// attribute at all while the whole suite stayed green. A document with no
    /// language is WCAG 3.1.1 failing on the one surface in this product built
    /// for reading a conversation by heading. Take the language from
    /// [`Self::wrap_prepared`] instead, which is not offered the question and
    /// so cannot answer it wrongly.
    fn wrap_prepared_in_language(
        &self,
        content: &str,
        way_out: &str,
        machine_language: Option<&str>,
    ) -> String {
        let language = language_attribute(machine_language);
        // Asked of the settings and of Windows together, so a machine set to
        // reduce animation gets an immediate scroll whatever this application
        // was told. The rule and the reason are in `application::scrolling`.
        let scrolling = crate::application::scrolling::how_to_scroll(
            crate::data::config::ConfigManager::load_stored()
                .map(|stored| stored.app_config().smooth_scrolling)
                .unwrap_or(false),
            crate::application::scrolling::system_motion(),
        )
        .css_scroll_behavior();
        format!(
            r#"<!DOCTYPE html>
<html{language}><head><meta charset="utf-8"><style>
html {{ scroll-behavior: {scrolling}; }}
@media (prefers-reduced-motion: reduce) {{
    html {{ scroll-behavior: auto; }}
}}
body {{
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    font-size: 14px; line-height: 1.6;
    color: #1a1a1a; background: #ffffff;
    margin: 12px; word-wrap: break-word;
}}
a {{ color: #0066cc; }}
img {{ max-width: 100%; height: auto; }}
pre, code {{ background: #f5f5f5; padding: 2px 4px; border-radius: 3px; }}
blockquote {{ border-left: 3px solid #ccc; margin-left: 0; padding-left: 12px; color: #555; }}
table {{ border-collapse: collapse; }} td, th {{ padding: 4px 8px; }}
@media (prefers-color-scheme: dark) {{
    body {{ background: #1e1e1e; color: #d4d4d4; }}
    a {{ color: #569cd6; }} pre, code {{ background: #2d2d2d; }}
    blockquote {{ border-color: #555; color: #999; }}
}}
.leave {{
    font-size: 13px; color: #1a1a1a; background: #f0f0f0;
    border: 1px solid #767676; border-radius: 3px;
    padding: 8px 12px; margin: 0 0 12px 0;
    min-height: 24px; min-width: 24px; cursor: pointer;
    font-family: inherit;
}}
.leave:focus {{ outline: 3px solid #0066cc; outline-offset: 2px; }}
@media (prefers-color-scheme: dark) {{
    .leave {{ background: #2d2d2d; color: #d4d4d4; border-color: #9a9a9a; }}
    .leave:focus {{ outline-color: #569cd6; }}
}}
</style></head><body>
<button type="button" class="leave" onclick="window.contextMenu.postMessage('{{&quot;kind&quot;:&quot;leave&quot;}}')">{}</button>
<main>{}</main></body></html>"#,
            html_escape::encode_text(way_out),
            content
        )
    }

    /// Render a whole conversation as one document.
    ///
    /// Every message is introduced by a heading, so `H` in a screen reader
    /// moves between them and the whole thread is navigable without tabbing
    /// through it. The heading carries the sender and, past level six, the
    /// real depth, because those are what someone navigates by; the subject is
    /// on the document rather than repeated on every reply.
    ///
    /// Levels cap at six and never skip. Skipping a heading level is a
    /// structure violation in its own right, and conversations go deeper than
    /// six, so the depth moves into the text rather than the markup.
    pub fn render_thread(&self, subject: &str, parts: &[ThreadPart]) -> String {
        let mut body = String::new();
        let title = html_escape::encode_text(if subject.trim().is_empty() {
            "No subject"
        } else {
            subject.trim()
        });
        body.push_str(&format!("<h1>{title}</h1>\n"));
        // Every message opens through here now, not only threads, so one part
        // is the common case rather than the odd one. Counting it out loud
        // gets the number agreement wrong and says the wrong thing about what
        // was opened, and it is the first line read aloud on the page.
        if parts.len() > 1 {
            body.push_str(&format!(
                "<p>{} messages in this conversation.</p>\n",
                parts.len()
            ));
        }

        for (position, part) in parts.iter().enumerate() {
            // Heading levels start at 2: the subject is the document's h1, and
            // a second h1 would give the page two titles.
            let level = (crate::application::threading::heading_level(part.depth) + 1).min(6);
            let role = if part.depth == 0 {
                "Message".to_string()
            } else if level < 6 {
                "Reply".to_string()
            } else {
                // The markup has run out of levels, so the depth is spoken.
                format!("Reply, level {}", part.depth + 1)
            };
            body.push_str(&format!(
                "<h{level}>{position}. {role} from {sender}</h{level}>
<p>{date}</p>
",
                level = level,
                position = position + 1,
                role = html_escape::encode_text(&role),
                sender = html_escape::encode_text(&part.sender),
                date = html_escape::encode_text(&part.date),
            ));
            // The kind is taken, not worked out, for the reason on `ThreadPart`.
            let content = match &part.body {
                MessageBody::Html(html) | MessageBody::Multipart { html, .. } => {
                    self.sanitize_html(html)
                }
                MessageBody::Plain(text) => format!(
                    "<pre style=\"white-space:pre-wrap;font-family:inherit\">{}</pre>",
                    html_escape::encode_text(text)
                ),
            };
            body.push_str(&content);
            body.push('\n');
        }

        // `wrap_prepared`, not `wrap_body`. Every body above has
        // already been through the sanitiser, and sanitising the assembled
        // document again would escape the headings this whole surface exists
        // to produce whenever the plain-text setting is on.
        self.wrap_prepared(&body, "Close this window (Escape)")
    }

    /// The only URLs this application will hand to the operating system.
    ///
    /// Returns the URL when it is safe to open externally, `None` otherwise.
    /// Everything a message body offers has to pass through here first.
    ///
    /// Opening a URL calls the platform's shell handler, which on Windows will
    /// launch executables, reach UNC paths across the network, and invoke any
    /// protocol handler that happens to be registered on that machine. None of
    /// that is a decision the sender of an email gets to make, so this allows a
    /// known set of schemes and refuses everything else rather than trying to
    /// enumerate what is dangerous.
    pub fn safe_external_url(url: &str) -> Option<String> {
        let trimmed = url.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.chars().any(|c| c.is_control()) {
            return None;
        }
        if SAFE_URL_SCHEMES
            .iter()
            .any(|scheme| lower.starts_with(scheme))
        {
            if lower.starts_with("http://") || lower.starts_with("https://") {
                let remainder = &trimmed[trimmed.find("://")? + 3..];
                // Intentionally reject userinfo URLs to reduce phishing obfuscation risks.
                if remainder.is_empty() || remainder.starts_with('/') || remainder.contains('@') {
                    return None;
                }
            }
            if lower.starts_with("mailto:") && !trimmed[7..].contains('@') {
                return None;
            }
            return Some(trimmed.to_string());
        }
        None
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Link information for accessibility
#[derive(Debug, Clone)]
pub struct LinkInfo {
    /// URL
    pub url: String,
    /// Link text
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_preview_document_starts_with_a_focusable_way_out() {
        // Two independent routes, because one is not enough here. The Escape
        // key depends on a keydown listener; this depends only on the document
        // having rendered. It is the first focusable thing on the page, so Tab
        // reaches it before anything the sender wrote.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_body(&MessageBody::Plain("Hello".into()));
        let button = html.find("<button").expect("no way out control");
        let content = html.find("<main").expect("no main");
        assert!(button < content, "the way out is not reachable first");
        assert!(html.contains("Back to message list"));
    }

    #[test]
    fn test_the_way_out_does_not_depend_on_the_injected_script() {
        // If it were wired up by the user script, both routes would fail
        // together, which is not two routes.
        let renderer = HtmlRenderer::new();
        assert!(
            renderer
                .wrap_body(&MessageBody::Plain("Hello".into()))
                .contains("onclick=")
        );
    }

    #[test]
    fn test_a_preview_document_has_a_main_landmark() {
        // A screen reader user arriving in the preview needs somewhere to be.
        // A bare run of text in a body with no landmark gives nothing to jump
        // to and no way to tell the message from the chrome around it.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_body(&MessageBody::Plain("Hello".into()));
        assert!(html.contains("<main"), "no main landmark: {}", html);
        assert!(html.contains("</main>"));
    }

    #[test]
    fn test_a_preview_document_declares_the_language_worked_out_for_it() {
        // Whether there is a lang attribute at all depends on whether this
        // machine will say what language it is set to, so asserting one is
        // present would be a test that reports on the build agent's locale
        // while reading like a test of the code. What is this code's to get
        // right is that the answer worked out is the answer that lands in the
        // tag.
        //
        // The expected side reads the machine through `system_language`, not
        // through `document_language`. It used to call `document_language`,
        // which is the same function the document calls, so a wrong answer
        // moved both sides of the assertion together and the test stayed
        // green through it.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_body(&MessageBody::Plain("Hello".into()));
        let expected = format!(
            "<html{}>",
            language_attribute(crate::service::spellcheck::system_language().as_deref())
        );

        assert!(
            html.contains(&expected),
            "{expected} is not in the document"
        );
    }

    #[test]
    fn test_a_conversation_document_declares_the_language_worked_out_for_it() {
        // The same claim as the preview test above, made about the other
        // surface, because the two documents are built by different functions
        // and only one of them was ever read for its opening tag. This is the
        // headings surface a screen reader user moves through with H, so it is
        // the last document in the product that should be handed to a reader
        // with no language on it (3.1.1).
        //
        // Read the same way as the preview test: the expected side asks the
        // machine through `system_language`, so nothing here reports on the
        // build agent's locale, and on a machine that will not name its
        // language both sides are empty and this test cannot fail. What it
        // pins is that whatever was worked out is what lands in the tag.
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread("Quarterly report", &[part("Ada", 0, "Body")]);
        let expected = format!(
            "<html{}>",
            language_attribute(crate::service::spellcheck::system_language().as_deref())
        );

        assert!(
            html.contains(&expected),
            "{expected} is not in the document"
        );
    }

    #[test]
    fn test_the_language_lookup_answers_the_machines_own_answer_and_not_a_substitute() {
        // No document here, on purpose. This reads the lookup on its own:
        // `document_language` memoises `system_language` and is allowed to do
        // nothing else, not answer a default, not answer a blank, not answer a
        // language nobody asked for. A wrong answer here reaches a live reader
        // as a claim about a whole message, which is worse than saying nothing
        // (3.1.1). What a rendered document does with the answer is the two
        // tests above, one per surface.
        //
        // What this kills, and where. A substituted blank or a made-up tag
        // dies on any machine, because `system_language` answers `None` rather
        // than `Some("")` when Windows writes nothing. A substituted `None`
        // dies only on a machine that will name its language, which is Windows
        // CI and a normal desktop. On a machine that will not, that one
        // survives and this test still passes.
        assert_eq!(
            document_language(),
            crate::service::spellcheck::system_language()
        );
    }

    #[test]
    fn test_a_document_carries_the_language_it_was_handed_into_its_opening_tag() {
        // A real document, rendered twice, read for the tag that came out.
        // Nothing here calls the machine, so it says the same thing on every
        // machine and there is no shared function on both sides of it.
        let renderer = HtmlRenderer::new();

        let german = renderer.wrap_prepared_in_language("<p>Hallo</p>", "Escape", Some("de-DE"));
        let unknown = renderer.wrap_prepared_in_language("<p>Hello</p>", "Escape", None);

        assert!(german.contains("<html lang=\"de-DE\">"), "{german}");
        assert!(unknown.contains("<html>"), "{unknown}");
        assert!(!unknown.contains("lang="), "{unknown}");
    }

    #[test]
    fn test_a_document_asks_to_be_read_in_the_machines_language() {
        // "Asks to be read in", not "is read in". Whether a reader acts on this
        // depends on its automatic language switching being on and on a voice
        // for that language being installed, and neither is this project's to
        // decide (WCAG 3.1.1).
        assert_eq!(language_attribute(Some("de-DE")), " lang=\"de-DE\"");
    }

    #[test]
    fn test_a_machine_that_will_not_say_its_language_is_not_answered_with_english() {
        // No attribute means "not known", and a reader carries on in the voice
        // its owner chose. `lang="en"` is a different thing: a claim that the
        // document is English, which a reader acts on by pronouncing every
        // other language as though it were.
        assert_eq!(language_attribute(None), "");
    }

    #[test]
    fn test_a_language_tag_cannot_carry_an_attribute_into_the_tag() {
        // The tag reaches a live browser engine, and on Windows it comes from
        // a locale name rather than anything this code wrote.
        let attribute = language_attribute(Some("en\" onload=\"alert(1)"));

        assert_eq!(
            attribute.matches('"').count(),
            2,
            "the tag broke out of its attribute: {attribute}"
        );
        assert!(!attribute.contains(' ') || attribute.starts_with(" lang=\""));
        assert!(!attribute.trim_start().contains(' '), "{attribute}");
    }

    #[test]
    fn test_the_sanitiser_keeps_a_senders_own_language_marking() {
        // A pin, green from the first run. A sender who marked a quotation as
        // French keeps that marking through the sanitiser, which is the whole
        // of WCAG 3.1.2 Language of Parts here, and it works by inheritance
        // from the sanitiser's defaults rather than by any decision made here.
        // The next pass that narrows what attributes survive would take it away
        // without anybody noticing.
        let renderer = HtmlRenderer::new();

        let cleaned = renderer.sanitize_html("<p lang=\"fr\">Bonjour</p>");

        assert!(cleaned.contains("lang=\"fr\""), "{cleaned}");
    }

    #[test]
    fn test_a_preview_document_says_how_to_leave_it() {
        // The preview is a WebView, which swallows the keystrokes that would
        // normally move focus out. Someone who cannot see the window has no
        // way to discover the way back unless the document says it.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_body(&MessageBody::Plain("Hello".into()));
        assert!(html.contains("Escape"), "no way out is stated: {}", html);
    }

    fn part(sender: &str, depth: usize, body: &str) -> ThreadPart {
        ThreadPart {
            sender: sender.to_string(),
            date: "2026-07-26".to_string(),
            subject: "Quarterly report".to_string(),
            body: MessageBody::Html(body.to_string()),
            depth,
        }
    }

    /// A part whose body kind is the point of the test.
    fn part_of(sender: &str, depth: usize, body: MessageBody) -> ThreadPart {
        ThreadPart {
            sender: sender.to_string(),
            date: "2026-07-26".to_string(),
            subject: "Quarterly report".to_string(),
            body,
            depth,
        }
    }

    #[test]
    fn test_a_plain_text_body_keeps_text_that_only_looks_like_markup() {
        // The bug `wrap_body` already had taken out of it, still here: working
        // the kind out from angle brackets reads "write to <ada@example.com>"
        // as a tag and hands it to the sanitiser, which deletes anything
        // tag-shaped. The address disappears out of the middle of the sentence
        // and nothing says so.
        let renderer = HtmlRenderer::new();
        let plain = MessageBody::Plain("write to <ada@example.com> today".to_string());

        let html = renderer.render_thread("Subject", &[part_of("Ada", 0, plain)]);

        assert!(html.contains("ada@example.com"), "{html}");
    }

    #[test]
    fn test_a_thread_document_has_one_heading_per_message() {
        // H moves between messages in a screen reader, which is the whole
        // point of rendering a conversation as one document.
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread(
            "Quarterly report",
            &[
                part("Ada", 0, "The numbers are attached."),
                part("Grace", 1, "Thanks."),
                part("Alan", 2, "Agreed."),
            ],
        );
        assert_eq!(html.matches("<h2").count(), 1);
        assert_eq!(html.matches("<h3").count(), 1);
        assert_eq!(html.matches("<h4").count(), 1);
        assert!(html.contains("from Ada"));
        assert!(html.contains("3 messages in this conversation"));
    }

    #[test]
    fn test_the_subject_is_the_only_h1() {
        // Two h1 elements give a document two titles, and a screen reader user
        // navigating by heading has no way to tell which is the real one.
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread("Quarterly report", &[part("Ada", 0, "Body")]);
        assert_eq!(html.matches("<h1").count(), 1);
    }

    #[test]
    fn test_deep_replies_stop_at_h6_and_say_their_real_depth() {
        // Levels cap rather than skip, and the depth moves into the text so
        // nothing is lost.
        let renderer = HtmlRenderer::new();
        let deep: Vec<ThreadPart> = (0..10).map(|d| part("Ada", d, "Body")).collect();
        let html = renderer.render_thread("Long thread", &deep);
        assert!(!html.contains("<h7"));
        assert!(html.contains("Reply, level 10"));
    }

    #[test]
    fn test_a_reply_the_heading_level_can_carry_is_not_also_given_a_number_out_loud() {
        // The depth is spoken only once the markup has run out of levels. A
        // reply at level three is already announced as a level three heading,
        // so saying the number again in the words makes every heading in an
        // ordinary thread carry the same fact twice.
        let renderer = HtmlRenderer::new();

        let html = renderer.render_thread(
            "Quarterly report",
            &[part("Ada", 0, "One"), part("Grace", 1, "Two")],
        );

        assert!(html.contains("Reply from Grace"), "{html}");
        assert!(!html.contains("Reply, level 2"), "{html}");
    }

    #[test]
    fn test_a_thread_body_is_still_sanitized() {
        // Being part of a conversation does not make a stranger's HTML safer.
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread(
            "Quarterly report",
            &[part("Ada", 0, "<script>alert(1)</script><p>Hello</p>")],
        );
        assert!(!html.contains("<script"));
        assert!(html.contains("Hello"));
    }

    #[test]
    fn test_a_plain_text_body_in_a_thread_is_escaped_not_rendered() {
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread("Subject", &[part("Ada", 0, "5 < 6 & 7 > 6")]);
        assert!(html.contains("5 &lt; 6 &amp; 7 &gt; 6"));
    }

    #[test]
    fn test_the_reading_window_says_its_button_closes_the_window() {
        // The same shell serves two surfaces that go to different places. The
        // preview sits beside the message list and goes back to it; this is a
        // window and closes. Telling somebody the button does the other thing
        // is worse than not labelling it, because they will not check.
        let html = HtmlRenderer::new().render_thread("Subject", &[part("Ada", 0, "Body")]);

        assert!(html.contains("Close this window"), "{html}");
        assert!(!html.contains("Back to message list"), "{html}");
    }

    #[test]
    fn test_one_message_is_not_announced_as_a_conversation() {
        // Every message opens through this composition now, not only threads,
        // so most of the time there is exactly one part. Greeting somebody
        // with "1 messages in this conversation" gets both the number
        // agreement and the fact wrong, and it is the first thing read aloud.
        let renderer = HtmlRenderer::new();

        let html = renderer.render_thread("Report", &[part("Ada", 0, "Body")]);

        assert!(!html.contains("1 messages"), "{html}");
        assert!(!html.contains("conversation"), "{html}");
    }

    #[test]
    fn test_a_real_conversation_still_says_how_many_messages_are_in_it() {
        let renderer = HtmlRenderer::new();

        let html =
            renderer.render_thread("Report", &[part("Ada", 0, "One"), part("Grace", 1, "Two")]);

        assert!(html.contains("2 messages in this conversation"), "{html}");
    }

    #[test]
    fn test_a_thread_with_no_subject_says_so() {
        let renderer = HtmlRenderer::new();
        let html = renderer.render_thread("   ", &[part("Ada", 0, "Body")]);
        assert!(html.contains("No subject"));
    }

    #[test]
    fn test_a_sender_name_cannot_inject_markup_through_the_heading() {
        // The sender field is attacker controlled on every message.
        let renderer = HtmlRenderer::new();
        let html =
            renderer.render_thread("Subject", &[part("<script>alert(1)</script>", 0, "Body")]);
        assert!(!html.contains("<script"));
    }

    #[test]
    fn test_html_renderer_creation() {
        let renderer = HtmlRenderer::new();
        assert!(!renderer.plain_text_only);
    }

    #[test]
    fn test_sanitize_html_removes_javascript() {
        let renderer = HtmlRenderer::new();
        let dangerous_html = r#"<p onclick="alert('xss')">Hello</p><script>alert('xss')</script>"#;
        let safe_html = renderer.sanitize_html(dangerous_html);

        assert!(!safe_html.contains("onclick"));
        assert!(!safe_html.contains("<script"));
        assert!(safe_html.contains("Hello"));
    }

    #[test]
    fn test_html_to_plain_text() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Hello <strong>World</strong>!</p><p>Second paragraph.</p>";
        let plain = renderer.html_to_plain_text(html);

        assert!(plain.contains("Hello World!"));
        assert!(plain.contains("Second paragraph."));
        assert!(!plain.contains("<p>"));
    }

    #[test]
    fn test_extract_image_alt_texts() {
        let renderer = HtmlRenderer::new();
        let html =
            r#"<img src="test.jpg" alt="Test Image"><img src="test2.jpg" alt="Another Image">"#;
        let alt_texts = renderer.extract_image_alt_texts(html);

        assert_eq!(alt_texts.len(), 2);
        assert_eq!(alt_texts[0], "Test Image");
        assert_eq!(alt_texts[1], "Another Image");
    }

    #[test]
    fn test_extract_link_texts() {
        let renderer = HtmlRenderer::new();
        let html =
            r#"<a href="https://example.com">Example Link</a><a href="https://test.com">Test</a>"#;
        let links = renderer.extract_link_texts(html);

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[0].text, "Example Link");
    }

    #[test]
    fn test_wrap_body_keeps_markup() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Hello <strong>World</strong></p>";
        let wrapped = renderer.wrap_body(&MessageBody::Html(html.into()));
        assert!(wrapped.contains("<!DOCTYPE html>"));
        assert!(wrapped.contains("Segoe UI"));
        assert!(wrapped.contains("<p>Hello <strong>World</strong></p>"));
    }

    #[test]
    fn test_wrap_body_keeps_text_as_text() {
        let renderer = HtmlRenderer::new();
        let text = "Just plain text, no HTML.";
        let wrapped = renderer.wrap_body(&MessageBody::Plain(text.into()));
        assert!(wrapped.contains("<pre"));
        assert!(wrapped.contains("Just plain text, no HTML."));
    }

    #[test]
    fn test_wrap_body_strips_scripts() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Safe</p><script>alert('xss')</script>";
        let wrapped = renderer.wrap_body(&MessageBody::Html(html.into()));
        assert!(wrapped.contains("Safe"));
        assert!(!wrapped.contains("<script"));
    }

    #[test]
    fn test_extract_links_filters_unsafe_schemes() {
        let renderer = HtmlRenderer::new();
        let html =
            r#"<a href="javascript:alert(1)">Bad</a><a href="mailto:test@example.com">Mail</a>"#;
        let links = renderer.extract_link_texts(html);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "mailto:test@example.com");
    }

    // ── Hostile input ───────────────────────────────────────────────────
    //
    // Message bodies arrive from strangers. Everything below is an assertion
    // about what a sender must not be able to make this client do.

    /// Snippets that have all been used against real mail clients.
    const HOSTILE: &[&str] = &[
        "<script>alert(1)</script>",
        "&lt;script&gt;alert(1)&lt;/script&gt;",
        "&amp;lt;script&amp;gt;alert(1)&amp;lt;/script&amp;gt;",
        "<img src=x onerror=alert(1)>",
        "<a href=\"javascript:alert(1)\">click</a>",
        "<a href=\"JaVaScRiPt:alert(1)\">click</a>",
        "<a href=\"data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==\">x</a>",
        "<iframe src=\"https://evil.example\"></iframe>",
        "<svg/onload=alert(1)>",
        "<body onload=alert(1)>",
        "<object data=\"evil.swf\"></object>",
        "<embed src=\"evil.swf\">",
        "<form action=\"https://evil.example\"><input name=p></form>",
        "<meta http-equiv=refresh content=\"0;url=https://evil.example\">",
        "<base href=\"https://evil.example/\">",
        "<link rel=stylesheet href=\"https://evil.example/x.css\">",
    ];

    /// The invariants that must hold for anything placed in the WebView.
    fn assert_no_live_markup(rendered: &str, label: &str) {
        let lower = rendered.to_ascii_lowercase();
        // The document template itself contains <style> and <meta>, so only
        // inspect the part of the document the sender controls.
        let body = lower.split("<body>").nth(1).unwrap_or(&lower).to_string();
        for forbidden in [
            "<script",
            "javascript:",
            "onerror=",
            "onload=",
            "<iframe",
            "<object",
            "<embed",
            "<meta",
            "<base",
            "<link",
            "<form",
        ] {
            assert!(
                !body.contains(forbidden),
                "{}: sender controlled body contains {:?}\n{}",
                label,
                forbidden,
                body
            );
        }
    }

    #[test]
    fn test_hostile_bodies_never_reach_the_webview_as_markup() {
        let renderer = HtmlRenderer::new();
        for html in HOSTILE {
            assert_no_live_markup(
                &renderer.wrap_body(&MessageBody::Html(html.to_string())),
                html,
            );
        }
    }

    #[test]
    fn test_hostile_bodies_are_inert_in_plain_text_mode_too() {
        // Plain text mode strips tags and then decodes entities, which turns
        // "&lt;script&gt;" back into "<script>". That is correct as plain
        // text and unsafe the moment it is placed in an HTML document.
        let renderer = HtmlRenderer::plain_text_only();
        for html in HOSTILE {
            assert_no_live_markup(
                &renderer.wrap_body(&MessageBody::Html(html.to_string())),
                html,
            );
        }
    }

    #[test]
    fn test_sanitize_output_is_safe_to_embed() {
        let renderer = HtmlRenderer::plain_text_only();
        let sanitized = renderer.sanitize_html("&lt;script&gt;alert(1)&lt;/script&gt;");
        assert!(
            !sanitized.to_ascii_lowercase().contains("<script"),
            "sanitize_html must return something safe to embed, got: {}",
            sanitized
        );
    }

    // ── Accessibility survives sanitizing ───────────────────────────────
    //
    // Stripping hostile markup must not also strip the structure a screen
    // reader navigates by. Security that costs the user their headings is
    // not a good trade.

    #[test]
    fn test_heading_structure_survives_sanitizing() {
        let renderer = HtmlRenderer::new();
        let html = "<h1>Quarterly report</h1><h2>Revenue</h2><p>Up.</p>";
        let safe = renderer.sanitize_html(html);
        assert!(safe.contains("<h1"), "h1 lost: {}", safe);
        assert!(safe.contains("<h2"), "h2 lost: {}", safe);
    }

    #[test]
    fn test_link_text_survives_sanitizing() {
        let renderer = HtmlRenderer::new();
        let safe = renderer.sanitize_html("<a href=\"https://example.com\">Quarterly report</a>");
        assert!(
            safe.contains("Quarterly report"),
            "link text lost: {}",
            safe
        );
        assert!(safe.contains("https://example.com"), "href lost: {}", safe);
    }

    #[test]
    fn test_list_structure_survives_sanitizing() {
        let renderer = HtmlRenderer::new();
        let safe = renderer.sanitize_html("<ul><li>one</li><li>two</li></ul>");
        assert!(safe.contains("<ul"), "list lost: {}", safe);
        assert!(safe.contains("<li"), "list items lost: {}", safe);
    }

    #[test]
    fn test_table_structure_survives_sanitizing() {
        let renderer = HtmlRenderer::new();
        let safe =
            renderer.sanitize_html("<table><tr><th>Month</th></tr><tr><td>May</td></tr></table>");
        assert!(safe.contains("<table"), "table lost: {}", safe);
        assert!(safe.contains("<th"), "header cell lost: {}", safe);
    }

    #[test]
    fn test_image_alt_text_survives_sanitizing() {
        let renderer = HtmlRenderer::new();
        let safe =
            renderer.sanitize_html("<img src=\"https://example.com/x.png\" alt=\"Revenue chart\">");
        assert!(safe.contains("Revenue chart"), "alt text lost: {}", safe);
    }

    // ── Fuzzing ─────────────────────────────────────────────────────────

    /// Deterministic generator, so any failure is reproducible from the seed
    /// printed in the assertion message.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }

        fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
            &items[(self.next() % items.len() as u64) as usize]
        }
    }

    /// Splice hostile snippets together with the characters that break naive
    /// parsers: unbalanced brackets, quotes, nulls, and direction overrides.
    fn fuzz_body(seed: u64) -> String {
        let mut rng = Lcg(seed);
        let noise = [
            "<",
            ">",
            "\"",
            "'",
            "&",
            "\0",
            "\n",
            "\t",
            "/",
            "=",
            " ",
            "\\",
            "&#0;",
            "&#x3c;",
            "&NewLine;",
            "\u{feff}",
            "\u{202e}",
            "%3cscript%3e",
        ];
        let mut body = String::new();
        for _ in 0..(rng.next() % 12 + 1) {
            if rng.next().is_multiple_of(2) {
                body.push_str(rng.pick(HOSTILE));
            }
            for _ in 0..(rng.next() % 6) {
                body.push_str(rng.pick(&noise));
            }
        }
        body
    }

    #[test]
    fn test_fuzz_webview_body_stays_inert() {
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            let rendered = HtmlRenderer::new().wrap_body(&MessageBody::Html(body.clone()));
            assert_no_live_markup(&rendered, &format!("seed {}", seed));
        }
    }

    #[test]
    fn test_fuzz_plain_text_mode_stays_inert() {
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            let rendered =
                HtmlRenderer::plain_text_only().wrap_body(&MessageBody::Html(body.clone()));
            assert_no_live_markup(&rendered, &format!("seed {}", seed));
        }
    }

    #[test]
    fn test_fuzz_renderer_never_panics() {
        // Every entry point a message body can reach.
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            let renderer = HtmlRenderer::new();
            let _ = renderer.sanitize_html(&body);
            let _ = renderer.html_to_plain_text(&body);
            let _ = renderer.extract_image_alt_texts(&body);
            let _ = renderer.extract_link_texts(&body);
            let _ = renderer.wrap_body(&MessageBody::Html(body.clone()));
            let _ = renderer.wrap_body(&MessageBody::Plain(body.clone()));
        }
    }

    #[test]
    fn test_fuzz_extracted_links_are_always_safe_schemes() {
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            for link in HtmlRenderer::new().extract_link_texts(&body) {
                let scheme = link.url.to_ascii_lowercase();
                assert!(
                    !scheme.starts_with("javascript:")
                        && !scheme.starts_with("data:")
                        && !scheme.starts_with("vbscript:"),
                    "seed {} produced a navigable {:?}",
                    seed,
                    link.url
                );
            }
        }
    }

    // ── External URL policy ─────────────────────────────────────────────
    //
    // Clicking a link in a message hands the URL to the operating system's
    // shell handler. On Windows that will launch executables, open UNC paths
    // across the network, and invoke any registered protocol handler. The
    // sender of an email must not be able to reach any of that.

    #[test]
    fn test_safe_external_url_allows_ordinary_web_links() {
        for url in [
            "https://example.com/report",
            "http://example.com",
            "mailto:ada@example.com",
            // Short ones too. A link shortener produces addresses of about a
            // dozen characters, and they are the ones people are handed most.
            "http://t.co",
            "http://a.io",
            "https://t.co/x",
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_some(),
                "{} should be openable",
                url
            );
        }
    }

    #[test]
    fn test_an_address_with_no_host_is_not_something_this_will_open() {
        // What gets handed to the platform's shell handler has to name
        // somewhere to go. A scheme with nothing after it, or with only
        // slashes, is not an address, and letting one through means the shell
        // decides what it meant.
        for url in ["https://", "http://", "https:///evil.example/x", "http:///"] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_none(),
                "{} names no host and must not reach the shell",
                url
            );
        }
    }

    #[test]
    fn test_safe_external_url_refuses_local_files() {
        for url in [
            "file:///C:/Windows/System32/calc.exe",
            "file://localhost/etc/passwd",
            "FILE:///C:/Windows/System32/calc.exe",
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_none(),
                "{} must never reach the shell",
                url
            );
        }
    }

    #[test]
    fn test_safe_external_url_refuses_unc_and_bare_paths() {
        for url in [
            r"\\evil.example\share\payload.exe",
            r"C:\Windows\System32\calc.exe",
            "//evil.example/share/payload.exe",
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_none(),
                "{} must never reach the shell",
                url
            );
        }
    }

    #[test]
    fn test_safe_external_url_refuses_script_and_data_schemes() {
        for url in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_none(),
                "{} must never reach the shell",
                url
            );
        }
    }

    #[test]
    fn test_safe_external_url_refuses_other_registered_handlers() {
        // Anything the platform happens to have registered is an attack
        // surface we did not choose. Allow a known set, refuse the rest.
        for url in [
            "ms-msdt:/id PCWDiagnostic",
            "search-ms:query=x",
            "shell:startup",
            "vscode://file/C:/x",
            "smb://evil.example/share",
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_none(),
                "{} must never reach the shell",
                url
            );
        }
    }

    #[test]
    fn test_safe_external_url_refuses_control_characters() {
        // A newline can split a command line once the value is passed on.
        assert!(HtmlRenderer::safe_external_url("https://example.com\n/x").is_none());
        assert!(HtmlRenderer::safe_external_url("https://example.com\u{0}").is_none());
    }

    #[test]
    fn test_safe_external_url_refuses_userinfo_phishing() {
        // "https://apple.com@evil.example" reads as Apple and goes to evil.
        assert!(HtmlRenderer::safe_external_url("https://apple.com@evil.example").is_none());
    }

    #[test]
    fn test_fuzz_no_fuzzed_body_yields_an_openable_dangerous_url() {
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            for link in HtmlRenderer::new().extract_link_texts(&body) {
                if let Some(openable) = HtmlRenderer::safe_external_url(&link.url) {
                    let lower = openable.to_ascii_lowercase();
                    assert!(
                        lower.starts_with("http://")
                            || lower.starts_with("https://")
                            || lower.starts_with("mailto:"),
                        "seed {} produced openable {:?}",
                        seed,
                        openable
                    );
                }
            }
        }
    }
}
