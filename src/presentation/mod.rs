//! Presentation layer — UI and accessibility components
//!
//! Native wxdragon (wxWidgets) UI with built-in accessibility support.

pub mod accessibility;
pub mod date_display;
pub mod html_renderer;
pub mod message_columns;
pub mod message_rows;
pub mod pim_rows;
pub mod read_aloud;
pub mod ui_types;
pub mod wx_account_manager;
pub mod wx_app;
pub mod wx_calendar;
pub mod wx_calendar_module;
pub mod wx_columns;
pub mod wx_compose;
pub mod wx_contacts_module;
pub mod wx_managers;
pub mod wx_notes_module;
pub mod wx_oauth;
pub mod wx_reminders_module;
pub mod wx_settings;
pub mod wx_tasks_module;
pub mod wx_thread_view;

pub use accessibility::Accessibility;
pub use html_renderer::HtmlRenderer;
pub use ui_types::*;
pub use wx_app::WxMailApp;
