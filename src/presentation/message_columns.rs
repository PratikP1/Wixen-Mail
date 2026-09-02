//! Which columns the message list shows, in what order, and how it sorts.
//!
//! This is a verbosity control before it is a display preference. The message
//! list runs in virtual mode on a native list control, where a row's accessible
//! name is assembled from its visible columns, so every column that is on is
//! another field a screen reader reads for every message while arrowing through
//! a mailbox. Six columns is the difference between skimming and wading.
//!
//! That is also why the flags are separate narrow columns rather than one merged
//! "status". A single column reading "unread, flagged, attachment" costs all
//! three on every row; separate ones let someone keep unread and drop the rest.

use crate::presentation::ui_types::MailSortOption;

/// A column the message list can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageColumn {
    Unread,
    Attachment,
    Subject,
    /// From in most folders, To in Sent and Drafts. A From column in Sent is
    /// your own address on every row, which is noise read aloud a thousand times.
    Correspondent,
    /// When the server received it.
    Received,
    /// When the sender says they sent it, which is sender controlled and often
    /// wrong. Sorting an inbox by it puts forged-date spam permanently on top.
    Sent,
    Snippet,
    Thread,
    Size,
    Flagged,
    /// Whether you have replied. Only meaningful once flags come from a server.
    Answered,
    /// Whether the message is an unsent draft.
    Draft,
    /// What the provider's spam and phishing filter made of it.
    ///
    /// Off by default. Most mail is ordinary, so on for everybody it would be
    /// an empty column read past on every row; on for those who want it, it is
    /// the fastest way to see the verdict without opening anything.
    Safety,
    To,
    Cc,
}

impl MessageColumn {
    /// Every column, in the order the column dialog lists them.
    pub const ALL: [MessageColumn; 15] = [
        MessageColumn::Unread,
        MessageColumn::Attachment,
        MessageColumn::Subject,
        MessageColumn::Correspondent,
        MessageColumn::Received,
        MessageColumn::Sent,
        MessageColumn::Snippet,
        MessageColumn::Thread,
        MessageColumn::Size,
        MessageColumn::Flagged,
        MessageColumn::Answered,
        MessageColumn::Draft,
        MessageColumn::Safety,
        MessageColumn::To,
        MessageColumn::Cc,
    ];

    /// The column heading, and what a screen reader reads for the column.
    ///
    /// No mnemonics: a list header cannot be activated by one, so an ampersand
    /// here would be announced without meaning anything.
    pub fn heading(&self) -> &'static str {
        match self {
            MessageColumn::Unread => "Unread",
            MessageColumn::Attachment => "Attachment",
            MessageColumn::Subject => "Subject",
            MessageColumn::Correspondent => "Correspondent",
            MessageColumn::Received => "Received",
            MessageColumn::Sent => "Sent",
            MessageColumn::Snippet => "Snippet",
            MessageColumn::Thread => "Thread",
            MessageColumn::Size => "Size",
            MessageColumn::Flagged => "Flagged",
            MessageColumn::Answered => "Answered",
            MessageColumn::Draft => "Draft",
            MessageColumn::Safety => "Safety",
            MessageColumn::To => "To",
            MessageColumn::Cc => "Cc",
        }
    }

    /// The identifier used when the layout is stored.
    pub fn key(&self) -> &'static str {
        match self {
            MessageColumn::Unread => "unread",
            MessageColumn::Attachment => "attachment",
            MessageColumn::Subject => "subject",
            MessageColumn::Correspondent => "correspondent",
            MessageColumn::Received => "received",
            MessageColumn::Sent => "sent",
            MessageColumn::Snippet => "snippet",
            MessageColumn::Thread => "thread",
            MessageColumn::Size => "size",
            MessageColumn::Flagged => "flagged",
            MessageColumn::Answered => "answered",
            MessageColumn::Draft => "draft",
            MessageColumn::Safety => "safety",
            MessageColumn::To => "to",
            MessageColumn::Cc => "cc",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        MessageColumn::ALL.into_iter().find(|c| c.key() == key)
    }

    /// The SQL expression this column sorts on.
    ///
    /// Fixed strings chosen by matching on the enum, never built from anything
    /// a user typed, because the result is interpolated into a query.
    fn sort_expression(&self) -> &'static str {
        match self {
            MessageColumn::Unread => "m.read",
            MessageColumn::Attachment => "m.has_attachments",
            MessageColumn::Subject => "m.subject COLLATE NOCASE",
            MessageColumn::Correspondent => "m.from_addr COLLATE NOCASE",
            // The server's arrival time, falling back to the sender's date for
            // rows stored before arrival times were kept.
            MessageColumn::Received => "COALESCE(m.internaldate, m.date)",
            MessageColumn::Sent => "m.date",
            MessageColumn::Snippet => "m.snippet COLLATE NOCASE",
            MessageColumn::Thread => "m.thread_id",
            MessageColumn::Size => "m.size_bytes",
            MessageColumn::Flagged => "m.starred",
            MessageColumn::Answered => "m.answered",
            MessageColumn::Draft => "m.draft",
            MessageColumn::Safety => "m.safety",
            MessageColumn::To => "m.to_addr COLLATE NOCASE",
            MessageColumn::Cc => "m.cc COLLATE NOCASE",
        }
    }

    /// The SQL expression this column takes over a whole conversation.
    ///
    /// D-02: a row describes the conversation, not its newest message, and the
    /// rule is stated once here and used twice. `conversations_in` selects
    /// these expressions to fill `ConversationItem`'s fields, and
    /// [`Sort::conversation_order_by_clause`] puts the same strings in the
    /// `ORDER BY`. The value a row shows and the value the list sorts by are
    /// therefore the same expression, and there is no second spelling of the
    /// rule to keep in step.
    ///
    /// A fourth method beside [`Self::sort_expression`] rather than that one
    /// with the aggregate wrapped round it. String surgery on a sort expression
    /// is exactly how the two halves come apart, and it would also give up the
    /// property the method above states: fixed strings chosen by matching on
    /// the enum, never built from anything a user typed, because the result is
    /// interpolated into a query.
    ///
    /// [`MessageColumn::ALL`] being a fixed-size array is what keeps this
    /// honest over time. Adding a variant makes every `match` in this file go
    /// red at compile time, and this is one of them, so a new column cannot
    /// arrive with a display rule and no sort rule.
    ///
    /// The families, in D-02's own words:
    ///
    /// - Dates and Snippet take the newest message's value.
    /// - `Attachment`, `Flagged`, `Answered` and `Draft` are true if any is.
    /// - `Safety` is the worst.
    /// - `Size` is the sum.
    /// - `Correspondent` is the distinct senders, and `To` and `Cc` follow it.
    /// - `Subject` is the conversation's name, per D-04.
    /// - `Thread` is the counts, per D-03.
    ///
    /// `m` is the reach: the rows this conversation has within the folders the
    /// setting says to count. `conversations_in` names it, so an expression
    /// here reads exactly like its message-level sibling above.
    pub(crate) fn conversation_sort_expression(&self) -> &'static str {
        match self {
            // How many have not been read, which is also the second number the
            // Thread column says. A conversation is unread when that is not
            // zero.
            MessageColumn::Unread => "SUM(CASE WHEN m.read = 0 THEN 1 ELSE 0 END)",
            MessageColumn::Attachment => "MAX(m.has_attachments)",
            // D-04, through the one function that answers it. Registered on
            // the connection rather than spelled again in SQL, the same way
            // `lower` is, because a second spelling of this rule is a second
            // answer to the question of what a conversation is called.
            MessageColumn::Subject => {
                "conversation_name(\
                 (SELECT o.subject FROM here o WHERE o.thread_id = m.thread_id \
                  ORDER BY o.received_at ASC, o.id ASC LIMIT 1))"
            }
            // One per line, not one per comma. SQLite's own separator is a
            // comma and a display name is allowed to contain one, so
            // "Smith, John <j@example.com>" would come back as two people.
            // The same reasoning `safety_reasons` already records for storing
            // its sentences a line apiece.
            MessageColumn::Correspondent => "GROUP_CONCAT(DISTINCT char(10) || m.from_addr)",
            // The server's arrival time, falling back to the sender's date for
            // rows stored before arrival times were kept. `received_at` is that
            // fallback, applied once where the reach is worked out.
            MessageColumn::Received => "MAX(m.received_at)",
            MessageColumn::Sent => "MAX(m.date)",
            MessageColumn::Snippet => {
                "(SELECT n.snippet FROM here n \
                 WHERE n.thread_id = m.thread_id \
                 ORDER BY n.received_at DESC, n.id DESC LIMIT 1)"
            }
            MessageColumn::Thread => "COUNT(*)",
            // Null where no message in it says what it weighs, which reads as
            // blank rather than as "0 bytes", a claim we cannot make.
            MessageColumn::Size => "SUM(m.size_bytes)",
            MessageColumn::Flagged => "MAX(m.starred)",
            MessageColumn::Answered => "MAX(m.answered)",
            MessageColumn::Draft => "MAX(m.draft)",
            // The worst, which is not the largest of the words stored. Safety
            // is written down as "ordinary", "suspicious", "spam" and
            // "phishing" so a stored mailbox can be read by a person, and in
            // that alphabet the worst of the three sorts first. So the ranking
            // is spelled here, worst last, matching the order the enum itself
            // is declared in.
            MessageColumn::Safety => {
                "MAX(CASE m.safety \
                 WHEN 'phishing' THEN 3 WHEN 'spam' THEN 2 WHEN 'suspicious' THEN 1 \
                 ELSE 0 END)"
            }
            MessageColumn::To => "GROUP_CONCAT(DISTINCT char(10) || m.to_addr)",
            MessageColumn::Cc => "GROUP_CONCAT(DISTINCT char(10) || m.cc)",
        }
    }
}

/// Which way a sort runs. Every column supports both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// How the direction is described when the new sort is announced.
    ///
    /// Dates read as "newest first" rather than "descending", because that is
    /// what the user asked for rather than how it is implemented.
    fn spoken_for(&self, column: MessageColumn) -> &'static str {
        let is_date = matches!(column, MessageColumn::Received | MessageColumn::Sent);
        match (self, is_date) {
            (SortDirection::Ascending, true) => "oldest first",
            (SortDirection::Descending, true) => "newest first",
            (SortDirection::Ascending, false) => "ascending",
            (SortDirection::Descending, false) => "descending",
        }
    }

    fn key(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "asc",
            SortDirection::Descending => "desc",
        }
    }
}

/// One column and which way it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct By {
    pub column: MessageColumn,
    pub direction: SortDirection,
}

impl By {
    /// The `ORDER BY` term for this one column.
    fn term(&self) -> String {
        let direction = match self.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };
        format!("{} {}", self.column.sort_expression(), direction)
    }

    /// The same term, over a whole conversation.
    ///
    /// The direction is worked out once, above and here, from the same enum, so
    /// a list of conversations and a list of messages cannot come to mean
    /// opposite things by the same word.
    fn conversation_term(&self) -> String {
        let direction = match self.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };
        format!(
            "{} {}",
            self.column.conversation_sort_expression(),
            direction
        )
    }

    /// How it is written in the stored layout.
    fn key(&self) -> String {
        format!("{}:{}", self.column.key(), self.direction.key())
    }

    /// Read one back, or nothing if it names a column this build has never
    /// heard of.
    fn from_key(stored: &str) -> Option<Self> {
        let (column, direction) = stored.split_once(':')?;
        Some(Self {
            column: MessageColumn::from_key(column)?,
            direction: match direction {
                "asc" => SortDirection::Ascending,
                "desc" => SortDirection::Descending,
                _ => return None,
            },
        })
    }

    /// What is said about it.
    fn spoken(&self) -> String {
        format!(
            "{}, {}",
            self.column.heading().to_lowercase(),
            self.direction.spoken_for(self.column)
        )
    }
}

/// How the list is ordered.
///
/// Two levels, because one is not enough for the question people actually ask
/// of a mailbox: newest first is right until there are twenty from today, and
/// then what is wanted is unread first among those. A single column cannot say
/// that, and sorting twice by hand does not survive the next refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: MessageColumn,
    pub direction: SortDirection,
    /// What decides the order between rows the first level calls equal.
    ///
    /// `None` for one level, which is what every stored layout written before
    /// this had and what most people want.
    pub then: Option<By>,
}

impl Sort {
    /// The sort a stored setting asks for.
    ///
    /// Settings has offered a starting sort order since it existed and
    /// nothing read it, so somebody who asked for oldest first, or for unread
    /// at the top, got newest first every time and had to sort by hand on
    /// every folder.
    ///
    /// `None` for a word this does not know, which is what a settings file
    /// written by a later version or edited by hand can hold: the built-in
    /// order is a better answer than a guess.
    pub fn from_setting(stored: &str) -> Option<Self> {
        let (column, direction) = match stored.trim() {
            "date_newest" => (MessageColumn::Received, SortDirection::Descending),
            "date_oldest" => (MessageColumn::Received, SortDirection::Ascending),
            "sender_az" => (MessageColumn::Correspondent, SortDirection::Ascending),
            "sender_za" => (MessageColumn::Correspondent, SortDirection::Descending),
            "subject_az" => (MessageColumn::Subject, SortDirection::Ascending),
            "subject_za" => (MessageColumn::Subject, SortDirection::Descending),
            // Unread at the top, and newest first among the rest, because
            // "unread first" says nothing about the order within either group
            // and the answer people expect underneath it is the ordinary one.
            "unread_first" => {
                return Some(
                    Self {
                        column: MessageColumn::Received,
                        direction: SortDirection::Descending,
                        then: None,
                    }
                    .with_unread_first(),
                );
            }
            _ => return None,
        };
        Some(Self {
            column,
            direction,
            then: None,
        })
    }

    /// The same sort, with unread rows brought to the top.
    fn with_unread_first(self) -> Self {
        Self {
            column: MessageColumn::Unread,
            direction: SortDirection::Ascending,
            then: Some(By {
                column: self.column,
                direction: self.direction,
            }),
        }
    }

    /// The first level on its own, as a value.
    pub fn first(&self) -> By {
        By {
            column: self.column,
            direction: self.direction,
        }
    }

    /// The `ORDER BY` body for this sort.
    ///
    /// Both levels when there are two, in that order, which is what SQL does
    /// with a comma. A second level naming the same column as the first is
    /// left out: it decides nothing and reads as though it might.
    pub fn order_by_clause(&self) -> String {
        match self.then.filter(|then| then.column != self.column) {
            Some(then) => format!("{}, {}", self.first().term(), then.term()),
            None => self.first().term(),
        }
    }

    /// The `ORDER BY` body for this sort, over a list of conversations.
    ///
    /// The same two levels and the same rule about a second level naming the
    /// first's column, because a sort means the same thing in either view.
    /// D-12: the sort column and direction survive a switch between the two
    /// unchanged, and the header announces the same thing before and after,
    /// which only holds if one `Sort` can express itself both ways.
    ///
    /// Every term comes from
    /// [`MessageColumn::conversation_sort_expression`], which is the same
    /// method `conversations_in` fills its fields from. So the list is ordered
    /// by the values its rows are showing.
    pub fn conversation_order_by_clause(&self) -> String {
        match self.then.filter(|then| then.column != self.column) {
            Some(then) => format!(
                "{}, {}",
                self.first().conversation_term(),
                then.conversation_term()
            ),
            None => self.first().conversation_term(),
        }
    }

    /// How the sort is announced once it has been applied.
    pub fn spoken(&self) -> String {
        match self.then.filter(|then| then.column != self.column) {
            Some(then) => format!(
                "Sorted by {}, then by {}",
                self.first().spoken(),
                then.spoken()
            ),
            None => format!("Sorted by {}", self.first().spoken()),
        }
    }

    /// The ordering the message list actually runs for this sort.
    ///
    /// Not every column has its own comparison. Rather than let a header
    /// announce a sort that nothing performs, columns without one fall to the
    /// nearest ordering that is meaningful: anything derived from the message
    /// body or its position sorts by date, and the flag columns group their
    /// flagged rows together. Every column therefore sorts, and the
    /// announcement is always true.
    pub fn as_mail_sort_option(&self) -> Option<MailSortOption> {
        let ascending = self.direction == SortDirection::Ascending;
        Some(match self.column {
            MessageColumn::Subject if ascending => MailSortOption::SubjectAZ,
            MessageColumn::Subject => MailSortOption::SubjectZA,
            MessageColumn::Correspondent | MessageColumn::To | MessageColumn::Cc if ascending => {
                MailSortOption::SenderAZ
            }
            MessageColumn::Correspondent | MessageColumn::To | MessageColumn::Cc => {
                MailSortOption::SenderZA
            }
            MessageColumn::Unread => MailSortOption::UnreadFirst,
            _ if ascending => MailSortOption::DateOldestFirst,
            _ => MailSortOption::DateNewestFirst,
        })
    }
}

/// The kind of folder being shown, which decides the default columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Inbox,
    Sent,
    Drafts,
}

impl FolderKind {
    /// Which of the three a stored folder is read as.
    ///
    /// Junk, Trash, Archive and anything somebody made themselves all hold
    /// mail that arrived, so they are read the way an inbox is: it is the
    /// correspondent and when it turned up that matter.
    ///
    /// Nothing asked this before. Every call site named `Inbox`, so the Sent
    /// and Drafts layout, which drops the Unread column and sorts by when a
    /// message was sent, was written and tested and never once used: the
    /// Unread column was read out on every row in Sent, where it says the
    /// same thing every time.
    pub fn for_folder(folder: crate::common::types::FolderType) -> Self {
        use crate::common::types::FolderType;
        match folder {
            FolderType::Sent => FolderKind::Sent,
            FolderType::Drafts => FolderKind::Drafts,
            _ => FolderKind::Inbox,
        }
    }
}

/// Something the caller asked for that cannot be done.
///
/// The sentence inside it is announced as it stands, by the column chooser
/// that raised it. There is deliberately no `Display`: one that formatted this
/// differently would be a second way for a refusal to read, reachable from
/// nowhere, and one of the two would drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnError(pub String);

/// The visible columns, their order, and the sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnLayout {
    /// Visible columns in display order. Hidden columns are simply absent.
    order: Vec<MessageColumn>,
    pub sort: Sort,
    /// Which sort of folder this layout was built for.
    ///
    /// Carried so the window can tell when moving to another folder means a
    /// different set of columns, and leave the layout alone when it does
    /// not: rebuilding on every folder change would throw away whatever
    /// somebody had just sorted by.
    pub kind: FolderKind,
}

impl ColumnLayout {
    /// The default layout for a kind of folder.
    ///
    /// Sent and Drafts drop unread, because everything in them has been read and
    /// a column identical on every row is pure verbosity when spoken. They also
    /// show the date that means something there rather than the received date.
    pub fn defaults_for(kind: FolderKind) -> Self {
        let (order, sort_column) = match kind {
            FolderKind::Inbox => (
                vec![
                    MessageColumn::Unread,
                    MessageColumn::Attachment,
                    MessageColumn::Subject,
                    MessageColumn::Correspondent,
                    MessageColumn::Received,
                    MessageColumn::Snippet,
                ],
                MessageColumn::Received,
            ),
            FolderKind::Sent | FolderKind::Drafts => (
                vec![
                    MessageColumn::Attachment,
                    MessageColumn::Subject,
                    MessageColumn::Correspondent,
                    MessageColumn::Sent,
                    MessageColumn::Snippet,
                ],
                MessageColumn::Sent,
            ),
        };

        Self {
            kind,
            order,
            sort: Sort {
                column: sort_column,
                direction: SortDirection::Descending,
                then: None,
            },
        }
    }

    /// Visible columns in display order.
    pub fn visible(&self) -> Vec<MessageColumn> {
        self.order.clone()
    }

    /// Whether a column is currently shown.
    pub fn is_visible(&self, column: MessageColumn) -> bool {
        self.order.contains(&column)
    }

    /// Show or hide a column.
    ///
    /// The last visible column cannot be hidden: a list with no columns reads as
    /// an empty row for every message.
    pub fn set_visible(
        &mut self,
        column: MessageColumn,
        visible: bool,
    ) -> Result<String, ColumnError> {
        if visible {
            if !self.order.contains(&column) {
                self.order.push(column);
            }
            return Ok(format!("{} shown", column.heading()));
        }

        if self.order.len() == 1 && self.order[0] == column {
            return Err(ColumnError(format!(
                "{} is the only column left, so it cannot be hidden",
                column.heading()
            )));
        }
        self.order.retain(|c| *c != column);
        Ok(format!("{} hidden", column.heading()))
    }

    /// Move a visible column by `offset` positions, clamped to the ends.
    ///
    /// Clamping rather than wrapping: a column that silently jumps from one end
    /// to the other is disorienting when the only feedback is spoken.
    pub fn move_by(&mut self, column: MessageColumn, offset: i32) -> Result<String, ColumnError> {
        let Some(from) = self.order.iter().position(|c| *c == column) else {
            return Err(ColumnError(format!(
                "{} is hidden, so it cannot be moved",
                column.heading()
            )));
        };

        let last = self.order.len().saturating_sub(1);
        let to = (from as i32 + offset).clamp(0, last as i32) as usize;
        let moved = self.order.remove(from);
        self.order.insert(to, moved);

        Ok(format!(
            "{} moved to position {} of {}",
            column.heading(),
            to + 1,
            self.order.len()
        ))
    }

    /// Return to the default layout for a kind of folder.
    /// Adopt the sort chosen from the Sort Messages menu.
    ///
    /// Sender maps back to Correspondent rather than To, because that is the
    /// column the menu item is about in every folder but Sent, and Sent shows
    /// Correspondent too.
    pub fn set_sort_from_option(&mut self, option: MailSortOption) {
        self.sort = match option {
            MailSortOption::DateNewestFirst => Sort {
                column: MessageColumn::Received,
                direction: SortDirection::Descending,
                then: None,
            },
            MailSortOption::DateOldestFirst => Sort {
                column: MessageColumn::Received,
                direction: SortDirection::Ascending,
                then: None,
            },
            MailSortOption::SenderAZ => Sort {
                column: MessageColumn::Correspondent,
                direction: SortDirection::Ascending,
                then: None,
            },
            MailSortOption::SenderZA => Sort {
                column: MessageColumn::Correspondent,
                direction: SortDirection::Descending,
                then: None,
            },
            MailSortOption::SubjectAZ => Sort {
                column: MessageColumn::Subject,
                direction: SortDirection::Ascending,
                then: None,
            },
            MailSortOption::SubjectZA => Sort {
                column: MessageColumn::Subject,
                direction: SortDirection::Descending,
                then: None,
            },
            MailSortOption::UnreadFirst => Sort {
                column: MessageColumn::Unread,
                direction: SortDirection::Descending,
                then: None,
            },
        };
    }

    /// Sort by a column, reversing if it is already the sort column.
    ///
    /// Returns what to announce. A header click that changes the order without
    /// saying so is a silent change to the thing under the user's cursor, and
    /// the first click has to pick the direction people actually want: newest
    /// first for a date, A to Z for text.
    pub fn sort_by(&mut self, column: MessageColumn) -> String {
        // The second level is kept across a change of the first, because it is
        // a standing preference rather than part of this click. Somebody who
        // has asked for unread first among equals still wants that after
        // sorting by sender.
        let then = self.sort.then;
        self.sort = if self.sort.column == column {
            Sort {
                column,
                direction: match self.sort.direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                },
                then,
            }
        } else {
            Sort {
                column,
                direction: match column {
                    MessageColumn::Received | MessageColumn::Sent => SortDirection::Descending,
                    _ => SortDirection::Ascending,
                },
                then,
            }
        };
        self.sort.spoken()
    }

    pub fn reset(&mut self, kind: FolderKind) {
        *self = Self::defaults_for(kind);
    }

    /// The layout in its stored form.
    pub fn to_stored(&self) -> String {
        let columns: Vec<&str> = self.order.iter().map(|c| c.key()).collect();
        // The second level is appended after a semicolon, so a layout written
        // before there was one still reads here and one written here still
        // reads in a build that only knows the first.
        let sort = match self.sort.then {
            Some(then) => format!("{};{}", self.sort.first().key(), then.key()),
            None => self.sort.first().key(),
        };
        format!("{}|{}", columns.join(","), sort)
    }

    /// Read a stored layout, falling back to the default for anything unusable.
    ///
    /// Unknown column names are skipped rather than rejected, so a layout
    /// written by a newer version that knows more columns still restores the
    /// ones this build understands instead of losing the lot.
    pub fn from_stored(stored: &str, kind: FolderKind) -> Self {
        let default = Self::defaults_for(kind);
        let Some((columns, sort)) = stored.split_once('|') else {
            return default;
        };

        let order: Vec<MessageColumn> = columns
            .split(',')
            .filter_map(MessageColumn::from_key)
            .collect();
        if order.is_empty() {
            return default;
        }

        let (first, then) = match sort.split_once(';') {
            Some((first, then)) => (first, By::from_key(then)),
            None => (sort, None),
        };
        let sort = By::from_key(first)
            .map(|by| Sort {
                column: by.column,
                direction: by.direction,
                then,
            })
            .unwrap_or(default.sort);

        Self { kind, order, sort }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_menu_sort_moves_the_layout_with_it() {
        // The menu and the column headers must not end up disagreeing about
        // how the list is ordered; the menu is the only place that states it
        // in words, so it is the one people check.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        for option in [
            MailSortOption::DateNewestFirst,
            MailSortOption::DateOldestFirst,
            MailSortOption::SenderAZ,
            MailSortOption::SenderZA,
            MailSortOption::SubjectAZ,
            MailSortOption::SubjectZA,
            MailSortOption::UnreadFirst,
        ] {
            layout.set_sort_from_option(option);
            assert_eq!(
                layout.sort.as_mail_sort_option(),
                Some(option),
                "{:?} did not survive the round trip",
                option
            );
        }
    }

    #[test]
    fn test_clicking_a_header_sorts_then_reverses() {
        // The first click picks a sensible direction for the column, the
        // second reverses it. Anything else means someone who cannot see the
        // header has no way to reach descending order by clicking at all.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.sort = Sort {
            column: MessageColumn::Received,
            direction: SortDirection::Descending,
            then: None,
        };

        let said = layout.sort_by(MessageColumn::Subject);
        assert_eq!(layout.sort.column, MessageColumn::Subject);
        assert_eq!(layout.sort.direction, SortDirection::Ascending);
        assert_eq!(said, "Sorted by subject, ascending");

        let said = layout.sort_by(MessageColumn::Subject);
        assert_eq!(layout.sort.direction, SortDirection::Descending);
        assert_eq!(said, "Sorted by subject, descending");
    }

    #[test]
    fn test_a_date_column_starts_at_newest_first() {
        // Nobody opens a mailbox wanting the oldest message at the top.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.sort = Sort {
            column: MessageColumn::Subject,
            direction: SortDirection::Ascending,
            then: None,
        };
        let said = layout.sort_by(MessageColumn::Received);
        assert_eq!(layout.sort.direction, SortDirection::Descending);
        assert_eq!(said, "Sorted by received, newest first");
    }

    #[test]
    fn test_every_column_maps_to_a_sort_that_runs() {
        // A header that announces a sort and does not perform one is worse
        // than a header that cannot be sorted.
        for column in MessageColumn::ALL {
            for direction in [SortDirection::Ascending, SortDirection::Descending] {
                let sort = Sort {
                    column,
                    direction,
                    then: None,
                };
                assert!(
                    sort.as_mail_sort_option().is_some(),
                    "{:?} {:?} has no sort to run",
                    column,
                    direction
                );
            }
        }
    }

    #[test]
    fn test_unread_sorts_unread_to_the_top_either_way() {
        // There is one unread-first ordering, so both directions land on it
        // rather than one of them silently doing nothing.
        for direction in [SortDirection::Ascending, SortDirection::Descending] {
            let sort = Sort {
                column: MessageColumn::Unread,
                direction,
                then: None,
            };
            assert_eq!(
                sort.as_mail_sort_option(),
                Some(crate::presentation::ui_types::MailSortOption::UnreadFirst)
            );
        }
    }

    #[test]
    fn test_a_second_level_decides_the_rows_the_first_calls_equal() {
        // Newest first is right until there are twenty from today, and then
        // what is wanted is unread first among those.
        let sort = Sort {
            column: MessageColumn::Received,
            direction: SortDirection::Descending,
            then: Some(By {
                column: MessageColumn::Unread,
                direction: SortDirection::Ascending,
            }),
        };

        assert_eq!(
            sort.order_by_clause(),
            "COALESCE(m.internaldate, m.date) DESC, m.read ASC"
        );
        assert_eq!(
            sort.spoken(),
            "Sorted by received, newest first, then by unread, ascending"
        );
    }

    #[test]
    fn test_one_level_is_written_and_said_the_way_it_always_was() {
        let sort = Sort {
            column: MessageColumn::Subject,
            direction: SortDirection::Ascending,
            then: None,
        };

        assert_eq!(sort.order_by_clause(), "m.subject COLLATE NOCASE ASC");
        assert_eq!(sort.spoken(), "Sorted by subject, ascending");
    }

    #[test]
    fn test_a_second_level_on_the_same_column_is_left_out() {
        // It decides nothing, and announcing it reads as though it might.
        let sort = Sort {
            column: MessageColumn::Subject,
            direction: SortDirection::Ascending,
            then: Some(By {
                column: MessageColumn::Subject,
                direction: SortDirection::Descending,
            }),
        };

        assert_eq!(sort.order_by_clause(), "m.subject COLLATE NOCASE ASC");
        assert!(!sort.spoken().contains("then by"), "{}", sort.spoken());
    }

    #[test]
    fn test_both_levels_survive_being_stored() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.sort.then = Some(By {
            column: MessageColumn::Unread,
            direction: SortDirection::Ascending,
        });

        let back = ColumnLayout::from_stored(&layout.to_stored(), FolderKind::Inbox);

        assert_eq!(back.sort, layout.sort);
    }

    #[test]
    fn test_a_layout_stored_before_there_was_a_second_level_still_reads() {
        // Written by every build up to now. It has one level and no semicolon,
        // and losing somebody's column arrangement over that would be a poor
        // trade for a feature they had not asked for yet.
        let old = "unread,subject,received|received:desc";

        let back = ColumnLayout::from_stored(old, FolderKind::Inbox);

        assert_eq!(back.sort.column, MessageColumn::Received);
        assert_eq!(back.sort.direction, SortDirection::Descending);
        assert_eq!(back.sort.then, None);
        assert_eq!(back.order.len(), 3);
    }

    #[test]
    fn test_a_second_level_naming_a_column_this_build_does_not_know_is_dropped() {
        // Rather than losing the whole layout. A newer build may know columns
        // this one does not.
        let newer = "unread,subject|subject:asc;invented:desc";

        let back = ColumnLayout::from_stored(newer, FolderKind::Inbox);

        assert_eq!(back.sort.column, MessageColumn::Subject);
        assert_eq!(back.sort.then, None);
    }

    #[test]
    fn test_changing_the_first_level_keeps_the_second() {
        // It is a standing preference rather than part of this click.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.sort.then = Some(By {
            column: MessageColumn::Unread,
            direction: SortDirection::Ascending,
        });

        layout.sort_by(MessageColumn::Correspondent);

        assert_eq!(
            layout.sort.then,
            Some(By {
                column: MessageColumn::Unread,
                direction: SortDirection::Ascending
            })
        );
    }

    #[test]
    fn test_inbox_defaults_match_the_agreed_set() {
        let layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        assert_eq!(
            layout.visible(),
            vec![
                MessageColumn::Unread,
                MessageColumn::Attachment,
                MessageColumn::Subject,
                MessageColumn::Correspondent,
                MessageColumn::Received,
                MessageColumn::Snippet,
            ]
        );
    }

    #[test]
    fn test_sent_and_drafts_drop_unread() {
        // Everything in Sent has been read, so the column is identical on every
        // row. A column that never varies is pure verbosity when it is spoken.
        for kind in [FolderKind::Sent, FolderKind::Drafts] {
            assert!(
                !ColumnLayout::defaults_for(kind).is_visible(MessageColumn::Unread),
                "{:?} should not show unread by default",
                kind
            );
        }
    }

    #[test]
    fn test_sent_shows_the_sent_date_not_the_received_date() {
        let layout = ColumnLayout::defaults_for(FolderKind::Sent);
        assert!(layout.is_visible(MessageColumn::Sent));
        assert!(!layout.is_visible(MessageColumn::Received));
    }

    #[test]
    fn test_every_column_can_sort_both_ways() {
        for column in MessageColumn::ALL {
            for direction in [SortDirection::Ascending, SortDirection::Descending] {
                let sort = Sort {
                    column,
                    direction,
                    then: None,
                };
                assert!(!sort.order_by_clause().is_empty());
            }
        }
    }

    #[test]
    fn test_sort_clause_is_a_fixed_string_per_column() {
        // The clause is interpolated into SQL, so it must never come from user
        // input. Matching on the enum is what guarantees that.
        let sort = Sort {
            column: MessageColumn::Received,
            direction: SortDirection::Descending,
            then: None,
        };
        assert_eq!(
            sort.order_by_clause(),
            "COALESCE(m.internaldate, m.date) DESC"
        );
    }

    #[test]
    fn test_a_safety_sort_is_a_ranking_with_a_direction_on_the_end_of_it() {
        // Every other column's expression is a column name, and appending a
        // direction to one of those is the one shape `term` was written
        // against. Safety's is a `CASE`, so the appending is asserted here, in
        // both directions, rather than left to read correctly.
        //
        // What order the ranking puts the verdicts in is not asked here. That
        // is a claim about rows and it is made against a real database, in
        // `data::message_cache::messages`, because this arm and the
        // conversation arm are built from one ranking and a check that read it
        // on both sides would agree with itself whatever it said.
        for (direction, ending) in [
            (SortDirection::Ascending, " ASC"),
            (SortDirection::Descending, " DESC"),
        ] {
            let clause = Sort {
                column: MessageColumn::Safety,
                direction,
                then: None,
            }
            .order_by_clause();

            assert!(
                clause.starts_with("CASE m.safety"),
                "the message view is ordering the stored words rather than ranking them: {clause}"
            );
            assert!(
                clause.ends_with(ending),
                "the direction did not survive being appended to a CASE: {clause}"
            );
        }
    }

    #[test]
    fn test_default_sort_is_newest_received_first() {
        let layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        assert_eq!(layout.sort.column, MessageColumn::Received);
        assert_eq!(layout.sort.direction, SortDirection::Descending);
    }

    #[test]
    fn test_hiding_a_column_removes_it_from_the_visible_order() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        assert!(layout.set_visible(MessageColumn::Snippet, false).is_ok());
        assert!(!layout.is_visible(MessageColumn::Snippet));
        assert!(!layout.visible().contains(&MessageColumn::Snippet));
    }

    #[test]
    fn test_the_last_visible_column_cannot_be_hidden() {
        // A list with no columns is a list with no content, and a screen reader
        // would read an empty row for every message.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        let all: Vec<_> = layout.visible();
        for column in all.iter().take(all.len() - 1) {
            layout.set_visible(*column, false).unwrap();
        }
        let last = layout.visible()[0];
        assert!(
            layout.set_visible(last, false).is_err(),
            "hiding the last column should be refused"
        );
        assert_eq!(layout.visible().len(), 1);
    }

    #[test]
    fn test_moving_a_column_changes_its_position() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        assert_eq!(layout.visible()[0], MessageColumn::Unread);
        layout.move_by(MessageColumn::Subject, -2).unwrap();
        assert_eq!(layout.visible()[0], MessageColumn::Subject);
    }

    #[test]
    fn test_moving_past_the_end_clamps_rather_than_wrapping() {
        // Wrapping would silently move a column to the opposite end, which is
        // disorienting when the only feedback is spoken.
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.move_by(MessageColumn::Unread, -5).unwrap();
        assert_eq!(layout.visible()[0], MessageColumn::Unread);
        layout.move_by(MessageColumn::Snippet, 5).unwrap();
        assert_eq!(*layout.visible().last().unwrap(), MessageColumn::Snippet);
    }

    #[test]
    fn test_moving_a_hidden_column_is_refused() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.set_visible(MessageColumn::Snippet, false).unwrap();
        assert!(layout.move_by(MessageColumn::Snippet, -1).is_err());
    }

    #[test]
    fn test_move_announcement_says_where_it_landed() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        let spoken = layout.move_by(MessageColumn::Subject, -1).unwrap();
        assert_eq!(spoken, "Subject moved to position 2 of 6");
    }

    #[test]
    fn test_reset_returns_to_the_folder_default() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        layout.set_visible(MessageColumn::Snippet, false).unwrap();
        layout.move_by(MessageColumn::Subject, -2).unwrap();
        layout.reset(FolderKind::Inbox);
        assert_eq!(layout, ColumnLayout::defaults_for(FolderKind::Inbox));
    }

    #[test]
    fn test_column_headings_are_plain_words() {
        for column in MessageColumn::ALL {
            let heading = column.heading();
            assert!(!heading.is_empty());
            assert!(
                !heading.contains('&'),
                "{:?} heading carries a mnemonic it cannot use",
                column
            );
        }
    }

    #[test]
    fn test_layout_round_trips_through_its_stored_form() {
        let mut layout = ColumnLayout::defaults_for(FolderKind::Sent);
        layout.set_visible(MessageColumn::Size, true).unwrap();
        layout.move_by(MessageColumn::Size, -1).unwrap();
        layout.sort = Sort {
            column: MessageColumn::Size,
            direction: SortDirection::Ascending,
            then: None,
        };

        let stored = layout.to_stored();
        let restored = ColumnLayout::from_stored(&stored, FolderKind::Sent);
        assert_eq!(restored, layout);
    }

    #[test]
    fn test_unreadable_stored_layout_falls_back_to_the_default() {
        // A corrupted preference must not leave someone with an empty list.
        let restored = ColumnLayout::from_stored("nonsense,,,", FolderKind::Inbox);
        assert_eq!(restored, ColumnLayout::defaults_for(FolderKind::Inbox));
    }

    #[test]
    fn test_stored_form_ignores_columns_it_does_not_recognise() {
        // Forward compatibility: a newer version may have written a column this
        // build has never heard of, and that must not lose the rest.
        let restored = ColumnLayout::from_stored(
            "unread,invented_column,subject|received:desc",
            FolderKind::Inbox,
        );
        assert_eq!(
            restored.visible(),
            vec![MessageColumn::Unread, MessageColumn::Subject]
        );
    }
}

#[cfg(test)]
mod the_sort_somebody_asked_for {
    use super::*;

    #[test]
    fn test_each_offered_order_maps_to_a_real_sort() {
        // The Settings list offers seven, and every one of them was ignored:
        // nothing read the setting, so the list always opened newest first
        // and anybody who wanted otherwise sorted by hand on every folder.
        for stored in [
            "date_newest",
            "date_oldest",
            "sender_az",
            "sender_za",
            "subject_az",
            "subject_za",
            "unread_first",
        ] {
            assert!(
                Sort::from_setting(stored).is_some(),
                "{stored} is offered in Settings and means nothing here"
            );
        }
    }

    #[test]
    fn test_oldest_and_newest_are_the_same_column_the_other_way_round() {
        let newest = Sort::from_setting("date_newest").expect("newest first");
        let oldest = Sort::from_setting("date_oldest").expect("oldest first");

        assert_eq!(newest.column, MessageColumn::Received);
        assert_eq!(oldest.column, MessageColumn::Received);
        assert_ne!(newest.direction, oldest.direction);
    }

    #[test]
    fn test_unread_first_still_puts_the_newest_at_the_top_of_each_group() {
        // "Unread first" says nothing about the order inside either group, and
        // the answer people expect underneath it is the ordinary one.
        let sort = Sort::from_setting("unread_first").expect("unread first");

        assert_eq!(sort.column, MessageColumn::Unread);
        assert_eq!(
            sort.then,
            Some(By {
                column: MessageColumn::Received,
                direction: SortDirection::Descending,
            })
        );
    }

    #[test]
    fn test_a_word_this_does_not_know_leaves_the_built_in_order_alone() {
        // A settings file from a later version, or edited by hand.
        assert_eq!(Sort::from_setting("by_moon_phase"), None);
        assert_eq!(Sort::from_setting(""), None);
    }
}

#[cfg(test)]
mod which_columns_a_folder_gets {
    use super::*;
    use crate::common::types::FolderType;

    #[test]
    fn test_sent_and_drafts_are_told_apart_from_everything_else() {
        // A layout that drops the Unread column and sorts by when a message
        // was sent has been written and tested since these columns existed,
        // and nothing ever asked for it: every call site named Inbox. So in
        // Sent and Drafts the Unread column was read out on every row, where
        // it is identical on all of them, and the date shown was when the
        // message arrived rather than when it went.
        assert_eq!(FolderKind::for_folder(FolderType::Sent), FolderKind::Sent);
        assert_eq!(
            FolderKind::for_folder(FolderType::Drafts),
            FolderKind::Drafts
        );
    }

    #[test]
    fn test_every_other_folder_is_read_like_an_inbox() {
        // Junk, Trash, Archive and anything somebody made themselves all hold
        // mail that arrived, so they are read the way an inbox is.
        for kind in [
            FolderType::Inbox,
            FolderType::Trash,
            FolderType::Spam,
            FolderType::Archive,
            FolderType::Custom,
        ] {
            assert_eq!(
                FolderKind::for_folder(kind),
                FolderKind::Inbox,
                "{kind:?} should be read like an inbox"
            );
        }
    }

    #[test]
    fn test_the_sent_layout_leaves_out_the_column_that_is_the_same_on_every_row() {
        let sent = ColumnLayout::defaults_for(FolderKind::Sent);

        assert!(
            !sent.visible().contains(&MessageColumn::Unread),
            "Unread is identical on every row in Sent and is pure verbosity spoken"
        );
    }
}
