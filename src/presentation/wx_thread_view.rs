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
use std::sync::Arc;
use wxdragon::prelude::*;

/// The dialog's third answer, alongside the stock OK and Cancel.
const ID_HEADINGS: Id = ID_HIGHEST + 950;

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
    /// Show the whole conversation as one document.
    WholeConversation,
    /// Show the whole conversation as a page, with real headings.
    ///
    /// A second reading surface rather than a replacement. The text one is
    /// focusable, arrow-navigable and searchable, and it has no headings, so
    /// this is the answer when a thread is long enough that moving between
    /// messages one key at a time is the slow way round.
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
    let headings = Button::builder(&dlg)
        .with_label("As &Headings")
        .with_id(ID_HEADINGS)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    buttons.add(&open, 0, SizerFlag::All, 4);
    buttons.add(&headings, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    // The root is the whole conversation, and every message hangs under its
    // own parent, so the control's own level announcement matches the reply
    // depth rather than something we invented.
    let Some(root) = tree.add_root(&root_label(nodes.len()), None, None) else {
        tracing::error!("Conversation tree root could not be created");
        return ThreadChoice::Cancelled;
    };

    // The message id rides on the item rather than being looked up by
    // position or by label. Labels repeat within a conversation, and a
    // position lookup breaks the moment the tree is built in another order.
    let mut items: Vec<Option<TreeItemId>> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let under = match node.parent.and_then(|index| items.get(index).cloned()) {
            Some(Some(parent_item)) => parent_item,
            _ => root.clone(),
        };
        let item = tree.append_item(&under, &node_label(node), None, None);
        if let Some(ref item) = item {
            tree.set_custom_data(item, node.message_id);
        }
        items.push(item);
    }
    tree.expand_all();
    tree.select_item(&root);
    tree.set_focus();

    let chosen = std::rc::Rc::new(std::cell::RefCell::new(ThreadChoice::WholeConversation));

    // Selection drives the choice, so pressing Enter, clicking Open, and
    // double-clicking a row all act on the same thing: the row you are on.
    tree.on_selection_changed({
        let chosen = chosen.clone();
        move |event| {
            let Some(item) = event.get_item() else {
                return;
            };
            // No data means the root, which is the whole conversation.
            *chosen.borrow_mut() = match tree
                .get_custom_data(&item)
                .and_then(|data| data.downcast_ref::<i64>().copied())
            {
                Some(id) => ThreadChoice::Message(id),
                None => ThreadChoice::WholeConversation,
            };
        }
    });

    tree.on_item_activated(move |_| dlg.end_modal(ID_OK));
    open.on_click(move |_| dlg.end_modal(ID_OK));
    headings.on_click(move |_| dlg.end_modal(ID_HEADINGS));
    cancel.on_click(move |_| dlg.end_modal(ID_CANCEL));

    let _ = a11y.announce(
        &format!("Conversation, {} messages", nodes.len()),
        Priority::Normal,
    );

    match dlg.show_modal() {
        ID_OK => chosen.borrow().clone(),
        // Always the whole thread, whichever row the tree was on. The point of
        // the page is the shape of the conversation, and one message has none.
        ID_HEADINGS => ThreadChoice::AsHeadings,
        _ => ThreadChoice::Cancelled,
    }
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
