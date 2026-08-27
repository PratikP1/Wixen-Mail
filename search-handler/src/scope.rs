//! Which crawl scope rule tells the indexer to come and look.
//!
//! # The gap this closes
//!
//! Registering a protocol handler teaches the indexer how to *read* a URL in our
//! scheme. It does not tell it to *go and look*. Without the second half the
//! handler is a correct answer to a question nobody asks: registration succeeds,
//! the indexer runs, no error appears anywhere, and no mail is ever found.
//!
//! The second half is a crawl scope rule, added through
//! `ISearchCrawlScopeManager`, which names a URL prefix and says the indexer may
//! index under it. This module works out what that rule should say. The calls
//! that put it there are in [`crate::com::crawl_scope`], because they cannot be
//! tested from here.
//!
//! # A rule and a root, not just a rule
//!
//! Two things are added and both are needed. The rule says a URL prefix is in
//! scope. The *search root* is the point a crawl starts from, and for a scheme
//! of our own there is nothing else to start it: a file rule inherits a root
//! from the file system, and we have no file system. A rule with no root is in
//! scope and never visited.
//!
//! # Why the rule names one person
//!
//! [`plan_for`] requires a security identifier rather than treating it as
//! optional. The handler runs outside any signed-in session, so a URL that does
//! not say whose mail it is falls back to the account the indexer's host process
//! happens to be running as, which is almost never the right one. A rule built
//! without an identifier would be a rule that registers cleanly and can never
//! find a message, which is the exact failure this module exists to remove. One
//! rule per person who wants this, and each names its own.
//!
//! # A default rule rather than a user rule
//!
//! `ISearchCrawlScopeManager` offers `AddDefaultScopeRule` and
//! `AddUserScopeRule`. The names are about precedence, not about which signed-in
//! account the rule belongs to: a default rule is what an application says its
//! own data looks like, and a user rule is a person's own choice, which wins
//! over a default and is what `RevertToDefaultScopes` throws away.
//!
//! This is a default rule. Installing Wixen Mail and ticking a box is the
//! application declaring where its store is, which is what a default rule means.
//! It also leaves the person in charge: a rule of their own beats ours, so
//! installing again cannot quietly overrule a choice they made. A user scope
//! rule would, and would also be thrown away by `RevertToDefaultScopes`, which
//! is a button in the Windows indexing settings.
//!
//! Two halves of that are on different footings. That the rule really is
//! recorded as the application's own has been watched on a real machine:
//! `IsDefault` comes back true and the setup tool reports it. That taking this
//! location out in Indexing Options writes a person's own rule which then wins
//! is Microsoft's documented model and has not been watched here.

use crate::url::{ItemUrl, Place};
use windows::Win32::System::Search::FF_INDEXCOMPLEXURLS;

/// Why a scope rule could not be worked out.
///
/// Flat, and it carries nothing that came out of a mailbox, for the same reason
/// [`crate::url::UrlError`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    /// The text handed in could not be a Windows security identifier.
    NotASecurityIdentifier,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotASecurityIdentifier => {
                write!(f, "that is not a Windows security identifier")
            }
        }
    }
}

/// What to tell the crawl scope manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePlan {
    /// The URL prefix the rule and the root both name.
    pub prefix: String,
    /// A URL below the prefix, used to ask the indexer whether the rule works.
    ///
    /// Asking about the prefix itself proves less than it looks: a rule can be
    /// present and still not cover the items under it. This is a real item URL
    /// in the shape the handler hands back, with names that could not be
    /// anybody's, so nothing private is carried into a question.
    pub sample_url: String,
    /// What the indexer is told about following URLs below the prefix.
    pub follow_flags: u32,
}

/// The account and folder names in [`ScopePlan::sample_url`].
///
/// Deliberately not a real account or folder. This URL is passed to Windows and
/// printed by the setup tool, so it must not be able to carry somebody's account
/// name or a folder name out of the store.
const SAMPLE_ACCOUNT: &str = "account";
const SAMPLE_FOLDER: &str = "folder";

/// Work out the rule for one person's mail.
///
/// `user` is a security identifier without braces, the form
/// `S-1-5-21-...` that `ConvertSidToStringSid` hands back.
///
/// The URL is built with the same code that writes every URL the handler hands
/// to the indexer, and then read back with the same code that parses one. That
/// round trip is the point: a prefix this handler would refuse is a prefix the
/// indexer would crawl and get nothing from.
pub fn plan_for(user: &str) -> Result<ScopePlan, PlanError> {
    let braced = format!("{{{user}}}");
    let root = ItemUrl {
        user: Some(braced.clone()),
        place: Place::Root,
    };
    let sample = ItemUrl {
        user: Some(braced),
        place: Place::Message {
            account: SAMPLE_ACCOUNT.to_string(),
            folder: SAMPLE_FOLDER.to_string(),
            uid: 1,
        },
    };

    let prefix = format!("{root}/");
    // Both have to be URLs this handler accepts. Building them from `ItemUrl`
    // is not enough on its own, because a security identifier that is not one
    // reaches the string unchecked and only the parser looks at it.
    ItemUrl::parse(&prefix).map_err(|_| PlanError::NotASecurityIdentifier)?;
    let sample_url = sample.to_string();
    ItemUrl::parse(&sample_url).map_err(|_| PlanError::NotASecurityIdentifier)?;

    Ok(ScopePlan {
        prefix,
        sample_url,
        // Index URLs that carry a query or an escape rather than treating them
        // as too complicated to follow. Nothing this handler writes today has a
        // query in it, so this is expected to change nothing; it is set because
        // the two mistakes are not the same size. The other flag,
        // `FF_SUPPRESSINDEXING`, would turn indexing off for everything under
        // the prefix and look exactly like the handler not working.
        follow_flags: FF_INDEXCOMPLEXURLS.0 as u32,
    })
}

/// Whether a rule the indexer already holds is the one this plan describes.
///
/// The crawl scope manager hands rules back as text, and the text it hands back
/// is not guaranteed to be the text that went in: a trailing slash and the case
/// of the scheme are both things Windows is free to normalise. Comparing the
/// two strings directly is how a rule that is already there gets added a second
/// time, or reported missing while it sits in the list.
///
/// A rule for something *below* our prefix is not our rule. It is somebody
/// naming one account or one folder, and reporting it as ours would say the
/// whole store is covered when one folder is.
pub fn same_rule(pattern: &str, prefix: &str) -> bool {
    fn tidy(url: &str) -> String {
        url.trim_end_matches('/').to_ascii_lowercase()
    }

    tidy(pattern) == tidy(prefix)
}

/// What the indexer answered about one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    /// Whether the rule puts the prefix in the index or keeps it out.
    pub includes: bool,
    /// Whether it is an application's default rule or a person's own choice.
    pub is_default: bool,
}

/// Which of the rules naming our prefix is the one that decides.
///
/// There can be more than one. The application adds a default rule saying where
/// its store is, and a person who then takes this location out in Indexing
/// Options gets a rule of their own for the same URL. Both sit in the same list,
/// and the person's own rule wins.
///
/// Reporting the first one found would say the mail is being indexed while the
/// indexer is skipping it, which is worse than saying nothing.
pub fn deciding_rule(rules: &[Rule]) -> Option<Rule> {
    rules
        .iter()
        .find(|rule| !rule.is_default)
        .or_else(|| rules.first())
        .copied()
}

/// What the crawl scope manager currently says about this handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeState {
    /// The rule naming our prefix, when there is one.
    pub rule: Option<Rule>,
    /// Whether a search root for our prefix is registered.
    pub root_registered: bool,
    /// What the indexer answers when asked about a message URL below the prefix.
    ///
    /// This is the answer that matters, because it is the question the indexer
    /// asks itself. A rule can be present and overruled by another one.
    pub sample_included: bool,
    /// How many rules in the catalog belong to something else.
    ///
    /// A count and never the rules themselves. Another application's rule names
    /// real folders on this machine, and printing the list would put somebody's
    /// folder layout on screen for no reason.
    pub other_rules: usize,
}

impl ScopeState {
    /// Whether everything the indexer needs before it will ask about mail is
    /// in place.
    ///
    /// Both halves, because either one alone is a silent nothing. A root with
    /// no including rule is a starting point the indexer is not allowed to
    /// index, and a rule with no root is a permission nothing acts on.
    pub fn ready_to_be_crawled(&self) -> bool {
        self.root_registered && self.sample_included
    }
}

impl std::fmt::Display for ScopeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = match self.rule {
            None => "not there".to_string(),
            Some(rule) => {
                let effect = match rule.includes {
                    true => "puts this location in the index",
                    false => "keeps this location out of the index",
                };
                let owner = match rule.is_default {
                    true => "set by the application",
                    false => "set by the person using this computer",
                };
                format!("present, {effect}, {owner}")
            }
        };
        writeln!(f, "Crawl scope rule: {rule}")?;
        writeln!(
            f,
            "Search root: {}",
            match self.root_registered {
                true => "registered",
                false => "not registered",
            }
        )?;
        writeln!(
            f,
            "The indexer says a message URL here is in scope: {}",
            match self.sample_included {
                true => "yes",
                false => "no",
            }
        )?;
        writeln!(
            f,
            "Rules in this catalog belonging to something else: {} (not listed, \
             because they name real folders on this computer)",
            self.other_rules
        )?;
        match self.ready_to_be_crawled() {
            true => write!(f, "The indexer has been told to look here."),
            false => write!(
                f,
                "The indexer has not been told to look here, so it will never ask \
                 about any mail."
            ),
        }
    }
}

/// What can be said about the URL the indexer is working on, without repeating it.
///
/// The indexer will happily say it is busy with
/// `file:///C:/Users/somebody/Documents/...`, or with one of ours, which carries
/// an account name and a folder name. Neither belongs on screen, so the answer
/// is one of three fixed sentences and never the URL.
pub fn describe_url_being_indexed(url: Option<&str>) -> &'static str {
    match url.map(str::trim).filter(|url| !url.is_empty()) {
        None => "The indexer is not working on anything right now.",
        Some(url) if is_ours(url) => "The indexer is working on a Wixen Mail URL right now.",
        Some(_) => "The indexer is working on something else right now.",
    }
}

/// Whether a URL belongs to this handler's scheme.
///
/// Sliced with `get` rather than by index. The URL came from Windows, so it can
/// be any text at all, and cutting a multi-byte character in half would panic
/// in the middle of reporting on somebody's machine.
fn is_ours(url: &str) -> bool {
    let scheme = format!("{}:", crate::url::SCHEME);
    url.get(..scheme.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(&scheme))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::SCHEME;

    const A_USER: &str = "S-1-5-21-1004336348-1177238915-682003330-512";

    fn a_plan() -> ScopePlan {
        plan_for(A_USER).expect("a real security identifier should give a plan")
    }

    #[test]
    fn test_the_prefix_is_a_prefix_of_every_url_the_handler_hands_back() {
        // This is the whole join between the two halves. The rule names a
        // prefix and the handler writes urls; if they do not share a beginning
        // then the indexer is crawling one place and the handler is describing
        // another, and the symptom is a scope rule that is present, correct
        // looking, and matches not one item.
        let plan = a_plan();
        let user = Some(format!("{{{A_USER}}}"));
        let places = [
            Place::Account {
                account: "work".to_string(),
            },
            Place::Folder {
                account: "work".to_string(),
                folder: "INBOX".to_string(),
            },
            Place::Message {
                account: "work".to_string(),
                folder: "INBOX".to_string(),
                uid: 4211,
            },
        ];

        for place in places {
            let url = ItemUrl {
                user: user.clone(),
                place,
            }
            .to_string();
            assert!(
                url.starts_with(&plan.prefix),
                "{url} is not under {}",
                plan.prefix
            );
        }
    }

    #[test]
    fn test_the_prefix_is_a_url_this_handler_would_accept() {
        // A prefix the handler refuses is a place the indexer will crawl and
        // get an error from on every single item. The two are written by
        // different code and only meet on a live machine, so the plan proves
        // the round trip here instead.
        let plan = a_plan();

        assert_eq!(
            ItemUrl::parse(&plan.prefix)
                .expect("the prefix should parse")
                .place,
            Place::Root
        );
        assert!(
            ItemUrl::parse(&plan.sample_url).is_ok(),
            "the sample url the indexer is asked about is not one of ours"
        );
    }

    #[test]
    fn test_a_scope_rule_has_to_say_whose_mail_it_is() {
        // The handler runs outside any signed-in session and cannot ask Windows
        // where "my" application data is. A url with no security identifier
        // falls back to whatever account the indexer's host process runs as,
        // finds no database there, and reports an empty mailbox. Refusing a
        // rule that names nobody is how that whole failure is kept off a
        // machine, because it registers cleanly and looks fine.
        for not_a_sid in [
            "",
            "  ",
            "not a sid",
            "S-1-5-21-1004336348 512",
            "{S-1-5-18}",
        ] {
            assert_eq!(
                plan_for(not_a_sid),
                Err(PlanError::NotASecurityIdentifier),
                "{not_a_sid}"
            );
        }
    }

    #[test]
    fn test_the_prefix_ends_at_a_slash_so_it_cannot_reach_a_neighbour() {
        // A prefix match is a text comparison at the indexer's end. Without the
        // final slash, "wixen-mail://{sid}/localhost" is also a prefix of
        // "wixen-mail://{sid}/localhostable", and a scope rule that reaches
        // further than it names is a scope rule nobody can reason about.
        let plan = a_plan();

        assert!(plan.prefix.ends_with('/'), "{}", plan.prefix);
        assert!(
            plan.prefix
                .starts_with(&format!("{SCHEME}://{{{A_USER}}}/")),
            "{}",
            plan.prefix
        );
    }

    #[test]
    fn test_the_rule_indexes_rather_than_suppressing_indexing() {
        // The two follow flags do opposite things and are one bit apart.
        // FF_SUPPRESSINDEXING would leave a rule that is present, enabled, and
        // indexes nothing, which is indistinguishable from the handler being
        // broken.
        let plan = a_plan();

        assert_eq!(plan.follow_flags, FF_INDEXCOMPLEXURLS.0 as u32);
        assert_eq!(
            plan.follow_flags & windows::Win32::System::Search::FF_SUPPRESSINDEXING.0 as u32,
            0,
            "the rule tells the indexer not to index"
        );
    }

    #[test]
    fn test_a_rule_already_there_is_recognised_however_windows_wrote_it_back() {
        // The crawl scope manager is free to hand a rule back with a different
        // trailing slash or a different case from the one that went in.
        // Comparing the strings directly is how the same rule gets added twice,
        // or reported missing while it sits in the list, and both look like the
        // add having failed.
        let plan = a_plan();
        let without_slash = plan.prefix.trim_end_matches('/').to_string();

        assert!(same_rule(&plan.prefix, &plan.prefix));
        assert!(same_rule(&without_slash, &plan.prefix));
        assert!(same_rule(&plan.prefix.to_uppercase(), &plan.prefix));
    }

    #[test]
    fn test_a_rule_for_one_folder_is_not_the_rule_for_the_whole_store() {
        // A rule naming something below our prefix covers one account or one
        // folder. Counting it as ours would report the whole store as indexed
        // when a single folder is, and taking ours out would then leave it
        // behind.
        let plan = a_plan();

        assert!(!same_rule(&format!("{}work", plan.prefix), &plan.prefix));
        assert!(!same_rule(
            &format!("{}work/INBOX", plan.prefix),
            &plan.prefix
        ));
    }

    #[test]
    fn test_another_persons_rule_is_not_this_persons_rule() {
        // One rule per person, each naming their own identifier. Treating one
        // as another would report a second account on the machine as set up
        // when it is not, and removing one would take the other away.
        let mine = a_plan();
        let theirs = plan_for("S-1-5-21-1004336348-1177238915-682003330-1001")
            .expect("another real identifier");

        assert!(!same_rule(&theirs.prefix, &mine.prefix));
    }

    #[test]
    fn test_somebody_elses_scheme_is_never_mistaken_for_ours() {
        // The catalog holds every rule on the machine, most of them file paths
        // and some of them other handlers. A loose match here would have the
        // tool report Outlook's mail rule as this handler's.
        let plan = a_plan();

        for other in [
            "file:///C:/Users/somebody/Documents",
            "mapi://{S-1-5-21-1004336348-1177238915-682003330-512}/",
            "wixen-mailbox://localhost/",
        ] {
            assert!(!same_rule(other, &plan.prefix), "{other}");
        }
    }

    #[test]
    fn test_a_rule_that_is_present_but_excludes_is_not_ready_to_be_crawled() {
        // Windows keeps exclusion rules in the same list as inclusion rules. A
        // check that only asked whether a rule exists would call an exclusion
        // success, and the person would be told the indexer is looking at mail
        // it has been told to skip.
        let state = ScopeState {
            rule: Some(Rule {
                includes: false,
                is_default: true,
            }),
            root_registered: true,
            sample_included: false,
            other_rules: 40,
        };

        assert!(!state.ready_to_be_crawled());
        assert!(
            state.to_string().contains("keeps this location out"),
            "{state}"
        );
        assert!(
            state.to_string().contains("never ask about any mail"),
            "{state}"
        );
    }

    #[test]
    fn test_a_rule_with_no_root_is_reported_as_not_ready() {
        // A rule says a url prefix may be indexed. A root is where a crawl
        // starts. For a scheme of our own there is no file system to inherit a
        // starting point from, so a rule on its own is a permission nothing
        // ever acts on, and it looks exactly like being set up correctly.
        let state = ScopeState {
            rule: Some(Rule {
                includes: true,
                is_default: true,
            }),
            root_registered: false,
            sample_included: true,
            other_rules: 40,
        };

        assert!(!state.ready_to_be_crawled());
        assert!(
            state.to_string().contains("Search root: not registered"),
            "{state}"
        );
    }

    #[test]
    fn test_both_halves_present_is_the_only_thing_reported_as_ready() {
        // The one combination that means the indexer will really come and ask.
        let state = ScopeState {
            rule: Some(Rule {
                includes: true,
                is_default: true,
            }),
            root_registered: true,
            sample_included: true,
            other_rules: 40,
        };

        assert!(state.ready_to_be_crawled());
        assert!(
            state.to_string().contains("has been told to look here"),
            "{state}"
        );
    }

    #[test]
    fn test_the_state_report_never_lists_another_applications_rules() {
        // Every rule in this catalog that is not ours names a real place on
        // this machine: somebody's Documents folder, their mail profile, an
        // application's data folder. Printing the list would put a person's
        // folder layout on screen to answer a question about Wixen Mail.
        let state = ScopeState {
            rule: None,
            root_registered: false,
            sample_included: false,
            other_rules: 37,
        };

        let report = state.to_string();
        assert!(report.contains("37"), "{report}");
        assert!(!report.contains("file:///"), "{report}");
        assert!(!report.contains(r"C:\"), "{report}");
    }

    #[test]
    fn test_the_url_the_indexer_is_busy_with_is_described_and_never_repeated() {
        // URLBeingIndexed hands back a real url. One of ours carries an account
        // name and a folder name, and one of somebody else's carries a path
        // through their profile. Neither may be printed, so the answer is one
        // of three fixed sentences.
        let ours = "wixen-mail://{S-1-5-21-99-1001}/localhost/personal/Medical/7";
        let theirs = r"file:///C:/Users/somebody/Documents/divorce.docx";

        assert!(describe_url_being_indexed(Some(ours)).contains("Wixen Mail"));
        for url in [ours, theirs] {
            let described = describe_url_being_indexed(Some(url));
            assert!(!described.contains("Medical"), "{described}");
            assert!(!described.contains("divorce"), "{described}");
            assert!(!described.contains("localhost"), "{described}");
        }
        assert!(describe_url_being_indexed(Some(theirs)).contains("something else"));
    }

    #[test]
    fn test_an_idle_indexer_is_said_to_be_idle_rather_than_working_on_nothing() {
        // URLBeingIndexed hands back nothing when the indexer is idle, and an
        // empty string is the same answer. Reading either as "working on
        // something else" would have the tool report activity that is not
        // happening.
        assert!(describe_url_being_indexed(None).contains("not working on anything"));
        assert!(describe_url_being_indexed(Some("")).contains("not working on anything"));
        assert!(describe_url_being_indexed(Some("   ")).contains("not working on anything"));
    }

    #[test]
    fn test_a_persons_own_choice_beats_the_rule_the_application_added() {
        // Both rules can name the same url at once: ours from installing, and
        // theirs from taking this location out in Indexing Options. Windows
        // hands both back in one list and honours theirs. Reporting ours would
        // tell somebody their mail is being indexed while the indexer skips it.
        let ours = Rule {
            includes: true,
            is_default: true,
        };
        let theirs = Rule {
            includes: false,
            is_default: false,
        };

        assert_eq!(deciding_rule(&[ours, theirs]), Some(theirs));
        assert_eq!(deciding_rule(&[theirs, ours]), Some(theirs));
        assert_eq!(deciding_rule(&[ours]), Some(ours));
        assert_eq!(deciding_rule(&[]), None);
    }

    #[test]
    fn test_the_sample_url_cannot_carry_a_real_account_or_folder_name() {
        // The sample is handed to Windows and printed by the setup tool. It is
        // built from fixed words rather than from anything in the store, so
        // there is no route by which somebody's account name reaches either.
        let plan = a_plan();

        assert!(plan.sample_url.contains(SAMPLE_ACCOUNT));
        assert!(plan.sample_url.contains(SAMPLE_FOLDER));
        assert_eq!(
            ItemUrl::parse(&plan.sample_url).expect("the sample").place,
            Place::Message {
                account: SAMPLE_ACCOUNT.to_string(),
                folder: SAMPLE_FOLDER.to_string(),
                uid: 1,
            }
        );
    }
}
