//! Service layer - Protocol implementations and services
//!
//! This layer handles email protocols, security, caching, and other services.

pub mod protocols;
pub mod security;
pub mod cache;
pub mod attachments;

pub use protocols::{imap, smtp, pop3};
pub use security::SecurityService;
pub use cache::CacheService;
pub use attachments::AttachmentHandler;
