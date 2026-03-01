//! HTML Rendering with Sanitization and Accessibility
//!
//! Renders HTML email content with security (XSS protection) and accessibility features.

use ammonia::clean;
use std::sync::OnceLock;

const SAFE_URL_SCHEMES: [&str; 3] = ["http://", "https://", "mailto:"];

fn html_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"<[^>]*>").expect("valid html tag regex"))
}

fn newline_compact_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\n\s*\n\s*\n+").expect("valid newline compact regex"))
}

fn image_alt_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(?is)<img[^>]*?alt=(?:"([^"]*)"|'([^']*)')[^>]*?>"#)
            .expect("valid image alt regex")
    })
}

fn link_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r#"(?is)<a[^>]*?href=(?:"([^"]*)"|'([^']*)')[^>]*?>([\s\S]*?)</a>"#)
            .expect("valid link regex")
    })
}

fn img_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<img\b").expect("valid image tag regex"))
}

fn anchor_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<a\b").expect("valid anchor tag regex"))
}

fn script_tag_re() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"(?is)<\s*script\b").expect("valid script tag regex"))
}

/// HTML renderer with sanitization
pub struct HtmlRenderer {
    /// Whether to strip all HTML and return plain text
    plain_text_only: bool,
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

    /// Sanitize HTML content for safe display
    ///
    /// This removes potentially dangerous HTML/JavaScript while preserving
    /// safe formatting and structure.
    pub fn sanitize_html(&self, html: &str) -> String {
        if self.plain_text_only {
            return self.html_to_plain_text(html);
        }

        clean(html)
    }

    /// Convert HTML to accessible plain text
    ///
    /// This is useful for screen readers and text-only displays.
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

    /// Render HTML into an accessible text representation for RichTextCtrl.
    ///
    /// Produces plain text with link annotations and image descriptions
    /// suitable for screen readers. Links are shown inline as "text [URL]".
    pub fn render_for_accessibility(&self, html: &str) -> AccessibleRenderedContent {
        let sanitized = self.sanitize_html(html);

        // Replace links with accessible inline format: "text [URL]"
        let with_links = link_re().replace_all(&sanitized, |caps: &regex::Captures| {
            let href = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("");
            let text = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let clean_text = self.html_to_plain_text(text);
            if let Some(safe_url) = Self::sanitize_url(href) {
                if clean_text.trim() == safe_url.trim() {
                    clean_text // Don't duplicate if link text is already the URL
                } else {
                    format!("{} [{}]", clean_text, safe_url)
                }
            } else {
                clean_text
            }
        }).to_string();

        // Replace images with alt text descriptions
        let with_images = image_alt_re().replace_all(&with_links, |caps: &regex::Captures| {
            let alt = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str()).unwrap_or("image");
            format!("[Image: {}]", alt)
        }).to_string();

        // Remove remaining img tags without alt text
        let cleaned = img_tag_re().replace_all(&with_images, "[Image]").to_string();

        let plain = self.html_to_plain_text(&cleaned);
        let links = self.extract_link_texts(&sanitized);
        let image_alt_texts = self.extract_image_alt_texts(&sanitized);

        // Build link summary for the bottom of the message
        let link_summary = if !links.is_empty() {
            let mut summary = String::from("\n\n--- Links ---\n");
            for (i, link) in links.iter().enumerate() {
                summary.push_str(&format!("  {}. {} — {}\n", i + 1, link.text, link.url));
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
            if let (Some(href), Some(text)) = (cap.get(1).or_else(|| cap.get(2)), cap.get(3)) {
                if let Some(safe_url) = Self::sanitize_url(href.as_str()) {
                    links.push(LinkInfo {
                        url: safe_url,
                        text: self.html_to_plain_text(text.as_str()),
                    });
                }
            }
        }

        links
    }

    /// Convert supported URL schemes to safe navigable values.
    fn sanitize_url(url: &str) -> Option<String> {
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

    fn build_warnings(
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
    fn test_extract_links_filters_unsafe_schemes() {
        let renderer = HtmlRenderer::new();
        let html =
            r#"<a href="javascript:alert(1)">Bad</a><a href="mailto:test@example.com">Mail</a>"#;
        let links = renderer.extract_link_texts(html);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "mailto:test@example.com");
    }
}
