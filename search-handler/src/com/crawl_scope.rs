//! Telling the Windows indexer to come and look, and taking that back.
//!
//! Nothing here can be unit tested, for the same reason nothing else in
//! [`crate::com`] can: it is calls into a COM service that only exists on a real
//! machine. Every decision this makes lives in [`crate::scope`], where it is
//! tested, and this file only carries the answers across.
//!
//! # What this needs to be allowed to do
//!
//! The catalog is `SystemIndex`, which belongs to the Windows Search service and
//! is shared by everybody on the machine. Reading it is open to anybody: asking
//! for the catalog, listing the rules and asking whether a URL is in scope all
//! work from an ordinary prompt. Changing it does not. `AddRoot`,
//! `AddDefaultScopeRule` and `SaveAll` alter what a system service crawls for
//! every account, so they need administrator rights. What a run without them
//! really returns is recorded in the crate's README rather than guessed at here.
//!
//! # Nothing here prints a URL
//!
//! Every URL in this catalog names a real place: a folder in somebody's profile,
//! another application's store, or one of ours, which carries an account name
//! and a folder name. So rules that are not ours are counted and never listed,
//! and the URL the indexer is busy with is described rather than repeated. Those
//! two decisions are in [`crate::scope`] with tests, because they are easy to
//! undo by accident while making a report friendlier.

use crate::scope::{
    Rule, ScopePlan, ScopeState, deciding_rule, describe_url_being_indexed, same_rule,
};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree};
use windows::Win32::System::Search::{
    CATALOG_PAUSED_REASON_DELAYED_RECOVERY, CATALOG_PAUSED_REASON_EXTERNAL,
    CATALOG_PAUSED_REASON_HIGH_CPU, CATALOG_PAUSED_REASON_HIGH_IO,
    CATALOG_PAUSED_REASON_HIGH_NTF_RATE, CATALOG_PAUSED_REASON_LOW_BATTERY,
    CATALOG_PAUSED_REASON_LOW_DISK, CATALOG_PAUSED_REASON_LOW_MEMORY, CATALOG_PAUSED_REASON_NONE,
    CATALOG_PAUSED_REASON_UPGRADING, CATALOG_PAUSED_REASON_USER_ACTIVE, CATALOG_STATUS_FULL_CRAWL,
    CATALOG_STATUS_IDLE, CATALOG_STATUS_INCREMENTAL_CRAWL, CATALOG_STATUS_PAUSED,
    CATALOG_STATUS_PROCESSING_NOTIFICATIONS, CATALOG_STATUS_RECOVERING,
    CATALOG_STATUS_SHUTTING_DOWN, CSearchManager, CSearchRoot, CatalogPausedReason, CatalogStatus,
    ISearchCatalogManager, ISearchCrawlScopeManager, ISearchManager, ISearchRoot, ISearchScopeRule,
};
use windows_core::{HRESULT, HSTRING, PWSTR};

/// The catalog every ordinary Windows Search query reads.
///
/// There is exactly one on a normal machine and this is its name. A second
/// catalog can be made by a server product; adding to one of those would index
/// mail into a place nothing on the desktop searches.
pub const CATALOG: &str = "SystemIndex";

/// Windows' own code for "you are not allowed to do that".
///
/// Written out rather than reached through a helper so the number a person sees
/// in a failure message and the number this compares against are the same one.
const ACCESS_DENIED: HRESULT = HRESULT(0x8007_0005_u32 as i32);

/// Why a crawl scope operation could not be done.
///
/// Each case names the step rather than the call, because the step is what a
/// person can act on. The code is carried through because it is the only thing
/// that separates "not allowed" from "the service is not running", and those
/// have completely different answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    NoSearchService(HRESULT),
    NoCatalog(HRESULT),
    NoCrawlScopeManager(HRESULT),
    CannotRead(HRESULT),
    CannotAddRule(HRESULT),
    CannotAddRoot(HRESULT),
    CannotRemoveRule(HRESULT),
    CannotRemoveRoot(HRESULT),
    CannotSave(HRESULT),
    CannotReindex(HRESULT),
}

impl ScopeError {
    /// The code Windows gave back.
    pub fn code(self) -> HRESULT {
        match self {
            Self::NoSearchService(code)
            | Self::NoCatalog(code)
            | Self::NoCrawlScopeManager(code)
            | Self::CannotRead(code)
            | Self::CannotAddRule(code)
            | Self::CannotAddRoot(code)
            | Self::CannotRemoveRule(code)
            | Self::CannotRemoveRoot(code)
            | Self::CannotSave(code)
            | Self::CannotReindex(code) => code,
        }
    }

    /// Whether this failed because the prompt was not an administrator one.
    ///
    /// Worth telling apart from every other failure, because it is the one a
    /// person can fix in ten seconds and the one they will hit first.
    pub fn needs_administrator(self) -> bool {
        self.code() == ACCESS_DENIED
    }
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doing = match self {
            Self::NoSearchService(_) => "reach the Windows Search service",
            Self::NoCatalog(_) => "open the Windows Search catalog",
            Self::NoCrawlScopeManager(_) => "open the list of places Windows Search looks",
            Self::CannotRead(_) => "read the list of places Windows Search looks",
            Self::CannotAddRule(_) => "add the rule that puts this location in the index",
            Self::CannotAddRoot(_) => "add the starting point for the crawl",
            Self::CannotRemoveRule(_) => "remove the rule",
            Self::CannotRemoveRoot(_) => "remove the starting point",
            Self::CannotSave(_) => "save the change",
            Self::CannotReindex(_) => "ask the indexer to look again",
        };
        write!(f, "could not {doing} (code {:#010X})", self.code().0)?;
        match self.needs_administrator() {
            true => write!(f, ". Run this from an administrator prompt."),
            false => Ok(()),
        }
    }
}

/// The two objects every operation here needs.
struct Catalog {
    catalog: ISearchCatalogManager,
    scope: ISearchCrawlScopeManager,
}

/// Open the system catalog and its crawl scope.
///
/// Reading is enough for this step. It is the writing further down that needs
/// administrator rights, so a `status` run gets this far on any prompt.
fn open() -> Result<Catalog, ScopeError> {
    let manager: ISearchManager = unsafe { CoCreateInstance(&CSearchManager, None, CLSCTX_ALL) }
        .map_err(|e| ScopeError::NoSearchService(e.code()))?;
    let catalog = unsafe { manager.GetCatalog(&HSTRING::from(CATALOG)) }
        .map_err(|e| ScopeError::NoCatalog(e.code()))?;
    let scope = unsafe { catalog.GetCrawlScopeManager() }
        .map_err(|e| ScopeError::NoCrawlScopeManager(e.code()))?;

    Ok(Catalog { catalog, scope })
}

/// Put this handler's URL prefix into the crawl scope.
///
/// Two things go in and both are needed. The rule says the prefix may be
/// indexed. The root is the point the crawl starts from, and a scheme of our own
/// has no file system to inherit one from, so a rule without a root is a
/// permission nothing acts on.
///
/// Running this twice is not an error. The root is only added when it is not
/// already there, and a scope rule for a URL that already has one replaces it,
/// so a second install leaves the same single rule rather than a duplicate.
///
/// `SaveAll` is what commits either of them. Without it the calls above succeed,
/// the object is thrown away, and nothing changed.
pub fn add(plan: &ScopePlan) -> Result<(), ScopeError> {
    let opened = open()?;
    let prefix = HSTRING::from(plan.prefix.as_str());

    if !root_is_registered(&opened.scope, plan)? {
        add_root(&opened.scope, &prefix)?;
    }

    unsafe {
        opened
            .scope
            .AddDefaultScopeRule(&prefix, true, plan.follow_flags)
            .map_err(|e| ScopeError::CannotAddRule(e.code()))?;
        opened
            .scope
            .SaveAll()
            .map_err(|e| ScopeError::CannotSave(e.code()))?;
    }

    Ok(())
}

/// Register the point a crawl of our URLs starts from.
fn add_root(scope: &ISearchCrawlScopeManager, prefix: &HSTRING) -> Result<(), ScopeError> {
    let root: ISearchRoot = unsafe { CoCreateInstance(&CSearchRoot, None, CLSCTX_ALL) }
        .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;

    unsafe {
        root.SetRootURL(prefix)
            .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;
        // The store is a hierarchy: the root holds accounts, an account holds
        // folders, a folder holds messages. Saying so is what makes the indexer
        // walk down it rather than asking about the one URL it was given.
        root.SetIsHierarchical(true)
            .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;
        // This handler has no way to tell the indexer that a message arrived.
        // Claiming otherwise would have the indexer wait for notifications that
        // never come and crawl nothing in between, which looks exactly like the
        // handler not working.
        root.SetProvidesNotifications(false)
            .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;
        root.SetUseNotificationsOnly(false)
            .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;
        scope
            .AddRoot(&root)
            .map_err(|e| ScopeError::CannotAddRoot(e.code()))?;
    }

    Ok(())
}

/// Take this handler's URL prefix back out of the crawl scope.
///
/// What is there is read first and only what is there is removed, rather than
/// calling remove and deciding which failures mean "it was not there". That
/// keeps removal working on a machine where only half of it was ever added,
/// which is what a failed install leaves behind.
///
/// A rule the person added themselves is removed too. Leaving it would leave the
/// indexer asking about a scheme that no longer has a handler, which is the
/// state this whole thing exists to avoid.
pub fn remove(plan: &ScopePlan) -> Result<(), ScopeError> {
    let opened = open()?;
    let prefix = HSTRING::from(plan.prefix.as_str());
    let (ours, _) = read_rules(&opened.scope, plan)?;
    let root_registered = root_is_registered(&opened.scope, plan)?;

    if ours.is_empty() && !root_registered {
        return Ok(());
    }

    unsafe {
        for rule in &ours {
            match rule.is_default {
                true => opened.scope.RemoveDefaultScopeRule(&prefix),
                false => opened.scope.RemoveScopeRule(&prefix),
            }
            .map_err(|e| ScopeError::CannotRemoveRule(e.code()))?;
        }
        if root_registered {
            opened
                .scope
                .RemoveRoot(&prefix)
                .map_err(|e| ScopeError::CannotRemoveRoot(e.code()))?;
        }
        opened
            .scope
            .SaveAll()
            .map_err(|e| ScopeError::CannotSave(e.code()))?;
    }

    Ok(())
}

/// What the crawl scope manager currently says about this handler.
///
/// Read only. This is the call behind the setup tool's `status`, and it is safe
/// to run from an ordinary prompt on a machine somebody is using.
pub fn read_state(plan: &ScopePlan) -> Result<ScopeState, ScopeError> {
    let opened = open()?;
    let (ours, all) = read_rules(&opened.scope, plan)?;
    let sample = HSTRING::from(plan.sample_url.as_str());
    let sample_included = unsafe { opened.scope.IncludedInCrawlScope(&sample) }
        .map_err(|e| ScopeError::CannotRead(e.code()))?
        .as_bool();

    Ok(ScopeState {
        rule: deciding_rule(&ours),
        root_registered: root_is_registered(&opened.scope, plan)?,
        sample_included,
        other_rules: all.saturating_sub(ours.len()),
    })
}

/// Ask the indexer to visit this handler's URLs again.
///
/// Scoped to our own root on purpose. `Reindex` on the catalog would throw away
/// and rebuild the index for the whole machine, which costs hours and is never
/// what somebody testing a mail handler meant.
pub fn reindex(plan: &ScopePlan) -> Result<(), ScopeError> {
    let opened = open()?;
    let prefix = HSTRING::from(plan.prefix.as_str());

    unsafe { opened.catalog.ReindexSearchRoot(&prefix) }
        .map_err(|e| ScopeError::CannotReindex(e.code()))
}

/// What the indexer is doing, in words that carry nothing private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogReport {
    /// What the catalog says it is doing.
    pub status: String,
    /// Why it is paused, when it is.
    pub paused_because: Option<&'static str>,
    /// How many items the whole index holds, across everything on the machine.
    ///
    /// Not a count of mail. There is no supported way to ask the catalog how
    /// many items came from one root, so this is here as a sign of life rather
    /// than an answer about this handler. The README says how to ask a question
    /// that really is about the mail.
    pub items_in_the_whole_index: i32,
    /// One fixed sentence about the URL being worked on, never the URL.
    pub busy_with: &'static str,
}

/// Ask the catalog how it is getting on.
pub fn report() -> Result<CatalogReport, ScopeError> {
    let opened = open()?;
    let mut status = CatalogStatus::default();
    let mut paused = CatalogPausedReason::default();

    unsafe { opened.catalog.GetCatalogStatus(&mut status, &mut paused) }
        .map_err(|e| ScopeError::CannotRead(e.code()))?;
    let items =
        unsafe { opened.catalog.NumberOfItems() }.map_err(|e| ScopeError::CannotRead(e.code()))?;

    // A failure here is not a failure of the report. The indexer being between
    // items is an ordinary thing to catch it doing, and it answers with an
    // error rather than an empty string when it is.
    let busy = unsafe { opened.catalog.URLBeingIndexed() }.ok().map(taken);

    Ok(CatalogReport {
        status: describe_status(status),
        paused_because: describe_pause(paused),
        items_in_the_whole_index: items,
        busy_with: describe_url_being_indexed(busy.as_deref()),
    })
}

/// The rules naming this handler's prefix, and how many rules there are in all.
///
/// One walk for both answers. The list is read out of a live service and can
/// change between calls, so two walks could report more rules of ours than
/// there are rules altogether.
fn read_rules(
    scope: &ISearchCrawlScopeManager,
    plan: &ScopePlan,
) -> Result<(Vec<Rule>, usize), ScopeError> {
    let mut ours = Vec::new();
    let mut seen = 0usize;

    for_each_rule(scope, |rule| {
        seen = seen.saturating_add(1);
        let pattern = unsafe { rule.PatternOrURL() }
            .ok()
            .map(taken)
            .unwrap_or_default();
        if !same_rule(&pattern, &plan.prefix) {
            return Ok(());
        }
        ours.push(Rule {
            includes: unsafe { rule.IsIncluded() }
                .map_err(|e| ScopeError::CannotRead(e.code()))?
                .as_bool(),
            is_default: unsafe { rule.IsDefault() }
                .map_err(|e| ScopeError::CannotRead(e.code()))?
                .as_bool(),
        });
        Ok(())
    })?;

    Ok((ours, seen))
}

/// Walk the catalog's scope rules one at a time.
///
/// One at a time rather than in blocks. The block form saves calls and needs an
/// array of optional interfaces whose unfilled entries have to be got right by
/// hand; this list is read once by a setup tool, and being obviously correct is
/// worth more than the calls saved.
fn for_each_rule(
    scope: &ISearchCrawlScopeManager,
    mut visit: impl FnMut(&ISearchScopeRule) -> Result<(), ScopeError>,
) -> Result<(), ScopeError> {
    let rules =
        unsafe { scope.EnumerateScopeRules() }.map_err(|e| ScopeError::CannotRead(e.code()))?;

    loop {
        let mut one = [None];
        let mut fetched = 0u32;
        // The enumerator answers S_FALSE at the end of the list, and the
        // windows crate turns that into an error rather than a value. Stopping
        // on any error would be the same code as stopping at the end, so how
        // many came back is what decides whether there is one to visit.
        let more = unsafe { rules.Next(&mut one, &mut fetched) }.is_ok();
        match one.first().and_then(Option::as_ref) {
            Some(rule) if fetched == 1 => visit(rule)?,
            _ => return Ok(()),
        }
        if !more {
            return Ok(());
        }
    }
}

/// Whether a search root for this handler's prefix is registered.
fn root_is_registered(
    scope: &ISearchCrawlScopeManager,
    plan: &ScopePlan,
) -> Result<bool, ScopeError> {
    let roots = unsafe { scope.EnumerateRoots() }.map_err(|e| ScopeError::CannotRead(e.code()))?;
    let mut found = false;

    loop {
        let mut one = [None];
        let mut fetched = 0u32;
        let more = unsafe { roots.Next(&mut one, &mut fetched) }.is_ok();
        match one.first().and_then(Option::as_ref) {
            Some(root) if fetched == 1 => {
                let url = unsafe { root.RootURL() }
                    .ok()
                    .map(taken)
                    .unwrap_or_default();
                found = found || same_rule(&url, &plan.prefix);
            }
            _ => return Ok(found),
        }
        if !more {
            return Ok(found);
        }
    }
}

/// Read a string a COM call allocated, and give the memory straight back.
///
/// Every one of these calls hands over a buffer the caller now owns. Reading it
/// and not freeing it leaks once per rule in the catalog, which on a real
/// machine is several dozen per run.
fn taken(text: PWSTR) -> String {
    if text.is_null() {
        return String::new();
    }

    // The pointer came from a COM call that promises a null-terminated string
    // and has not been freed yet, which is what both of these calls need.
    let read = unsafe { text.to_string() }.unwrap_or_default();
    unsafe { CoTaskMemFree(Some(text.0.cast())) };

    read
}

/// What a catalog status means, in words.
fn describe_status(status: CatalogStatus) -> String {
    let known = [
        (CATALOG_STATUS_IDLE, "idle"),
        (CATALOG_STATUS_PAUSED, "paused"),
        (CATALOG_STATUS_RECOVERING, "recovering"),
        (CATALOG_STATUS_FULL_CRAWL, "doing a full crawl"),
        (CATALOG_STATUS_INCREMENTAL_CRAWL, "catching up"),
        (
            CATALOG_STATUS_PROCESSING_NOTIFICATIONS,
            "working through changes it was told about",
        ),
        (CATALOG_STATUS_SHUTTING_DOWN, "shutting down"),
    ];

    known
        .iter()
        .find(|(value, _)| *value == status)
        .map(|(_, said)| (*said).to_string())
        // A status Windows added after this was written. Saying the number is
        // more use than calling it unknown, because it can be looked up.
        .unwrap_or_else(|| format!("status {}", status.0))
}

/// Why the catalog is paused, when it is.
fn describe_pause(reason: CatalogPausedReason) -> Option<&'static str> {
    let known = [
        (CATALOG_PAUSED_REASON_HIGH_IO, "the disk is busy"),
        (CATALOG_PAUSED_REASON_HIGH_CPU, "the processor is busy"),
        (
            CATALOG_PAUSED_REASON_HIGH_NTF_RATE,
            "too many files are changing at once",
        ),
        (CATALOG_PAUSED_REASON_LOW_BATTERY, "the battery is low"),
        (CATALOG_PAUSED_REASON_LOW_MEMORY, "memory is short"),
        (CATALOG_PAUSED_REASON_LOW_DISK, "disk space is short"),
        (
            CATALOG_PAUSED_REASON_DELAYED_RECOVERY,
            "it is waiting to recover",
        ),
        (
            CATALOG_PAUSED_REASON_USER_ACTIVE,
            "somebody is using the computer",
        ),
        (CATALOG_PAUSED_REASON_EXTERNAL, "something else paused it"),
        (CATALOG_PAUSED_REASON_UPGRADING, "it is upgrading"),
    ];

    match reason == CATALOG_PAUSED_REASON_NONE {
        true => None,
        false => Some(
            known
                .iter()
                .find(|(value, _)| *value == reason)
                .map(|(_, said)| *said)
                .unwrap_or("for a reason this tool does not know"),
        ),
    }
}
