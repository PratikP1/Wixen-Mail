//! Check boxes on a native tree, set and read through Windows directly.
//!
//! A `SysTreeView32` with the `TVS_CHECKBOXES` style draws a check box beside
//! every item and keeps its state as the item's state image: 1 for unchecked,
//! 2 for checked. NVDA reads that state from the control itself, with
//! `TVM_GETITEMSTATE` and `TVIS_STATEIMAGEMASK` (its `sysTreeView32.py`, read
//! 2026-09-18), and Windows' own accessible object for the tree answers a
//! check button with `STATE_SYSTEM_CHECKED` on a ticked item. So a tick set
//! this way reaches a screen reader with no object of this program's in the
//! path, which is what the folder chooser needs after the object it used to
//! write turned out to say read-only (#70, and the paragraph in
//! `accessibility/names.rs` where that object stood).
//!
//! # Why this is not the toolkit's own call
//!
//! wxdragon 0.9.17's `TreeCtrl` offers no check state and no style for one.
//! Its sys crate exports `wxd_TreeCtrl_SetItemState` and
//! `wxd_TreeCtrl_GetItemState`, but each takes a `wxd_TreeItemId_t` the
//! wrapper keeps `pub(crate)`, and `u64::from(&TreeItemId)` answers the Rust
//! struct's address rather than the `HTREEITEM`. There is therefore no way to
//! get from an item wx created to its handle through the wrapper. The handles
//! come instead from walking the control with `TVM_GETNEXTITEM`, each item
//! then its children then its next sibling, which is the order the items were
//! appended when the caller appends them in that order; the walk is the path
//! and not a fallback, and `wx_folder_choice::nesting` answers that order.
//!
//! # What was measured before this was written
//!
//! `tests/a_kept_folder_reads_as_a_checked_check_box.rs`, reading B, on
//! 2026-09-18: a wx `TreeCtrl` given `TVS_CHECKBOXES` through
//! `SetWindowLongPtrW` after creation and before its first item answers state
//! image 2 on `TVM_GETITEMSTATE` for the item ticked through `TVM_SETITEMW`
//! and 1 on the others; over `AccessibleObjectFromWindow` the ticked item
//! carries `STATE_SYSTEM_CHECKED` and the others do not, none is read-only,
//! every item answers role check button, and `accValue` is the level. A
//! `WM_KEYDOWN` of `VK_SPACE` sent to the tree moved the selected item from 1
//! to 2 under wx, so the control toggles on Space by itself and nothing here
//! handles the key.
//!
//! The style has to be added after the control exists and before the first
//! item, which is the order Windows documents: the control builds its state
//! image list on the first insertion and a tick set on an item inserted
//! earlier may not show.
//!
//! **Windows only**, as `wxAccessible` is. The declarations are made by hand
//! from the Windows headers rather than through a crate feature, because the
//! `windows` crate's tree view feature would be compiled into the shipping
//! binary for three messages. Elsewhere every call here accepts and does
//! nothing, and [`CHECK_STATES_REACH_A_SCREEN_READER`] says so, so a caller
//! can decline to show a dialog whose ticks would reach nobody.

use wxdragon::prelude::*;

/// Whether a tick set through this module reaches the accessibility tree on
/// this build. A statement about the platform: true on Windows, where the
/// native tree carries its own check boxes, and false everywhere else, where
/// no bridge exists yet.
pub const CHECK_STATES_REACH_A_SCREEN_READER: bool = cfg!(target_os = "windows");

/// A native item handle: `HTREEITEM` on Windows, never handed out elsewhere.
pub type Item = isize;

/// The control's own window handle, as an integer the messages take.
fn handle_of(tree: &TreeCtrl) -> isize {
    tree.get_handle() as isize
}

/// Give the tree a check box beside every item. Call this after the tree is
/// built and before its first item.
pub fn add_check_boxes(tree: &TreeCtrl) {
    platform::add_check_boxes(handle_of(tree));
}

/// Every item in the tree, each followed by its children and then its next
/// sibling: the order the items were appended when each parent's children
/// were appended in order, and the order `wx_folder_choice::nesting` gives.
pub fn items_in_walk_order(tree: &TreeCtrl) -> Vec<Item> {
    platform::items_in_walk_order(handle_of(tree))
}

/// Tick or untick one item through the control's own state.
pub fn set_checked(tree: &TreeCtrl, item: Item, on: bool) {
    platform::set_state_image(handle_of(tree), item, if on { 2 } else { 1 });
}

/// Whether the control holds the item ticked, read from the control and not
/// from anything remembered here, so what is read is what is shown.
pub fn is_checked(tree: &TreeCtrl, item: Item) -> bool {
    platform::state_image(handle_of(tree), item) == 2
}

#[cfg(target_os = "windows")]
mod platform {
    use super::Item;

    /// winuser.h: the style word of a window.
    const GWL_STYLE: i32 = -16;
    /// commctrl.h: the tree view draws a check box beside each item.
    const TVS_CHECKBOXES: isize = 0x0100;
    /// commctrl.h: `TV_FIRST` is 0x1100; each message is an offset from it.
    const TVM_GETNEXTITEM: u32 = 0x1100 + 10;
    const TVM_GETITEMSTATE: u32 = 0x1100 + 39;
    const TVM_SETITEMW: u32 = 0x1100 + 63;
    /// commctrl.h: which item `TVM_GETNEXTITEM` answers with.
    const TVGN_ROOT: usize = 0;
    const TVGN_NEXT: usize = 1;
    const TVGN_CHILD: usize = 4;
    /// commctrl.h: which `TVITEMW` fields a message reads.
    const TVIF_STATE: u32 = 0x0008;
    const TVIF_HANDLE: u32 = 0x0010;
    /// commctrl.h: the four state bits that hold the state image index.
    const TVIS_STATEIMAGEMASK: u32 = 0xF000;

    /// `TVITEMW` as commctrl.h lays it out on 64-bit Windows: 56 bytes, the
    /// handle at offset 8 and `lParam` at 48. Only `mask`, `hItem`, `state`
    /// and `stateMask` are read for the one message this module sends with it.
    #[repr(C)]
    struct TreeItemW {
        mask: u32,
        h_item: isize,
        state: u32,
        state_mask: u32,
        psz_text: *mut u16,
        cch_text_max: i32,
        i_image: i32,
        i_selected_image: i32,
        c_children: i32,
        l_param: isize,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    }

    fn send(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize {
        if hwnd == 0 {
            return 0;
        }
        // SAFETY: a window handle the caller still holds; the messages sent
        // here read or set an item's state and walk the items, and each
        // answers 0 for a handle the control does not know.
        unsafe { SendMessageW(hwnd, message, wparam, lparam) }
    }

    pub(super) fn add_check_boxes(hwnd: isize) {
        if hwnd == 0 {
            return;
        }
        // SAFETY: a live window handle; the style word is read, one bit added
        // and written back, before any item exists.
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            SetWindowLongPtrW(hwnd, GWL_STYLE, style | TVS_CHECKBOXES);
        }
    }

    pub(super) fn items_in_walk_order(hwnd: isize) -> Vec<Item> {
        fn walk(hwnd: isize, item: Item, into: &mut Vec<Item>) {
            into.push(item);
            let mut child = send(hwnd, TVM_GETNEXTITEM, TVGN_CHILD, item);
            while child != 0 {
                walk(hwnd, child, into);
                child = send(hwnd, TVM_GETNEXTITEM, TVGN_NEXT, child);
            }
        }
        let mut items = Vec::new();
        let mut item = send(hwnd, TVM_GETNEXTITEM, TVGN_ROOT, 0);
        while item != 0 {
            walk(hwnd, item, &mut items);
            item = send(hwnd, TVM_GETNEXTITEM, TVGN_NEXT, item);
        }
        items
    }

    pub(super) fn set_state_image(hwnd: isize, item: Item, index: u32) {
        let mut tvitem = TreeItemW {
            mask: TVIF_STATE | TVIF_HANDLE,
            h_item: item,
            state: index << 12,
            state_mask: TVIS_STATEIMAGEMASK,
            psz_text: std::ptr::null_mut(),
            cch_text_max: 0,
            i_image: 0,
            i_selected_image: 0,
            c_children: 0,
            l_param: 0,
        };
        send(
            hwnd,
            TVM_SETITEMW,
            0,
            &mut tvitem as *mut TreeItemW as isize,
        );
    }

    pub(super) fn state_image(hwnd: isize, item: Item) -> u32 {
        let state = send(
            hwnd,
            TVM_GETITEMSTATE,
            item as usize,
            TVIS_STATEIMAGEMASK as isize,
        );
        (state as u32 & TVIS_STATEIMAGEMASK) >> 12
    }

    #[cfg(test)]
    mod tests {
        use super::TreeItemW;

        #[test]
        fn test_the_item_struct_is_laid_out_as_the_header_lays_it_out() {
            // A wrong layout is undefined behaviour inside the control rather
            // than an error, so the size is held here, where it costs nothing.
            assert_eq!(std::mem::size_of::<TreeItemW>(), 56);
            assert_eq!(std::mem::offset_of!(TreeItemW, h_item), 8);
            assert_eq!(std::mem::offset_of!(TreeItemW, state), 16);
            assert_eq!(std::mem::offset_of!(TreeItemW, state_mask), 20);
            assert_eq!(std::mem::offset_of!(TreeItemW, l_param), 48);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::Item;

    pub(super) fn add_check_boxes(_hwnd: isize) {}

    pub(super) fn items_in_walk_order(_hwnd: isize) -> Vec<Item> {
        Vec::new()
    }

    pub(super) fn set_state_image(_hwnd: isize, _item: Item, _index: u32) {}

    pub(super) fn state_image(_hwnd: isize, _item: Item) -> u32 {
        0
    }
}
