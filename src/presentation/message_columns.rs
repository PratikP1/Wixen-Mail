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

use std::fmt;

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
    Answered,
    Draft,
    To,
    Cc,
    Tags,
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
        MessageColumn::To,
        MessageColumn::Cc,
        MessageColumn::Tags,
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
            MessageColumn::To => "To",
            MessageColumn::Cc => "Cc",
            MessageColumn::Tags => "Tags",
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
            MessageColumn::To => "to",
            MessageColumn::Cc => "cc",
            MessageColumn::Tags => "tags",
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
            MessageColumn::Received => "m.internaldate",
            MessageColumn::Sent => "m.date",
            MessageColumn::Snippet => "m.snippet COLLATE NOCASE",
            MessageColumn::Thread => "m.thread_id",
            MessageColumn::Size => "m.size_bytes",
            MessageColumn::Flagged => "m.starred",
            MessageColumn::Answered => "m.answered",
            MessageColumn::Draft => "m.draft",
            MessageColumn::To => "m.to_addr COLLATE NOCASE",
            MessageColumn::Cc => "m.cc COLLATE NOCASE",
            MessageColumn::Tags => "m.tags COLLATE NOCASE",
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
    pub fn spoken_for(&self, column: MessageColumn) -> &'static str {
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

/// How the list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub column: MessageColumn,
    pub direction: SortDirection,
}

impl Sort {
    /// The `ORDER BY` body for this sort.
    pub fn order_by_clause(&self) -> String {
        let direction = match self.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };
        format!("{} {}", self.column.sort_expression(), direction)
    }

    /// How the sort is announced once it has been applied.
    pub fn spoken(&self) -> String {
        format!(
            "Sorted by {}, {}",
            self.column.heading().to_lowercase(),
            self.direction.spoken_for(self.column)
        )
    }
}

/// The kind of folder being shown, which decides the default columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Inbox,
    Sent,
    Drafts,
}

/// Something the caller asked for that cannot be done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnError(pub String);

impl fmt::Display for ColumnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The visible columns, their order, and the sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnLayout {
    /// Visible columns in display order. Hidden columns are simply absent.
    order: Vec<MessageColumn>,
    pub sort: Sort,
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
            order,
            sort: Sort {
                column: sort_column,
                direction: SortDirection::Descending,
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
    pub fn reset(&mut self, kind: FolderKind) {
        *self = Self::defaults_for(kind);
    }

    /// The layout in its stored form.
    pub fn to_stored(&self) -> String {
        let columns: Vec<&str> = self.order.iter().map(|c| c.key()).collect();
        format!(
            "{}|{}:{}",
            columns.join(","),
            self.sort.column.key(),
            self.sort.direction.key()
        )
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

        let sort = sort
            .split_once(':')
            .and_then(|(column, direction)| {
                let column = MessageColumn::from_key(column)?;
                let direction = match direction {
                    "asc" => SortDirection::Ascending,
                    "desc" => SortDirection::Descending,
                    _ => return None,
                };
                Some(Sort { column, direction })
            })
            .unwrap_or(default.sort);

        Self { order, sort }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                let sort = Sort { column, direction };
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
        };
        assert_eq!(sort.order_by_clause(), "m.internaldate DESC");
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
