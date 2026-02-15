//! Presentation layer - UI and accessibility components
//!
//! This layer handles all user interface rendering and accessibility features.

pub mod accessibility;
pub mod account_manager;
pub mod composition;
pub mod contact_manager;
pub mod filter_manager;
pub mod html_renderer;
pub mod oauth_manager;
pub mod signature_manager;
pub mod tag_manager;
pub mod ui;
pub mod ui_integrated;

pub use accessibility::Accessibility;
pub use account_manager::{AccountAction, AccountManagerWindow};
pub use composition::{CompositionAction, CompositionWindow};
pub use contact_manager::{ContactAction, ContactManagerWindow};
pub use filter_manager::{FilterManagerWindow, FilterRuleAction};
pub use html_renderer::HtmlRenderer;
pub use oauth_manager::{OAuthAction, OAuthManagerWindow};
pub use signature_manager::{SignatureAction, SignatureManagerWindow, SignatureSelector};
pub use tag_manager::{QuickTagAction, QuickTagMenu, TagAction, TagManagerWindow};
pub use ui::UI;
pub use ui_integrated::IntegratedUI;
