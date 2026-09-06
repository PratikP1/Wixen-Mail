//! HTML Rendering with Sanitization and Accessibility
//!
//! Renders HTML email content with security (XSS protection) and accessibility features.

use crate::common::types::MessageBody;
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

/// A whole `<img>` tag, so it can be replaced rather than picked apart.
pub fn img_tag_whole_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<img\b[^>]*>").expect("valid whole image regex"))
}

/// One attribute out of a tag ammonia has written.
///
/// Only ever run over cleaned markup, where every value is in double quotes,
/// which is why this can be a pattern rather than a parser.
fn one_attribute(tag: &str, name: &str) -> Option<String> {
    let looking_for = format!(r#"(?i)\b{name}="([^"]*)""#);
    regex::Regex::new(&looking_for)
        .ok()?
        .captures(tag)?
        .get(1)
        .map(|found| found.as_str().to_string())
}

/// An `alt` attribute that is present and empty, which is the decorative mark.
///
/// Only ever run over a tag ammonia has written, where every value is in
/// double quotes and an internal quote is an entity, so these six characters
/// cannot occur anywhere but the attribute itself.
fn empty_alt_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r#"(?i)\balt="""#).expect("valid empty alt regex"))
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

/// The cleaner, told to admit the pictures this application carried in.
///
/// `ammonia::clean` refuses `data:` addresses, which is right for a sender's
/// own: one can be an SVG, and an SVG is a document that can run a script. The
/// pictures written into a body by `pictures::carry_the_pictures` are not the
/// sender's, they are raster formats only, and by then every `data:` address
/// the sender wrote has already been taken out. So the filter admits exactly
/// the shape this application writes and nothing else beginning `data:`.
///
/// Built once. It compiles a set of allowed tags and attributes, and doing that
/// per message showed up when a folder of a thousand was opened.
fn cleaner() -> &'static ammonia::Builder<'static> {
    static CLEANER: OnceLock<ammonia::Builder<'static>> = OnceLock::new();
    CLEANER.get_or_init(|| {
        let mut builder = ammonia::Builder::default();
        let mut schemes = builder.clone_url_schemes();
        schemes.insert("data");
        schemes.insert("cid");
        builder.url_schemes(schemes);
        builder.attribute_filter(|element, attribute, value| {
            let looks_like_an_address = matches!(attribute, "src" | "href" | "background");
            if !looks_like_an_address {
                return Some(value.into());
            }
            let lowered = value.to_ascii_lowercase();
            // Only ours, and only on a picture. A `data:` address anywhere else
            // is a document the browser would run.
            if lowered.starts_with("data:") {
                return (element == "img"
                    && crate::application::pictures::is_a_picture_we_carried(value))
                .then(|| value.into());
            }
            // A `cid:` that reached here is one nothing could carry, so it
            // names a part that is missing or too large. Dropping the address
            // leaves the picture to be reported as held back, with whatever
            // the sender called it.
            if lowered.starts_with("cid:") {
                return None;
            }
            Some(value.into())
        });
        builder
    })
}

/// What the stored settings say about fetching pictures.
///
/// Blocked when the settings cannot be read at all, which is the safe way to be
/// wrong: a message shown without its remote pictures is a message somebody can
/// still read, and one shown with them is a message that has already reported
/// them.
/// Both picture answers the stored settings hold, read in one go.
///
/// One read rather than two. [`HtmlRenderer::new`] is called from
/// `editor_document::body_from_editor`, which runs every time the editor is
/// read, and a second `load_stored` there doubles a file read on the path a
/// message is autosaved and sent through.
///
/// Both fall back the safe way when the settings cannot be read at all, and
/// "safe" is the opposite direction for the two. A message shown without its
/// remote pictures is still readable and one shown with them has already
/// reported the reader, so pictures stay blocked. A reader told a picture was
/// there can ignore the line and one not told cannot ask, so announcing stays
/// on.
fn what_the_settings_say() -> (
    crate::application::pictures::Fetching,
    crate::application::pictures::Announcing,
) {
    use crate::application::pictures::{Announcing, Fetching};

    let stored = crate::data::config::ConfigManager::load_stored();
    let (blocked, announce) = stored
        .map(|stored| {
            let app = stored.app_config();
            (
                app.hold_back_remote_pictures,
                app.announce_decorative_pictures,
            )
        })
        .unwrap_or((true, true));
    (
        Fetching::from_setting(blocked),
        Announcing::from_setting(announce),
    )
}

/// HTML renderer with sanitization
pub struct HtmlRenderer {
    /// Whether to strip all HTML and return plain text
    plain_text_only: bool,
    /// Whether a picture the message only points at may be fetched.
    ///
    /// Fetching one tells the server it came from that this message was opened,
    /// by this computer, at this moment, which is the whole of how mail
    /// tracking works. Held back unless somebody has said otherwise.
    fetching: crate::application::pictures::Fetching,
    /// Whether a picture the sender marked decorative is said to be there.
    ///
    /// The reader's answer rather than the sender's. Read here rather than at
    /// the seam that uses it, so the settings are read once per renderer.
    announcing: crate::application::pictures::Announcing,
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
        let (fetching, announcing) = what_the_settings_say();
        Self {
            plain_text_only: false,
            fetching,
            announcing,
        }
    }

    /// Create a renderer that returns plain text only
    pub fn plain_text_only() -> Self {
        let (fetching, announcing) = what_the_settings_say();
        Self {
            plain_text_only: true,
            // Nothing is fetched in plain text either way, since there is no
            // browser to fetch with, but the field decides what the words say
            // where a picture would have been. `announcing` reaches nothing at
            // all here: `html_to_plain_text` strips every tag, so no picture
            // says anything in that path, described or decorative.
            fetching,
            announcing,
        }
    }

    /// A renderer told outright whether pictures may be fetched.
    ///
    /// For tests, which must not depend on whatever this machine's settings
    /// happen to say. Announcing takes the answer this ships with, so a test
    /// about fetching does not have to know this setting exists.
    pub fn with_fetching(fetching: crate::application::pictures::Fetching) -> Self {
        Self {
            plain_text_only: false,
            fetching,
            announcing: crate::application::pictures::Announcing::OutLoud,
        }
    }

    /// A renderer told outright about both picture answers.
    ///
    /// For tests of the announcing seam, which have to drive both directions.
    pub fn with_fetching_and_announcing(
        fetching: crate::application::pictures::Fetching,
        announcing: crate::application::pictures::Announcing,
    ) -> Self {
        Self {
            plain_text_only: false,
            fetching,
            announcing,
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

        // Cleaning only. Holding pictures back is about reading a message,
        // not about making one safe, and this same call sanitises a message on
        // its way out to somebody else: replacing a picture here would send
        // the words "Picture not shown" to the person being written to.
        cleaner().clean(html).to_string()
    }

    /// The same, and how many pictures were held back.
    ///
    /// A caller that shows a message wants both: the markup to show and the
    /// sentence to put above it. Counting a second time somewhere else would be
    /// two answers to one question.
    pub fn sanitize_and_count_held_back(&self, html: &str) -> (String, usize) {
        if self.plain_text_only {
            return (self.sanitize_html(html), 0);
        }
        self.hold_back_what_would_be_fetched(&cleaner().clean(html).to_string())
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

    /// Hold back the pictures that would have to be fetched.
    ///
    /// Run over the cleaned markup rather than the sender's, so the tags being
    /// matched are ones ammonia has already written and the shape of them is
    /// known. A picture held back leaves the sender's own description in its
    /// place, so somebody can tell what they would be asking for.
    ///
    /// Returns the markup and how many were held back, because the count is
    /// what the sentence above the message reports and counting twice would be
    /// two answers to one question.
    fn hold_back_what_would_be_fetched(&self, cleaned: &str) -> (String, usize) {
        use crate::application::pictures::{Showing, what_stands_in_for_it, what_to_do_about};

        let mut held_back = 0;
        let out = img_tag_whole_re()
            .replace_all(cleaned, |caught: &regex::Captures<'_>| {
                let tag = &caught[0];
                let address = one_attribute(tag, "src").unwrap_or_default();
                match what_to_do_about(&address, self.fetching) {
                    // The picture is shown, so it is not replaced. What may
                    // change is one attribute on it.
                    Showing::ItIsCarried | Showing::ItWillBeFetched => {
                        self.say_where_a_decorative_picture_is(tag)
                    }
                    Showing::HeldBack => {
                        held_back += 1;
                        let described = one_attribute(tag, "alt").unwrap_or_default();
                        format!(
                            "<span class=\"held-back\">{}</span>",
                            html_escape::encode_text(&what_stands_in_for_it(&described))
                        )
                    }
                }
            })
            .into_owned();
        (out, held_back)
    }

    /// Put words on a picture the sender marked decorative, if this reader
    /// asked for them.
    ///
    /// The attribute is rewritten and the picture is not replaced. A held-back
    /// picture becomes a span because it is not shown at all; a decorative one
    /// is shown, so replacing it would take it away from a sighted reader to
    /// serve somebody else, which is the trade this application exists not to
    /// make.
    ///
    /// Only an `alt` that is present and empty. A picture with a description
    /// is untouched, and so is one with no `alt` at all: that is a sender who
    /// said nothing rather than a sender who said there was nothing to say,
    /// and this program does not know the difference is safe to guess.
    ///
    /// It lives here rather than in `sanitize_html` on purpose, and the reason
    /// is in that function's own comment: the same call sanitises a message on
    /// its way out to somebody else, so words written there would be sent to
    /// the recipient and would overwrite the mark this program just made.
    fn say_where_a_decorative_picture_is(&self, tag: &str) -> String {
        use crate::application::pictures::{Announcing, WHAT_A_DECORATIVE_PICTURE_SAYS};

        if self.announcing == Announcing::Silently {
            return tag.to_string();
        }
        // Present and empty, told apart from absent. `one_attribute` answers
        // `None` for an attribute that is not there and `Some("")` for one
        // that is there and empty, which is the whole distinction.
        if one_attribute(tag, "alt").as_deref() != Some("") {
            return tag.to_string();
        }
        // Anchored on the attribute. `a`, `l` and `t` are all base64
        // characters, so a picture's own data can hold the letters, and a
        // replacement over the whole tag would corrupt the picture rather than
        // describe it. `\b` refuses a longer attribute name ending in `alt`,
        // and a `"` cannot appear inside a value ammonia has written, so the
        // six characters `alt=""` occur exactly where the empty description
        // is.
        empty_alt_re()
            .replace(
                tag,
                format!(
                    r#"alt="{}""#,
                    html_escape::encode_double_quoted_attribute(WHAT_A_DECORATIVE_PICTURE_SAYS)
                )
                .as_str(),
            )
            .into_owned()
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
            // Read rather than written, so pictures that would have to be
            // fetched are held back here.
            MessageBody::Html(html) | MessageBody::Multipart { html, .. } => {
                self.sanitize_and_count_held_back(html).0
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
                    self.sanitize_and_count_held_back(html).0
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

    #[test]
    fn test_a_picture_the_message_carries_is_shown() {
        // The whole point of carrying it. Before this the safe pictures were
        // the ones that did not appear and the tracking ones were the ones
        // that did.
        use crate::application::pictures::{Carried, Fetching, carry_the_pictures};
        let stored = carry_the_pictures(
            r#"<img src="cid:logo@x" alt="Company logo">"#,
            &[Carried {
                named: "logo@x".to_string(),
                kind: "image/png".to_string(),
                bytes: vec![0x89, b'P', b'N', b'G', 1, 2, 3, 4],
            }],
        );

        let (shown, held_back) =
            HtmlRenderer::with_fetching(Fetching::Blocked).sanitize_and_count_held_back(&stored);

        assert_eq!(held_back, 0, "a carried picture was held back: {shown}");
        assert!(shown.contains("data:image/png;base64,"), "{shown}");
        assert!(shown.contains("Company logo"), "{shown}");
    }

    #[test]
    fn test_a_drawing_the_sender_wrote_in_cannot_bring_a_script() {
        // The reason the filter admits only what this application wrote. An
        // SVG is a document and can run code, and a sender can write one
        // straight into a `src`.
        use crate::application::pictures::{Fetching, carry_the_pictures};
        let stored = carry_the_pictures(
            r#"<img src="data:image/svg+xml,<svg onload=alert(1)>" alt="Nasty">"#,
            &[],
        );

        let (shown, _) =
            HtmlRenderer::with_fetching(Fetching::Blocked).sanitize_and_count_held_back(&stored);

        assert!(!shown.contains("onload"), "a script survived: {shown}");
        assert!(!shown.contains("svg"), "{shown}");
    }

    #[test]
    fn test_a_message_being_written_keeps_its_pictures() {
        // The same call sanitises a message on its way out to somebody else.
        // Holding a picture back there would send them the words "Picture not
        // shown" in place of the picture, which is worse than either answer.
        use crate::application::pictures::Fetching;
        let written = r#"<p>Look</p><img src="https://cdn.example/x.jpg" alt="A chart">"#;

        let out = HtmlRenderer::with_fetching(Fetching::Blocked).sanitize_html(written);

        assert!(
            out.contains("cdn.example"),
            "sanitising a message being written held a picture back, so the \
             recipient would be sent a note about it instead: {out}"
        );
    }

    #[test]
    fn test_a_tracking_pixel_is_not_fetched() {
        // One invisible pixel is the whole of how mail tracking works. This is
        // the test that says the default protects against it.
        use crate::application::pictures::Fetching;
        let (shown, held_back) = HtmlRenderer::with_fetching(Fetching::Blocked)
            .sanitize_and_count_held_back(
                r#"<p>Hello</p><img src="https://tracker.example/pixel.gif" width="1" height="1">"#,
            );

        assert_eq!(held_back, 1, "nothing was held back: {shown}");
        assert!(
            !shown.contains("tracker.example"),
            "the address survived, so the browser will still fetch it: {shown}"
        );
        assert!(shown.contains("Hello"), "the message itself went: {shown}");
    }

    #[test]
    fn test_a_held_back_picture_leaves_the_senders_words_in_its_place() {
        use crate::application::pictures::Fetching;
        let (shown, _) = HtmlRenderer::with_fetching(Fetching::Blocked)
            .sanitize_and_count_held_back(
                r#"<img src="https://cdn.example/x.jpg" alt="Our spring range">"#,
            );

        assert!(shown.contains("Our spring range"), "{shown}");
        assert!(!shown.contains("cdn.example"), "{shown}");
    }

    #[test]
    fn test_somebody_who_allows_them_gets_them() {
        use crate::application::pictures::Fetching;
        let (shown, held_back) = HtmlRenderer::with_fetching(Fetching::Allowed)
            .sanitize_and_count_held_back(
                r#"<img src="https://cdn.example/x.jpg" alt="Our spring range">"#,
            );

        assert_eq!(held_back, 0);
        assert!(shown.contains("cdn.example"), "{shown}");
    }

    /// The trip a picture really takes between being inserted and being seen
    /// again, one stage at a time so a failure says which stage lost it.
    ///
    /// Named for the functions rather than for the story, because the point of
    /// this fixture is that every one of them is really called. A test that
    /// sanitises a string twice and finds it unchanged is a test about the
    /// sanitiser, and what can go wrong is a step somewhere else.
    ///
    /// The storage stage is not here. It needs a real database and it lives in
    /// `data::message_cache::drafts`, where a fixture already opens one.
    struct RoundTrip {
        /// What `pictures::a_picture_to_send` wrote.
        composed: String,
        /// What `editor_document::body_from_editor` handed back, which is the
        /// sanitiser over what the page returned. This is what a draft stores.
        left_the_editor: String,
        /// The same body put into the page again by
        /// `editor_document::editor_document`, which sanitises a second time.
        back_in_the_page: String,
    }

    impl RoundTrip {
        /// Run the trip for a body somebody wrote in the composer.
        fn of(composed: &str) -> Self {
            use crate::presentation::editor_document::{body_from_editor, editor_document};

            // As the page hands it back: a JSON string on some backends, which
            // is what `body_from_editor` unwraps before anything looks at it.
            let as_the_page_answers =
                serde_json::to_string(composed).expect("a body a page could answer with");
            let left_the_editor = body_from_editor(&as_the_page_answers);
            let back_in_the_page =
                editor_document(&MessageBody::Html(left_the_editor.clone()), "en-GB", false);
            Self {
                composed: composed.to_string(),
                left_the_editor,
                back_in_the_page,
            }
        }

        /// The `img` tags at each stage, so an assertion can name the stage.
        fn pictures_at_each_stage(&self) -> [(&'static str, Vec<String>); 3] {
            let tags = |markup: &str| {
                img_tag_whole_re()
                    .find_iter(markup)
                    .map(|found| found.as_str().to_string())
                    .collect::<Vec<_>>()
            };
            [
                ("as it was composed", tags(&self.composed)),
                ("as it left the editor", tags(&self.left_the_editor)),
                (
                    "as it went back into the page",
                    tags(&self.back_in_the_page),
                ),
            ]
        }
    }

    #[test]
    fn test_a_picture_and_its_description_survive_the_whole_trip() {
        // Criterion 2 of this phase says a description survives a draft save
        // and reload, and until now nothing asserted any of it. Every stage is
        // named so a failure says which end lost it, rather than "the
        // description is gone" about a trip with four stages in it.
        use crate::application::pictures::{
            WhatThePictureSays, a_picture_to_send, is_a_picture_we_carried,
        };

        let composed = a_picture_to_send(
            "image/png",
            &a_tiny_png(),
            &WhatThePictureSays::InWords("A chart of sales".to_string()),
        )
        .expect("a described picture");
        let trip = RoundTrip::of(&composed);

        for (stage, pictures) in trip.pictures_at_each_stage() {
            assert_eq!(
                pictures.len(),
                1,
                "the picture was lost {stage}: {}",
                trip.back_in_the_page
            );
            let tag = &pictures[0];
            let address = one_attribute(tag, "src").unwrap_or_default();
            assert!(
                is_a_picture_we_carried(&address),
                "the picture stopped being one this program carried {stage}: {tag}"
            );
            assert_eq!(
                one_attribute(tag, "alt").as_deref(),
                Some("A chart of sales"),
                "the description was lost {stage}: {tag}"
            );
        }
    }

    #[test]
    fn test_an_awkward_description_survives_the_whole_trip_unmangled() {
        // Three characters the escaping and the sanitiser could each mangle in
        // a different way, and the trip escapes and unescapes more than once.
        // A quote would close the attribute, an angle bracket would start a
        // tag, and a non-ASCII character is where an encoding mistake shows.
        use crate::application::pictures::{WhatThePictureSays, a_picture_to_send};

        let awkward = r#"Ada's "3 < 4" café"#;
        let composed = a_picture_to_send(
            "image/png",
            &a_tiny_png(),
            &WhatThePictureSays::InWords(awkward.to_string()),
        )
        .expect("a described picture");
        let trip = RoundTrip::of(&composed);

        for (stage, pictures) in trip.pictures_at_each_stage() {
            let tag = pictures
                .first()
                .unwrap_or_else(|| panic!("no picture {stage}"));
            // Read back through the same decoding the send path uses, because
            // what is stored is escaped and what a reader hears is not.
            let stored = one_attribute(tag, "alt").unwrap_or_default();
            let spoken = html_escape::decode_html_entities(&stored);
            assert_eq!(
                spoken, awkward,
                "the description came back different {stage}: {tag}"
            );
            assert!(
                !tag.contains(r#"alt="Ada's "3"#),
                "the quote broke out of the attribute {stage}: {tag}"
            );
        }
    }

    #[test]
    fn test_two_pictures_keep_their_own_descriptions_in_order() {
        // One picture cannot tell a swap from a survival. Two can, and a
        // message with a signature under a chart is the ordinary case.
        use crate::application::pictures::{WhatThePictureSays, a_picture_to_send};

        let in_words = |words: &str| WhatThePictureSays::InWords(words.to_string());
        let first = a_picture_to_send("image/png", &a_tiny_png(), &in_words("First, a chart"))
            .expect("one");
        let second = a_picture_to_send("image/jpeg", &a_tiny_png(), &in_words("Second, a photo"))
            .expect("two");
        let trip = RoundTrip::of(&format!("<p>{first}</p><p>{second}</p>"));

        for (stage, pictures) in trip.pictures_at_each_stage() {
            assert_eq!(pictures.len(), 2, "a picture was lost {stage}");
            assert_eq!(
                pictures
                    .iter()
                    .map(|tag| one_attribute(tag, "alt").unwrap_or_default())
                    .collect::<Vec<_>>(),
                vec!["First, a chart", "Second, a photo"],
                "the descriptions did not come back in order {stage}"
            );
        }
    }

    #[test]
    fn test_a_data_address_that_is_not_a_carried_picture_is_still_removed() {
        // So proving a picture survives has not widened what else does. An SVG
        // is a document and can carry a script, and this is the exact address
        // shape the filter exists to refuse.
        let trip = RoundTrip::of(
            r#"<img src="data:image/svg+xml;base64,PHN2Zz48c2NyaXB0Pjwvc2NyaXB0Pjwvc3ZnPg==" alt="A drawing">"#,
        );

        for (stage, pictures) in trip.pictures_at_each_stage() {
            if stage == "as it was composed" {
                continue;
            }
            let addresses: Vec<String> = pictures
                .iter()
                .filter_map(|tag| one_attribute(tag, "src"))
                .collect();
            assert!(
                addresses.is_empty(),
                "a data: address that is not a carried picture survived {stage}: {addresses:?}"
            );
        }
    }

    #[test]
    fn test_a_role_of_presentation_does_not_survive_the_sanitiser() {
        // The finding the decorative design rests on, asserted here rather
        // than read out of a crate's source. `role` is in neither of ammonia's
        // allowed lists, so it is removed, and a decorative mark written that
        // way would be lost the first time a draft was saved with nothing
        // saying so.
        let cleaned = HtmlRenderer::new().sanitize_html(&format!(
            r#"<img src="data:image/png;base64,{}" alt="" role="presentation">"#,
            a_tiny_encoded_png()
        ));

        assert!(
            !cleaned.contains("role"),
            "role survived the sanitiser, so the decorative mark had a second \
             representation after all: {cleaned}"
        );
        assert!(
            cleaned.contains("data:image/png;base64,"),
            "the picture itself was removed: {cleaned}"
        );
    }

    #[test]
    fn test_an_empty_description_is_kept_and_is_not_the_same_as_having_none() {
        // The decorative mark is an explicit empty `alt`, which is what WCAG
        // says a decorative image carries. It is only a mark if the sanitiser
        // keeps it, and only a mark if a picture that carries it can be told
        // apart from a picture whose sender said nothing at all. If ammonia
        // added an empty `alt` to every picture, or dropped an empty one, the
        // two would be the same string and there would be no mark here.
        let renderer = HtmlRenderer::new();
        let carried = a_tiny_encoded_png();

        let marked = renderer.sanitize_html(&format!(
            r#"<img src="data:image/png;base64,{carried}" alt="">"#
        ));
        let said_nothing =
            renderer.sanitize_html(&format!(r#"<img src="data:image/png;base64,{carried}">"#));

        assert_eq!(
            one_attribute(&marked, "alt").as_deref(),
            Some(""),
            "the empty description did not survive, so the decorative mark has \
             no representation: {marked}"
        );
        assert_eq!(
            one_attribute(&said_nothing, "alt"),
            None,
            "a picture whose sender said nothing came back carrying an empty \
             description, so the mark cannot be told from an absence: {said_nothing}"
        );
    }

    #[test]
    fn test_a_decorative_picture_survives_the_whole_trip_and_is_still_carried() {
        // A picture it stops recognising is not stripped of its mark, it is
        // deleted: the attribute filter admits a `data:` address only when
        // `is_a_picture_we_carried` says yes, so an empty description that
        // made that answer no would take the picture away with nothing said.
        use crate::application::pictures::{
            WhatThePictureSays, a_picture_to_send, is_a_picture_we_carried,
        };

        // Written by the real writer rather than by hand, so this is the trip
        // a decorative picture somebody inserted really takes. Hand-built
        // markup would prove the sanitiser keeps an empty `alt` and say
        // nothing about whether anything produces one.
        let trip = RoundTrip::of(
            &a_picture_to_send("image/png", &a_tiny_png(), &WhatThePictureSays::Decorative)
                .expect("a decorative picture"),
        );

        for (stage, pictures) in trip.pictures_at_each_stage() {
            assert_eq!(pictures.len(), 1, "the decorative picture was lost {stage}");
            let tag = &pictures[0];
            assert!(
                is_a_picture_we_carried(&one_attribute(tag, "src").unwrap_or_default()),
                "an empty description stopped this being a carried picture {stage}: {tag}"
            );
            assert_eq!(
                one_attribute(tag, "alt").as_deref(),
                Some(""),
                "the decorative mark was lost {stage}: {tag}"
            );
        }
    }

    /// A carried picture, described however the caller says.
    ///
    /// `None` writes no `alt` attribute at all, which is a sender who said
    /// nothing. `Some("")` writes an empty one, which is the decorative mark.
    fn a_carried_picture(described: Option<&str>) -> String {
        let carried = a_tiny_encoded_png();
        match described {
            Some(words) => format!(r#"<img src="data:image/png;base64,{carried}" alt="{words}">"#),
            None => format!(r#"<img src="data:image/png;base64,{carried}">"#),
        }
    }

    /// The reading path, told outright what this reader has chosen.
    fn read_with(announcing: Announcing, html: &str) -> String {
        HtmlRenderer::with_fetching_and_announcing(Fetching::Blocked, announcing)
            .sanitize_and_count_held_back(html)
            .0
    }

    #[test]
    fn test_a_decorative_picture_is_said_to_be_there_when_the_reader_asked_for_that() {
        // The sender's mark can be wrong, honestly or lazily, and the reader
        // is the one who pays. This is the final say moving to the receiving
        // side.
        let shown = read_with(Announcing::OutLoud, &a_carried_picture(Some("")));

        assert_eq!(
            one_attribute(&shown, "alt").as_deref(),
            Some(WHAT_A_DECORATIVE_PICTURE_SAYS),
            "the reader asked to be told a picture was there and was not: {shown}"
        );
        assert!(
            shown.contains("data:image/png;base64,"),
            "the picture itself was taken away rather than described: {shown}"
        );
    }

    #[test]
    fn test_a_decorative_picture_is_left_alone_when_the_reader_asked_for_silence() {
        // The other direction, and the one that makes the setting a setting.
        // Somebody who wants furniture silent gets exactly what the sender
        // meant.
        let marked = a_carried_picture(Some(""));

        assert_eq!(
            read_with(Announcing::Silently, &marked),
            marked,
            "the mark was rewritten for a reader who asked for silence"
        );
    }

    #[test]
    fn test_a_described_picture_is_untouched_whatever_the_reader_chose() {
        // The words go where the sender said nothing, never over what they
        // did say. Overwriting a description would be this program deciding it
        // knows better than somebody who took the trouble.
        let described = a_carried_picture(Some("A chart of sales"));

        for announcing in [Announcing::OutLoud, Announcing::Silently] {
            assert_eq!(
                read_with(announcing, &described),
                described,
                "a described picture was rewritten under {announcing:?}"
            );
        }
    }

    #[test]
    fn test_a_picture_with_no_description_at_all_is_untouched_whatever_the_reader_chose() {
        // The distinction the whole mark rests on. A missing `alt` is a sender
        // who said nothing; an empty one is a sender who said there is nothing
        // to say. Writing "the sender marked this decorative" over the first
        // would put words in the mouth of somebody who never opened it.
        let silent = a_carried_picture(None);

        for announcing in [Announcing::OutLoud, Announcing::Silently] {
            let shown = read_with(announcing, &silent);
            assert_eq!(
                one_attribute(&shown, "alt"),
                None,
                "a picture whose sender said nothing was reported as marked \
                 decorative under {announcing:?}: {shown}"
            );
        }
    }

    #[test]
    fn test_only_the_decorative_picture_changes_when_two_are_shown() {
        // One picture cannot tell "the right one changed" from "everything
        // changed". The ordinary message has both kinds in it.
        let both = format!(
            "<p>{}</p><p>{}</p>",
            a_carried_picture(Some("A chart of sales")),
            a_carried_picture(Some(""))
        );

        let shown = read_with(Announcing::OutLoud, &both);
        let descriptions: Vec<String> = img_tag_whole_re()
            .find_iter(&shown)
            .map(|found| one_attribute(found.as_str(), "alt").unwrap_or_default())
            .collect();

        assert_eq!(
            descriptions,
            vec![
                "A chart of sales".to_string(),
                WHAT_A_DECORATIVE_PICTURE_SAYS.to_string()
            ],
            "{shown}"
        );
    }

    #[test]
    fn test_the_rewrite_is_anchored_to_the_attribute_and_not_to_a_substring() {
        // `a`, `l` and `t` are all base64 characters, so a picture's own data
        // can hold the letters this looks for. A rewrite done by searching the
        // whole tag for text would corrupt the picture rather than describe
        // it, and the picture would simply not appear.
        let data = "YWx0YWx0YWx0YWx0";
        let awkward = format!(r#"<img src="data:image/png;base64,{data}" alt="">"#);

        let shown = read_with(Announcing::OutLoud, &awkward);

        assert!(
            shown.contains(&format!("base64,{data}")),
            "the picture's own data was rewritten: {shown}"
        );
        assert_eq!(
            one_attribute(&shown, "alt").as_deref(),
            Some(WHAT_A_DECORATIVE_PICTURE_SAYS)
        );
    }

    #[test]
    fn test_the_words_are_not_the_ones_used_for_a_picture_that_was_not_shown() {
        // Three different facts and three different sentences. A held-back
        // picture is one nobody has seen; a picture nobody described is a
        // sender who said nothing; this one is shown and its sender said there
        // was nothing to say. A reader who heard the same words for all three
        // would learn nothing from any of them.
        assert_ne!(WHAT_A_DECORATIVE_PICTURE_SAYS, what_stands_in_for_it(""));
        assert_ne!(
            WHAT_A_DECORATIVE_PICTURE_SAYS,
            what_stands_in_for_it(WHAT_A_DECORATIVE_PICTURE_SAYS)
        );
        assert!(
            !what_stands_in_for_it("").contains(WHAT_A_DECORATIVE_PICTURE_SAYS),
            "the held-back sentence swallowed this one"
        );
        // And it attributes the claim rather than making it, which is the
        // whole reason a reader can weigh it.
        assert!(WHAT_A_DECORATIVE_PICTURE_SAYS.contains("sender"));
    }

    #[test]
    fn test_a_message_on_its_way_out_keeps_its_mark_whatever_this_reader_chose() {
        // The trap this seam was placed to avoid. `sanitize_html` is the same
        // call that runs over a message being composed and sent, so words
        // written there would go to the recipient and would overwrite the mark
        // this program made a moment earlier. The rewrite lives in the reading
        // seam for that reason and this is what says it stayed there.
        let marked = a_carried_picture(Some(""));

        for announcing in [Announcing::OutLoud, Announcing::Silently] {
            let renderer =
                HtmlRenderer::with_fetching_and_announcing(Fetching::Blocked, announcing);
            let going_out = renderer.sanitize_html(&marked);

            assert_eq!(
                one_attribute(&going_out, "alt").as_deref(),
                Some(""),
                "a message on its way out picked up this reader's own words \
                 under {announcing:?}: {going_out}"
            );
        }
    }

    #[test]
    fn test_a_held_back_picture_still_says_what_it_always_said() {
        // A remote picture that is never fetched already has a sentence for an
        // empty description, and changing that would change what every blocked
        // message reads like. Different question, and not this one's.
        let (shown, held_back) =
            HtmlRenderer::with_fetching_and_announcing(Fetching::Blocked, Announcing::OutLoud)
                .sanitize_and_count_held_back(
                    r#"<img src="https://cdn.example/pixel.gif" alt="">"#,
                );

        assert_eq!(held_back, 1);
        assert!(shown.contains(&what_stands_in_for_it("")), "{shown}");
        assert!(
            !shown.contains(WHAT_A_DECORATIVE_PICTURE_SAYS),
            "the decorative words reached the held-back path: {shown}"
        );
    }

    #[test]
    fn test_the_renderer_every_message_opens_through_takes_the_answer_from_the_settings() {
        // `with_fetching_and_announcing` is a test door. `new` and
        // `plain_text_only` are the ones every message really opens through,
        // so an answer wired only into the test door would satisfy every test
        // above and change nothing anybody reads.
        //
        // Compared with what the settings say rather than with a fixed value,
        // because a test asserting `OutLoud` would pass on this machine for
        // the wrong reason and fail on a machine where somebody had turned it
        // off. What this pins is that the constructors ask, not what the
        // answer happens to be here.
        let (_, from_the_settings) = what_the_settings_say();

        assert_eq!(HtmlRenderer::new().announcing, from_the_settings);
        assert_eq!(
            HtmlRenderer::plain_text_only().announcing,
            from_the_settings
        );
    }

    /// Bytes of a kind worth carrying, under the size limit. Not a real PNG:
    /// nothing on this trip decodes one.
    fn a_tiny_png() -> Vec<u8> {
        vec![0x89, b'P', b'N', b'G', 1, 2, 3, 4]
    }

    /// The same bytes as they appear inside a `data:` address.
    fn a_tiny_encoded_png() -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(a_tiny_png())
    }

    use super::*;
    use crate::application::pictures::{
        Announcing, Fetching, WHAT_A_DECORATIVE_PICTURE_SAYS, what_stands_in_for_it,
    };

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
