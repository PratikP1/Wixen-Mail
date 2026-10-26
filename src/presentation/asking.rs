//! How a question with two answers is put, and how its answer is read.
//!
//! Every such question is a `Question`, and this module decides three things
//! about each, once: what its two buttons say, which of them Enter gives, and
//! that a box closed without a button is neither answer. `ask` is the one way
//! to put one up.
//!
//! **What the buttons say.** Each names what it does, Delete and Keep, Run and
//! Don't Run, rather than Yes and No (Pratik's answer to ledger 967 on
//! 2026-10-25). A screen reader user hears the question and then the button
//! the keyboard is on, and "Keep, button" says what Enter will do where
//! "No, button" asked them to remember how the question was worded. Each
//! button's access letter is its own within the box, and none is G, which
//! 13.4-12.4's "Don't ask me again" takes in every box.
//!
//! **Which answer Enter gives.** On a question asked before something that
//! cannot be undone, the one that keeps things as they are. Somebody who
//! presses Enter partway through hearing the question has otherwise deleted
//! the thing before the sentence finished. Anybody can be caught by that and
//! it costs a screen reader user more: hearing a question takes longer than
//! reading one, so there is more of it left when the finger moves, and Enter
//! is how a person working by keyboard answers everything.
//!
//! Three questions give Enter the first answer on purpose, because there it is
//! the thing somebody came to do and the whole question is in its words.
//! Sending a message the spell checker has doubts about: somebody who meant to
//! send and heard the warning should not have to go looking for a button.
//! Running a rule that does not delete, which `running_a_rule_now` decides.
//! And moving the mail already here that a new block catches, which the user
//! guide describes.
//!
//! **The box is Windows' own task dialog**, the same kind of window the
//! wxWidgets message box was, because that box already called
//! `TaskDialogIndirect` underneath under the common controls this program's
//! manifest asks for (`src/msw/msgdlg.cpp:525-559` at wxWidgets 3.3.4). Calling
//! it here is what lets 13.4-12.4 add a check box, which wxDragon's message box
//! cannot carry. Of what wxWidgets does around the call, this reproduces: every
//! other shown, enabled window of the thread disabled while the box is up and
//! enabled again after (`wxWindowDisabler`, `src/common/utilscmn.cpp:1529-1571`);
//! the owner (`src/common/dlgcmn.cpp:179-204`, in part, as `ask` says); its
//! flags, sized to the words and placed on an owner that is not minimised
//! (`msgdlg.cpp:675-690`); a first line followed by a blank one made the line
//! that stands out (`msgdlg.cpp:641-655`, `:699-707`); the two buttons as
//! `IDYES` and `IDNO` with the default named (`:732-745`, `:786-808`); and no
//! icon, as a question had none (`:710-727`). It adds one thing: Escape,
//! Alt+F4 and the title bar's Close close the box, which on a box with no
//! Cancel button they could not do (`:759-762`), and that is neither answer.
//!
//! What it does not reproduce. The callback (`:591-624`) only turns on
//! wxWidgets' dark mode, which this program never does. The modal hooks
//! (`include/wx/modalhook.h:113-117`) are not exposed by wxDragon; the two
//! wxWidgets registers are a veto of closing a frame while a modal box is up,
//! whose work the disabled windows do, and a flush of a log window this
//! program does not have. The event loop is the task dialog's own, as it was:
//! timers still reach their windows, and wxDragon's queued calls, drained on
//! idle, wait until the box closes.
//!
//! A house style check holds that nothing outside this file builds a box with
//! two answers.
//!
//! **"Don't ask me again".** Every box carries the task dialog's own check
//! box with those words, Alt+G, unticked when it opens (Pratik's answer of
//! 2026-10-25). It counts only with the answer the question remembers, the
//! one that goes ahead, or Describe It on the decorative-picture question:
//! ticked with that answer, the question is not asked again and the program
//! does what that answer does; ticked with the other answer, or with the box
//! closed by Escape, nothing is remembered. A stopped question is a setting,
//! offered on Settings, Questions, where ticking its box asks it again.

use crate::application::questions_before_acting::{Question, QuestionsNotAsked};
use crate::data::config::AppConfig;

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

/// What a wxWidgets message box with two answers came back as, which the
/// fallback off Windows reads.
///
/// It used to be written once for each question, and a second copy of a
/// mapping this small is a copy nobody would notice going wrong.
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
    let (first, second) = match question {
        Question::DeleteAnItem
        | Question::DeleteAContainer
        | Question::DeleteAConversation
        | Question::DeleteASavedSearch
        | Question::DeleteAFolder
        | Question::DeleteASoundScheme => ("&Delete", "&Keep"),
        Question::TakeAwayACopy => ("&Take Away", "&Keep"),
        Question::DeleteAConversationPermanently => ("&Delete Permanently", "&Keep"),
        Question::MoveAFolder => ("&Move", "&Don't Move"),
        Question::EmptyAFolder => ("&Empty", "&Don't Empty"),
        Question::EmptyAFolderForGood => ("&Empty for Good", "&Don't Empty"),
        Question::RunARule | Question::RunARuleThatDeletes => ("&Run", "&Don't Run"),
        Question::RemoveFoldersTheServerNoLongerLists | Question::RemoveAKey => {
            ("&Remove", "&Keep")
        }
        Question::ImportAKey => ("&Import", "&Don't Import"),
        Question::ApplyAViewToOtherFolders => ("&Apply", "&Don't Apply"),
        Question::OpenTheReleasesPage => ("&Open the Page", "&Don't Open"),
        Question::InstallAnUpdate => ("&Install Now", "&Don't Install"),
        Question::MoveTheMailABlockCatches => ("&Move Them", "&Leave Them"),
        Question::SendDespiteSpelling => ("&Send Anyway", "&Back to the Message"),
        Question::MarkAPictureDecorative => ("&Mark as Decorative", "&Describe It"),
    };
    TwoAnswers { first, second }
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

/// The answer a ticked "Don't ask me again" remembers, and so the answer a
/// stopped question gives without a window.
///
/// The first answer, which goes ahead, everywhere but the decorative-picture
/// question. There it is Describe It: remembering Mark as Decorative would
/// send every later small picture with no description, and the person who
/// pays is the one reading the message with a screen reader, who is told
/// nothing. A question added later remembers its first answer too, which is
/// the one its call site goes ahead on.
pub fn remembers(question: Question) -> Answered {
    match question {
        Question::MarkAPictureDecorative => Answered::No,
        _ => Answered::Yes,
    }
}

/// The question to stop asking after one was answered: `question` when "Don't
/// ask me again" was ticked and the answer is the one it remembers, and
/// nothing otherwise. A box closed without an answer is never remembered.
pub fn what_to_remember(
    _question: Question,
    _answered: Answered,
    _ticked: bool,
) -> Option<Question> {
    None
}

/// The answer a stopped question gives without a window, or `None` when it is
/// still asked.
pub fn answered_without_asking(
    _question: Question,
    _not_asked: &QuestionsNotAsked,
) -> Option<Answered> {
    None
}

/// Write the stop for `question` into `settings`: the spelling question turns
/// off "Check spelling before sending a message", which is where it comes
/// back; every other question joins the set Settings, Questions offers.
pub fn stop_asking(_question: Question, _settings: &mut AppConfig) {}

/// The check box on Settings, Questions that asks `question` again, with its
/// access letter, or `None` for the spelling question, which comes back
/// through its own setting on General.
pub fn in_settings(_question: Question) -> Option<&'static str> {
    None
}

/// The words on every box's check box, with G as its letter, the one
/// 13.4-12.3 kept free in every box.
pub const DONT_ASK_ME_AGAIN: &str = "Don't ask me a&gain";

/// What a box came back with: the answer, and whether "Don't ask me again"
/// was ticked when it closed, whichever way it closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shown {
    pub answered: Answered,
    pub ticked: bool,
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
    /// The words of the check box under the buttons, unticked when the box
    /// opens, or `None` for a box without one.
    pub verification: Option<&'static str>,
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
        cancellation_allowed: true,
        sized_to_content: true,
        verification: None,
    }
}

/// Ask `question` over the window `over`, with `title` and `words`, and hand
/// back the answer.
#[cfg(windows)]
pub fn ask(
    over: &dyn wxdragon::window::WxWidget,
    question: Question,
    title: &str,
    words: &str,
) -> Answered {
    show(&the_box(question, title, words), owner_of(over)).answered
}

/// The window a question asked over `over` belongs to: the top-level window
/// `over` sits in when that is shown, the first of the three owners wxWidgets
/// tries (`src/common/dlgcmn.cpp:179-204`); its other two, the active window
/// and the main window, are that same frame or nothing in this program, since
/// every question names the frame or a dialog on it. A hidden one gives no
/// owner.
#[cfg(windows)]
fn owner_of(over: &dyn wxdragon::window::WxWidget) -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GA_ROOT, GetAncestor, IsWindowVisible};

    let asking = HWND(over.get_handle());
    // SAFETY: a window this program built, or null, which both calls refuse.
    let top = unsafe { GetAncestor(asking, GA_ROOT) };
    // SAFETY: as above.
    (!top.is_invalid() && unsafe { IsWindowVisible(top) }.as_bool()).then_some(top)
}

/// Ask `question` over the window `over`, off Windows: wxWidgets' own message
/// box with the two labels and the answer Enter gives, and no "Don't ask me
/// again" box, which it cannot carry. Nothing this project runs by default
/// compiles this; `.github/workflows/other-platforms.yml` does, when it is
/// dispatched.
#[cfg(not(windows))]
pub fn ask(
    over: &dyn wxdragon::window::WxWidget,
    question: Question,
    title: &str,
    words: &str,
) -> Answered {
    use wxdragon::dialogs::message_dialog::{MessageDialog, MessageDialogStyle};

    let two_answers = MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion;
    let style = match enter_gives(question) {
        Answered::Yes => two_answers,
        Answered::No | Answered::Neither => two_answers | MessageDialogStyle::NoDefault,
    };
    let TwoAnswers { first, second } = answers(question);
    let dialog = MessageDialog::builder(over, words, title)
        .with_style(style)
        .build();
    dialog.set_yes_no_labels(first, second);
    which_of_the_two(dialog.show_modal())
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

/// Put `the_box` up over `owner` and wait for its answer, and whether its
/// check box was ticked when it closed.
#[cfg(windows)]
pub fn show(the_box: &TheBox, owner: Option<windows::Win32::Foundation::HWND>) -> Shown {
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
    let answered = match unsafe { TaskDialogIndirect(&config, Some(&mut pressed), None, None) } {
        Ok(()) => answered_from(pressed),
        Err(why) => {
            tracing::warn!(
                "The question \"{}\" could not be shown: {why}",
                the_box.title
            );
            Answered::Neither
        }
    };
    Shown {
        answered,
        ticked: false,
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
                verification: Some("Don't ask me a&gain"),
            }
        );
    }

    #[test]
    fn test_every_question_carries_dont_ask_me_again() {
        // The spelling question included: its box turns off the spelling
        // check, which is its stop.
        let without: Vec<_> = Question::ALL
            .iter()
            .filter(|question| {
                the_box(**question, "A title", "Words").verification != Some("Don't ask me a&gain")
            })
            .collect();
        assert!(without.is_empty(), "no Don't ask me again on {without:?}");
    }

    #[test]
    fn test_the_box_counts_only_with_the_answer_the_question_remembers() {
        let saved_search = Question::DeleteASavedSearch;
        assert_eq!(
            what_to_remember(saved_search, Answered::Yes, true),
            Some(saved_search),
            "ticked with Delete"
        );
        assert_eq!(
            what_to_remember(saved_search, Answered::No, true),
            None,
            "ticked with Keep"
        );
        assert_eq!(
            what_to_remember(saved_search, Answered::Neither, true),
            None,
            "ticked and closed with Escape"
        );
        assert_eq!(
            what_to_remember(saved_search, Answered::Yes, false),
            None,
            "Delete, not ticked"
        );

        // The decorative question remembers Describe It, its second answer.
        let picture = Question::MarkAPictureDecorative;
        assert_eq!(
            what_to_remember(picture, Answered::No, true),
            Some(picture),
            "ticked with Describe It"
        );
        assert_eq!(
            what_to_remember(picture, Answered::Yes, true),
            None,
            "ticked with Mark as Decorative"
        );
    }

    #[test]
    fn test_a_stop_is_written_where_its_question_comes_back() {
        let mut settings = AppConfig::default();
        stop_asking(Question::DeleteASavedSearch, &mut settings);
        assert!(
            settings
                .questions_not_asked
                .holds(Question::DeleteASavedSearch)
        );
        assert!(settings.check_spelling_before_send);

        let mut settings = AppConfig::default();
        stop_asking(Question::SendDespiteSpelling, &mut settings);
        assert!(
            !settings.check_spelling_before_send,
            "the spelling question's stop is its own setting on General"
        );
        assert_eq!(
            settings.questions_not_asked,
            QuestionsNotAsked::default(),
            "and it is not held a second time on the Questions page"
        );
    }

    #[test]
    fn test_a_stopped_question_is_answered_without_asking() {
        let mut not_asked = QuestionsNotAsked::default();
        assert_eq!(
            answered_without_asking(Question::DeleteASavedSearch, &not_asked),
            None
        );
        assert_eq!(
            answered_without_asking(Question::MarkAPictureDecorative, &not_asked),
            None
        );
        not_asked.stop(Question::DeleteASavedSearch);
        not_asked.stop(Question::MarkAPictureDecorative);
        assert_eq!(
            answered_without_asking(Question::DeleteASavedSearch, &not_asked),
            Some(Answered::Yes),
            "Delete"
        );
        assert_eq!(
            answered_without_asking(Question::MarkAPictureDecorative, &not_asked),
            Some(Answered::No),
            "Describe It"
        );
        assert_eq!(
            answered_without_asking(Question::MoveAFolder, &not_asked),
            None,
            "a question nobody stopped"
        );
    }

    /// Settings, Questions, a box per question, written from 13.4-12.4's
    /// premise 5 rather than read from `in_settings`.
    const ON_THE_QUESTIONS_PAGE: [(Question, Option<&str>); 22] = [
        (
            Question::DeleteAnItem,
            Some("Ask before deleting a &contact, event, task, note or reminder"),
        ),
        (
            Question::TakeAwayACopy,
            Some("Ask before Undo takes away a cop&y"),
        ),
        (
            Question::DeleteAContainer,
            Some("Ask before deleting a &list, calendar, notebook or contact group"),
        ),
        (
            Question::DeleteAConversation,
            Some("Ask before deleting a con&versation"),
        ),
        (
            Question::DeleteAConversationPermanently,
            Some("Ask before deleting a conversation &permanently"),
        ),
        (
            Question::DeleteASavedSearch,
            Some("Ask before deleting a &saved search"),
        ),
        (Question::MoveAFolder, Some("Ask before &moving a folder")),
        (
            Question::DeleteAFolder,
            Some("Ask before deleting a &folder from the server"),
        ),
        (
            Question::EmptyAFolder,
            Some("Ask before &emptying a folder into the Trash"),
        ),
        (
            Question::EmptyAFolderForGood,
            Some("Ask before emptying a folder for &good"),
        ),
        (
            Question::RunARule,
            Some("Ask before running a &rule over a folder"),
        ),
        (
            Question::RunARuleThatDeletes,
            Some("Ask before running a rule that &deletes"),
        ),
        (
            Question::RemoveFoldersTheServerNoLongerLists,
            Some("Ask before removing folders the mail server &no longer lists"),
        ),
        (
            Question::ImportAKey,
            Some("Ask before importing a &key from an attachment"),
        ),
        (
            Question::ApplyAViewToOtherFolders,
            Some("Ask before applying a vie&w to other folders"),
        ),
        (
            Question::OpenTheReleasesPage,
            Some("Ask before &opening the releases page"),
        ),
        (
            Question::InstallAnUpdate,
            Some("Ask before &installing an update"),
        ),
        (
            Question::MoveTheMailABlockCatches,
            Some("Ask before moving mail already here when &blocking a sender"),
        ),
        (Question::SendDespiteSpelling, None),
        (
            Question::MarkAPictureDecorative,
            Some("Ask whether a small pic&ture is decorative"),
        ),
        (Question::RemoveAKey, Some("&Ask before removing a PGP key")),
        (
            Question::DeleteASoundScheme,
            Some("Ask before deleting a sound sc&heme"),
        ),
    ];

    /// Two of `labels` sharing a letter, or a label with none.
    fn letters_shared_or_missing(labels: &[&str]) -> Vec<String> {
        let mut seen = std::collections::BTreeMap::new();
        let mut wrong = Vec::new();
        for label in labels {
            match letter_of(label) {
                None => wrong.push(format!("{label:?} has no letter")),
                Some(letter) => {
                    if let Some(first) = seen.insert(letter, *label) {
                        wrong.push(format!("{first:?} and {label:?} both take {letter}"));
                    }
                }
            }
        }
        wrong
    }

    #[test]
    fn test_settings_offers_every_question_but_spelling_each_with_a_letter_of_its_own() {
        for (question, label) in ON_THE_QUESTIONS_PAGE {
            assert_eq!(in_settings(question), label, "{question:?}");
        }
        assert_eq!(
            ON_THE_QUESTIONS_PAGE
                .iter()
                .map(|row| row.0)
                .collect::<Vec<_>>(),
            Question::ALL.to_vec(),
            "the table above no longer lists every question in order"
        );
        let labels: Vec<&str> = Question::ALL.into_iter().filter_map(in_settings).collect();
        assert_eq!(labels.len(), 21, "{labels:#?}");
        assert!(
            labels
                .iter()
                .all(|label| label.replace('&', "").starts_with("Ask")),
            "{labels:#?}"
        );
        assert_eq!(letters_shared_or_missing(&labels), Vec::<String>::new());

        // The reading can see each fault, so the pass above is not a pass
        // over nothing.
        assert_eq!(
            letters_shared_or_missing(&["&Ask one", "&Another", "No letter"]).len(),
            2
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
