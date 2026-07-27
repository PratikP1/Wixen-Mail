//! Application layer - Business logic and managers
//!
//! This layer contains the core business logic and management components.

pub mod accounts;
pub mod caldav_sync;
pub mod calendar;
pub mod composition;
pub mod contacts;
pub mod contacts_sync;
pub mod filters;
pub mod mail_controller;
pub mod messages;
pub mod notes;
pub mod reminders;
pub mod search;
pub mod tasks;
pub mod threading;

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
