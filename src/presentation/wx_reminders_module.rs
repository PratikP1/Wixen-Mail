//! Reminders module panel — view and manage reminders.

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
pub fn build_reminders_panel(parent: &Panel) -> RemindersPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Toolbar
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let btn_new = Button::builder(&panel).with_label("&New Reminder").build();
    toolbar_sizer.add(&btn_new, 0, SizerFlag::All, 4);

    // Reminder list
    let reminder_list = ListCtrl::builder(&panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    reminder_list.insert_column(0, "Done", ListColumnFormat::Centre, 50);
    reminder_list.insert_column(1, "Title", ListColumnFormat::Left, 300);
    reminder_list.insert_column(2, "Due", ListColumnFormat::Left, 180);
    reminder_list.insert_column(3, "Priority", ListColumnFormat::Left, 80);

    sizer.add_sizer(&toolbar_sizer, 0, SizerFlag::Expand, 0);
    sizer.add(&reminder_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    RemindersPanelHandles {
        panel,
        btn_new,
        reminder_list,
    }
}

/// Build the reminders sidebar panel.
pub fn build_reminders_sidebar(parent: &Panel) -> RemindersSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&panel).build();
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
