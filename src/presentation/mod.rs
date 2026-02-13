//! Presentation layer - UI and accessibility components
//!
//! This layer handles all user interface rendering and accessibility features.

pub mod accessibility;
pub mod ui;
pub mod ui_integrated;
pub mod html_renderer;
pub mod composition;

pub use accessibility::Accessibility;
pub use ui::UI;
pub use ui_integrated::IntegratedUI;
pub use html_renderer::HtmlRenderer;
pub use composition::{CompositionWindow, CompositionAction};
