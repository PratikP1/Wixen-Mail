//! Notes module panel: view and manage notes with folder organization.

use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use wxdragon::prelude::*;

/// Handles to interactive elements in the notes content panel.
pub struct NotesPanelHandles {
    pub panel: Panel,
    pub btn_new: Button,
    pub btn_save: Button,
    pub note_list: ListCtrl,
    pub title_input: TextCtrl,
    pub body_input: TextCtrl,
}

/// Handles to interactive elements in the notes sidebar panel.
pub struct NotesSidebarHandles {
    pub panel: Panel,
    pub tree: TreeCtrl,
    pub btn_new_folder: Button,
    pub btn_delete_folder: Button,
}

/// Build the notes module content panel.
pub fn build_notes_panel(parent: &Panel) -> NotesPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Horizontal).build();

    // Left: note list
    let list_panel = Panel::builder(&panel).build();
    let list_sizer = BoxSizer::builder(Orientation::Vertical).build();

    let btn_new = Button::builder(&list_panel).with_label("&New Note").build();

    let note_list = ListCtrl::builder(&list_panel)
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
    set_accessible_name(&note_list, "Notes");
    note_list.insert_column(0, "Title", ListColumnFormat::Left, 200);
    note_list.insert_column(1, "Last Modified", ListColumnFormat::Left, 150);

    list_sizer.add(&btn_new, 0, SizerFlag::Expand | SizerFlag::All, 2);
    list_sizer.add(&note_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    list_panel.set_sizer(list_sizer, true);

    // Right: note editor
    let editor_panel = Panel::builder(&panel).build();
    let editor_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Visible labels as well as accessible names. Two boxes with nothing
    // written beside them say what they are to a screen reader and to nobody
    // else (WCAG 3.3.2).
    let title_label = StaticText::builder(&editor_panel)
        .with_label("Title")
        .build();
    let title_input = TextCtrl::builder(&editor_panel).build();

    set_accessible_name(&title_input, "Note title");
    let body_label = StaticText::builder(&editor_panel)
        .with_label("Body, in Markdown")
        .build();
    let body_input = TextCtrl::builder(&editor_panel)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();

    // The description, not the name, so it is said once on arrival rather than
    // on every visit to a field somebody is typing in.
    set_accessible_name_and_description(
        &body_input,
        "Note body",
        "Markdown headings and lists are read back when this note is read aloud",
    );

    // Without this the editor accepts typing and throws it away on the next
    // selection, which is worse than being read-only.
    let btn_save = Button::builder(&editor_panel)
        .with_label("&Save Note")
        .build();

    editor_sizer.add(&title_label, 0, SizerFlag::Left | SizerFlag::Top, 4);
    editor_sizer.add(&title_input, 0, SizerFlag::Expand | SizerFlag::All, 4);
    editor_sizer.add(&body_label, 0, SizerFlag::Left | SizerFlag::Top, 4);
    editor_sizer.add(&body_input, 1, SizerFlag::Expand | SizerFlag::All, 4);
    editor_sizer.add(&btn_save, 0, SizerFlag::All, 4);
    editor_panel.set_sizer(editor_sizer, true);

    sizer.add(&list_panel, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&editor_panel, 2, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    NotesPanelHandles {
        panel,
        btn_new,
        btn_save,
        note_list,
        title_input,
        body_input,
    }
}

/// Build the notes sidebar panel.
pub fn build_notes_sidebar(parent: &Panel) -> NotesSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&panel).build();
    set_accessible_name(&tree, "Note folders");
    if let Some(root) = tree.add_root("Note Folders", None, None) {
        tree.append_item(&root, "All Notes", None, None);
        tree.expand(&root);
    }

    let btn_new_folder = Button::builder(&panel).with_label("New &Folder").build();
    let btn_delete_folder = Button::builder(&panel).with_label("Delete F&older").build();

    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_new_folder, 0, SizerFlag::Expand | SizerFlag::All, 2);
    set_accessible_name(&btn_delete_folder, "Delete note folder");
    sizer.add(&btn_delete_folder, 0, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    NotesSidebarHandles {
        panel,
        tree,
        btn_new_folder,
        btn_delete_folder,
    }
}
