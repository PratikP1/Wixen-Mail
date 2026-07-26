//! Presentation layer — UI and accessibility components
//!
//! Native wxdragon (wxWidgets) UI with built-in accessibility support.

pub mod accessibility;
pub mod html_renderer;
pub mod ui_types;
pub mod wx_account_manager;
pub mod wx_app;
pub mod wx_calendar;
pub mod wx_calendar_module;
pub mod wx_compose;
pub mod wx_contacts_module;
pub mod wx_managers;
pub mod wx_notes_module;
pub mod wx_oauth;
pub mod wx_reminders_module;
pub mod wx_settings;
pub mod wx_tasks_module;

pub use accessibility::Accessibility;
pub use html_renderer::HtmlRenderer;
pub use ui_types::*;
pub use wx_app::WxMailApp;
