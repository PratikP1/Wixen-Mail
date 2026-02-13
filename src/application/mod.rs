//! Application layer - Business logic and managers
//!
//! This layer contains the core business logic and management components.

pub mod accounts;
pub mod composition;
pub mod contacts;
pub mod filters;
pub mod messages;
pub mod search;

pub use accounts::AccountManager;
pub use composition::CompositionManager;
pub use contacts::ContactManager;
pub use filters::FilterEngine;
pub use messages::MessageManager;
pub use search::SearchEngine;
