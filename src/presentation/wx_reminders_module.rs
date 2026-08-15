//! Reminders module panel: view and manage reminders.

use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::theme;
use wxdragon::prelude::*;

/// Handles to interactive elements in the reminders content panel.
pub struct RemindersPanelHandles {
    pub panel: Panel,
    pub btn_new: Button,
    pub reminder_list: ListCtrl,
}

/// Handles to interactive elements in the reminders sidebar panel.
pub struct RemindersSidebarHandles {
    pub panel: Panel,
    pub tree: TreeCtrl,
}

/// Build the reminders module content panel.
///
/// `palette` is `None` under Windows high contrast, or when the system is set
/// up in a way nothing here has an opinion about; either way nothing is
/// painted and Windows keeps deciding.
pub fn build_reminders_panel(
    parent: &Panel,
    palette: Option<theme::Palette>,
) -> RemindersPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Toolbar
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let btn_new = Button::builder(&panel).with_label("&New Reminder").build();
    toolbar_sizer.add(&btn_new, 0, SizerFlag::All, 4);

    // Reminder list
    let reminder_list = ListCtrl::builder(&panel)
        .with_style(
            ListCtrlStyle::Report
                | ListCtrlStyle::SingleSel
                | ListCtrlStyle::HRules
                // Virtual, for the same reason the message list is:
                // a native list filled row by row stops being usable
                // somewhere around ten thousand items, and an address
                // book or a task history reaches that. In virtual mode
                // UI Automation still reports the real count, so a
                // screen reader says "row 12 of 40,000" and means it.
                | ListCtrlStyle::Virtual,
        )
        .build();
    set_accessible_name(&reminder_list, "Reminders");
    reminder_list.insert_column(0, "Done", ListColumnFormat::Centre, 50);
    reminder_list.insert_column(1, "Title", ListColumnFormat::Left, 300);
    reminder_list.insert_column(2, "Due", ListColumnFormat::Left, 180);
    reminder_list.insert_column(3, "Priority", ListColumnFormat::Left, 80);

    sizer.add_sizer(&toolbar_sizer, 0, SizerFlag::Expand, 0);
    sizer.add(&reminder_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    if let Some(palette) = palette {
        theme::paint(&panel, palette.main_surface());
        theme::paint(&reminder_list, palette.main_surface());
    }

    RemindersPanelHandles {
        panel,
        btn_new,
        reminder_list,
    }
}

/// Build the reminders sidebar panel.
///
/// `palette` follows the same rule as [`build_reminders_panel`].
pub fn build_reminders_sidebar(
    parent: &Panel,
    palette: Option<theme::Palette>,
) -> RemindersSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&panel).build();
    set_accessible_name(&tree, "Reminder groups");
    if let Some(palette) = palette {
        theme::paint(&panel, palette.second_surface());
        theme::paint(&tree, palette.second_surface());
    }
    if let Some(root) = tree.add_root("Reminders", None, None) {
        tree.append_item(&root, "Upcoming", None, None);
        tree.append_item(&root, "Today", None, None);
        tree.append_item(&root, "This Week", None, None);
        tree.append_item(&root, "Overdue", None, None);
        tree.append_item(&root, "Completed", None, None);
        tree.expand(&root);
    }

    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    RemindersSidebarHandles { panel, tree }
}
