//! A second kind of backend behind the notes seam, and the questions it
//! answers differently.
//!
//! `05.1-03` put one implementation behind
//! [`wixen_mail::application::notes_backend::NotesService`]. One implementation
//! cannot say whether a seam is a seam: everything the contract assumed about
//! backends and everything it assumed about *that* backend look the same from
//! inside it. This file puts a second one there, shaped from what a hosted note
//! service really does, and runs the same test bodies against both.
//!
//! # Why it lives here and not in the crate
//!
//! An integration test links the library the way somebody else's crate would,
//! so it sees only what is `pub`. A second implementation written inside the
//! crate could reach private items and would therefore prove nothing about
//! whether the seam can be implemented from outside it. Everything below is
//! written against the published API and nothing else, which is the stronger of
//! the two placements and is the one that would have failed loudly if the trait
//! or its types had been `pub(crate)`.
//!
//! What that placement costs is that the real CalDAV backend cannot be driven
//! from here: it holds a live client and every call it makes wants a socket. So
//! the first backend is represented by a stand-in with CalDAV's *shape* rather
//! than by the backend itself, and the summary says so rather than claiming a
//! network test that does not exist.
//!
//! # The four ways the second one disagrees, and where each comes from
//!
//! Read from Microsoft's own reference on 2026-09-06 and quoted with their
//! pages in this phase's `README.md`. They are facts with a date, not
//! invention, which is the whole reason a second implementation written the
//! same week as the seam is worth anything at all.
//!
//! 1. **No ETag, and no `If-Match`.** `onenotePage` has no `eTag` property and
//!    the update reference names no `If-Match`. What it has is a
//!    `lastModifiedDateTime` the service owns: a clock reading rather than an
//!    opaque token, which the service can move for its own reasons and which
//!    stands still when two changes land inside one of its ticks. That is what
//!    this one gives, because giving nothing at all is a shape no real service
//!    takes and the plan's own source says so. What giving nothing would cost
//!    is measured in one test below, and it is the argument for the contract
//!    change this plan makes.
//! 2. **No whole-document write.** The supported-actions table gives the `body`
//!    target append yes, replace no, insert no. To make a page say something
//!    else you delete it and make another one.
//! 3. **An identifier that moves.** Deleting and remaking a page gives a new
//!    identifier, and the same page says generated identifiers "might change
//!    after a page update".
//! 4. **A container more than one level deep.** notebook, section group,
//!    section, page. The one here refuses a container that is not three parts.
//!
//! # What this proves and what it does not
//!
//! It is a fake, not a network. Nothing here has met a timeout, a partial
//! response, a redirect, a rate limit or a sign-in that expired mid-operation.
//! Phase 5.2 replaces it with a real Graph client and is required to report
//! every place this one was wrong.

use std::sync::Mutex;

use wixen_mail::application::notes_backend::{
    ANoteAsItStands, ANoteThere, NotesService, WhatTheBackendKept, WhatTheBackendSaid,
};
use wixen_mail::application::notes_sync::{NoteSyncResult, sync_notes};
use wixen_mail::common::Result;
use wixen_mail::data::message_cache::{MessageCache, NoteBody, NoteEntry, NoteFolderEntry};

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

// ── What a test may ask of a backend, whichever one it is ───────────────────

/// The things a test does to a backend that are not part of the seam.
///
/// Standing in for the person at the other end and for the service itself: a
/// note somebody wrote in the web application, a note they changed there, a
/// page they deleted, and the setting on this computer refusing a write before
/// it leaves. None of it is a seam operation and none of it is reachable from
/// [`NotesService`], which is the point: a test body written against this trait
/// and the seam's cannot tell which backend it is driving except by the
/// parameter it was handed.
trait ABackendToDrive: NotesService {
    /// Where this backend's notes go, in whatever shape it insists on.
    fn container(&self) -> String;

    /// A container this backend will not accept.
    fn a_container_it_refuses(&self) -> String;

    /// Somebody wrote a note at the other end. What the backend calls it.
    fn somebody_else_wrote(&self, title: &str, body: &str) -> String;

    /// Somebody changed a note at the other end.
    fn somebody_else_changed(&self, named: &str, title: &str, body: &str);

    /// Somebody deleted a note at the other end.
    fn somebody_else_deleted(&self, named: &str);

    /// What this backend holds now: what it calls each note, its title, its
    /// body.
    fn what_it_holds(&self) -> Vec<(String, String, String)>;

    /// The next write is refused by what this program is allowed to change.
    ///
    /// Real in both backends: the gate is applied where the client is built, so
    /// a client that may not change anything refuses before the network. A
    /// backend with no way to be told to refuse leaves every error branch
    /// untestable by construction.
    fn the_setting_refuses_the_next_write(&self);

    /// How many times a note's whole body has been replaced in one call.
    ///
    /// Zero for a backend that has no such operation, which is the second of
    /// the four disagreements and is asserted rather than described.
    fn whole_bodies_written(&self) -> usize;

    /// Every container string this backend was handed, in order.
    ///
    /// The seam says a container is opaque and is handed back unchanged. That
    /// is checkable only by recording what really arrived.
    fn containers_it_was_handed(&self) -> Vec<String>;

    /// The service wrote to a note without changing what it says.
    ///
    /// Real, and named in the seam's own contract: a marker is not promised to
    /// change only when the content does. A re-index, a move between sections,
    /// a migration. What it costs is the subject of two tests below.
    fn the_other_end_touched_it_without_changing_it(&self, named: &str);
}

// ── The first shape: one document, one address, one strong marker ───────────

/// One note as a document-per-address backend holds it.
#[derive(Clone)]
struct ADocument {
    at: String,
    title: String,
    body: String,
    tag: String,
}

/// A backend shaped like a calendar server's journal collection.
///
/// The shape the seam was written against, standing in for
/// `service::caldav_journal` because that one cannot be driven without a
/// socket. Four properties, and each is the opposite of one of the four above:
/// a whole document is written in one call, the address the client chose is the
/// address for ever, the marker is an opaque token the server changes exactly
/// when the document changes, and the collection is one string of any shape.
struct ACollectionOfDocuments {
    documents: Mutex<Vec<ADocument>>,
    /// What the next tag will be built from. A token, never a clock.
    tags_given: Mutex<u64>,
    setting_refuses_the_next_write: Mutex<bool>,
    whole_bodies_written: Mutex<usize>,
    containers: Mutex<Vec<String>>,
}

impl ACollectionOfDocuments {
    fn new() -> Self {
        Self {
            documents: Mutex::new(Vec::new()),
            tags_given: Mutex::new(0),
            setting_refuses_the_next_write: Mutex::new(false),
            whole_bodies_written: Mutex::new(0),
            containers: Mutex::new(Vec::new()),
        }
    }

    fn a_new_tag(&self) -> String {
        let mut given = self.tags_given.lock().expect("the tag counter");
        *given += 1;
        format!("\"tag-{given}\"")
    }

    fn saw(&self, container: &str) {
        self.containers
            .lock()
            .expect("the containers seen")
            .push(container.to_string());
    }

    fn refused_by_the_setting(&self) -> bool {
        let mut refusing = self
            .setting_refuses_the_next_write
            .lock()
            .expect("the setting");
        std::mem::replace(&mut refusing, false)
    }
}

impl NotesService for ACollectionOfDocuments {
    async fn notes_it_holds(&self, container: &str) -> Result<Vec<ANoteThere>> {
        self.saw(container);
        Ok(self
            .documents
            .lock()
            .expect("the documents")
            .iter()
            .map(|held| ANoteThere {
                named: held.at.clone(),
                version: Some(held.tag.clone()),
            })
            .collect())
    }

    async fn what_a_note_says(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> Result<Option<ANoteAsItStands>> {
        self.saw(container);
        Ok(self
            .documents
            .lock()
            .expect("the documents")
            .iter()
            .find(|held| held.at == known_as.named)
            .map(|held| ANoteAsItStands {
                known_as: ANoteThere {
                    named: held.at.clone(),
                    version: Some(held.tag.clone()),
                },
                title: held.title.clone(),
                body: held.body.clone(),
            }))
    }

    async fn leave_a_note_saying(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        title: &str,
        body: &str,
    ) -> Result<WhatTheBackendSaid> {
        self.saw(container);
        if self.refused_by_the_setting() {
            return Ok(WhatTheBackendSaid::NotAllowedToChangeAnything);
        }
        let tag = self.a_new_tag();
        let mut documents = self.documents.lock().expect("the documents");
        let Some(there) = known_as else {
            let at = format!("{container}/{}.ics", documents.len() + 1);
            documents.push(ADocument {
                at: at.clone(),
                title: title.to_string(),
                body: body.to_string(),
                tag: tag.clone(),
            });
            *self
                .whole_bodies_written
                .lock()
                .expect("the whole-body count") += 1;
            return Ok(WhatTheBackendSaid::done(ANoteThere {
                named: at,
                version: Some(tag),
            }));
        };
        let Some(held) = documents.iter_mut().find(|held| held.at == there.named) else {
            return Ok(WhatTheBackendSaid::ItIsNotThere);
        };
        if there.version.as_deref() != Some(held.tag.as_str()) {
            return Ok(WhatTheBackendSaid::ItMovedFirst {
                version_now: Some(held.tag.clone()),
            });
        }
        // The whole document goes in one call, which is the operation the other
        // backend does not have.
        held.title = title.to_string();
        held.body = body.to_string();
        held.tag = tag.clone();
        *self
            .whole_bodies_written
            .lock()
            .expect("the whole-body count") += 1;
        Ok(WhatTheBackendSaid::done(ANoteThere {
            named: there.named.clone(),
            version: Some(tag),
        }))
    }

    async fn take_a_note_away(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> Result<WhatTheBackendSaid> {
        self.saw(container);
        if self.refused_by_the_setting() {
            return Ok(WhatTheBackendSaid::NotAllowedToChangeAnything);
        }
        let mut documents = self.documents.lock().expect("the documents");
        match documents.iter().position(|held| held.at == known_as.named) {
            Some(at) => {
                documents.remove(at);
                Ok(WhatTheBackendSaid::done(known_as.clone()))
            }
            None => Ok(WhatTheBackendSaid::ItIsNotThere),
        }
    }
}

impl ABackendToDrive for ACollectionOfDocuments {
    fn container(&self) -> String {
        "https://example.test/dav/journals".to_string()
    }

    fn a_container_it_refuses(&self) -> String {
        // It refuses nothing, which is a fact about this shape rather than a
        // gap in the fixture. A collection address is one string and any string
        // is one.
        self.container()
    }

    fn somebody_else_wrote(&self, title: &str, body: &str) -> String {
        let tag = self.a_new_tag();
        let mut documents = self.documents.lock().expect("the documents");
        let at = format!("{}/{}.ics", self.container(), documents.len() + 1);
        documents.push(ADocument {
            at: at.clone(),
            title: title.to_string(),
            body: body.to_string(),
            tag,
        });
        at
    }

    fn somebody_else_changed(&self, named: &str, title: &str, body: &str) {
        let tag = self.a_new_tag();
        let mut documents = self.documents.lock().expect("the documents");
        let held = documents
            .iter_mut()
            .find(|held| held.at == named)
            .expect("a document somebody else could change");
        held.title = title.to_string();
        held.body = body.to_string();
        held.tag = tag;
    }

    fn somebody_else_deleted(&self, named: &str) {
        let mut documents = self.documents.lock().expect("the documents");
        let at = documents
            .iter()
            .position(|held| held.at == named)
            .expect("a document somebody else could delete");
        documents.remove(at);
    }

    fn what_it_holds(&self) -> Vec<(String, String, String)> {
        self.documents
            .lock()
            .expect("the documents")
            .iter()
            .map(|held| (held.at.clone(), held.title.clone(), held.body.clone()))
            .collect()
    }

    fn the_setting_refuses_the_next_write(&self) {
        *self
            .setting_refuses_the_next_write
            .lock()
            .expect("the setting") = true;
    }

    fn whole_bodies_written(&self) -> usize {
        *self
            .whole_bodies_written
            .lock()
            .expect("the whole-body count")
    }

    fn containers_it_was_handed(&self) -> Vec<String> {
        self.containers.lock().expect("the containers seen").clone()
    }

    fn the_other_end_touched_it_without_changing_it(&self, named: &str) {
        let tag = self.a_new_tag();
        let mut documents = self.documents.lock().expect("the documents");
        let held = documents
            .iter_mut()
            .find(|held| held.at == named)
            .expect("a document the server could touch");
        held.tag = tag;
    }
}

// ── The second shape: a page in a section, appended to and never replaced ───

/// One note as a page-in-a-section backend holds it.
#[derive(Clone)]
struct APage {
    name: String,
    title: String,
    /// What the service really keeps, which is the text of an HTML document
    /// rather than the bytes it was handed.
    body: String,
    /// The service's own clock when this page was last written.
    changed_at: String,
}

/// A backend shaped like pages in a section of a notebook.
///
/// The four disagreements, each as a refusal rather than as a comment. Its
/// marker is a reading of a clock the service owns rather than an opaque token,
/// so it moves for reasons the content did not cause and stands still when two
/// changes land inside one tick; it has no operation that replaces a body, so
/// reaching an asked-for end state means removing the page and making another;
/// that gives the note a new name every time its body changes; and its
/// container is three parts and it refuses anything else.
struct ASectionOfPages {
    pages: Mutex<Vec<APage>>,
    names_given: Mutex<u64>,
    /// The service's clock, in minutes past an hour nobody cares about. A
    /// timestamp rather than a counter, because the shape is the point: nothing
    /// reading this may order it, parse it as a date or read a change in it as
    /// proof the text changed.
    clock: Mutex<u64>,
    /// The clock does not advance while this is set, which is how two changes
    /// inside one of the service's ticks are driven.
    clock_stands_still: Mutex<bool>,
    /// This one answers no marker at all, which no real service does.
    ///
    /// Its own field rather than its own backend, because it exists for exactly
    /// one test: the measurement that says what the seam's "a version marker is
    /// optional" really costs. A second whole implementation for one assertion
    /// would be a second thing to keep working, and this project has already
    /// written down that a second way to script a fake is worth avoiding.
    never_gives_a_marker: bool,
    setting_refuses_the_next_write: Mutex<bool>,
    containers: Mutex<Vec<String>>,
}

/// What the service keeps when it is handed a body.
///
/// A page is an HTML document, and HTML collapses a run of whitespace to one
/// space and cannot hold a trailing one at the end of a line. So the bytes that
/// come back are the bytes that went in only when the bytes that went in had
/// none of either. That is a property of the format, in the same way the
/// calendar format's single escape for a line break is, and no client speaking
/// it can do better.
fn as_a_page_keeps_it(body: &str) -> String {
    body.lines()
        .map(|line| {
            line.split([' ', '\t'])
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// What a page really kept, where that is not what it was handed.
///
/// Answered from the page itself rather than from the rule, so that a change to
/// what the service does cannot leave this saying it kept what it did not.
fn what_a_page_could_keep(page: &APage, title: &str, body: &str) -> Option<WhatTheBackendKept> {
    if page.title == title && page.body == body {
        return None;
    }
    Some(WhatTheBackendKept {
        title: page.title.clone(),
        body: page.body.clone(),
    })
}

impl ASectionOfPages {
    fn new() -> Self {
        Self {
            pages: Mutex::new(Vec::new()),
            names_given: Mutex::new(0),
            clock: Mutex::new(0),
            clock_stands_still: Mutex::new(false),
            never_gives_a_marker: false,
            setting_refuses_the_next_write: Mutex::new(false),
            containers: Mutex::new(Vec::new()),
        }
    }

    /// The same backend with the one thing no real service does.
    ///
    /// Not a shape anybody should ship. It exists so that the cost of the
    /// contract's sentence "a version marker is optional" can be measured
    /// rather than argued about.
    fn that_gives_no_marker() -> Self {
        Self {
            never_gives_a_marker: true,
            ..Self::new()
        }
    }

    /// The name the service generates. Never one the client chose.
    fn a_new_name(&self) -> String {
        let mut given = self.names_given.lock().expect("the name counter");
        *given += 1;
        format!("1-page{given}!OneNote")
    }

    /// The clock reading a write is stamped with.
    ///
    /// It advances first, so two writes get two readings, unless the test has
    /// held it still to drive two changes inside one of the service's ticks.
    fn now(&self) -> String {
        let mut clock = self.clock.lock().expect("the service's clock");
        if !*self.clock_stands_still.lock().expect("whether it advances") {
            *clock += 1;
        }
        format!("2026-09-10T09:{:02}:00Z", *clock)
    }

    /// What this backend says a page's marker is.
    fn the_marker_of(&self, page: &APage) -> Option<String> {
        if self.never_gives_a_marker {
            return None;
        }
        Some(page.changed_at.clone())
    }

    /// A container is a notebook, a section group and a section.
    ///
    /// Refused rather than guessed at. A backend handed one level of a
    /// four-level hierarchy has been handed another backend's addressing, and
    /// answering anyway would let the seam go on assuming a container is a URL.
    fn a_section(container: &str) -> std::result::Result<(), WhatTheBackendSaid> {
        if container.split('/').filter(|part| !part.is_empty()).count() == 3 {
            return Ok(());
        }
        Err(WhatTheBackendSaid::CouldNotBeReached(format!(
            "{container} is not a notebook, a section group and a section"
        )))
    }

    fn saw(&self, container: &str) {
        self.containers
            .lock()
            .expect("the containers seen")
            .push(container.to_string());
    }

    fn refused_by_the_setting(&self) -> bool {
        let mut refusing = self
            .setting_refuses_the_next_write
            .lock()
            .expect("the setting");
        std::mem::replace(&mut refusing, false)
    }
}

impl NotesService for ASectionOfPages {
    async fn notes_it_holds(&self, container: &str) -> Result<Vec<ANoteThere>> {
        self.saw(container);
        if let Err(refused) = Self::a_section(container) {
            return Err(wixen_mail::common::Error::Other(format!("{refused:?}")));
        }
        Ok(self
            .pages
            .lock()
            .expect("the pages")
            .iter()
            .map(|page| ANoteThere {
                named: page.name.clone(),
                version: self.the_marker_of(page),
            })
            .collect())
    }

    async fn what_a_note_says(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> Result<Option<ANoteAsItStands>> {
        self.saw(container);
        if let Err(refused) = Self::a_section(container) {
            return Err(wixen_mail::common::Error::Other(format!("{refused:?}")));
        }
        Ok(self
            .pages
            .lock()
            .expect("the pages")
            .iter()
            .find(|page| page.name == known_as.named)
            .map(|page| ANoteAsItStands {
                known_as: ANoteThere {
                    named: page.name.clone(),
                    version: self.the_marker_of(page),
                },
                title: page.title.clone(),
                body: page.body.clone(),
            }))
    }

    async fn leave_a_note_saying(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        title: &str,
        body: &str,
    ) -> Result<WhatTheBackendSaid> {
        self.saw(container);
        if let Err(refused) = Self::a_section(container) {
            return Ok(refused);
        }
        if self.refused_by_the_setting() {
            return Ok(WhatTheBackendSaid::NotAllowedToChangeAnything);
        }
        let name = self.a_new_name();
        let now = self.now();
        let kept = as_a_page_keeps_it(body);
        let mut pages = self.pages.lock().expect("the pages");
        let Some(there) = known_as else {
            let page = APage {
                name: name.clone(),
                title: title.to_string(),
                body: kept,
                changed_at: now,
            };
            let said = self.the_marker_of(&page);
            let could_keep = what_a_page_could_keep(&page, title, body);
            pages.push(page);
            return Ok(WhatTheBackendSaid::Done {
                known_as: ANoteThere {
                    named: name,
                    version: said,
                },
                what_it_could_keep: could_keep,
            });
        };
        let Some(at) = pages.iter().position(|page| page.name == there.named) else {
            return Ok(WhatTheBackendSaid::ItIsNotThere);
        };
        // Read before writing, because there is no `If-Match` to enforce it at
        // the service. The window between this comparison and the write is real
        // and is one of the things this fake cannot show.
        let held = self.the_marker_of(&pages[at]);
        if held.is_some() && held != there.version {
            return Ok(WhatTheBackendSaid::ItMovedFirst { version_now: held });
        }
        if pages[at].body == kept {
            // Only the title moved, and a title can be replaced in one call.
            // The page survives and so does its name.
            pages[at].title = title.to_string();
            pages[at].changed_at = now;
            let said = self.the_marker_of(&pages[at]);
            let could_keep = what_a_page_could_keep(&pages[at], title, body);
            return Ok(WhatTheBackendSaid::Done {
                known_as: ANoteThere {
                    named: there.named.clone(),
                    version: said,
                },
                what_it_could_keep: could_keep,
            });
        }
        // A body cannot be replaced. Reaching the asked-for end state means
        // removing this page and making another, and the note is called
        // something else afterwards.
        pages.remove(at);
        let page = APage {
            name: name.clone(),
            title: title.to_string(),
            body: kept,
            changed_at: now,
        };
        let said = self.the_marker_of(&page);
        let could_keep = what_a_page_could_keep(&page, title, body);
        pages.push(page);
        Ok(WhatTheBackendSaid::Done {
            known_as: ANoteThere {
                named: name,
                version: said,
            },
            what_it_could_keep: could_keep,
        })
    }

    async fn take_a_note_away(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> Result<WhatTheBackendSaid> {
        self.saw(container);
        if let Err(refused) = Self::a_section(container) {
            return Ok(refused);
        }
        if self.refused_by_the_setting() {
            return Ok(WhatTheBackendSaid::NotAllowedToChangeAnything);
        }
        let mut pages = self.pages.lock().expect("the pages");
        match pages.iter().position(|page| page.name == known_as.named) {
            Some(at) => {
                pages.remove(at);
                Ok(WhatTheBackendSaid::done(known_as.clone()))
            }
            None => Ok(WhatTheBackendSaid::ItIsNotThere),
        }
    }
}

impl ABackendToDrive for ASectionOfPages {
    fn container(&self) -> String {
        "notebook-work/group-reference/section-notes".to_string()
    }

    fn a_container_it_refuses(&self) -> String {
        "section-notes".to_string()
    }

    fn somebody_else_wrote(&self, title: &str, body: &str) -> String {
        let name = self.a_new_name();
        let now = self.now();
        self.pages.lock().expect("the pages").push(APage {
            name: name.clone(),
            title: title.to_string(),
            body: as_a_page_keeps_it(body),
            changed_at: now,
        });
        name
    }

    fn somebody_else_changed(&self, named: &str, title: &str, body: &str) {
        let now = self.now();
        let mut pages = self.pages.lock().expect("the pages");
        let page = pages
            .iter_mut()
            .find(|page| page.name == named)
            .expect("a page somebody else could change");
        page.title = title.to_string();
        page.body = as_a_page_keeps_it(body);
        page.changed_at = now;
    }

    fn somebody_else_deleted(&self, named: &str) {
        let mut pages = self.pages.lock().expect("the pages");
        let at = pages
            .iter()
            .position(|page| page.name == named)
            .expect("a page somebody else could delete");
        pages.remove(at);
    }

    fn what_it_holds(&self) -> Vec<(String, String, String)> {
        self.pages
            .lock()
            .expect("the pages")
            .iter()
            .map(|page| (page.name.clone(), page.title.clone(), page.body.clone()))
            .collect()
    }

    fn the_setting_refuses_the_next_write(&self) {
        *self
            .setting_refuses_the_next_write
            .lock()
            .expect("the setting") = true;
    }

    fn whole_bodies_written(&self) -> usize {
        // There is no such operation, so the count cannot be anything else.
        0
    }

    fn containers_it_was_handed(&self) -> Vec<String> {
        self.containers.lock().expect("the containers seen").clone()
    }

    fn the_other_end_touched_it_without_changing_it(&self, named: &str) {
        let now = self.now();
        let mut pages = self.pages.lock().expect("the pages");
        let page = pages
            .iter_mut()
            .find(|page| page.name == named)
            .expect("a page the service could touch");
        page.changed_at = now;
    }
}

impl ASectionOfPages {
    /// Two changes land inside one of the service's ticks from here on.
    ///
    /// Not a trick. A `lastModifiedDateTime` has a resolution, and two writes
    /// inside it carry the same reading, which is the one thing an ETag never
    /// does and the one direction the seam's contract had not written down.
    fn the_services_clock_stands_still(&self) {
        *self.clock_stands_still.lock().expect("whether it advances") = true;
    }
}

// ── The fixtures every body below shares ────────────────────────────────────

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().to_path_buf(), None).expect("a store to write into")
}

fn run<F: std::future::Future>(work: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime")
        .block_on(work)
}

/// The note folder one container is, made if it is not there yet.
///
/// What `notes_backend` does before it asks for a sync, and the reason it has
/// to happen in these fixtures too: one backend container is one note folder,
/// so a note waiting to be sent to a container has to be in that container's
/// folder or the push is not about it.
///
/// Idempotent, so the two fixtures below can both ask without either having to
/// know whether the other went first.
fn the_folder_for(cache: &MessageCache, container: &str) -> NoteFolderEntry {
    cache
        .a_note_folder_for(ACCOUNT, container, "Notes")
        .expect("the folder this container is")
}

/// A note somebody made here, in this container's folder, waiting to be sent.
fn a_note_made_here(
    cache: &MessageCache,
    container: &str,
    id: &str,
    title: &str,
    body: &str,
) -> NoteEntry {
    let folder = the_folder_for(cache, container);
    let now = "2026-09-10T09:00:00Z".to_string();
    let note = NoteEntry {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        folder_id: Some(folder.id),
        title: title.to_string(),
        body: body.to_string(),
        format: NoteBody::AsTyped,
        pinned: false,
        created_at: now.clone(),
        updated_at: now,
        pending: true,
        known_as: None,
        known_version: None,
    };
    cache.save_note(&note).expect("a note to store");
    note
}

/// The note as it stands here now.
fn the_note_here(cache: &MessageCache, id: &str) -> NoteEntry {
    cache
        .get_all_notes_for_account(ACCOUNT)
        .expect("the notes here")
        .into_iter()
        .find(|note| note.id == id)
        .expect("the note this test is about")
}

/// One sync of this account's notes against this backend.
fn a_sync<S: NotesService + ABackendToDrive>(cache: &MessageCache, backend: &S) -> NoteSyncResult {
    the_folder_for(cache, &backend.container());
    run(sync_notes(cache, backend, ACCOUNT, &backend.container())).expect("a sync")
}

// ── The same question of both backends ──────────────────────────────────────
//
// Every body below is written once, against the seam's trait and
// [`ABackendToDrive`] and nothing else, so a reader cannot tell which backend
// an assertion is about except by the parameter. Under each one sit the two
// tests that run it, one line each.
//
// **They are written out rather than generated, and that is not a style
// choice.** A macro taking the body and the two names is three lines shorter
// and makes both of them invisible to this project's own tooling:
// `test_every_test_a_guard_record_names_is_a_test_that_exists` looks for
// `fn <name>(` in the suite file, and a name that reaches the compiler only as
// a macro argument is never written that way. A guard record naming one is
// refused on every commit, which was found by writing the macro first and
// watching the check speak.

// ── A note out, and back, whatever the backend calls it ─────────────────────

fn a_note_made_here_reaches_the_backend<S: NotesService + ABackendToDrive>(backend: &S) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );

    let did = a_sync(&cache, backend);

    assert_eq!(did.sent, 1, "{did:?}");
    assert!(did.errors.is_empty(), "{did:?}");

    let held = backend.what_it_holds();
    assert_eq!(held.len(), 1, "{held:?}");
    assert_eq!(held[0].1, "Wiring colours");

    // Identified afterwards by whatever the backend calls it, which is not what
    // the client asked for and is never parsed here.
    let after = the_note_here(&cache, "note-1");
    assert_eq!(
        after.known_as.as_deref(),
        Some(held[0].0.as_str()),
        "the note here is not called what the backend called it"
    );
    assert!(
        !after.pending,
        "the note is still waiting after it was sent"
    );
}

#[test]
fn test_a_note_made_here_reaches_a_collection_of_documents() {
    a_note_made_here_reaches_the_backend(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_note_made_here_reaches_a_section_of_pages() {
    a_note_made_here_reaches_the_backend(&ASectionOfPages::new());
}

fn a_note_changed_here_is_written_without_replacing_a_whole_body<
    S: NotesService + ABackendToDrive,
>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);

    let sent = the_note_here(&cache, "note-1");
    let changed = NoteEntry {
        body: "Live is brown. Neutral is blue.".to_string(),
        pending: true,
        ..sent.clone()
    };
    cache.save_note(&changed).expect("the change to store");

    let did = a_sync(&cache, backend);

    assert_eq!(did.sent, 1, "{did:?}");
    assert!(did.errors.is_empty(), "{did:?}");
    let held = backend.what_it_holds();
    assert_eq!(held.len(), 1, "a second note was made instead: {held:?}");
    assert!(held[0].2.contains("Neutral is blue"), "{held:?}");

    // Whatever it is called now, that is what this computer calls it. A caller
    // left holding the old name is a caller whose next write makes a second
    // note, which is what the seam's contract says about a write that may
    // change an identity.
    let after = the_note_here(&cache, "note-1");
    assert_eq!(after.known_as.as_deref(), Some(held[0].0.as_str()));
}

#[test]
fn test_a_note_changed_here_reaches_a_collection_of_documents() {
    a_note_changed_here_is_written_without_replacing_a_whole_body(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_note_changed_here_reaches_a_section_of_pages() {
    a_note_changed_here_is_written_without_replacing_a_whole_body(&ASectionOfPages::new());
}

fn a_note_changed_at_the_backend_arrives_here<S: NotesService + ABackendToDrive>(backend: &S) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);
    let named = the_note_here(&cache, "note-1")
        .known_as
        .expect("a name from the backend");

    backend.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");

    let did = a_sync(&cache, backend);

    let after = the_note_here(&cache, "note-1");
    assert!(
        after.body.contains("Earth is green"),
        "a change made at the backend never arrived: {after:?} {did:?}"
    );
}

#[test]
fn test_a_change_at_a_collection_of_documents_arrives_here() {
    a_note_changed_at_the_backend_arrives_here(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_change_at_a_section_of_pages_arrives_here() {
    a_note_changed_at_the_backend_arrives_here(&ASectionOfPages::new());
}

fn a_note_only_the_backend_has_arrives_here<S: NotesService + ABackendToDrive>(backend: &S) {
    // The operation `05.1-03` had to add to the seam, asked of a second
    // backend: a listing answers identities, so without a way to read a note's
    // words a sync can learn that a backend holds a note it has never seen and
    // have nothing to write down.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    backend.somebody_else_wrote("Bins", "Green bin on Tuesday.");

    let did = a_sync(&cache, backend);

    assert_eq!(did.stored, 1, "{did:?}");
    let here = cache
        .get_all_notes_for_account(ACCOUNT)
        .expect("the notes here");
    assert_eq!(here.len(), 1, "{here:?}");
    assert_eq!(here[0].title, "Bins");
    assert_eq!(here[0].body, "Green bin on Tuesday.");
    assert!(
        !here[0].pending,
        "a note that arrived is waiting to be sent"
    );
}

#[test]
fn test_a_document_only_the_backend_has_arrives_here() {
    a_note_only_the_backend_has_arrives_here(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_page_only_the_backend_has_arrives_here() {
    a_note_only_the_backend_has_arrives_here(&ASectionOfPages::new());
}

// ── The conflict, which is where a seam written about one backend shows ─────

fn a_note_changed_in_both_places_is_held_for_somebody_to_choose<
    S: NotesService + ABackendToDrive,
>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);
    let sent = the_note_here(&cache, "note-1");
    let named = sent.known_as.clone().expect("a name from the backend");

    // Both copies move between syncs.
    backend.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");
    let changed_here = NoteEntry {
        body: "Live is brown. Neutral is blue.".to_string(),
        pending: true,
        ..sent
    };
    cache.save_note(&changed_here).expect("the change to store");

    let did = a_sync(&cache, backend);

    assert_eq!(
        did.held, 1,
        "a note that moved in two places was not held for anybody to choose: {did:?}"
    );
    assert!(
        cache
            .is_held_for_a_choice("note-1")
            .expect("whether it is held"),
        "nothing is waiting for a choice"
    );
    let after = the_note_here(&cache, "note-1");
    assert_eq!(
        after.body, "Live is brown. Neutral is blue.",
        "the copy typed here was written over before anybody chose"
    );
}

#[test]
fn test_both_copies_of_a_document_are_held_for_a_choice() {
    a_note_changed_in_both_places_is_held_for_somebody_to_choose(&ACollectionOfDocuments::new());
}

#[test]
fn test_both_copies_of_a_page_are_held_for_a_choice() {
    a_note_changed_in_both_places_is_held_for_somebody_to_choose(&ASectionOfPages::new());
}

/// Two copies that moved to the same words are not a question.
///
/// The other half of the body above, and the one a backend whose marker is a
/// clock meets far more often. A marker that moved is the whole of what the
/// push has to go on, so it reports a clash; but a clash is two copies that
/// disagree, and `conflict_choice` exists to ask which to keep. Asked about two
/// identical copies it is a question with one answer given twice, read aloud as
/// a title and a body that are the same on both sides.
///
/// It happens for a reason nobody would call an edge case: somebody fixes the
/// same typo in both places. It happens far more often on a backend whose
/// marker the service writes for its own reasons, because then any touch at all
/// puts a waiting change into this state.
fn a_note_changed_to_what_the_backend_already_says_is_not_a_question<
    S: NotesService + ABackendToDrive,
>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);
    let sent = the_note_here(&cache, "note-1");
    let named = sent.known_as.clone().expect("a name from the backend");

    // The same words arrive at both ends. The marker moves, so the push has
    // every reason to think the copies disagree, and they do not.
    backend.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");
    let changed_here = NoteEntry {
        body: "Live is brown. Earth is green.".to_string(),
        pending: true,
        ..sent
    };
    cache.save_note(&changed_here).expect("the change to store");

    let did = a_sync(&cache, backend);

    assert_eq!(
        did.held, 0,
        "two copies saying the same thing were held for somebody to choose between: {did:?}"
    );
    assert!(
        !cache
            .is_held_for_a_choice("note-1")
            .expect("whether it is held"),
        "somebody is being asked to choose between two copies that agree"
    );
    let after = the_note_here(&cache, "note-1");
    assert!(
        !after.pending,
        "a note the backend already agrees with is still waiting to be sent"
    );
    // And the marker is written down, or the next sync asks the same
    // unanswerable question again.
    assert!(
        after.known_version.is_some(),
        "the marker the backend reported was not written down"
    );
}

#[test]
fn test_a_document_changed_to_what_the_server_already_says_is_not_a_question() {
    a_note_changed_to_what_the_backend_already_says_is_not_a_question(
        &ACollectionOfDocuments::new(),
    );
}

#[test]
fn test_a_page_changed_to_what_onenote_already_says_is_not_a_question() {
    a_note_changed_to_what_the_backend_already_says_is_not_a_question(&ASectionOfPages::new());
}

fn a_change_the_setting_held_is_not_written_over_by_the_read<S: NotesService + ABackendToDrive>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);
    let sent = the_note_here(&cache, "note-1");
    let named = sent.known_as.clone().expect("a name from the backend");

    backend.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");
    let changed_here = NoteEntry {
        body: "Live is brown. Neutral is blue.".to_string(),
        pending: true,
        ..sent
    };
    cache.save_note(&changed_here).expect("the change to store");
    backend.the_setting_refuses_the_next_write();

    let did = a_sync(&cache, backend);

    assert_eq!(did.waiting_on_the_setting, 1, "{did:?}");
    let after = the_note_here(&cache, "note-1");
    assert_eq!(
        after.body, "Live is brown. Neutral is blue.",
        "the setting held the change here and the read destroyed it"
    );
    assert!(
        after.pending
            || cache
                .is_held_for_a_choice("note-1")
                .expect("whether it is held"),
        "the change is neither still waiting nor waiting on a choice, so it is gone"
    );
}

#[test]
fn test_the_setting_and_a_document_that_moved_do_not_lose_the_change() {
    a_change_the_setting_held_is_not_written_over_by_the_read(&ACollectionOfDocuments::new());
}

#[test]
fn test_the_setting_and_a_page_that_moved_do_not_lose_the_change() {
    a_change_the_setting_held_is_not_written_over_by_the_read(&ASectionOfPages::new());
}

// ── A name the backend no longer knows ──────────────────────────────────────

fn a_note_the_backend_no_longer_holds_is_made_again<S: NotesService + ABackendToDrive>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);
    let sent = the_note_here(&cache, "note-1");
    let named = sent.known_as.clone().expect("a name from the backend");

    // Somebody removed it at the other end, and there is a change here waiting.
    backend.somebody_else_deleted(&named);
    let changed_here = NoteEntry {
        body: "Live is brown. Neutral is blue.".to_string(),
        pending: true,
        ..sent
    };
    cache.save_note(&changed_here).expect("the change to store");

    let did = a_sync(&cache, backend);

    // The seam's contract says a name the backend has never given is not a
    // name: a note this program has that the backend does not know about is a
    // note to create. Reporting it instead leaves the change here for ever and
    // says the same thing on every sync from now on.
    assert!(
        did.errors.is_empty(),
        "a note the backend no longer holds was reported rather than made again: {did:?}"
    );
    assert_eq!(did.sent, 1, "{did:?}");
    let held = backend.what_it_holds();
    assert_eq!(held.len(), 1, "{held:?}");
    assert!(held[0].2.contains("Neutral is blue"), "{held:?}");
    let after = the_note_here(&cache, "note-1");
    assert!(!after.pending, "the change is still waiting to be sent");
    assert_eq!(after.known_as.as_deref(), Some(held[0].0.as_str()));
}

#[test]
fn test_a_document_the_backend_no_longer_holds_is_made_again() {
    a_note_the_backend_no_longer_holds_is_made_again(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_page_the_backend_no_longer_holds_is_made_again() {
    a_note_the_backend_no_longer_holds_is_made_again(&ASectionOfPages::new());
}

// ── Deletion, and the container ─────────────────────────────────────────────

fn a_note_deleted_here_is_taken_away_and_does_not_come_back<S: NotesService + ABackendToDrive>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);

    cache.delete_note("note-1").expect("the note to go");
    let did = a_sync(&cache, backend);

    assert_eq!(did.sent, 1, "{did:?}");
    assert!(
        backend.what_it_holds().is_empty(),
        "the backend still holds a note somebody deleted here"
    );
    assert!(
        cache
            .get_all_notes_for_account(ACCOUNT)
            .expect("the notes here")
            .is_empty(),
        "the note came back"
    );
}

#[test]
fn test_a_deleted_document_stays_deleted() {
    a_note_deleted_here_is_taken_away_and_does_not_come_back(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_deleted_page_stays_deleted() {
    a_note_deleted_here_is_taken_away_and_does_not_come_back(&ASectionOfPages::new());
}

fn the_container_is_handed_back_exactly_as_it_was_given<S: NotesService + ABackendToDrive>(
    backend: &S,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, backend);

    let handed = backend.containers_it_was_handed();
    assert!(
        !handed.is_empty(),
        "the backend was never given a container"
    );
    for one in &handed {
        assert_eq!(
            one,
            &backend.container(),
            "the container was taken apart or built somewhere before it arrived"
        );
    }
}

#[test]
fn test_a_collection_address_arrives_as_it_was_given() {
    the_container_is_handed_back_exactly_as_it_was_given(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_three_part_section_arrives_as_it_was_given() {
    the_container_is_handed_back_exactly_as_it_was_given(&ASectionOfPages::new());
}

// ── The two questions that are about one backend each ───────────────────────

#[test]
fn test_a_backend_with_no_operation_that_replaces_a_body_is_never_asked_for_one() {
    // The second of the four disagreements, asserted rather than described.
    // The seam asks for an end state, and the count of whole-body writes a
    // page-in-a-section backend performed is the evidence that "write the whole
    // document" was never the operation.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let pages = ASectionOfPages::new();
    a_note_made_here(
        &cache,
        &pages.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, &pages);
    let sent = the_note_here(&cache, "note-1");
    cache
        .save_note(&NoteEntry {
            body: "Live is brown. Neutral is blue.".to_string(),
            pending: true,
            ..sent
        })
        .expect("the change to store");
    a_sync(&cache, &pages);

    assert_eq!(pages.whole_bodies_written(), 0);
    assert_eq!(pages.what_it_holds().len(), 1);

    // And the shape the seam was written against does have the operation, which
    // is why one implementation could not have told anybody.
    let documents = ACollectionOfDocuments::new();
    let elsewhere = tempfile::tempdir().expect("a directory");
    let second = a_store(&elsewhere);
    a_note_made_here(
        &second,
        &documents.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&second, &documents);
    assert_eq!(documents.whole_bodies_written(), 1);
}

#[test]
fn test_a_container_that_is_not_the_shape_this_backend_needs_is_said_rather_than_guessed_at() {
    // The fourth disagreement. A backend handed one level of a four-level
    // hierarchy has been handed another backend's addressing, and answering
    // anyway is how a seam goes on believing a container is a URL. The refusal
    // reaches the sync as a problem it reports rather than as a clean sync over
    // somebody's missing notes.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let pages = ASectionOfPages::new();
    a_note_made_here(
        &cache,
        &pages.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );

    let did = run(sync_notes(
        &cache,
        &pages,
        ACCOUNT,
        &pages.a_container_it_refuses(),
    ))
    .expect("a sync");

    assert!(!did.errors.is_empty(), "{did:?}");
    assert_eq!(did.stored, 0, "{did:?}");
    assert!(
        pages.what_it_holds().is_empty(),
        "a note was written into a container the backend refused"
    );
}

#[test]
fn test_a_backend_that_gives_no_marker_loses_a_change_made_at_the_other_end_and_says_nothing() {
    // The measurement behind this plan's largest contract change. The seam said
    // a version marker is optional, and a backend that gives none has every
    // copy treated as having moved. Read on the push side that same absence
    // means the opposite: nothing is known to have moved there, so this
    // computer's copy goes over whatever is at the other end.
    //
    // Two readings of one absence, in one sync, and this is what they cost. It
    // is asserted as a loss rather than as a fix because there is nothing here
    // to fix: without a marker and without keeping the last copy seen, which
    // PIM-08 forbids, the two cases cannot be told apart. What changes is the
    // contract, which now requires a marker and says why.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let blind = ASectionOfPages::that_gives_no_marker();
    a_note_made_here(
        &cache,
        &blind.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, &blind);
    let sent = the_note_here(&cache, "note-1");
    let named = sent.known_as.clone().expect("a name from the backend");

    // Somebody at the other end writes something nobody here has seen, and
    // somebody here changes the same note.
    blind.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");
    cache
        .save_note(&NoteEntry {
            body: "Live is brown. Neutral is blue.".to_string(),
            pending: true,
            ..sent
        })
        .expect("the change to store");

    let did = a_sync(&cache, &blind);

    assert_eq!(
        did.held, 0,
        "this backend can now raise a clash, so the measurement below is stale"
    );
    assert!(did.errors.is_empty(), "{did:?}");
    let there = blind.what_it_holds();
    assert_eq!(there.len(), 1, "{there:?}");
    assert!(
        !there[0].2.contains("Earth is green"),
        "the change made at the other end survived, so this is no longer the cost"
    );
    // Nothing anywhere says it happened. Not a count, not a problem, not a
    // question. That sentence is the finding.
    assert_eq!(did.sent, 1, "{did:?}");
}

#[test]
fn test_a_marker_that_stood_still_while_the_note_moved_hides_the_change_from_the_read() {
    // The direction the contract had not written down. It says a marker is not
    // promised to change only when the content does, which is the harmless
    // direction: a marker that moves for its own reasons costs a fetch nobody
    // needed. The other direction costs somebody their note, and a clock
    // reading has it: two changes inside one tick carry one reading.
    //
    // An opaque token cannot do this, which is why one implementation could not
    // have found it.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let pages = ASectionOfPages::new();
    a_note_made_here(
        &cache,
        &pages.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, &pages);
    let named = the_note_here(&cache, "note-1")
        .known_as
        .expect("a name from the backend");

    pages.the_services_clock_stands_still();
    pages.somebody_else_changed(&named, "Wiring colours", "Live is brown. Earth is green.");

    let did = a_sync(&cache, &pages);

    assert_eq!(
        did.unchanged, 1,
        "the marker moved after all, so this measurement is stale: {did:?}"
    );
    let after = the_note_here(&cache, "note-1");
    assert!(
        !after.body.contains("Earth is green"),
        "the change arrived, so a standing-still marker no longer hides one"
    );
}

#[test]
fn test_the_name_a_page_backend_gives_moves_when_the_body_does_and_the_seam_keeps_up() {
    // The third disagreement, and the one most easily missed: a test that
    // writes once never sees it. The name after the second write is not the
    // name after the first, and what this computer holds is the later one.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let pages = ASectionOfPages::new();
    a_note_made_here(
        &cache,
        &pages.container(),
        "note-1",
        "Wiring colours",
        "Live is brown.",
    );
    a_sync(&cache, &pages);
    let first = the_note_here(&cache, "note-1")
        .known_as
        .expect("a name from the backend");

    let sent = the_note_here(&cache, "note-1");
    cache
        .save_note(&NoteEntry {
            body: "Live is brown. Neutral is blue.".to_string(),
            pending: true,
            ..sent
        })
        .expect("the change to store");
    a_sync(&cache, &pages);
    let second = the_note_here(&cache, "note-1")
        .known_as
        .expect("a name from the backend");

    assert_ne!(first, second, "the fixture did not move the name at all");
    assert_eq!(
        pages.what_it_holds()[0].0,
        second,
        "this computer is holding a name the backend no longer uses"
    );
}

// ── The round trip, and what a backend cannot promise about it ──────────────

/// A note with the things a body really holds and a format really loses.
///
/// Markdown, because that is what PIM-04 says a note's stored form is: two
/// levels of indentation that mean something, a run of spaces used to line
/// something up, and a trailing space at the end of a line. Every one of those
/// is a byte somebody typed on purpose and none of them is safe everywhere.
const AS_IT_WAS_TYPED: &str = "# Wiring colours\n\n- Live is brown\n  - Older cable: red\n- Neutral is blue\n\nTerminal  Colour\nL         brown \n";

fn a_note_goes_out_and_comes_back<S: NotesService + ABackendToDrive>(
    backend: &S,
) -> (String, String) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );
    a_sync(&cache, backend);
    let there = backend.what_it_holds();
    assert_eq!(there.len(), 1, "{there:?}");
    (the_note_here(&cache, "note-1").body, there[0].2.clone())
}

#[test]
fn test_a_note_is_the_bytes_it_went_out_as_after_a_round_trip_through_a_collection_of_documents() {
    // Compared on bytes and never on anything parsed. Two documents that parse
    // to the same structure can be two different byte strings, and the byte
    // string is the criterion.
    let (here, there) = a_note_goes_out_and_comes_back(&ACollectionOfDocuments::new());

    assert_eq!(here, AS_IT_WAS_TYPED);
    assert_eq!(there, AS_IT_WAS_TYPED, "a byte moved on the way out");
}

#[test]
fn test_a_note_a_section_of_pages_could_not_keep_says_exactly_which_bytes_moved() {
    // The other answer, and the one phase 5.2 will really meet. A page is an
    // HTML document, HTML collapses a run of whitespace to one space and cannot
    // hold one at the end of a line, so the indentation that makes a nested
    // list a nested list does not survive.
    //
    // Named rather than summarised. "It normalises" is a sentence somebody
    // reads past; the three lines below are what it costs.
    let (here, there) = a_note_goes_out_and_comes_back(&ASectionOfPages::new());

    // The two copies agree afterwards, which is the decision this plan made and
    // the one it costs most to get wrong either way. Left as it was typed, the
    // copy here and the copy there differ from this moment with nothing said,
    // until the first thing that moves a marker at the backend brings the
    // backend's version down over somebody's note for no reason they can see.
    assert_eq!(
        here, there,
        "the two copies disagree after a sync that said it worked"
    );
    assert_ne!(there, AS_IT_WAS_TYPED);

    let moved: Vec<(&str, &str)> = AS_IT_WAS_TYPED
        .lines()
        .zip(there.lines())
        .filter(|(typed, kept)| typed != kept)
        .collect();
    assert_eq!(
        moved,
        vec![
            ("  - Older cable: red", "- Older cable: red"),
            ("Terminal  Colour", "Terminal Colour"),
            ("L         brown ", "L brown"),
        ],
        "the bytes that moved are not the ones written down here"
    );
}

fn a_backend_that_could_not_keep_a_note_says_so<S: NotesService + ABackendToDrive>(
    backend: &S,
    expected: usize,
) {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );

    let did = a_sync(&cache, backend);

    assert_eq!(
        did.not_kept_exactly, expected,
        "a backend that could not keep a note reported that it took it: {did:?}"
    );
    // And what it kept is what this computer now holds, so the two agree from
    // the moment of the push rather than diverging in silence until something
    // at the other end moves a marker.
    let here = the_note_here(&cache, "note-1");
    assert_eq!(here.body, backend.what_it_holds()[0].2);
    assert!(!here.pending, "{here:?}");
}

#[test]
fn test_a_collection_of_documents_that_kept_the_note_says_nothing_about_keeping_it() {
    // The half that stops this being a count that is always one. A backend that
    // kept the bytes says nothing, so the sentence a person hears is about a
    // real loss rather than about every sync.
    a_backend_that_could_not_keep_a_note_says_so(&ACollectionOfDocuments::new(), 0);
}

#[test]
fn test_a_section_of_pages_that_could_not_keep_a_note_says_so_rather_than_reporting_it_took_it() {
    a_backend_that_could_not_keep_a_note_says_so(&ASectionOfPages::new(), 1);
}

#[test]
fn test_the_summary_says_a_backend_could_not_keep_a_note_only_when_one_really_could_not() {
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    let pages = ASectionOfPages::new();
    a_note_made_here(
        &cache,
        &pages.container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );
    let could_not = a_sync(&cache, &pages).summary();

    assert!(could_not.contains("could not be kept"), "{could_not}");
    assert!(could_not.contains("notes backend"), "{could_not}");

    let elsewhere = tempfile::tempdir().expect("a directory");
    let second = a_store(&elsewhere);
    let documents = ACollectionOfDocuments::new();
    a_note_made_here(
        &second,
        &documents.container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );
    let kept = a_sync(&second, &documents).summary();

    assert!(!kept.contains("could not be kept"), "{kept}");
}

fn a_note_the_other_end_only_touched_is_not_written_down_again<
    S: NotesService + ABackendToDrive,
>(
    backend: &S,
) {
    // A marker that moved for a reason the content did not cause. The contract
    // already says a backend may do this, and called the cost a fetch nobody
    // needed. It is more than that: the row was rewritten, its changed time
    // became the time of the sync, and the sync counted a note as stored when
    // nothing was.
    //
    // `NoteSyncResult::stored`'s own comment says it must not be the number
    // seen, because a sync that rewrites everything every time can only report
    // the size of the container. This is how it came to.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    a_note_made_here(
        &cache,
        &backend.container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );
    a_sync(&cache, backend);
    let after_the_push = the_note_here(&cache, "note-1");
    let named = after_the_push.known_as.clone().expect("a name");

    backend.the_other_end_touched_it_without_changing_it(&named);
    let did = a_sync(&cache, backend);

    assert_eq!(
        did.stored, 0,
        "a note nothing changed was counted as stored: {did:?}"
    );
    assert_eq!(did.unchanged, 1, "{did:?}");
    let after = the_note_here(&cache, "note-1");
    assert_eq!(
        after.updated_at, after_the_push.updated_at,
        "the row was written again, so the note's changed time is now the sync's"
    );
    assert_eq!(after.body, after_the_push.body);
    // The marker is still written down, or the next sync asks the same
    // question and the one after that as well.
    assert_ne!(
        after.known_version, after_the_push.known_version,
        "the marker the backend gives now was not written down"
    );
}

#[test]
fn test_a_document_the_server_only_touched_is_not_written_down_again() {
    a_note_the_other_end_only_touched_is_not_written_down_again(&ACollectionOfDocuments::new());
}

#[test]
fn test_a_page_the_service_only_touched_is_not_written_down_again() {
    a_note_the_other_end_only_touched_is_not_written_down_again(&ASectionOfPages::new());
}

#[test]
fn test_the_stored_form_of_a_note_did_not_change_to_take_a_second_backend() {
    // PIM-08's first line, asked of the store rather than of the seam. A note
    // written the way every note has always been written reads back the bytes
    // it went in as, its format is still the one word this program writes, and
    // nothing about a second kind of backend needed a migration to say so.
    let dir = tempfile::tempdir().expect("a directory");
    let cache = a_store(&dir);
    // Any container will do: this one is about the store rather than about a
    // backend, and no sync runs below.
    a_note_made_here(
        &cache,
        &ACollectionOfDocuments::new().container(),
        "note-1",
        "Wiring colours",
        AS_IT_WAS_TYPED,
    );

    let here = the_note_here(&cache, "note-1");
    assert_eq!(here.body, AS_IT_WAS_TYPED);
    assert_eq!(here.format, NoteBody::AsTyped);
    assert_eq!(here.known_as, None);
    assert_eq!(here.known_version, None);
}
