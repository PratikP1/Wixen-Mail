//! HTML Rendering with Sanitization and Accessibility
//!
//! Renders HTML email content with security (XSS protection) and accessibility features.

use crate::common::Result;
use ammonia::clean;

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
        
        // Use ammonia to clean HTML
        // It removes:
        // - JavaScript
        // - onclick/onerror/etc handlers
        // - data: URLs (potential XSS)
        // - Dangerous CSS
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
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        text = re.replace_all(&text, "").to_string();
        
        // Decode HTML entities
        text = html_escape::decode_html_entities(&text).to_string();
        
        // Clean up whitespace
        let re = regex::Regex::new(r"\n\s*\n\s*\n+").unwrap();
        text = re.replace_all(&text, "\n\n").to_string();
        
        text.trim().to_string()
    }
    
    /// Render HTML for egui display
    ///
    /// Converts sanitized HTML to a format suitable for egui rendering.
    /// Returns a structured representation for accessibility.
    pub fn render_for_egui(&self, html: &str) -> RenderedContent {
        let sanitized = self.sanitize_html(html);
        let plain_text = self.html_to_plain_text(&sanitized);
        
        RenderedContent {
            html: sanitized,
            plain_text,
            has_images: html.contains("<img"),
            has_links: html.contains("<a "),
        }
    }
    
    /// Extract alt text from images for accessibility
    pub fn extract_image_alt_texts(&self, html: &str) -> Vec<String> {
        let mut alt_texts = Vec::new();
        
        // Simple regex to extract alt attributes
        let re = regex::Regex::new(r#"<img[^>]*alt=["']([^"']*)["'][^>]*>"#).unwrap();
        
        for cap in re.captures_iter(html) {
            if let Some(alt) = cap.get(1) {
                alt_texts.push(alt.as_str().to_string());
            }
        }
        
        alt_texts
    }
    
    /// Extract link texts for accessibility
    pub fn extract_link_texts(&self, html: &str) -> Vec<LinkInfo> {
        let mut links = Vec::new();
        
        // Extract href and link text
        let re = regex::Regex::new(r#"<a[^>]*href=["']([^"']*)["'][^>]*>(.*?)</a>"#).unwrap();
        
        for cap in re.captures_iter(html) {
            if let (Some(href), Some(text)) = (cap.get(1), cap.get(2)) {
                links.push(LinkInfo {
                    url: href.as_str().to_string(),
                    text: self.html_to_plain_text(text.as_str()),
                });
            }
        }
        
        links
    }
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
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
        let html = r#"<img src="test.jpg" alt="Test Image"><img src="test2.jpg" alt="Another Image">"#;
        let alt_texts = renderer.extract_image_alt_texts(html);
        
        assert_eq!(alt_texts.len(), 2);
        assert_eq!(alt_texts[0], "Test Image");
        assert_eq!(alt_texts[1], "Another Image");
    }
    
    #[test]
    fn test_extract_link_texts() {
        let renderer = HtmlRenderer::new();
        let html = r#"<a href="https://example.com">Example Link</a><a href="https://test.com">Test</a>"#;
        let links = renderer.extract_link_texts(html);
        
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com");
        assert_eq!(links[0].text, "Example Link");
    }
    
    #[test]
    fn test_render_for_egui() {
        let renderer = HtmlRenderer::new();
        let html = "<p>Hello <strong>World</strong>!</p>";
        let content = renderer.render_for_egui(html);
        
        assert!(!content.html.is_empty());
        assert!(!content.plain_text.is_empty());
        assert!(!content.has_images);
        assert!(!content.has_links);
    }
}
