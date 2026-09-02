//! The conversation tree.
//!
//! A threaded message list forces one shape on everybody: either the list is
//! flat and the conversation structure is lost, or it is a tree and every
//! arrow press walks branches you did not ask for. This keeps the list flat and
//! puts the structure behind `Enter`, so the structure is there when you want
//! it and out of the way when you do not.
//!
//! A native `TreeCtrl`, so the control announces the level itself. A screen
//! reader says "level 3" without us describing it, and it says it the way that
//! user has configured their screen reader to say it.

use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::theme;
use crate::presentation::tree_walk;
use std::sync::Arc;
use wxdragon::prelude::*;

/// The dialog's third answer, alongside the stock OK and Cancel.
const ID_PLAIN_TEXT: Id = ID_HIGHEST + 950;

/// One message as the tree shows it.
#[derive(Debug, Clone)]
pub struct ThreadNode {
    /// The cache row id, so a choice can be acted on.
    pub message_id: i64,
    /// The message's UID on the server, so its attachments can be fetched.
    pub uid: u32,
    pub sender: String,
    pub subject: String,
    pub date: String,
    pub read: bool,
    pub depth: usize,
    /// Index of the parent in the same slice, where there is one.
    pub parent: Option<usize>,
}

/// What the user chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreadChoice {
    /// Show the whole conversation as one plain document.
    ///
    /// The text control: focusable, arrow-navigable, searchable, and with no
    /// structure in it at all. Worth having, and not what a conversation should
    /// open into, which is why it is no longer what Enter does.
    WholeConversation,
    /// Show the whole conversation as a page, with real headings and links.
    ///
    /// What opening a conversation does. A conversation is a shape, and the
    /// text control has no way to express one: no headings for `H` to move
    /// between, no links a screen reader can list, no way to tell where one
    /// message ends and the next begins except by reading it all.
    AsHeadings,
    /// Show this message alone.
    Message(i64),
    /// Nothing; go back to the list.
    Cancelled,
}

/// How a message reads in the tree.
///
/// The unread state is first because it is the one thing that changes what you
/// do next, and last would mean listening past the sender and the date to
/// reach it.
fn node_label(node: &ThreadNode) -> String {
    let unread = if node.read { "" } else { "Unread. " };
    let subject = if node.subject.trim().is_empty() {
        "No subject"
    } else {
        node.subject.trim()
    };
    format!("{}{}. {}. {}", unread, node.sender, subject, node.date)
}

/// One row of the conversation tree, in the order a depth-first walk of the
/// built tree meets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    /// Which node in the slice this row shows.
    node: usize,
    /// Which node's row it hangs under. `None` means the root.
    under: Option<usize>,
}

/// The rows of the conversation tree, in the order the built tree is walked.
///
/// Not the order the nodes arrive in, and the two differ whenever a message
/// replies to something other than the one before it: a reply sits under its
/// parent, so the walk meets it immediately after that parent rather than
/// where it sat in the slice.
///
/// This is what makes a vector beside the control exact rather than
/// approximate. The rows are appended in this order and each id is pushed in
/// the same step, so the control's walk and the vector are one sequence rather
/// than two that have to be kept in step.
fn rows_in_walk_order(nodes: &[ThreadNode]) -> Vec<Row> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| Row {
            node: index,
            under: node.parent.filter(|parent| *parent < index),
        })
        .collect()
}

/// What a selection at `position` in the tree's walk means.
///
/// `None` is the root, which is not walked and so has no position, and the
/// root means the whole conversation. A position past the end of `ids` means
/// the same: the safe answer to a row this cannot place is the conversation,
/// never a message nobody pointed at.
fn what_a_selection_means(ids: &[i64], position: Option<usize>) -> ThreadChoice {
    match position.and_then(|position| ids.get(position)) {
        Some(id) => ThreadChoice::Message(*id),
        None => ThreadChoice::AsHeadings,
    }
}

/// How the root of the tree reads.
///
/// It says what `Enter` will do rather than repeating the subject, because
/// `Enter` means two different things in this tree and the row has to say
/// which. A row labelled with the subject would give no clue that activating
/// it opens all five messages rather than that one.
fn root_label(count: usize) -> String {
    format!(
        "Whole conversation, {} message{}",
        count,
        if count == 1 { "" } else { "s" }
    )
}

/// Show the conversation tree and return what the user chose.
///
/// `nodes` must be in display order, parents before their children.
pub fn show_thread_dialog(
    parent: &Frame,
    subject: &str,
    nodes: &[ThreadNode],
    a11y: &Arc<Accessibility>,
) -> ThreadChoice {
    use crate::presentation::accessibility::announcements::Priority;

    if nodes.is_empty() {
        return ThreadChoice::Cancelled;
    }

    let Some((dlg, _tree, chosen)) =
        build_thread_dialog(parent, subject, nodes, theme::current_from_stored_config())
    else {
        return ThreadChoice::Cancelled;
    };

    let _ = a11y.announce(
        &format!("Conversation, {} messages", nodes.len()),
        Priority::Normal,
    );

    match dlg.show_modal() {
        ID_OK => chosen.borrow().clone(),
        // Always the whole thread, whichever row the tree was on. The plain
        // reading is of the conversation, and one message already has its own
        // way in through Enter.
        ID_PLAIN_TEXT => ThreadChoice::WholeConversation,
        _ => ThreadChoice::Cancelled,
    }
}

/// Build the Conversation tree dialog without showing it.
///
/// Everything `show_thread_dialog` used to do up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` or make an
/// announcement at all.
///
/// `None` when the tree's root could not be created, mirroring what
/// `show_thread_dialog` itself used to do on the same failure.
///
/// Returns what was chosen alongside the dialog: selecting a row in the tree
/// keeps this current, so reading it back is all `show_thread_dialog` has to
/// do once the dialog closes.
///
/// The tree comes back too, which `show_thread_dialog` does not need and a
/// test does: choosing a row is the whole of what this dialog does, and
/// without the control there is no way to choose one and watch the answer
/// change. `tests/tree_dialogs_resolve_the_row_somebody_is_on.rs` does exactly
/// that.
pub fn build_thread_dialog(
    parent: &Frame,
    subject: &str,
    nodes: &[ThreadNode],
    palette: Option<theme::Palette>,
) -> Option<(
    Dialog,
    TreeCtrl,
    std::rc::Rc<std::cell::RefCell<ThreadChoice>>,
)> {
    let dlg = Dialog::builder(parent, "Conversation")
        .with_size(620, 460)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let hint = StaticText::builder(&dlg)
        .with_label(
            "Enter on the first row opens the whole conversation. Enter on a message opens \
             that message. As Headings opens the whole conversation as a page, where H moves \
             between messages. Escape goes back to the list.",
        )
        .build();
    sizer.add(&hint, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let tree = TreeCtrl::builder(&dlg).build();
    set_accessible_name(&tree, &format!("Conversation: {}", subject));
    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let open = Button::builder(&dlg)
        .with_label("&Open")
        .with_id(ID_OK)
        .build();
    let plain = Button::builder(&dlg)
        .with_label("As Plain &Text")
        .with_id(ID_PLAIN_TEXT)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    buttons.add(&open, 0, SizerFlag::All, 4);
    buttons.add(&plain, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    // The root is the whole conversation, and every message hangs under its
    // own parent, so the control's own level announcement matches the reply
    // depth rather than something we invented.
    let Some(root) = tree.add_root(&root_label(nodes.len()), None, None) else {
        tracing::error!("Conversation tree root could not be created");
        return None;
    };

    // The message ids are held here rather than hung off the rows. Labels
    // repeat within a conversation, so a label lookup is out; what makes a
    // position lookup safe is that the row and its id are produced in the same
    // step below, in the one order [`rows_in_walk_order`] gives, and that this
    // tree is built once and never touched again while the dialog is open.
    // Nothing is inserted, removed or re-sorted under it.
    //
    // Hanging the id off the row instead is what this used to do, and it is
    // not the cheap option it looks like: `set_custom_data` puts the id in a
    // process-global map in `wxdragon` and nothing takes it out, so every
    // opening of this dialog left one entry per message behind for the life of
    // the program. `tests/tree_rows_leave_no_registry_entry.rs` counts them.
    let mut items: Vec<Option<TreeItemId>> = vec![None; nodes.len()];
    let mut ids: Vec<i64> = Vec::with_capacity(nodes.len());
    for row in rows_in_walk_order(nodes) {
        let under = row
            .under
            .and_then(|parent| items[parent].clone())
            .unwrap_or_else(|| root.clone());
        let Some(item) = tree.append_item(&under, &node_label(&nodes[row.node]), None, None) else {
            // A row that could not be appended would leave the walk one row
            // short of the ids beside it, and every row after it would resolve
            // to the message before it. Better no conversation window at all
            // than one that opens the wrong message, which is the same
            // judgement the missing root above makes.
            tracing::error!("Conversation tree row could not be created");
            return None;
        };
        items[row.node] = Some(item);
        ids.push(nodes[row.node].message_id);
    }
    tree.expand_all();
    tree.select_item(&root);
    tree.set_focus();

    let chosen = std::rc::Rc::new(std::cell::RefCell::new(ThreadChoice::AsHeadings));

    // Selection drives the choice, so pressing Enter, clicking Open, and
    // double-clicking a row all act on the same thing: the row you are on.
    tree.on_selection_changed({
        let chosen = chosen.clone();
        move |_event| {
            // The control is asked which row is selected rather than the event
            // being asked which row it is about. `TreeItemId` has no equality,
            // so an item the event hands over cannot be matched against
            // anything; `is_selected`, asked of each row in turn, needs no
            // comparison and gives an exact position.
            *chosen.borrow_mut() =
                what_a_selection_means(&ids, tree_walk::where_the_selection_sits(&tree));
        }
    });

    tree.on_item_activated(move |_| dlg.end_modal(ID_OK));
    open.on_click(move |_| dlg.end_modal(ID_OK));
    plain.on_click(move |_| dlg.end_modal(ID_PLAIN_TEXT));
    cancel.on_click(move |_| dlg.end_modal(ID_CANCEL));

    // Painted last. The tree is left to Windows here, the same as every
    // `Choice`, `ComboBox`, `RadioButton` and `CheckBox` elsewhere in this
    // round. `None` means high contrast is on, or the system is set up in a
    // way this application should not paint over, so nothing is set here and
    // Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
    }

    Some((dlg, tree, chosen))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(sender: &str, read: bool) -> ThreadNode {
        ThreadNode {
            uid: 0,
            message_id: 1,
            sender: sender.to_string(),
            subject: "Quarterly report".to_string(),
            date: "2026-07-26".to_string(),
            read,
            depth: 0,
            parent: None,
        }
    }

    /// One message of a conversation, identified by the id a choice acts on.
    fn message(message_id: i64, parent: Option<usize>) -> ThreadNode {
        ThreadNode {
            message_id,
            uid: message_id as u32,
            parent,
            depth: parent.map_or(0, |_| 1),
            ..node("Ada Lovelace", true)
        }
    }

    /// Four messages whose arrival order and walk order are not the same
    /// sequence.
    ///
    /// The third message replies to the first, so the built tree meets it
    /// straight after the first while the slice holds it two places later.
    /// A fixture whose two orders agreed would let the arrival order pass as
    /// the walk order, and the test would read as green on arrival while
    /// measuring nothing.
    fn a_conversation_whose_two_orders_differ() -> Vec<ThreadNode> {
        vec![
            message(11, None),
            message(22, None),
            message(33, Some(0)),
            message(44, None),
        ]
    }

    fn ids(nodes: &[ThreadNode], rows: &[Row]) -> Vec<i64> {
        rows.iter().map(|row| nodes[row.node].message_id).collect()
    }

    #[test]
    fn test_the_fixtures_two_orders_really_do_differ() {
        // Guarding the fixture rather than the code. Every test below is only
        // as strong as this difference, and a later simplification that made
        // the two orders agree would leave them all passing against an
        // ordering function that returned its input.
        let nodes = a_conversation_whose_two_orders_differ();
        let arrived: Vec<i64> = nodes.iter().map(|node| node.message_id).collect();
        let walked = ids(&nodes, &rows_in_walk_order(&nodes));

        assert_ne!(arrived, walked, "the fixture cannot tell the two apart");
    }

    #[test]
    fn test_a_reply_is_walked_under_the_message_it_replies_to_not_where_it_arrived() {
        let nodes = a_conversation_whose_two_orders_differ();

        assert_eq!(
            ids(&nodes, &rows_in_walk_order(&nodes)),
            vec![11, 33, 22, 44],
            "the reply to the first message is walked straight after it"
        );
    }

    #[test]
    fn test_two_replies_to_one_message_sit_under_it_in_the_order_they_arrived() {
        let nodes = vec![
            message(11, None),
            message(22, Some(0)),
            message(33, None),
            message(44, Some(0)),
        ];

        assert_eq!(
            ids(&nodes, &rows_in_walk_order(&nodes)),
            vec![11, 22, 44, 33]
        );
    }

    #[test]
    fn test_the_root_is_not_a_row_so_the_first_walked_row_is_the_first_message() {
        // The root is the whole conversation and is never appended, so nothing
        // in the walk stands for it and position 0 is a real message.
        let nodes = a_conversation_whose_two_orders_differ();
        let rows = rows_in_walk_order(&nodes);

        assert_eq!(rows.len(), nodes.len());
        assert_eq!(rows[0].under, None);
        assert_eq!(ids(&nodes, &rows)[0], 11);
        // And the row after it hangs under it rather than under the root,
        // which is what puts it there.
        assert_eq!(rows[1].under, Some(0));
    }

    #[test]
    fn test_a_row_resolves_to_its_own_message() {
        let nodes = a_conversation_whose_two_orders_differ();
        let ids = ids(&nodes, &rows_in_walk_order(&nodes));

        // Each position written out rather than read back off the ordering.
        // Walking the ordering and asserting each row against itself would
        // pass whatever the ordering said, which is no assertion at all.
        for (position, id) in [(0, 11), (1, 33), (2, 22), (3, 44)] {
            assert_eq!(
                what_a_selection_means(&ids, Some(position)),
                ThreadChoice::Message(id),
                "position {position}"
            );
        }
    }

    #[test]
    fn test_the_root_means_the_whole_conversation() {
        // The root is not walked, so it has no position, and neither has a
        // position past the end of the rows. Both mean the conversation rather
        // than a message nobody pointed at.
        let nodes = a_conversation_whose_two_orders_differ();
        let ids = ids(&nodes, &rows_in_walk_order(&nodes));

        assert_eq!(what_a_selection_means(&ids, None), ThreadChoice::AsHeadings);
        assert_eq!(
            what_a_selection_means(&ids, Some(ids.len())),
            ThreadChoice::AsHeadings
        );
    }

    #[test]
    fn test_unread_is_said_first_because_it_changes_what_you_do_next() {
        // Last would mean listening past the sender and the date to reach the
        // one piece of state that matters.
        let label = node_label(&node("Ada Lovelace", false));
        assert!(label.starts_with("Unread. Ada Lovelace"));
        assert!(!node_label(&node("Ada Lovelace", true)).contains("Unread"));
    }

    #[test]
    fn test_a_node_with_no_subject_says_so() {
        let mut n = node("Ada", true);
        n.subject = "  ".to_string();
        assert!(node_label(&n).contains("No subject"));
    }

    #[test]
    fn test_the_root_says_what_enter_will_do_rather_than_the_subject() {
        // Enter means two things in this tree, so the row has to say which.
        assert_eq!(root_label(5), "Whole conversation, 5 messages");
        assert_eq!(root_label(1), "Whole conversation, 1 message");
    }
}
