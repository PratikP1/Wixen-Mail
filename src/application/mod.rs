//! Application layer - Business logic and managers
//!
//! This layer contains the core business logic and management components.

pub mod accounts;
pub mod allowed;
pub mod autosave;
pub mod caldav_sync;
pub mod calendar;
pub mod collection_sync;
pub mod composition;
pub mod contacts;
pub mod contacts_sync;
pub mod filters;
pub mod forget;
pub mod item_fields;
pub mod mail_auth;
pub mod mail_controller;
pub mod mail_sync;
pub mod messages;
pub mod new_item;
pub mod notes;
pub mod pim_command;
pub mod reminders;
pub mod reply;
pub mod running;
pub mod search;
pub mod spell_session;
pub mod tasks;
pub mod tasks_sync;
pub mod threading;
pub mod words;

pub use accounts::AccountManager;
pub use caldav_sync::{refresh_subscription, sync_caldav_calendar};
pub use calendar::CalendarManager;
pub use composition::CompositionManager;
pub use contacts::ContactManager;
pub use contacts_sync::{sync_google_contacts, sync_microsoft_contacts};
pub use filters::FilterEngine;
pub use mail_controller::{MailController, SendEmailRequest};
pub use messages::MessageManager;
pub use notes::NoteManager;
pub use reminders::ReminderManager;
pub use search::SearchEngine;
pub use tasks::TaskManager;
