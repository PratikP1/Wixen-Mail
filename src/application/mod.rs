//! Application layer - Business logic and managers
//!
//! This layer contains the core business logic and management components.

pub mod accounts;
pub mod messages;
pub mod composition;
pub mod search;
pub mod filters;
pub mod contacts;

pub use accounts::AccountManager;
pub use messages::MessageManager;
pub use composition::CompositionManager;
pub use search::SearchEngine;
pub use filters::FilterEngine;
pub use contacts::ContactManager;
