//! Notes module panel — view and manage notes with folder organization.

use crate::presentation::accessibility::names::set_accessible_name;
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
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
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

    let title_input = TextCtrl::builder(&editor_panel).build();

    set_accessible_name(&title_input, "Note title");
    let body_input = TextCtrl::builder(&editor_panel)
        .with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::WordWrap)
        .build();

    set_accessible_name(&body_input, "Note body");

    // Without this the editor accepts typing and throws it away on the next
    // selection, which is worse than being read-only.
    let btn_save = Button::builder(&editor_panel)
        .with_label("&Save Note")
        .build();

    editor_sizer.add(&title_input, 0, SizerFlag::Expand | SizerFlag::All, 4);
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

    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_new_folder, 0, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    NotesSidebarHandles {
        panel,
        tree,
        btn_new_folder,
    }
}
