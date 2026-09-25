//! Application layer - Business logic and managers
//!
//! This layer contains the core business logic and management components.

pub mod about;
pub mod account_order;
pub mod accounts;
pub mod address_book_source;
pub mod allowed;
pub mod answered_meetings;
pub mod answering;
pub mod asking_when_free;
pub mod attaching;
pub mod autosave;
pub mod blocking;
pub mod body_safety;
pub mod bringing_everything_down;
pub mod caldav_sync;
pub mod calendar;
pub mod calendar_conflict;
pub mod calendar_source;
pub mod carddav_sync;
pub mod categories;
pub mod checking_on_a_schedule;
pub mod checking_signatures;
pub mod choosing_messages;
pub mod closing;
pub mod collection_sync;
pub mod conflict_choice;
pub mod contact_groups;
pub mod contact_names;
pub mod contacts_sync;
pub mod context_menu;
pub mod conversations;
pub mod deleting_at_the_server;
pub mod deletions;
pub mod describing_pictures;
pub mod destinations;
pub mod draft_copy;
pub mod draft_message;
pub mod due;
pub mod editing;
pub mod emptying;
pub mod encrypted_mail;
pub mod event_alerts;
pub mod export_tree;
pub mod favourites;
pub mod feedback_report;
pub mod filing;
pub mod filters;
pub mod finding_what_was_deleted;
pub mod flag_changes_waiting;
pub mod folder_settings;
pub mod folders_underneath;
pub mod font_choice;
pub mod forget;
pub mod from_message;
pub mod handover;
pub mod help;
/// What a sender hid is not read, and a reader is told when words were left out.
pub mod hidden_text;
pub mod how_far_it_got;
pub mod import_tree;
pub mod importing_an_outlook_data_file;
pub mod importing_contacts;
pub mod importing_messages;
pub mod invitations;
pub mod item_fields;
pub mod keeping_message_text;
pub mod links_in_text;
pub mod local_delete;
pub mod local_folders;
pub mod long_text;
pub mod looking_people_up;
pub mod mail_across_accounts;
pub mod mail_auth;
pub mod mail_controller;
pub mod mail_session;
pub mod mail_sync;
pub mod mailto;
pub mod marking_read;
pub mod message_files;
pub mod message_id;
pub mod messages;
pub mod moves_waiting;
pub mod new_item;
pub mod notes_backend;
pub mod notes_sync;
pub mod occurrences;
pub mod opening;
/// Where a link in a message opens, and the one decision from the setting and the ask to the route (#80).
pub mod opening_links;
pub mod opening_pgp;
pub mod other_items;
pub mod phone_numbers;
pub mod pictures;
pub mod pim_command;
pub mod pop_sync;
pub mod printing;
/// What a message shows and says, decided once for every surface that shows one.
pub mod reading_a_message;
pub mod reading_habits;
pub mod reading_style;
pub mod receipts;
pub mod reordering;
pub mod repeating;
pub mod reply;
pub mod running;
pub mod saved_searches;
pub mod scrolling;
pub mod search;
pub mod sending_later;
pub mod sent_copy;
pub mod server_delete;
pub mod server_thread_ids;
pub mod sign_off;
/// Which signature an account's messages start with, one set for every account with
/// one default, and whether a change of From account may replace the block (#43).
pub mod signatures;
/// What a message row's snippet says: the first relevant words, by written rules, so a
/// row read aloud on every arrow press is a hint about the message and not its first address.
pub mod snippet;
pub mod spell_session;
/// The shape every sentence the status bar shows is written to: the one wording for a
/// refusal when nothing was chosen, the words a status sentence may not use, and the endings.
pub mod status_sentences;
pub mod summing_up;
pub mod sync_marker;
pub mod tagging;
pub mod tasks_sync;
pub mod text_history;
pub mod the_network_coming_and_going;
pub mod thread_identity;
pub mod threading;
pub mod time_blocks;
pub mod trying_again;
pub mod what_is_said_while_fetching;
pub mod when_people_are_free;
pub mod who_is_coming;
pub mod words;

pub use accounts::AccountManager;
pub use caldav_sync::{refresh_subscription, sync_caldav_calendar};
pub use filters::FilterEngine;
pub use mail_controller::{MailController, SendEmailRequest};
pub use search::SearchEngine;
