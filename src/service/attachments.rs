//! Attachment handler
//!
//! Manages email attachments.

use crate::common::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Email attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub mime_type: String,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Attachment handler
pub struct AttachmentHandler;

impl AttachmentHandler {
    /// Create a new attachment handler
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Save attachment to disk
    pub fn save(&self, attachment: &Attachment, path: &str) -> Result<()> {
        let target = PathBuf::from(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, &attachment.data)?;
        Ok(())
    }

    /// Load attachment from disk
    pub fn load(&self, path: &str) -> Result<Attachment> {
        let target = PathBuf::from(path);
        let data = fs::read(&target)?;
        let filename = target
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| crate::common::Error::Other("Invalid attachment filename".to_string()))?
            .to_string();
        let mime_type = infer_mime_type(&target).to_string();
        Ok(Attachment {
            filename,
            mime_type,
            size: data.len(),
            data,
        })
    }
}

fn infer_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("txt") => "text/plain",
        Some("html" | "htm") => "text/html",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
}

impl Default for AttachmentHandler {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_attachment_handler_creation() {
        let handler = AttachmentHandler::new();
        assert!(handler.is_ok());
    }

    #[test]
    fn test_attachment_save_and_load() {
        let handler = AttachmentHandler::new().unwrap();
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("wixen_attachment_{}.txt", nanos));
        let attachment = Attachment {
            filename: "note.txt".to_string(),
            mime_type: "text/plain".to_string(),
            size: 5,
            data: b"hello".to_vec(),
        };
        handler.save(&attachment, path.to_str().unwrap()).unwrap();
        let loaded = handler.load(path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.data, b"hello");
        assert_eq!(loaded.mime_type, "text/plain");
        let _ = std::fs::remove_file(path);
    }
}
