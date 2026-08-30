//! Presentation layer: UI and accessibility components
//!
//! Native wxdragon (wxWidgets) UI with built-in accessibility support.

pub mod accessibility;
pub mod art;
pub mod asking;
pub mod command_line;
pub mod compose_toolbar;
pub mod contact_convert;
pub mod date_display;
pub mod default_app_words;
pub mod editor_document;
#[cfg(test)]
pub mod editor_page_harness;
pub mod finding_people;
pub mod first_run;
pub mod folder_tree;
pub mod help_page;
pub mod html_renderer;
pub mod manager_words;
pub mod managers;
pub mod markdown_input;
pub mod message_columns;
pub mod message_rows;
pub mod panes;
pub mod pim_rows;
pub mod read_aloud;
pub mod reader_text;
pub mod scan_target;
pub mod status_line;
pub mod theme;
pub mod ui_types;
pub mod wx_account_manager;
pub mod wx_add_calendar;
pub mod wx_app;
pub mod wx_calendar;
pub mod wx_calendar_module;
pub mod wx_columns;
pub mod wx_compose;
pub mod wx_contacts_module;
pub mod wx_context_menu;
pub mod wx_destination;
pub mod wx_first_run;
pub mod wx_folder_choice;
pub mod wx_item_form;
pub mod wx_managers;
pub mod wx_notes_module;
pub mod wx_reader;
pub mod wx_reminder_alert;
pub mod wx_reminders_module;
pub mod wx_settings;
pub mod wx_tasks_module;
pub mod wx_thread_view;
pub mod wx_tray;
pub mod wx_which_days;

pub use accessibility::Accessibility;
pub use html_renderer::HtmlRenderer;
pub use ui_types::*;
pub use wx_app::WxMailApp;
