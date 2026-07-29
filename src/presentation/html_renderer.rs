//! HTML Rendering with Sanitization and Accessibility
//!
//! Renders HTML email content with security (XSS protection) and accessibility features.

use ammonia::clean;
use std::sync::OnceLock;

// The patterns below are literals compiled once. An `expect` here can only fire
// if one of them is edited into something invalid, which every test in this
// module would catch on the first run.

const SAFE_URL_SCHEMES: [&str; 3] = ["http://", "https://", "mailto:"];

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
    pub body: String,
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
    fn sanitize_html(&self, html: &str) -> String {
        if self.plain_text_only {
            return html_escape::encode_text(&self.html_to_plain_text(html)).to_string();
        }

        clean(html)
    }

    /// Convert HTML to accessible plain text
    ///
    /// This is useful for screen readers and text-only displays.
    fn html_to_plain_text(&self, html: &str) -> String {
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

    /// Render HTML into an accessible text representation for RichTextCtrl.
    ///
    /// Produces plain text with link annotations and image descriptions
    /// suitable for screen readers. Links are shown inline as "text [URL]".
    pub fn render_for_accessibility(&self, html: &str) -> AccessibleRenderedContent {
        let sanitized = self.sanitize_html(html);

        // Replace links with accessible inline format: "text [URL]"
        let with_links = link_re()
            .replace_all(&sanitized, |caps: &regex::Captures| {
                let href = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                let text = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let clean_text = self.html_to_plain_text(text);
                if let Some(safe_url) = Self::safe_external_url(href) {
                    if clean_text.trim() == safe_url.trim() {
                        clean_text // Don't duplicate if link text is already the URL
                    } else {
                        format!("{} [{}]", clean_text, safe_url)
                    }
                } else {
                    clean_text
                }
            })
            .to_string();

        // Replace images with alt text descriptions
        let with_images = image_alt_re()
            .replace_all(&with_links, |caps: &regex::Captures| {
                let alt = caps
                    .get(1)
                    .or_else(|| caps.get(2))
                    .map(|m| m.as_str())
                    .unwrap_or("image");
                format!("[Image: {}]", alt)
            })
            .to_string();

        // Remove remaining img tags without alt text
        let cleaned = img_tag_re()
            .replace_all(&with_images, "[Image]")
            .to_string();

        let plain = self.html_to_plain_text(&cleaned);
        let links = self.extract_link_texts(&sanitized);
        let image_alt_texts = self.extract_image_alt_texts(&sanitized);

        // Build link summary for the bottom of the message
        let link_summary = if !links.is_empty() {
            let mut summary = String::from("\n\n--- Links ---\n");
            for (i, link) in links.iter().enumerate() {
                summary.push_str(&format!("  {}. {}: {}\n", i + 1, link.text, link.url));
            }
            summary
        } else {
            String::new()
        };

        let accessible_text = format!("{}{}", plain, link_summary);

        AccessibleRenderedContent {
            accessible_text,
            links,
            image_alt_texts,
        }
    }

    /// Render HTML into a structured representation.
    ///
    /// Sanitizes the HTML and returns plain text plus metadata
    /// (links, images, headings) for accessibility.
    pub fn render_for_egui(&self, html: &str) -> RenderedContent {
        let sanitized = self.sanitize_html(html);
        let plain_text = self.html_to_plain_text(&sanitized);
        let image_alt_texts = self.extract_image_alt_texts(&sanitized);
        let links = self.extract_link_texts(&sanitized);
        let warnings = self.build_warnings(html, &sanitized, &image_alt_texts, &links);

        RenderedContent {
            html: sanitized,
            plain_text,
            has_images: html.contains("<img"),
            has_links: html.contains("<a "),
            links,
            image_alt_texts,
            warnings,
        }
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

    /// Wrap sanitized HTML in a full document for WebView display.
    ///
    /// Applies readable typography, dark-mode support, image containment,
    /// and blockquote styling. Plain-text bodies are wrapped in `<pre>`.
    pub fn wrap_for_webview(&self, body: &str) -> String {
        let is_html = body.contains('<') && body.contains('>');
        let content = if is_html {
            self.sanitize_html(body)
        } else {
            format!(
                "<pre style=\"white-space:pre-wrap;font-family:inherit\">{}</pre>",
                html_escape::encode_text(body)
            )
        };
        self.wrap_prepared(&content)
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
    fn wrap_prepared(&self, content: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><style>
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
<button type="button" class="leave" onclick="window.contextMenu.postMessage('{{&quot;kind&quot;:&quot;leave&quot;}}')">Back to message list (Escape)</button>
<main>{}</main></body></html>"#,
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
        body.push_str(&format!(
            "<h1>{}</h1>
<p>{} messages in this conversation.</p>
",
            html_escape::encode_text(if subject.trim().is_empty() {
                "No subject"
            } else {
                subject.trim()
            }),
            parts.len()
        ));

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
            let content = if part.body.contains('<') && part.body.contains('>') {
                self.sanitize_html(&part.body)
            } else {
                format!(
                    "<pre style=\"white-space:pre-wrap;font-family:inherit\">{}</pre>",
                    html_escape::encode_text(&part.body)
                )
            };
            body.push_str(&content);
            body.push('\n');
        }

        // `wrap_prepared`, not `wrap_for_webview`. Every body above has
        // already been through the sanitiser, and sanitising the assembled
        // document again would escape the headings this whole surface exists
        // to produce whenever the plain-text setting is on.
        self.wrap_prepared(&body)
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

    pub fn build_warnings(
        &self,
        original_html: &str,
        sanitized_html: &str,
        image_alt_texts: &[String],
        links: &[LinkInfo],
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        let original_lower = original_html.to_lowercase();
        let sanitized_lower = sanitized_html.to_lowercase();
        if script_tag_re().is_match(&original_lower) && !script_tag_re().is_match(&sanitized_lower)
        {
            warnings.push("Potentially unsafe scripts were removed.".to_string());
        }
        if original_lower.contains("onerror=") || original_lower.contains("onclick=") {
            warnings.push("Inline event handlers were removed for safety.".to_string());
        }
        let image_count = img_tag_re().find_iter(original_html).count();
        if image_count > image_alt_texts.len() {
            warnings.push("Images without alt text may reduce accessibility.".to_string());
        }
        if anchor_tag_re().is_match(original_html) && links.is_empty() {
            warnings.push("Unsupported/unsafe links were omitted from preview.".to_string());
        }
        warnings
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Accessible rendering of HTML content for RichTextCtrl / screen readers.
#[derive(Debug, Clone)]
pub struct AccessibleRenderedContent {
    /// Full accessible text with link annotations and image descriptions
    pub accessible_text: String,
    /// Extracted safe links
    pub links: Vec<LinkInfo>,
    /// Image alt texts
    pub image_alt_texts: Vec<String>,
}

/// Rendered HTML content with accessibility information
#[derive(Debug, Clone)]
pub struct RenderedContent {
    /// Sanitized HTML
    pub html: String,
    /// Plain text version for accessibility
    pub plain_text: String,
    /// Whether content has images
    pub has_images: bool,
    /// Whether content has links
    pub has_links: bool,
    /// Safe extracted links
    pub links: Vec<LinkInfo>,
    /// Extracted alt text from images
    pub image_alt_texts: Vec<String>,
    /// Renderer warnings and safety notes
    pub warnings: Vec<String>,
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
        let html = renderer.wrap_for_webview("Hello");
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
        assert!(renderer.wrap_for_webview("Hello").contains("onclick="));
    }

    #[test]
    fn test_a_preview_document_has_a_main_landmark() {
        // A screen reader user arriving in the preview needs somewhere to be.
        // A bare run of text in a body with no landmark gives nothing to jump
        // to and no way to tell the message from the chrome around it.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_for_webview("Hello");
        assert!(html.contains("<main"), "no main landmark: {}", html);
        assert!(html.contains("</main>"));
    }

    #[test]
    fn test_a_preview_document_declares_its_language() {
        // Without a lang attribute a screen reader reads the message in
        // whatever voice it was last using, which turns English mail read by a
        // German voice into noise (WCAG 3.1.1).
        let renderer = HtmlRenderer::new();
        assert!(renderer.wrap_for_webview("Hello").contains("<html lang="));
    }

    #[test]
    fn test_a_preview_document_says_how_to_leave_it() {
        // The preview is a WebView, which swallows the keystrokes that would
        // normally move focus out. Someone who cannot see the window has no
        // way to discover the way back unless the document says it.
        let renderer = HtmlRenderer::new();
        let html = renderer.wrap_for_webview("Hello");
        assert!(html.contains("Escape"), "no way out is stated: {}", html);
    }

    fn part(sender: &str, depth: usize, body: &str) -> ThreadPart {
        ThreadPart {
            sender: sender.to_string(),
            date: "2026-07-26".to_string(),
            subject: "Quarterly report".to_string(),
            body: body.to_string(),
            depth,
        }
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
    fn test_render_for_egui() {
        let renderer = HtmlRenderer::new();
        let html =
            r#"<p>Hello <strong>World</strong>!</p><a href="https://example.com">Example</a>"#;
        let content = renderer.render_for_egui(html);

        assert!(!content.html.is_empty());
        assert!(!content.plain_text.is_empty());
        assert!(!content.has_images);
        assert!(content.has_links);
        assert_eq!(content.links.len(), 1);
    }

    #[test]
    fn test_wrap_for_webview_html() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Hello <strong>World</strong></p>";
        let wrapped = renderer.wrap_for_webview(html);
        assert!(wrapped.contains("<!DOCTYPE html>"));
        assert!(wrapped.contains("Segoe UI"));
        assert!(wrapped.contains("<p>Hello <strong>World</strong></p>"));
    }

    #[test]
    fn test_wrap_for_webview_plain_text() {
        let renderer = HtmlRenderer::new();
        let text = "Just plain text, no HTML.";
        let wrapped = renderer.wrap_for_webview(text);
        assert!(wrapped.contains("<pre"));
        assert!(wrapped.contains("Just plain text, no HTML."));
    }

    #[test]
    fn test_wrap_for_webview_strips_scripts() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Safe</p><script>alert('xss')</script>";
        let wrapped = renderer.wrap_for_webview(html);
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
            assert_no_live_markup(&renderer.wrap_for_webview(html), html);
        }
    }

    #[test]
    fn test_hostile_bodies_are_inert_in_plain_text_mode_too() {
        // Plain text mode strips tags and then decodes entities, which turns
        // "&lt;script&gt;" back into "<script>". That is correct as plain
        // text and unsafe the moment it is placed in an HTML document.
        let renderer = HtmlRenderer::plain_text_only();
        for html in HOSTILE {
            assert_no_live_markup(&renderer.wrap_for_webview(html), html);
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
            let rendered = HtmlRenderer::new().wrap_for_webview(&body);
            assert_no_live_markup(&rendered, &format!("seed {}", seed));
        }
    }

    #[test]
    fn test_fuzz_plain_text_mode_stays_inert() {
        for seed in 0..4000u64 {
            let body = fuzz_body(seed);
            let rendered = HtmlRenderer::plain_text_only().wrap_for_webview(&body);
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
            let _ = renderer.render_for_accessibility(&body);
            let _ = renderer.render_for_egui(&body);
            let _ = renderer.extract_image_alt_texts(&body);
            let _ = renderer.extract_link_texts(&body);
            let _ = renderer.wrap_for_webview(&body);
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
        ] {
            assert!(
                HtmlRenderer::safe_external_url(url).is_some(),
                "{} should be openable",
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
