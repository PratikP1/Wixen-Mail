//! Tasks module panel: view and manage tasks and task lists.

use crate::presentation::accessibility::names::set_accessible_name;
use wxdragon::prelude::*;

/// Handles to interactive elements in the tasks content panel.
pub struct TasksPanelHandles {
    pub panel: Panel,
    pub btn_new: Button,
    pub task_list: ListCtrl,
}

/// Handles to interactive elements in the tasks sidebar panel.
pub struct TasksSidebarHandles {
    pub panel: Panel,
    pub tree: TreeCtrl,
    pub btn_new_list: Button,
    pub btn_delete_list: Button,
}

/// Build the tasks module content panel.
pub fn build_tasks_panel(parent: &Panel) -> TasksPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Toolbar
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let btn_new = Button::builder(&panel).with_label("&New Task").build();
    toolbar_sizer.add(&btn_new, 0, SizerFlag::All, 4);

    // Task list
    let task_list = ListCtrl::builder(&panel)
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
    set_accessible_name(&task_list, "Tasks");
    task_list.insert_column(0, "Done", ListColumnFormat::Centre, 50);
    task_list.insert_column(1, "Title", ListColumnFormat::Left, 300);
    task_list.insert_column(2, "Due Date", ListColumnFormat::Left, 120);
    task_list.insert_column(3, "Priority", ListColumnFormat::Left, 80);

    sizer.add_sizer(&toolbar_sizer, 0, SizerFlag::Expand, 0);
    sizer.add(&task_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    TasksPanelHandles {
        panel,
        btn_new,
        task_list,
    }
}

/// Build the tasks sidebar panel.
pub fn build_tasks_sidebar(parent: &Panel) -> TasksSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&panel).build();
    set_accessible_name(&tree, "Task lists");
    if let Some(root) = tree.add_root("Task Lists", None, None) {
        tree.append_item(&root, "My Tasks", None, None);
        tree.expand(&root);
    }

    let btn_new_list = Button::builder(&panel).with_label("New &List").build();
    let btn_delete_list = Button::builder(&panel).with_label("Delete Li&st").build();

    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_new_list, 0, SizerFlag::Expand | SizerFlag::All, 2);
    set_accessible_name(&btn_delete_list, "Delete task list");
    sizer.add(&btn_delete_list, 0, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    TasksSidebarHandles {
        panel,
        tree,
        btn_new_list,
        btn_delete_list,
    }
}
