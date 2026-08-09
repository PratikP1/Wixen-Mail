//! Service layer - Protocol implementations and services
//!
//! This layer handles email protocols, security, caching, and other services.

pub mod attachment_name;
pub mod cache;
pub mod caldav;
pub mod credentials;
pub mod google_api;
pub mod ical_subscription;
pub mod microsoft_graph;
pub mod mime;
pub mod oauth;
pub mod oauth_credentials;
pub mod outward;
pub mod pdf;
pub mod protocols;
pub mod safebrowsing;
pub mod safety;
pub mod security;
pub mod spellcheck;
pub mod tasks_api;
pub mod vtimezone;

pub use cache::CacheService;
pub use caldav::{CalDavClient, CalDavEvent};
pub use google_api::GoogleApiClient;
pub use ical_subscription::ICalSubscriptionClient;
pub use microsoft_graph::MsGraphClient;
pub use oauth::{AuthManager, OAuthProvider, OAuthService, OAuthTokenSet};
pub use protocols::{imap, pop3, smtp};
pub use security::SecurityService;
pub use spellcheck::{I18n, Locale, SpellChecker};
