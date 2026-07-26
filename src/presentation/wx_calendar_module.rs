//! Calendar module panel — full calendar view with sidebar and event list.
//!
//! This panel lives inside the main window content area (not a dialog).
//! It provides a calendar container tree in the sidebar and an event list
//! in the content area.

use crate::presentation::accessibility::names::set_accessible_name;
use wxdragon::prelude::*;

/// Handles to interactive elements in the calendar content panel.
pub struct CalendarPanelHandles {
    pub panel: Panel,
    pub btn_today: Button,
    pub btn_prev: Button,
    pub btn_next: Button,
    pub date_label: StaticText,
    pub event_list: ListCtrl,
}

/// Handles to interactive elements in the calendar sidebar panel.
pub struct CalendarSidebarHandles {
    pub panel: Panel,
    pub tree: TreeCtrl,
    pub btn_new: Button,
    pub btn_manage: Button,
}

/// Build the calendar module content panel.
///
/// Returns handles to the panel and its interactive widgets.
pub fn build_calendar_panel(parent: &Panel) -> CalendarPanelHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Toolbar row
    let toolbar_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let btn_today = Button::builder(&panel).with_label("&Today").build();
    let btn_prev = Button::builder(&panel).with_label("&< Prev").build();
    let btn_next = Button::builder(&panel).with_label("&Next >").build();
    toolbar_sizer.add(&btn_today, 0, SizerFlag::All, 4);
    toolbar_sizer.add(&btn_prev, 0, SizerFlag::All, 4);
    toolbar_sizer.add(&btn_next, 0, SizerFlag::All, 4);

    // Date heading
    let date_label = StaticText::builder(&panel)
        .with_label("Calendar — Select an account to view events")
        .build();

    // Event list
    let event_list = ListCtrl::builder(&panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&event_list, "Calendar events");
    event_list.insert_column(0, "Time", ListColumnFormat::Left, 120);
    event_list.insert_column(1, "Summary", ListColumnFormat::Left, 300);
    event_list.insert_column(2, "Calendar", ListColumnFormat::Left, 120);
    event_list.insert_column(3, "Location", ListColumnFormat::Left, 150);
    event_list.insert_column(4, "Status", ListColumnFormat::Left, 80);

    sizer.add_sizer(&toolbar_sizer, 0, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&date_label, 0, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add(&event_list, 1, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    CalendarPanelHandles {
        panel,
        btn_today,
        btn_prev,
        btn_next,
        date_label,
        event_list,
    }
}

/// Build the calendar sidebar panel.
///
/// Contains a tree of calendar containers with checkboxes.
pub fn build_calendar_sidebar(parent: &Panel) -> CalendarSidebarHandles {
    let panel = Panel::builder(parent).build();
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let label = StaticText::builder(&panel).with_label("Calendars").build();

    let tree = TreeCtrl::builder(&panel).build();
    set_accessible_name(&tree, "Calendars");
    if let Some(root) = tree.add_root("All Calendars", None, None) {
        tree.append_item(&root, "My Calendar", None, None);
        tree.expand(&root);
    }

    let btn_new = Button::builder(&panel).with_label("&New Calendar").build();
    let btn_manage = Button::builder(&panel)
        .with_label("&Manage Calendars")
        .build();

    sizer.add(&label, 0, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_new, 0, SizerFlag::Expand | SizerFlag::All, 2);
    sizer.add(&btn_manage, 0, SizerFlag::Expand | SizerFlag::All, 2);
    panel.set_sizer(sizer, true);

    CalendarSidebarHandles {
        panel,
        tree,
        btn_new,
        btn_manage,
    }
}
