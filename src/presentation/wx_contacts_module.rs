//! Contacts module panel: full contacts view with sidebar and detail pane.
//!
//! This panel lives inside the main window content area (not a dialog).

use crate::presentation::accessibility::names::set_accessible_name;
use wxdragon::prelude::*;

/// Handles to interactive elements in the contacts content panel.
pub struct ContactsPanelHandles {
    pub panel: Panel,
    pub search_input: TextCtrl,
    pub contact_list: ListCtrl,
    pub detail_label: StaticText,
}

/// Handles to interactive elements in the contacts sidebar panel.
pub struct ContactsSidebarHandles {
    pub panel: Panel,
    pub tree: TreeCtrl,
    pub btn_new_group: Button,
    pub btn_delete_group: Button,
    pub btn_import: Button,
    pub btn_export: Button,
}

/// Build the contacts module content panel.
pub fn build_contacts_panel(parent: &Panel) -> ContactsPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Search row
    let search_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let search_label = StaticText::builder(&panel).with_label("&Search:").build();
    let search_input = TextCtrl::builder(&panel).build();
    set_accessible_name(&search_input, "Search contacts");
    search_sizer.add(
        &search_label,
        0,
        SizerFlag::All | SizerFlag::AlignCenterVertical,
        4,
    );
    search_sizer.add(&search_input, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Contact list
    let contact_list = ListCtrl::builder(&panel)
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
    set_accessible_name(&contact_list, "Contacts");
    contact_list.insert_column(0, "Name", ListColumnFormat::Left, 200);
    contact_list.insert_column(1, "Email", ListColumnFormat::Left, 250);
    contact_list.insert_column(2, "Phone", ListColumnFormat::Left, 150);
    contact_list.insert_column(3, "Company", ListColumnFormat::Left, 150);

    // Detail panel placeholder
    let detail = Panel::builder(&panel).build();
    let detail_sizer = BoxSizer::builder(Orientation::Vertical).build();
    let detail_label = StaticText::builder(&detail)
        .with_label("Select a contact to view details")
        .build();
    detail_sizer.add(&detail_label, 0, SizerFlag::Expand | SizerFlag::All, 8);
    detail.set_sizer(detail_sizer, true);

    sizer.add_sizer(&search_sizer, 0, SizerFlag::Expand, 0);
    sizer.add(&contact_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&detail, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    ContactsPanelHandles {
        panel,
        search_input,
        contact_list,
        detail_label,
    }
}

/// Build the contacts sidebar panel.
pub fn build_contacts_sidebar(parent: &Panel) -> ContactsSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&panel).build();
    set_accessible_name(&tree, "Contact groups");
    if let Some(root) = tree.add_root("Contacts", None, None) {
        tree.append_item(&root, "All Contacts", None, None);
        tree.append_item(&root, "Favorites", None, None);
        // The same branch the groups are filled into, so the tree has the
        // same shape before anything has loaded as it has afterwards. Without
        // it, somebody with no account at all finds nowhere to press the menu
        // key and no sign that groups exist.
        tree.append_item(
            &root,
            &crate::application::contact_groups::the_groups_branch(0),
            None,
            None,
        );
        tree.expand(&root);
    }

    let btn_new_group = Button::builder(&panel).with_label("New &Group").build();
    let btn_delete_group = Button::builder(&panel).with_label("Delete G&roup").build();
    let btn_import = Button::builder(&panel).with_label("&Import vCard").build();
    let btn_export = Button::builder(&panel).with_label("&Export vCard").build();

    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_new_group, 0, SizerFlag::Expand | SizerFlag::All, 2);
    set_accessible_name(&btn_delete_group, "Delete contact group");
    sizer.add(&btn_delete_group, 0, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_import, 0, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_export, 0, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    ContactsSidebarHandles {
        panel,
        tree,
        btn_new_group,
        btn_delete_group,
        btn_import,
        btn_export,
    }
}
