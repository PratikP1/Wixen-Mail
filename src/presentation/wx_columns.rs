//! The dialog for choosing which columns appear and in what order.
//!
//! Reordering columns is the classic place an application requires dragging,
//! which WCAG 2.5.7 rules out and which no keyboard user can do at all. Here it
//! is a list, a checkbox, and two buttons, so everything works from the
//! keyboard by default rather than as an alternative path bolted on afterwards.
//!
//! Every change announces what happened. Moving something in a list you cannot
//! see is otherwise a silent action with an invisible result.

use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::message_columns::{ColumnLayout, FolderKind, MessageColumn};
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_MOVE_UP: Id = ID_HIGHEST + 820;
const ID_MOVE_DOWN: Id = ID_HIGHEST + 821;
const ID_RESET: Id = ID_HIGHEST + 822;

/// What the dialog decided.
pub enum ColumnDialogResult {
    /// Apply this layout.
    Updated(Box<ColumnLayout>),
    /// Leave things as they were.
    Cancelled,
}

/// Describe one row of the column list.
///
/// The position is spoken as part of the row because moving an item is
/// otherwise indistinguishable from nothing happening.
fn row_label(column: MessageColumn, shown: bool, position: usize, total: usize) -> String {
    format!(
        "{}, {}, position {} of {}",
        column.heading(),
        if shown { "shown" } else { "hidden" },
        position + 1,
        total
    )
}

/// Every column in display order, visible ones first, then the rest.
fn ordered(layout: &ColumnLayout) -> Vec<MessageColumn> {
    let mut columns = layout.visible();
    for column in MessageColumn::ALL {
        if !columns.contains(&column) {
            columns.push(column);
        }
    }
    columns
}

fn fill(list: &CheckListBox, layout: &ColumnLayout) {
    list.clear();
    let columns = ordered(layout);
    let total = columns.len();
    for (position, column) in columns.iter().enumerate() {
        let shown = layout.is_visible(*column);
        list.append(&row_label(*column, shown, position, total));
        list.check(position as u32, shown);
    }
}

/// Show the column chooser.
pub fn show_column_dialog(
    parent: &Frame,
    current: &ColumnLayout,
    kind: FolderKind,
    a11y: &Arc<Accessibility>,
) -> ColumnDialogResult {
    let dlg = Dialog::builder(parent, "Columns")
        .with_size(460, 460)
        .build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let hint = StaticText::builder(&dlg)
        .with_label(
            "Space shows or hides a column. Alt+Up and Alt+Down move it. \
             The last remaining column cannot be hidden.",
        )
        .build();
    sizer.add(&hint, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let list = CheckListBox::builder(&dlg).build();
    set_accessible_name(&list, "Columns");
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let up = Button::builder(&dlg)
        .with_label("Move &Up")
        .with_id(ID_MOVE_UP)
        .build();
    let down = Button::builder(&dlg)
        .with_label("Move &Down")
        .with_id(ID_MOVE_DOWN)
        .build();
    let reset = Button::builder(&dlg)
        .with_label("&Reset")
        .with_id(ID_RESET)
        .build();
    let ok = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    for button in [&up, &down, &reset, &ok, &cancel] {
        buttons.add(button, 0, SizerFlag::All, 4);
    }
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dlg.set_sizer(sizer, true);

    let working = std::rc::Rc::new(std::cell::RefCell::new(current.clone()));
    fill(&list, &working.borrow());
    list.set_selection(0, true);

    // Space toggles visibility. The refusal to hide the last column is
    // announced rather than the checkbox silently springing back.
    let on_toggle = {
        let working = working.clone();
        let a11y = a11y.clone();
        move |index: u32| {
            let columns = ordered(&working.borrow());
            let Some(column) = columns.get(index as usize).copied() else {
                return;
            };
            let shown = working.borrow().is_visible(column);
            let outcome = working.borrow_mut().set_visible(column, !shown);
            match outcome {
                Ok(said) => {
                    let _ = a11y.announce(&said, Priority::Normal);
                }
                Err(refused) => {
                    let _ = a11y.announce(&refused.0, Priority::High);
                }
            }
        }
    };

    list.on_toggled({
        let working = working.clone();
        let on_toggle = on_toggle.clone();
        move |event| {
            let index = event.get_selection().unwrap_or(0);
            on_toggle(index);
            fill(&list, &working.borrow());
            list.set_selection(index, true);
        }
    });

    let move_by = {
        let working = working.clone();
        let a11y = a11y.clone();
        move |offset: i32| {
            let index = list.get_selection().unwrap_or(0);
            let columns = ordered(&working.borrow());
            let Some(column) = columns.get(index as usize).copied() else {
                return;
            };
            let outcome = working.borrow_mut().move_by(column, offset);
            match outcome {
                Ok(said) => {
                    let _ = a11y.announce(&said, Priority::Normal);
                    fill(&list, &working.borrow());
                    let moved = ordered(&working.borrow())
                        .iter()
                        .position(|c| *c == column)
                        .unwrap_or(0);
                    list.set_selection(moved as u32, true);
                }
                Err(refused) => {
                    let _ = a11y.announce(&refused.0, Priority::High);
                }
            }
        }
    };

    up.on_click({
        let move_by = move_by.clone();
        move |_| move_by(-1)
    });
    down.on_click({
        let move_by = move_by.clone();
        move |_| move_by(1)
    });

    // Alt+Up and Alt+Down do the same from the list itself, so moving a column
    // does not mean leaving it to reach a button and coming back.
    list.on_key_down({
        let move_by = move_by.clone();
        move |event| {
            if let WindowEventData::Keyboard(ref key) = event
                && key.alt_down()
            {
                match key.get_key_code() {
                    Some(315) => move_by(-1),
                    Some(317) => move_by(1),
                    _ => {}
                }
            }
        }
    });

    reset.on_click({
        let working = working.clone();
        let a11y = a11y.clone();
        move |_| {
            working.borrow_mut().reset(kind);
            fill(&list, &working.borrow());
            list.set_selection(0, true);
            let _ = a11y.announce("Columns reset to the default", Priority::Normal);
        }
    });

    ok.on_click(move |_| dlg.end_modal(ID_OK));
    cancel.on_click(move |_| dlg.end_modal(ID_CANCEL));

    if dlg.show_modal() == ID_OK {
        let layout = working.borrow().clone();
        ColumnDialogResult::Updated(Box::new(layout))
    } else {
        ColumnDialogResult::Cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_row_says_where_it_is() {
        // Moving something in a list you cannot see is otherwise silent.
        assert_eq!(
            row_label(MessageColumn::Subject, true, 2, 6),
            "Subject, shown, position 3 of 6"
        );
        assert_eq!(
            row_label(MessageColumn::Size, false, 5, 6),
            "Size, hidden, position 6 of 6"
        );
    }

    #[test]
    fn test_hidden_columns_are_listed_after_the_visible_ones() {
        let layout = ColumnLayout::defaults_for(FolderKind::Inbox);
        let columns = ordered(&layout);
        let visible = layout.visible();
        assert_eq!(&columns[..visible.len()], &visible[..]);
        assert_eq!(columns.len(), MessageColumn::ALL.len());
    }

    #[test]
    fn test_every_column_appears_exactly_once() {
        let layout = ColumnLayout::defaults_for(FolderKind::Sent);
        let columns = ordered(&layout);
        for column in MessageColumn::ALL {
            assert_eq!(
                columns.iter().filter(|c| **c == column).count(),
                1,
                "{:?} should be listed once",
                column
            );
        }
    }
}
