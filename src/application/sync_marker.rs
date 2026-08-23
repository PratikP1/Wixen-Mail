//! The one thing every provider sync does last: write down how far it got,
//! so the next run can ask for only what changed since.
//!
//! `sync_google_contacts`, `sync_microsoft_contacts`, `sync_google_calendar`
//! and `sync_microsoft_calendar` each built this by hand, from the same
//! pieces every time: the row already on file, the account, the marker the
//! provider just handed back, and whether this run read everything rather
//! than a change since last time. The four had already drifted apart by
//! nothing more than which string names the provider and which field holds
//! the marker, Google's `sync_token` or Microsoft's `delta_link`; a shared
//! function is what keeps the next difference between them a choice rather
//! than an accident.

use crate::common::Result;
use crate::data::message_cache::{MessageCache, SyncState};

/// The marker a provider handed back, in whichever of the two forms it uses.
///
/// Google hands back a `sync_token` and Microsoft a `delta_link`; a call site
/// sets the one its provider gave and leaves the other empty. Two fields
/// rather than one, because that is how [`SyncState`] and the table behind it
/// are already shaped, and this is a save, not a redesign of either.
#[derive(Debug)]
pub struct SyncMarker {
    pub sync_token: Option<String>,
    pub delta_link: Option<String>,
}

/// Remember the marker this sync leaves behind, so the next one can ask for
/// only what changed since.
///
/// `state` is the row this account already had, if any. Its id is carried
/// over rather than replaced, so this sync updates that row in place instead
/// of leaving an orphan behind every time it runs. `is_a_full_sync` decides
/// whether this moment becomes the account's new `last_full_sync`, or
/// whether whatever it already had carries forward unchanged.
pub fn remember_this_syncs_marker(
    cache: &MessageCache,
    state: Option<&SyncState>,
    account_id: &str,
    sync_type: &str,
    provider: &str,
    marker: SyncMarker,
    is_a_full_sync: bool,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let new_state = SyncState {
        id: state
            .map(|s| s.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        account_id: account_id.to_string(),
        sync_type: sync_type.to_string(),
        provider: provider.to_string(),
        sync_token: marker.sync_token,
        delta_link: marker.delta_link,
        last_full_sync: if is_a_full_sync {
            Some(now.clone())
        } else {
            state.and_then(|s| s.last_full_sync.clone())
        },
        last_incremental_sync: Some(now),
    };
    cache.save_sync_state(&new_state)
}
