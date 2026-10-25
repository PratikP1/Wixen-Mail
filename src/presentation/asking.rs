//! Which answer Enter gives on a question with two answers.
//!
//! A wxWidgets message box with Yes and No on it makes Yes the answer Enter
//! gives unless it is told otherwise, and that is the wrong way round for a
//! question asked before something that cannot be undone. Somebody who presses
//! Enter partway through hearing the question has deleted the thing before the
//! sentence finished.
//!
//! Anybody can be caught by that and it costs a screen reader user more, for
//! two reasons. Hearing a question takes longer than reading one, so there is
//! more of it left when the finger moves. And Enter is how a person working by
//! keyboard answers everything, so it is already on its way.
//!
//! Not every question wants No, though. The composer asks whether to send a
//! message the spell checker has doubts about, and there Yes on Enter is the
//! point: somebody who meant to send and heard the warning should not have to
//! go looking for a button, and the whole question is in the words, so it can
//! be answered from hearing it.
//!
//! So the answer Enter gives is a decision, and both answers live here so that
//! asking a new question means picking one of them. A house style check holds
//! that: nothing outside this file names the flag that puts two answers on a
//! box.

use crate::application::questions_before_acting::Question;
use wxdragon::dialogs::message_dialog::MessageDialogStyle;

/// The flag that stops Enter answering Yes.
///
/// `wxNO_DEFAULT`, which the binding calls `NoDefault`. This project calls it
/// by what it does here, so a question reads as a decision about Enter rather
/// than about a default button.
pub const ENTER_DOES_NOT_ANSWER_YES: MessageDialogStyle = MessageDialogStyle::NoDefault;

/// A question with two answers, where Enter answers No.
///
/// For anything that cannot be undone. The question still says what Yes does,
/// because the buttons say Yes and No, so "Delete Ada Lovelace?" has to be a
/// question those two words answer. The binding can give a message box other
/// button labels; this module keeps Yes and No, because other labels would
/// change what people hear, which is a decision rather than a tidy-up.
pub fn yes_no_where_enter_answers_no() -> MessageDialogStyle {
    let two_answers = yes_no_where_enter_answers_yes();
    // `|` here could be `^` and nothing would change: `ENTER_DOES_NOT_ANSWER_YES`
    // is bit 7 (128) and `two_answers` never sets it, the same disjointness
    // `yes_no_where_enter_answers_yes` checks below for its own two flags. The
    // assert is what would notice a future wxdragon giving `NoDefault` a
    // bit one of `MessageDialogStyle::YesNo` or `IconQuestion` already uses,
    // which is the only way `|` and `^` could ever start disagreeing here.
    debug_assert_eq!(
        two_answers.bits() & ENTER_DOES_NOT_ANSWER_YES.bits(),
        0,
        "yes_no_where_enter_answers_yes now shares a bit with \
         ENTER_DOES_NOT_ANSWER_YES, so combining them is no longer safe to \
         read as an OR of two independent flags"
    );
    two_answers | ENTER_DOES_NOT_ANSWER_YES
}

/// A question with two answers, where Enter answers Yes.
///
/// For a question standing between somebody and the thing they came to do,
/// where the answer that gets on with it is the one they meant. Choosing this
/// is choosing that pressing Enter without hearing the end of the question does
/// the thing, so it belongs only where the thing can be undone or was asked
/// for.
pub fn yes_no_where_enter_answers_yes() -> MessageDialogStyle {
    // `|` and `^` compute the same value here, always: wxWidgets gives every
    // style flag its own bit, and `YesNo` (10, bits 1 and 3) and
    // `IconQuestion` (1024, bit 10) share none of them, which is the one case
    // where OR and XOR cannot disagree. A mutation test replacing this `|`
    // with `^` found exactly that: nothing failed, because nothing could.
    // The assert stands in for that proof at run time, so a future flag value
    // that starts overlapping fails loudly here instead of quietly changing
    // what this function returns.
    debug_assert_eq!(
        MessageDialogStyle::YesNo.bits() & MessageDialogStyle::IconQuestion.bits(),
        0,
        "YesNo and IconQuestion now share a bit, so combining them is no \
         longer safe to read as an OR of two independent flags"
    );
    MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion
}

/// Which of a question's two answers came back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answered {
    /// They chose the first answer.
    Yes,
    /// They chose the second answer.
    No,
    /// The window was closed without an answer.
    ///
    /// Not the same as either of the other two, and this variant exists to
    /// stop it being folded into one of them. A question worth asking is one
    /// where both answers do something, so answering it on somebody's behalf
    /// because they closed the window does one of those things without being
    /// asked to.
    Neither,
}

/// What a two-answer question came back as.
///
/// Here beside the flag that decides which answer Enter gives, because the two
/// are one subject: how a question with two answers is put and how its answer
/// is read. It used to be written once for each question, and a second copy of
/// a mapping this small is a copy nobody would notice going wrong.
pub fn which_of_the_two(from_the_dialog: wxdragon::Id) -> Answered {
    match from_the_dialog {
        wxdragon::id::ID_YES => Answered::Yes,
        wxdragon::id::ID_NO => Answered::No,
        _ => Answered::Neither,
    }
}

/// What the first button answers with, Windows' `IDYES`.
pub const IDYES: i32 = 6;
/// What the second button answers with, Windows' `IDNO`.
pub const IDNO: i32 = 7;
/// What a box closed without a button answers with, Windows' `IDCANCEL`:
/// Escape, Alt+F4 or the title bar's Close.
pub const IDCANCEL: i32 = 2;

/// A question's two buttons, each with its access letter marked by `&`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwoAnswers {
    /// The button that does the thing asked about.
    pub first: &'static str,
    /// The other one.
    pub second: &'static str,
}

/// What each of a question's two buttons says.
pub fn answers(question: Question) -> TwoAnswers {
    let _ = question;
    TwoAnswers {
        first: "&Yes",
        second: "&No",
    }
}

/// The answer Enter gives on a question: the button the keyboard is on when
/// the box opens.
pub fn enter_gives(question: Question) -> Answered {
    match question {
        Question::RunARule | Question::MoveTheMailABlockCatches | Question::SendDespiteSpelling => {
            Answered::Yes
        }
        Question::DeleteAnItem
        | Question::TakeAwayACopy
        | Question::DeleteAContainer
        | Question::DeleteAConversation
        | Question::DeleteAConversationPermanently
        | Question::DeleteASavedSearch
        | Question::MoveAFolder
        | Question::DeleteAFolder
        | Question::EmptyAFolder
        | Question::EmptyAFolderForGood
        | Question::RunARuleThatDeletes
        | Question::RemoveFoldersTheServerNoLongerLists
        | Question::ImportAKey
        | Question::ApplyAViewToOtherFolders
        | Question::OpenTheReleasesPage
        | Question::InstallAnUpdate
        | Question::MarkAPictureDecorative
        | Question::RemoveAKey
        | Question::DeleteASoundScheme => Answered::No,
    }
}

/// Everything Windows is handed to put one question up, as data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBox {
    /// The window's title.
    pub title: String,
    /// The line that stands out above the rest, when the words have one.
    pub main_instruction: Option<String>,
    /// The words of the question.
    pub content: String,
    /// The two buttons, first and second, each as the id it answers with and
    /// its label.
    pub buttons: [(i32, &'static str); 2],
    /// The id of the button the keyboard is on when the box opens.
    pub default_button: i32,
    /// Whether Escape, Alt+F4 and the title bar's Close close the box.
    pub cancellation_allowed: bool,
    /// Whether the box grows to fit its words rather than cutting them short.
    pub sized_to_content: bool,
}

/// The box that asks `question`, with `title` and `words`.
pub fn the_box(question: Question, title: &str, words: &str) -> TheBox {
    let TwoAnswers { first, second } = answers(question);
    let (main_instruction, content) = the_words_split_the_way_wx_splits_them(words);
    TheBox {
        title: title.to_string(),
        main_instruction,
        content,
        buttons: [(IDYES, first), (IDNO, second)],
        default_button: match enter_gives(question) {
            Answered::Yes => IDYES,
            Answered::No | Answered::Neither => IDNO,
        },
        cancellation_allowed: false,
        sized_to_content: true,
    }
}

/// A first line followed by a blank one stands out above the rest, the way
/// wxWidgets splits a message box's words (`src/msw/msgdlg.cpp:641-655` and
/// `:699-707` at 3.3.4); anything else is the words as they are.
fn the_words_split_the_way_wx_splits_them(words: &str) -> (Option<String>, String) {
    match words.split_once('\n') {
        Some((first, after)) => match after.strip_prefix('\n') {
            Some("") => (None, first.to_string()),
            Some(rest) => (Some(first.to_string()), rest.to_string()),
            None => (None, words.to_string()),
        },
        None => (None, words.to_string()),
    }
}

/// Which answer the button a box came back with is.
pub fn answered_from(pressed: i32) -> Answered {
    match pressed {
        IDYES => Answered::Yes,
        IDNO => Answered::No,
        _ => Answered::Neither,
    }
}

/// A top-level window of the thread asking, as the box finds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopWindow {
    /// Its handle, as a number.
    pub handle: isize,
    /// Whether it is shown.
    pub visible: bool,
    /// Whether it takes input.
    pub enabled: bool,
}

/// The windows to disable while a question is up: every shown, enabled one but
/// the owner, which Windows disables itself. What `wxWindowDisabler` does
/// around a message box (`src/common/utilscmn.cpp:1529-1571`).
pub fn the_windows_to_disable(windows: &[TopWindow], owner: Option<isize>) -> Vec<isize> {
    windows
        .iter()
        .filter(|window| window.visible && window.enabled && Some(window.handle) != owner)
        .map(|window| window.handle)
        .collect()
}

/// Put `the_box` up over `owner` and wait for its answer.
#[cfg(windows)]
pub fn show(the_box: &TheBox, owner: Option<windows::Win32::Foundation::HWND>) -> Answered {
    use windows::Win32::UI::Controls::{
        TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TDF_ALLOW_DIALOG_CANCELLATION, TDF_EXPAND_FOOTER_AREA,
        TDF_POSITION_RELATIVE_TO_WINDOW, TDF_SIZE_TO_CONTENT, TaskDialogIndirect,
    };
    use windows::Win32::UI::WindowsAndMessaging::IsIconic;
    use windows::core::PCWSTR;

    let _disabled = OthersDisabled::around(owner);
    let wide = |text: &str| -> Vec<u16> { text.encode_utf16().chain([0]).collect() };
    let title = wide(&the_box.title);
    let main_instruction = the_box.main_instruction.as_deref().map(wide);
    let content = wide(&the_box.content);
    let labels = the_box.buttons.map(|(_, label)| wide(label));
    let buttons = [0, 1].map(|at| TASKDIALOG_BUTTON {
        nButtonID: the_box.buttons[at].0,
        pszButtonText: PCWSTR(labels[at].as_ptr()),
    });

    let mut flags = TDF_EXPAND_FOOTER_AREA;
    if the_box.sized_to_content {
        flags |= TDF_SIZE_TO_CONTENT;
    }
    if the_box.cancellation_allowed {
        flags |= TDF_ALLOW_DIALOG_CANCELLATION;
    }
    // SAFETY: a window handle, or none; IsIconic only reads it.
    if owner.is_some_and(|owner| !unsafe { IsIconic(owner) }.as_bool()) {
        flags |= TDF_POSITION_RELATIVE_TO_WINDOW;
    }
    // Built by value: the structure is packed, so no field of it is borrowed.
    // Every string it points at is owned above and lives until this returns.
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        hwndParent: owner.unwrap_or_default(),
        dwFlags: flags,
        pszWindowTitle: PCWSTR(title.as_ptr()),
        pszMainInstruction: main_instruction
            .as_ref()
            .map_or(PCWSTR::null(), |line| PCWSTR(line.as_ptr())),
        pszContent: PCWSTR(content.as_ptr()),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: the_box.default_button,
        ..Default::default()
    };
    let mut pressed = 0i32;
    // SAFETY: the configuration and everything it points at outlive the call,
    // and the out pointer is a local.
    match unsafe { TaskDialogIndirect(&config, Some(&mut pressed), None, None) } {
        Ok(()) => answered_from(pressed),
        Err(why) => {
            tracing::warn!(
                "The question \"{}\" could not be shown: {why}",
                the_box.title
            );
            Answered::Neither
        }
    }
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn EnableWindow(window: isize, enable: i32) -> i32;
    fn IsWindowEnabled(window: isize) -> i32;
}

/// The other windows of the asking thread, disabled while a question is up and
/// enabled again when this is dropped, so no way out of `show` can leave one
/// disabled.
#[cfg(windows)]
struct OthersDisabled(Vec<isize>);

#[cfg(windows)]
impl OthersDisabled {
    fn around(owner: Option<windows::Win32::Foundation::HWND>) -> Self {
        let chosen = the_windows_to_disable(
            &top_windows_of_this_thread(),
            owner.map(|owner| owner.0 as isize),
        );
        for &window in &chosen {
            // SAFETY: a window of this thread, found a moment ago.
            unsafe { EnableWindow(window, 0) };
        }
        OthersDisabled(chosen)
    }
}

#[cfg(windows)]
impl Drop for OthersDisabled {
    fn drop(&mut self) {
        for &window in &self.0 {
            // SAFETY: a handle this disabled; a window destroyed since is
            // refused by Windows rather than touched.
            unsafe { EnableWindow(window, 1) };
        }
    }
}

/// Every top-level window of the calling thread, as `EnumThreadWindows` lists
/// them.
#[cfg(windows)]
fn top_windows_of_this_thread() -> Vec<TopWindow> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{EnumThreadWindows, IsWindowVisible};

    unsafe extern "system" fn add(window: HWND, found: LPARAM) -> windows::core::BOOL {
        // SAFETY: `found` is the vector handed in below, alive for the call.
        let found = unsafe { &mut *(found.0 as *mut Vec<TopWindow>) };
        let handle = window.0 as isize;
        found.push(TopWindow {
            handle,
            // SAFETY: a live window handle Windows just handed over.
            visible: unsafe { IsWindowVisible(window) }.as_bool(),
            // SAFETY: as above.
            enabled: unsafe { IsWindowEnabled(handle) } != 0,
        });
        true.into()
    }

    let mut found: Vec<TopWindow> = Vec::new();
    // SAFETY: the callback writes only to `found`, which outlives the call.
    unsafe {
        let _ = EnumThreadWindows(
            GetCurrentThreadId(),
            Some(add),
            LPARAM(&mut found as *mut Vec<TopWindow> as isize),
        );
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What wxWidgets calls it, so the two answers can be told apart by number
    /// rather than by the name this file gives them.
    const WX_NO_DEFAULT: i64 = 0x0000_0080;

    #[test]
    fn test_a_question_that_cannot_be_undone_does_not_let_enter_answer_yes() {
        // The whole point. Without the flag, wxWidgets puts the focus on Yes
        // and Enter deletes the thing.
        let style = yes_no_where_enter_answers_no();

        assert_eq!(
            style.bits() & WX_NO_DEFAULT,
            WX_NO_DEFAULT,
            "Enter still answers Yes on a question asked before a deletion"
        );
    }

    #[test]
    fn test_the_deliberate_one_still_lets_enter_answer_yes() {
        // The composer's send question. Getting this wrong in the other
        // direction is quieter and still wrong: somebody who meant to send
        // would have to find a button.
        let style = yes_no_where_enter_answers_yes();

        assert_eq!(
            style.bits() & WX_NO_DEFAULT,
            0,
            "Enter no longer sends a message somebody chose to send"
        );
    }

    #[test]
    fn test_both_still_ask_a_question_with_two_answers() {
        // The flag is added to the style rather than replacing it. Losing the
        // Yes and No answers would leave a box with one button on it, and the
        // deletion would go ahead on any answer.
        for style in [
            yes_no_where_enter_answers_no(),
            yes_no_where_enter_answers_yes(),
        ] {
            assert!(
                style.contains(MessageDialogStyle::YesNo),
                "a question with no answers to choose between"
            );
            assert!(
                style.contains(MessageDialogStyle::IconQuestion),
                "a question that does not look like one"
            );
        }
    }

    #[test]
    fn test_the_flag_is_the_number_wxwidgets_uses() {
        // Read from the binding rather than written here, so this is the one
        // place the two are compared. A wrong number would be a style flag
        // meaning something else, which is the kind of mistake that looks like
        // nothing at all until a dialog behaves oddly.
        assert_eq!(ENTER_DOES_NOT_ANSWER_YES.bits(), WX_NO_DEFAULT);
    }

    #[test]
    fn test_closing_the_window_is_neither_answer() {
        // The one that matters. A closed window folded into No answers the
        // question on somebody's behalf, and folded into Yes it carries out
        // the thing they were being asked about.
        assert_eq!(which_of_the_two(wxdragon::id::ID_YES), Answered::Yes);
        assert_eq!(which_of_the_two(wxdragon::id::ID_NO), Answered::No);
        assert_eq!(which_of_the_two(wxdragon::id::ID_CANCEL), Answered::Neither);
        assert_eq!(which_of_the_two(wxdragon::id::ID_OK), Answered::Neither);
    }

    /// Every question's two buttons, first and second, written from what each
    /// answer does (13.4-12.3's premise 2), not read from `answers`.
    const THE_BUTTONS: [(Question, &str, &str); 22] = [
        (Question::DeleteAnItem, "&Delete", "&Keep"),
        (Question::TakeAwayACopy, "&Take Away", "&Keep"),
        (Question::DeleteAContainer, "&Delete", "&Keep"),
        (Question::DeleteAConversation, "&Delete", "&Keep"),
        (
            Question::DeleteAConversationPermanently,
            "&Delete Permanently",
            "&Keep",
        ),
        (Question::DeleteASavedSearch, "&Delete", "&Keep"),
        (Question::MoveAFolder, "&Move", "&Don't Move"),
        (Question::DeleteAFolder, "&Delete", "&Keep"),
        (Question::EmptyAFolder, "&Empty", "&Don't Empty"),
        (
            Question::EmptyAFolderForGood,
            "&Empty for Good",
            "&Don't Empty",
        ),
        (Question::RunARule, "&Run", "&Don't Run"),
        (Question::RunARuleThatDeletes, "&Run", "&Don't Run"),
        (
            Question::RemoveFoldersTheServerNoLongerLists,
            "&Remove",
            "&Keep",
        ),
        (Question::ImportAKey, "&Import", "&Don't Import"),
        (Question::ApplyAViewToOtherFolders, "&Apply", "&Don't Apply"),
        (
            Question::OpenTheReleasesPage,
            "&Open the Page",
            "&Don't Open",
        ),
        (Question::InstallAnUpdate, "&Install Now", "&Don't Install"),
        (
            Question::MoveTheMailABlockCatches,
            "&Move Them",
            "&Leave Them",
        ),
        (
            Question::SendDespiteSpelling,
            "&Send Anyway",
            "&Back to the Message",
        ),
        (
            Question::MarkAPictureDecorative,
            "&Mark as Decorative",
            "&Describe It",
        ),
        (Question::RemoveAKey, "&Remove", "&Keep"),
        (Question::DeleteASoundScheme, "&Delete", "&Keep"),
    ];

    #[test]
    fn test_every_question_says_what_each_of_its_answers_does() {
        // "Keep, button" says what Enter will do; "No, button" asked somebody
        // to remember how the question was worded.
        let wrong: Vec<_> = THE_BUTTONS
            .iter()
            .filter(|&&(question, first, second)| answers(question) != TwoAnswers { first, second })
            .map(|(question, first, second)| (question, answers(*question), first, second))
            .collect();
        assert!(
            wrong.is_empty(),
            "these questions' buttons do not say what each does, as given and as wanted: {wrong:#?}"
        );
        assert_eq!(
            THE_BUTTONS.iter().map(|row| row.0).collect::<Vec<_>>(),
            Question::ALL.to_vec(),
            "the table above no longer lists every question in order"
        );
    }

    /// The access letter of a label: the character after its `&`, upper case.
    fn letter_of(label: &str) -> Option<char> {
        let (_, after) = label.split_once('&')?;
        after
            .chars()
            .next()
            .map(|letter| letter.to_ascii_uppercase())
    }

    /// Why one box's two letters will not do, or nothing.
    fn what_is_wrong_with_the_letters(answers: TwoAnswers) -> Option<String> {
        let first = letter_of(answers.first);
        let second = letter_of(answers.second);
        match (first, second) {
            (None, _) | (_, None) => Some("a button with no letter".to_string()),
            (Some(first), Some(second)) if first == second => {
                Some(format!("both buttons take {first}"))
            }
            (Some('G'), _) | (_, Some('G')) => {
                Some("G is the check box's, added by 13.4-12.4".to_string())
            }
            _ => None,
        }
    }

    #[test]
    fn test_no_box_gives_one_letter_to_both_answers_or_takes_the_check_boxs_g() {
        // A letter two buttons share lands on whichever comes first, and G is
        // "Don't ask me a&gain" in every box from 13.4-12.4.
        let clashing: Vec<_> = Question::ALL
            .iter()
            .filter_map(|question| {
                what_is_wrong_with_the_letters(answers(*question)).map(|why| (question, why))
            })
            .collect();
        assert!(clashing.is_empty(), "{clashing:#?}");

        // The reading can see each fault, so the pass above is not a pass over
        // nothing.
        let planted = |first, second| what_is_wrong_with_the_letters(TwoAnswers { first, second });
        assert!(
            planted("&Delete", "&Don't Delete").is_some(),
            "a shared letter"
        );
        assert!(planted("&Go", "&Keep").is_some(), "the check box's G");
        assert!(planted("Delete", "&Keep").is_some(), "no letter");
        assert!(
            planted("&Delete", "&Keep").is_none(),
            "two letters of their own"
        );
    }

    #[test]
    fn test_enter_keeps_things_as_they_are_except_where_it_was_chosen() {
        // Written row by row from 13.4-12.3's premise 2, "Enter after": the
        // second answer on nineteen questions, the first on the three whose
        // call sites chose it on purpose.
        let gives_the_first = [
            Question::RunARule,
            Question::MoveTheMailABlockCatches,
            Question::SendDespiteSpelling,
        ];
        for question in Question::ALL {
            let wanted = if gives_the_first.contains(&question) {
                Answered::Yes
            } else {
                Answered::No
            };
            assert_eq!(enter_gives(question), wanted, "{question:?}");
        }
    }

    #[test]
    fn test_a_box_carries_its_questions_buttons_with_enter_on_keep() {
        let the_box = the_box(
            Question::DeleteASavedSearch,
            "Delete Saved Search",
            "Delete the saved search Weekly? The mail it listed stays where it is.",
        );
        assert_eq!(
            the_box,
            TheBox {
                title: "Delete Saved Search".to_string(),
                main_instruction: None,
                content: "Delete the saved search Weekly? The mail it listed stays where it is."
                    .to_string(),
                buttons: [(6, "&Delete"), (7, "&Keep")],
                default_button: 7,
                cancellation_allowed: true,
                sized_to_content: true,
            }
        );
    }

    #[test]
    fn test_escape_may_close_any_question() {
        // Without this Windows gives Escape and Alt+F4 nothing to do on a box
        // with no Cancel button, so a question could only be answered.
        let closable: Vec<_> = Question::ALL
            .iter()
            .filter(|question| !the_box(**question, "A title", "Words").cancellation_allowed)
            .collect();
        assert!(closable.is_empty(), "Escape cannot close {closable:?}");
    }

    #[test]
    fn test_the_words_split_the_way_wx_splits_them() {
        let split = |words| {
            let the_box = the_box(Question::InstallAnUpdate, "Install Update", words);
            (the_box.main_instruction, the_box.content)
        };
        assert_eq!(
            split("First line\n\nThe rest"),
            (Some("First line".to_string()), "The rest".to_string())
        );
        assert_eq!(split("One line"), (None, "One line".to_string()));
        assert_eq!(split("Two\nlines"), (None, "Two\nlines".to_string()));
        assert_eq!(split("Only\n\n"), (None, "Only".to_string()));
        assert_eq!(
            split("First\n\nSecond\n\nThird"),
            (Some("First".to_string()), "Second\n\nThird".to_string())
        );
    }

    #[test]
    fn test_a_box_closed_without_a_button_is_neither_answer() {
        assert_eq!(answered_from(6), Answered::Yes);
        assert_eq!(answered_from(7), Answered::No);
        assert_eq!(answered_from(2), Answered::Neither, "Escape or Alt+F4");
        assert_eq!(answered_from(0), Answered::Neither, "the box never opened");
    }

    #[test]
    fn test_every_other_shown_window_is_disabled_while_a_question_is_up() {
        let window = |handle, visible, enabled| TopWindow {
            handle,
            visible,
            enabled,
        };
        let found = [
            window(10, true, true),
            window(20, true, true),
            window(30, false, true),
            window(40, true, false),
            window(50, true, true),
        ];
        // The owner Windows disables itself; a hidden window and one already
        // disabled are left alone, so nothing is enabled afterwards that was
        // not enabled before.
        assert_eq!(the_windows_to_disable(&found, Some(20)), vec![10, 50]);
        assert_eq!(the_windows_to_disable(&found, None), vec![10, 20, 50]);
    }
}
