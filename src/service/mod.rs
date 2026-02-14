//! Service layer - Protocol implementations and services
//!
//! This layer handles email protocols, security, caching, and other services.

pub mod attachments;
pub mod cache;
pub mod oauth;
pub mod protocols;
pub mod security;

pub use attachments::AttachmentHandler;
pub use cache::CacheService;
pub use oauth::{OAuthProvider, OAuthService, OAuthTokenSet};
pub use protocols::{imap, pop3, smtp};
pub use security::SecurityService;
