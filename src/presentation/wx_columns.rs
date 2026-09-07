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
use crate::presentation::theme;
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

/// Which kind of folder Restore Defaults puts back.
///
/// The layout's own, because the dialog is opened on the folder somebody is in
/// and the layout in effect was built for that folder. `docs/KEYBOARD_SHORTCUTS.md`
/// has promised "the default columns for this kind of folder" since the key was
/// documented, and what it gave was the inbox's every time: the one production
/// caller passed `FolderKind::Inbox` as a literal, so `Alt+R` in Sent put back
/// the Unread column, which says the same thing on every row there, and the
/// Received sort. `98546f8` corrected that literal where a folder is switched
/// and left this one.
///
/// A layout that does not say which folder it was arranged in gets the inbox's,
/// and that is one keypress in one situation: a stored string written before
/// layouts recorded a kind, before the first folder has been opened, which is
/// what rebuilds it.
fn what_reset_restores(current: &ColumnLayout) -> FolderKind {
    current.kind.unwrap_or(FolderKind::Inbox)
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
    a11y: &Arc<Accessibility>,
) -> ColumnDialogResult {
    let (dlg, working) =
        build_column_dialog(parent, current, a11y, theme::current_from_stored_config());

    if dlg.show_modal() == ID_OK {
        let layout = working.borrow().clone();
        ColumnDialogResult::Updated(Box::new(layout))
    } else {
        ColumnDialogResult::Cancelled
    }
}

/// Build the column chooser without showing it.
///
/// Everything `show_column_dialog` used to do up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Returns the working layout alongside the dialog. It is what
/// `show_column_dialog` still needs after a real `.show_modal()`: the list
/// keeps it current as somebody ticks and moves rows, so reading it back is
/// all that is left to do once the dialog closes.
///
/// There is no parameter saying which kind of folder this is, and there was
/// one. It was the only way to open this dialog and it was passed a literal, so
/// deleting it is what stops the mistake rather than a comment asking nobody to
/// make it again. [`what_reset_restores`] reads the layout instead.
pub fn build_column_dialog(
    parent: &Frame,
    current: &ColumnLayout,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) -> (Dialog, std::rc::Rc<std::cell::RefCell<ColumnLayout>>) {
    let restores = what_reset_restores(current);
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
            working.borrow_mut().reset(restores);
            fill(&list, &working.borrow());
            list.set_selection(0, true);
            let _ = a11y.announce("Columns reset to the default", Priority::Normal);
        }
    });

    ok.on_click(move |_| dlg.end_modal(ID_OK));
    cancel.on_click(move |_| dlg.end_modal(ID_CANCEL));

    // Painted last. The `CheckListBox` draws its own check marks rather than
    // going through a control the established pattern paints, so it is left
    // to Windows here, the same as every `Choice`, `ComboBox`, `RadioButton`
    // and `CheckBox` elsewhere in this round. `None` means high contrast is
    // on, or the system is set up in a way this application should not paint
    // over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dlg, palette.main_surface());
    }

    (dlg, working)
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
    fn test_restore_defaults_puts_back_the_columns_for_the_folder_somebody_is_in() {
        // `docs/KEYBOARD_SHORTCUTS.md` promises `Alt+R` puts back "the default
        // columns for this kind of folder". In Sent it put back the inbox's,
        // which is the Unread column reading the same word on every row and a
        // sort by when a message arrived rather than when it went.
        for kind in [FolderKind::Inbox, FolderKind::Sent, FolderKind::Drafts] {
            let mut arranged = ColumnLayout::defaults_for(kind);
            arranged.set_visible(MessageColumn::Size, true).unwrap();

            assert_eq!(
                what_reset_restores(&arranged),
                kind,
                "Restore Defaults in a {kind:?} folder restores another folder's columns"
            );
        }
    }

    #[test]
    fn test_a_layout_that_does_not_say_where_it_belongs_restores_the_inbox_columns() {
        // The one case with no better answer: a layout read out of a string
        // written before layouts said which folder they were arranged in, in a
        // window where no folder has been opened yet. Opening one rebuilds it.
        let older = ColumnLayout::from_stored("unread,subject|subject:asc")
            .expect("a layout written before there was a kind");

        assert_eq!(what_reset_restores(&older), FolderKind::Inbox);
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
