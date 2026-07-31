//! The page the message editor is.
//!
//! The composer's body is a `contenteditable` in a web view rather than a
//! `wxRichTextCtrl`, and the reason is accessibility rather than formatting.
//! `wxRichTextCtrl` is drawn by wxWidgets on every platform, so it exposes no
//! per-range accessibility attributes anywhere, which means no misspelling can
//! ever be marked, no heading can report itself, and every announcement has to
//! be made by hand and will be slightly wrong forever.
//!
//! A web view gets all of that from the engine. `spellcheck` alone produces
//! native spelling annotations on all three platforms: UIA spelling errors from
//! Chromium, `AXMarkedMisspelled` from WebKit, AT-SPI `invalid:spelling` from
//! WebKitGTK. Each screen reader then announces them itself, which is always
//! better than us doing its job.
//!
//! # The security point, which is the whole reason this file is testable
//!
//! **A reply quotes a stranger's message.** The original body came from
//! whoever sent it, and putting it into a `contenteditable` puts it into a live
//! DOM in a real browser engine. The reader could get away with less care
//! because a text control renders nothing; this cannot.
//!
//! So every quoted body goes through the sanitiser before it reaches the page,
//! and what comes back out goes through it again, because somebody can paste
//! anything into an editor and the result is sent to another person.
//!
//! # Why the keys are bound in the page
//!
//! A web view swallows keys once it has focus. That is what trapped people in
//! the preview pane. Here the page is ours, so the keys that have to escape are
//! bound in it and posted back out: Escape, Ctrl+Enter to send, Ctrl+S to save.
//!
//! The formatting keys are bound there too, for the same reason plus one more.
//! The composer is a dialog, a dialog cannot own a menu bar, and a popup menu
//! does not register the accelerators its labels advertise. So the page is the
//! only place they can live, and [`Format::keystroke`] is what both the page
//! and the menu label are built from, because a menu promising a key nothing
//! binds is worse than a menu with no key on it at all.
//!
//! Everything else is the editor's, which is what somebody typing a message
//! wants.

use crate::application::words::{Position, TextNode};
use crate::common::types::MessageBody;
use crate::presentation::html_renderer::HtmlRenderer;

/// The name the page posts messages to.
///
/// Registered with `add_script_message_handler` on the Rust side. One channel
/// with a `kind` field rather than several, because each one is a separate
/// registration to get wrong.
pub const CHANNEL: &str = "wixenEditor";

/// The element the message is typed into.
const BODY_ID: &str = "wixen-body";

/// The page function that gives the caret a block to stand in.
///
/// Named once here because two things call it: the keydown handler inside the
/// page, and every script the toolbar and menu run from the Rust side.
const BLOCK_FUNCTION: &str = "wixenBlock";

/// Put text into the page as text rather than as markup.
///
/// Two things have to survive. The angle brackets, because a bare address in a
/// message with no HTML part is not a tag, and the sanitiser deletes anything
/// tag-shaped: "reply to <ada@example.com>." came back as "reply to .".
///
/// And the line breaks. A `div` collapses them, and `innerText` reads what is
/// rendered rather than what is in the tree, so a quoted original arrived as
/// one unbroken line in both halves of the message that went out. They become
/// `<br>` rather than a `white-space` rule because this is sent, not just
/// shown, and the sanitiser on the way out keeps a `<br>` and drops a `style`.
///
/// Runs of spaces are not kept. Preserving them needs the CSS that does not
/// survive the trip, and plain-text mail almost never depends on them.
fn escaped_plain_text(text: &str) -> String {
    html_escape::encode_text(text)
        .replace("\r\n", "\n")
        .replace('\n', "<br>")
}

/// Build the editor page for a message body.
///
/// `body` is whatever the composer starts with: empty for a new message, a
/// quoted original for a reply or forward, a saved draft. It is treated as
/// untrusted in every case, because in two of those three it is.
///
/// It arrives typed because the two halves need opposite treatment and no
/// amount of looking at the string tells them apart. "if a < b and c > d" is
/// prose that every markup test calls markup, and running the sanitiser over
/// it deletes the middle of the sentence. The MIME part the body came from is
/// the only thing that knows, so the answer travels with the value.
///
/// `language` is the spelling language as a BCP 47 tag, which the engine reads
/// from `lang` to choose its dictionary. Getting this wrong does not fail
/// loudly, it just marks the wrong words, so it comes from the same setting the
/// send-time check uses.
pub fn editor_document(body: &MessageBody, language: &str, mark_spelling: bool) -> String {
    let renderer = HtmlRenderer::new();
    // Markup is sanitised because the page is a live DOM in a browser engine
    // and a reply quotes a stranger. Text is escaped, which is the same
    // protection by the other route, and keeps what the sanitiser would eat.
    let safe_body = match body {
        MessageBody::Html(html) | MessageBody::Multipart { html, .. } => {
            renderer.sanitize_html(html)
        }
        MessageBody::Plain(text) => escaped_plain_text(text),
    };
    let language = safe_language(language);
    let spellcheck = if mark_spelling { "true" } else { "false" };
    // The sound at the end of a wrong word is the same setting as the marking,
    // so the handler is not there at all when it is off. A handler that runs
    // and decides to do nothing still runs on every keystroke.
    let word_watch = if mark_spelling {
        // Apostrophes and hyphens are inside words rather than after them.
        // Without them here every contraction sounds wrong halfway through
        // being typed, because "don" is checked the moment the apostrophe
        // lands and "don" is not a word.
        "if (!/[\\p{L}\\p{M}'’\\-]/u.test(event.data)) { wordFinished(at); }"
    } else {
        ""
    };
    // The same check for input that arrives as a finished word rather than a
    // character at a time: dictation, and composition for the scripts that
    // need it. Without these two the sound was off for anybody not typing one
    // key at a time, with nothing saying so.
    let phrase_watch = if mark_spelling { "wordAt(at);" } else { "" };
    let format_keys = format_key_table();
    let reach_keys = reach_key_table();
    let link_id = LINK_PLACEHOLDER_ID;
    let block_markers = markdown_block_table();
    // One definition of the limit, shared by the insert dialog and by Tab.
    let max_rows = MAX_TABLE_ROWS;
    let inline_markers = markdown_inline_table();

    format!(
        r#"<!DOCTYPE html>
<html lang="{language}"><head><meta charset="utf-8"><style>
html, body {{ height: 100%; margin: 0; }}
body {{
    font-family: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
    font-size: 1rem; line-height: 1.6;
    color: #1a1a1a; background: #ffffff;
}}
#{BODY_ID} {{
    min-height: 100%; padding: 12px; outline: none;
    word-wrap: break-word;
}}
/* 2.4.11 and 2.4.13. The editor is where the caret lives, so the one time it
   is not focused it still has to be obvious which control it is. */
#{BODY_ID}:focus-visible {{ outline: 3px solid #0066cc; outline-offset: -3px; }}
blockquote {{
    border-left: 3px solid #767676; margin-left: 0; padding-left: 12px;
    color: #555;
}}
img {{ max-width: 100%; height: auto; }}
@media (prefers-color-scheme: dark) {{
    body {{ background: #1e1e1e; color: #d4d4d4; }}
    blockquote {{ border-color: #9a9a9a; color: #bbb; }}
    #{BODY_ID}:focus-visible {{ outline-color: #569cd6; }}
}}
</style></head><body>
<div id="{BODY_ID}" contenteditable="true" spellcheck="{spellcheck}"
     role="textbox" aria-multiline="true" aria-label="Message body">{safe_body}</div>
<script>
(function () {{
  var body = document.getElementById({BODY_ID:?});
  function post(message) {{
    try {{ window.chrome.webview.postMessage(JSON.stringify(message)); }}
    catch (e) {{
      try {{ window.webkit.messageHandlers[{CHANNEL:?}].postMessage(JSON.stringify(message)); }}
      catch (e2) {{ }}
    }}
  }}
  // Give the caret a block to stand in.
  //
  // A formatting command works on the block the caret is in, and an empty
  // message has none: the editor is an empty div until something is typed into
  // it. `insertOrderedList` makes its own block and `insertUnorderedList` does
  // not, so on a blank message Ctrl+Shift+L announced "Bulleted list" and left
  // plain text behind, and the Enters after it behaved like plain text, which
  // reads as a list that will not end.
  function block() {{
    if (body.firstChild) {{ return; }}
    // A command from the menu or the toolbar arrives while the menu has the
    // focus, and a caret put into an editor nothing is in does not stick. Only
    // when the focus is somewhere else: calling focus on an editor that
    // already has it throws the caret away, and doing that here undid the
    // command for every key press, which is the direction this arrived from.
    if (document.activeElement !== body) {{ body.focus(); }}
    body.innerHTML = '<div><br></div>';
    var caret = document.createRange();
    caret.setStart(body.firstChild, 0);
    caret.collapse(true);
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(caret);
  }}
  // Also on `window`, because the toolbar and the menu apply their commands by
  // running a script from the Rust side rather than through the keydown
  // handler, and an empty message is empty whichever way the command arrives.
  window.{BLOCK_FUNCTION} = block;
  // The formatting keys, built from the same list the menu is built from.
  var formats = {format_keys};
  // The underlined letters, built from the same list the window is built from.
  var reaches = {reach_keys};
  // The keys that have to leave, and the ones that act here. Everything else
  // belongs to the editor, which is what somebody typing a message wants, and
  // is why this is a window of its own rather than a pane sharing one.
  body.addEventListener('keydown', function (event) {{
    if (event.key === 'Escape') {{
      event.preventDefault();
      post({{ kind: 'cancel' }});
      return;
    }}
    if (event.key === 'Enter' && event.ctrlKey) {{
      event.preventDefault();
      post({{ kind: 'send' }});
      return;
    }}
    if (event.key === 's' && event.ctrlKey && !event.shiftKey && !event.altKey) {{
      event.preventDefault();
      post({{ kind: 'save' }});
      return;
    }}
    // F7 is the spelling key in every Windows application that has one, and
    // this is a web view, so it has to be caught here or it never arrives.
    if (event.key === 'F7' && !event.ctrlKey && !event.altKey && !event.shiftKey) {{
      event.preventDefault();
      post({{ kind: 'spelling' }});
      return;
    }}
    if (event.key === 'Tab' && !event.ctrlKey && !event.altKey) {{
      if (tableTab(event.shiftKey)) {{ event.preventDefault(); return; }}
      // Outside a table the body used to keep Tab and do nothing with it, so
      // the only ways back to the Subject line were the mouse and Escape, and
      // Escape throws the message away. Left to itself the browser does not
      // hand focus out at all, so the key is taken here and the window is
      // asked to move focus.
      event.preventDefault();
      post({{ kind: 'leave', back: event.shiftKey }});
      return;
    }}
    // Alt and a letter, which is how a Windows dialog is worked without a
    // mouse. Held with Ctrl as well it is a formatting key, and those are
    // matched below.
    if (event.altKey && !event.ctrlKey && event.key.length === 1) {{
      var letter = event.key.toLowerCase();
      for (var r = 0; r < reaches.length; r++) {{
        if (reaches[r].key === letter) {{
          event.preventDefault();
          post({{ kind: 'reach', index: reaches[r].index }});
          return;
        }}
      }}
    }}
    // A key that types a character keeps typing it. On some layouts AltGr,
    // which arrives as Ctrl and Alt together, produces a real character on the
    // digit row, and taking it would cost somebody a character they need to
    // save them a trip to the menu.
    var types = event.key.length === 1;
    for (var n = 0; n < formats.length; n++) {{
      var f = formats[n];
      if (event.ctrlKey !== f.ctrl || event.shiftKey !== f.shift
          || event.altKey !== f.alt) {{
        continue;
      }}
      if (event.key.toLowerCase() !== f.key && (types || event.code !== f.code)) {{
        continue;
      }}
      event.preventDefault();
      block();
      document.execCommand(f.command, false, f.value);
      post({{ kind: 'format', index: f.index }});
      return;
    }}
  }});
  // ── The words, for the spelling check ──────────────────────────────────
  //
  // On `window` rather than in here, because these are called from outside:
  // the check runs a script and reads the answer back. Everything about where
  // a word is lives on this side, and the Rust side only ever says "the
  // seventh one".
  //
  // Walked afresh each time rather than remembered. A remembered list is one
  // that goes stale the moment somebody types, and a stale position replaces
  // the wrong word.
  // What counts as a block, for saying whether two words are next to each
  // other. Two words either side of a paragraph break look adjacent in a flat
  // list and are not, and the fix offered for a repeated word is to delete it.
  var BLOCKS = /^(ADDRESS|ARTICLE|ASIDE|BLOCKQUOTE|BODY|DD|DIV|DL|DT|FIGCAPTION|FIGURE|FOOTER|FORM|H1|H2|H3|H4|H5|H6|HEADER|LI|MAIN|NAV|OL|P|PRE|SECTION|TABLE|TD|TH|TR|UL)$/;

  function blockOf(node) {{
    var holder = node.parentElement;
    while (holder && holder !== body && !BLOCKS.test(holder.tagName)) {{
      holder = holder.parentElement;
    }}
    return holder || body;
  }}

  function textNodes() {{
    var walker = document.createTreeWalker(body, NodeFilter.SHOW_TEXT, null);
    var found = [];
    var node;
    while ((node = walker.nextNode())) {{ found.push(node); }}
    return found;
  }}

  // Read afresh every time rather than remembered. A remembered list goes
  // stale the moment somebody types, and a stale position replaces the wrong
  // word.
  window.wixenText = function () {{
    var nodes = textNodes();
    var blocks = [];
    var out = [];
    for (var n = 0; n < nodes.length; n++) {{
      var holder = blockOf(nodes[n]);
      var at = blocks.indexOf(holder);
      if (at < 0) {{ at = blocks.length; blocks.push(holder); }}
      out.push({{ text: nodes[n].data, block: at }});
    }}
    return JSON.stringify(out);
  }};

  function rangeOver(index, start, end) {{
    var nodes = textNodes();
    var node = nodes[index];
    if (!node) {{ return null; }}
    var range = document.createRange();
    range.setStart(node, Math.min(start, node.data.length));
    range.setEnd(node, Math.min(end, node.data.length));
    return range;
  }}

  function selectRange(range) {{
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
    body.focus();
  }}

  window.wixenSelectWord = function (index, start, end) {{
    var range = rangeOver(index, start, end);
    if (!range) {{ return false; }}
    selectRange(range);
    var holder = range.startContainer.parentElement;
    if (holder && holder.scrollIntoView) {{ holder.scrollIntoView({{ block: 'nearest' }}); }}
    return true;
  }};

  // Replace a word, and say where that left the caret.
  //
  // Reported rather than worked out, and that is the point. What a replacement
  // does to everything after it is the page's business, so the check carries on
  // from a place the page named rather than from a sum over how many words the
  // replacement turned out to be.
  window.wixenReplaceWord = function (index, start, end, replacement) {{
    var nodes = textNodes();
    var node = nodes[index];
    if (!node) {{ return JSON.stringify(null); }}
    var from = start;
    if (!replacement.length) {{
      // Deleting a repeated word takes the space in front of it, or the
      // sentence is left with a gap where the word was.
      while (from > 0 && /\s/.test(node.data.charAt(from - 1))) {{ from--; }}
    }}
    var range = rangeOver(index, from, end);
    if (!range) {{ return JSON.stringify(null); }}
    selectRange(range);
    if (replacement.length) {{
      document.execCommand('insertText', false, replacement);
    }} else {{
      document.execCommand('delete');
    }}
    var selection = window.getSelection();
    if (!selection.rangeCount) {{ return JSON.stringify(null); }}
    var after = selection.getRangeAt(0);
    var now = textNodes();
    for (var n = 0; n < now.length; n++) {{
      if (now[n] === after.endContainer) {{
        return JSON.stringify({{ node: n, offset: after.endOffset }});
      }}
    }}
    return JSON.stringify(null);
  }};

  // ── Moving through a table ─────────────────────────────────────────────
  //
  // Tab means "next cell" inside a table and "next control" everywhere else,
  // which is what people expect from every other editor and is why it is worth
  // the special case.
  //
  // Nobody is trapped, and there are two ways out rather than one. Shift+Tab
  // from the first cell leaves, and so does Tab from the last cell of a table
  // that has reached the row limit. That is the rule this cannot get wrong,
  // because a keyboard trap in a message body is a person unable to reach the
  // Send button.
  //
  // What Tab does is a question about three numbers, so it is separate from
  // the doing and is tested. It used to take every forward Tab and add a row,
  // without a limit and with no way out forwards at all.
  function tableStep(at, total, rows, backwards) {{
    if (backwards) {{ return at === 0 ? null : {{ cell: at - 1 }}; }}
    if (at + 1 < total) {{ return {{ cell: at + 1 }}; }}
    // Adding a row off the end is what every editor does. Not without end,
    // though: past the same limit the insert path refuses, Tab stops taking
    // the key, so holding it down cannot grow a table forever and there is a
    // way out going forwards.
    if (rows >= {max_rows}) {{ return null; }}
    return {{ addRow: true }};
  }}

  function cellAround(node) {{
    while (node && node !== body) {{
      if (node.nodeType === 1 &&
          (node.tagName === 'TD' || node.tagName === 'TH')) {{
        return node;
      }}
      node = node.parentNode;
    }}
    return null;
  }}

  function allCells(cell) {{
    var table = cell;
    while (table && table.tagName !== 'TABLE') {{ table = table.parentNode; }}
    if (!table) {{ return null; }}
    return {{ table: table, cells: table.querySelectorAll('th, td') }};
  }}

  function caretInto(cell) {{
    var range = document.createRange();
    range.selectNodeContents(cell);
    range.collapse(true);
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  }}

  // A row shaped like the last one, so a table stays rectangular however many
  // rows get added by tabbing off the end of it.
  function addRow(table) {{
    var rows = table.rows;
    var last = rows[rows.length - 1];
    var row = document.createElement('tr');
    for (var n = 0; n < last.cells.length; n++) {{
      var cell = document.createElement('td');
      cell.appendChild(document.createElement('br'));
      row.appendChild(cell);
    }}
    last.parentNode.appendChild(row);
    return row.cells[0];
  }}

  function tableTab(backwards) {{
    var selection = window.getSelection();
    if (!selection || !selection.focusNode) {{ return false; }}
    var cell = cellAround(selection.focusNode);
    if (!cell) {{ return false; }}
    var found = allCells(cell);
    if (!found) {{ return false; }}
    var at = -1;
    for (var n = 0; n < found.cells.length; n++) {{
      if (found.cells[n] === cell) {{ at = n; break; }}
    }}
    if (at < 0) {{ return false; }}
    var step = tableStep(at, found.cells.length, found.table.rows.length, backwards);
    // Nothing to do here means the key belongs to the dialog, which is how
    // somebody leaves the table without a mouse.
    if (!step) {{ return false; }}
    if (step.addRow) {{
      caretInto(addRow(found.table));
      post({{ kind: 'row' }});
      return true;
    }}
    caretInto(found.cells[step.cell]);
    return true;
  }}

  // ── Markdown, as it is typed ───────────────────────────────────────────
  //
  // What each marker means is decided in Rust and generated into these two
  // tables, so what the editor recognises and what the documentation promises
  // are one definition.
  var mdBlocks = {block_markers};
  var mdInline = {inline_markers};
  var mdNumbered = /^[0-9]+\.$/;

  function escapeText(text) {{
    return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }}

  // Where the caret is, but only when it is somewhere this can reason about:
  // in a text node, with the offset inside it.
  function caretText() {{
    var selection = window.getSelection();
    if (!selection || !selection.isCollapsed) {{ return null; }}
    var node = selection.focusNode;
    if (!node || node.nodeType !== 3) {{ return null; }}
    return {{ node: node, offset: selection.focusOffset,
             before: node.data.slice(0, selection.focusOffset) }};
  }}

  function selectBack(node, from, to) {{
    var range = document.createRange();
    range.setStart(node, from);
    range.setEnd(node, to);
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(range);
  }}

  // Put the caret after what was just inserted rather than inside it, so the
  // next word is not swallowed by the formatting that was meant to end.
  function caretAfterInserted() {{
    var placed = body.querySelector('[data-md]');
    if (!placed) {{ return; }}
    placed.removeAttribute('data-md');
    var after = document.createRange();
    after.setStartAfter(placed);
    after.collapse(true);
    var selection = window.getSelection();
    selection.removeAllRanges();
    selection.addRange(after);
  }}

  // ── Recognising, kept apart from applying ──────────────────────────────
  //
  // Everything in this block is a string question with a string answer: no
  // document, no selection, no command. That is what makes it testable, and
  // it is where every rule bug found so far has been. `window.wixenRules`
  // below is how the tests reach them; nothing in the page uses that name.

  // A marker only counts as a marker when it is the whole line so far.
  // Otherwise a sentence ending in a hyphen becomes a bullet at the next
  // space, in the middle of writing, which is the behaviour people hate.
  function blockRule(line) {{
    var trimmed = line.replace(/^\s+/, '');
    for (var n = 0; n < mdBlocks.length; n++) {{
      if (mdBlocks[n].marker === trimmed) {{ return mdBlocks[n]; }}
    }}
    if (mdNumbered.test(trimmed)) {{ return mdBlocks.numbered; }}
    return null;
  }}

  function inlineRule(typed, before) {{
    for (var n = 0; n < mdInline.length; n++) {{
      var style = mdInline[n];
      var mark = style.delimiter;
      if (typed !== mark.charAt(mark.length - 1)) {{ continue; }}
      if (before.slice(-mark.length) !== mark) {{ continue; }}
      var inner = before.slice(0, before.length - mark.length);
      var open = inner.lastIndexOf(mark);
      if (open < 0) {{ continue; }}
      // "**word*" is not an italic wrapping an asterisk. Without this the
      // single-character rule fires one keystroke before the double one, and
      // bold is unreachable by typing.
      if (open > 0 && inner.charAt(open - 1) === mark.charAt(0)) {{ continue; }}
      var content = inner.slice(open + mark.length);
      if (!content.length || /^\s|\s$/.test(content)) {{ continue; }}
      return {{ style: style, open: open, content: content }};
    }}
    return null;
  }}

  function linkParts(before) {{
    var open = before.lastIndexOf('[');
    if (open < 0) {{ return null; }}
    var typed = before.slice(open);
    var middle = typed.indexOf('](');
    if (middle < 1) {{ return null; }}
    var text = typed.slice(1, middle);
    var url = typed.slice(middle + 2, typed.length - 1);
    if (!text.length || !url.length) {{ return null; }}
    // An address can contain brackets: Wikipedia puts them in article titles.
    // This runs on every close bracket typed, so it sees the address one
    // character at a time, and the bracket that ends the link is the one that
    // balances. An unmatched open bracket means there is more address coming.
    //
    // Taking it anyway produced a truncated address, which still passes every
    // check and becomes a link to somewhere else with nothing said.
    var depth = 0;
    for (var c = 0; c < url.length; c++) {{
      if (url.charAt(c) === '(') {{ depth++; }}
      else if (url.charAt(c) === ')') {{ depth--; }}
    }}
    if (depth !== 0) {{ return null; }}
    return {{ open: open, text: text, url: url }};
  }}

  // Which kinds of input the typing rules run for.
  //
  // `insertText` is somebody typing. `insertReplacementText` is dictation and
  // Windows Voice Access, which put a finished phrase in at once, and leaving
  // it out meant somebody dictating a message got no spelling sound at all.
  // That is the group these guardrails name first, and nothing said it was
  // off.
  //
  // Composition is deliberately not here. It fires on every intermediate
  // state while a character is still being chosen, so a rule running then
  // would fire in the middle of a word being decided. `compositionend` is
  // handled on its own, where the word is finished.
  //
  // Paste is not here either. These rules are about what was just written; a
  // pasted block is checked by F7 with the rest of the message.
  function typingInput(inputType) {{
    return inputType === 'insertText' || inputType === 'insertReplacementText';
  }}

  function wordEnded(before) {{
    var match = /[\p{{L}}\p{{M}}][\p{{L}}\p{{M}}'’\-]*$/u.exec(before);
    if (!match) {{ return null; }}
    var word = match[0].replace(/['’\-]+$/u, '');
    // One letter is "a" or "I", and a sound at the end of those would be a
    // sound at the end of half the sentences somebody writes.
    return word.length > 1 ? word : null;
  }}

  window.wixenRules = {{ block: blockRule, inline: inlineRule,
                        link: linkParts, word: wordEnded, table: tableStep,
                        input: typingInput }};

  // ── Applying ───────────────────────────────────────────────────────────

  function blockMarkdown(at) {{
    if (at.node.previousSibling) {{ return; }}
    var rule = blockRule(at.before.slice(0, at.before.length - 1));
    if (!rule) {{ return; }}
    selectBack(at.node, 0, at.offset);
    document.execCommand('delete');
    document.execCommand(rule.command, false, rule.value);
    post({{ kind: 'format', index: rule.index }});
  }}

  function inlineMarkdown(typed, at) {{
    var hit = inlineRule(typed, at.before);
    if (!hit) {{ return; }}
    selectBack(at.node, hit.open, at.offset);
    document.execCommand('insertHTML', false,
      '<' + hit.style.tag + ' data-md="1">' + escapeText(hit.content)
      + '</' + hit.style.tag + '>');
    caretAfterInserted();
    post({{ kind: 'style', index: hit.style.index }});
  }}

  // The words are marked and the address is sent to be checked. Whether it
  // becomes a link is not the page's decision: it goes to another person.
  function linkMarkdown(at) {{
    var parts = linkParts(at.before);
    if (!parts) {{ return; }}
    selectBack(at.node, parts.open, at.offset);
    document.execCommand('insertHTML', false,
      '<span id="{link_id}">' + escapeText(parts.text) + '</span>');
    post({{ kind: 'link', url: parts.url }});
  }}

  // The word that has just been finished, if one has been.
  //
  // Sent as a word rather than checked here, because the dictionary is the
  // one Windows holds and the words somebody has taught it are in it. An
  // engine's own marking and our checker agreeing matters: on Windows they
  // are the same ISpellChecker.
  // A delimiter was just typed, so the word is the one before it.
  function wordFinished(at) {{
    var word = wordEnded(at.before.slice(0, at.before.length - 1));
    if (word) {{ post({{ kind: 'word', text: word }}); }}
  }}

  // No delimiter: the word ends at the caret. Dictation and composition both
  // land a finished word in one go, so there is nothing to strip off it.
  function wordAt(at) {{
    var word = wordEnded(at.before);
    if (word) {{ post({{ kind: 'word', text: word }}); }}
  }}

  body.addEventListener('input', function (event) {{
    if (!typingInput(event.inputType) || !event.data) {{ return; }}
    var at = caretText();
    if (!at) {{ return; }}
    // Dictation puts in a finished phrase rather than a character, so there is
    // no delimiter to look at, and the Markdown rules have nothing to match: a
    // marker is typed, and none of these is one character long.
    if (event.inputType === 'insertReplacementText') {{ {phrase_watch} return; }}
    {word_watch}
    if (event.data === ' ') {{ blockMarkdown(at); return; }}
    if (event.data === ')') {{ linkMarkdown(at); return; }}
    inlineMarkdown(event.data, at);
  }});

  // A word finished by composition, which is how Japanese, Chinese and Korean
  // are written. The Markdown rules are not run from here: their markers are
  // typed directly, and one arriving through composition is a coincidence.
  body.addEventListener('compositionend', function () {{
    var at = caretText();
    if (!at) {{ return; }}
    {phrase_watch}
  }});

  // Focus lands in the message, at the start of it. A reply that opens with
  // the caret after the quoted original means typing below somebody else's
  // words, which is not where a reply goes.
  body.focus();
  var range = document.createRange();
  range.setStart(body, 0);
  range.collapse(true);
  var selection = window.getSelection();
  selection.removeAllRanges();
  selection.addRange(range);
}})();
</script>
</body></html>"#
    )
}

/// The Markdown line markers, as the object the page walks.
///
/// The numbered-list rule hangs off the same object under `numbered`, because
/// it is not one string to compare against: Markdown takes any number, and
/// somebody resuming a list types the one they are up to.
fn markdown_block_table() -> String {
    use crate::presentation::markdown_input::BLOCK_RULES;
    let entries: Vec<String> = BLOCK_RULES
        .iter()
        .map(|rule| {
            format!(
                "{{marker:{:?},{}}}",
                rule.marker,
                command_fields(rule.format)
            )
        })
        .collect();
    format!(
        "Object.assign([{}], {{numbered:{{{}}}}})",
        entries.join(","),
        command_fields(Format::NumberedList)
    )
}

/// The Markdown inline markers, as the array the page walks.
fn markdown_inline_table() -> String {
    use crate::presentation::markdown_input::INLINE_STYLES;
    let entries: Vec<String> = INLINE_STYLES
        .iter()
        .enumerate()
        .map(|(index, style)| {
            format!(
                "{{delimiter:{:?},tag:{:?},index:{index}}}",
                style.delimiter, style.tag
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// The command, its argument and its place in [`Format::ALL`], as JavaScript.
///
/// The place is what comes back in the message, and is how the announcement
/// finds its words again without the page carrying any of them.
fn command_fields(format: Format) -> String {
    let (command, argument) = format.as_command();
    let value = match argument {
        Some(argument) => format!("{argument:?}"),
        None => "null".to_string(),
    };
    let index = Format::ALL
        .iter()
        .position(|candidate| *candidate == format)
        .unwrap_or_default();
    format!("command:{command:?},value:{value},index:{index}")
}

/// The widest table this will build.
///
/// Not arbitrary. A table is read one cell at a time by the people this is for,
/// and past about this width you cannot hold the column you are in while you
/// move across the row. A message is not a spreadsheet, and refusing a
/// twenty-column table out loud is kinder than building one nobody can read.
pub const MAX_TABLE_COLUMNS: usize = 10;

/// The tallest table this will build. Rows can be added by tabbing afterwards.
pub const MAX_TABLE_ROWS: usize = 50;

/// The cell the caret is put in once a table has been inserted.
pub const TABLE_FIRST_CELL_ID: &str = "wixen-first-cell";

/// The markup for a table.
///
/// `<th scope="col">` rather than a bold first row is the entire accessibility
/// argument for this feature. It is what lets the person receiving the message
/// hear "Total, column 3" as they move across it, instead of a wall of numbers
/// they have to keep a mental map of. A table laid out with spaces, which is
/// what people do without this, gives them nothing at all.
///
/// `<br>` in each empty cell because a cell with nothing in it cannot be
/// clicked into or reached by the caret in any engine.
fn table_markup(rows: usize, columns: usize, header: bool) -> String {
    let mut markup = String::from("<table>");
    let mut first_cell = true;
    let mut cell = |tag: &str, extra: &str, markup: &mut String| {
        let id = if first_cell {
            first_cell = false;
            format!(" id=\"{TABLE_FIRST_CELL_ID}\"")
        } else {
            String::new()
        };
        markup.push_str(&format!("<{tag}{extra}{id}><br></{tag}>"));
    };

    if header {
        markup.push_str("<thead><tr>");
        for _ in 0..columns {
            cell("th", " scope=\"col\"", &mut markup);
        }
        markup.push_str("</tr></thead>");
    }
    markup.push_str("<tbody>");
    for _ in 0..rows {
        markup.push_str("<tr>");
        for _ in 0..columns {
            cell("td", "", &mut markup);
        }
        markup.push_str("</tr>");
    }
    markup.push_str("</tbody></table><p><br></p>");
    markup
}

/// The script that puts a table in the message and the caret in its first cell.
///
/// `rows` is the body rows, so a table asked for with a header row has that
/// many rows underneath it. `None` when the size is not one this will build,
/// and then the caller says so rather than inserting something else.
pub fn insert_table_script(rows: usize, columns: usize, header: bool) -> Option<String> {
    if rows == 0 || columns == 0 || rows > MAX_TABLE_ROWS || columns > MAX_TABLE_COLUMNS {
        return None;
    }
    let markup = table_markup(rows, columns, header);
    Some(format!(
        "(function () {{            document.execCommand('insertHTML', false, {markup:?});            var first = document.getElementById({TABLE_FIRST_CELL_ID:?});            if (first) {{              first.removeAttribute('id');              var range = document.createRange();              range.setStart(first, 0);              range.collapse(true);              var selection = window.getSelection();              selection.removeAllRanges();              selection.addRange(range);            }}            document.getElementById({BODY_ID:?}).focus(); }})();"
    ))
}

/// What is said when a table is inserted.
///
/// It says how to move through it because this is the one moment somebody needs
/// to know, and because Tab meaning "next cell" here and "next control"
/// everywhere else is exactly the kind of thing that has to be told rather than
/// discovered.
pub fn table_spoken(rows: usize, columns: usize, header: bool) -> String {
    let header = if header { ", with a header row" } else { "" };
    format!(
        "Table added, {rows} rows by {columns} columns{header}.          In the first cell. Tab moves to the next cell,          and Tab in the last cell adds a row."
    )
}

/// Ask the page for every word in the message, in order.
///
/// The page owns where words are, because an offset into a string cannot be
/// mapped back into a document: what somebody reads has line breaks between
/// blocks that the tree does not contain. Everything on this side is an index
/// into the list this returns.
pub fn text_script() -> String {
    "window.wixenText()".to_string()
}

/// Select the word at that position, and bring it into view.
///
/// Selecting rather than just moving the caret, because a selected word is
/// announced by the screen reader as the check arrives at it, and because the
/// next thing the check does is replace it.
pub fn select_word_script(at: Position, end: usize) -> String {
    let (node, start) = (at.node, at.offset);
    format!("window.wixenSelectWord({node}, {start}, {end})")
}

/// Replace the word at that position.
///
/// An empty replacement deletes the word and the space in front of it, which is
/// what fixing a repeated word means.
pub fn replace_word_script(at: Position, end: usize, replacement: &str) -> String {
    let (node, start) = (at.node, at.offset);
    format!("window.wixenReplaceWord({node}, {start}, {end}, {replacement:?})")
}

/// Both halves of the message, or nothing when the editor did not answer.
///
/// `run_script` returns `None` on any failure, and every call site used to
/// turn that into an empty string, which makes a failed read and an empty
/// message the same thing. They are not, and the difference cost twice: the
/// autosave timer wrote the empty result over the draft it exists to protect,
/// and the send path queued a message with no body and no error.
///
/// So the caller is handed nothing and has to decide. Both callers decide the
/// same way, which is to leave the stored draft alone and say so.
pub fn message_from_editor(
    html: Option<String>,
    plain: Option<String>,
) -> Option<(String, String)> {
    Some((body_from_editor(&html?), plain_from_editor(&plain?)))
}

/// Take the message's text back out of what the page returned.
///
/// Nothing here is trusted to be what was asked for: an answer that is not
/// means no text rather than a panic in the middle of checking a message.
pub fn text_from_editor(raw: &str) -> Vec<TextNode> {
    let unwrapped = serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string());
    serde_json::from_str::<Vec<TextNode>>(&unwrapped).unwrap_or_default()
}

/// Where a replacement left the caret, as the page reported it.
///
/// `None` when the page could not say, which is a reason to stop the pass
/// rather than to guess: carrying on from the wrong place corrects the wrong
/// word, and that is worse than a check that ends early and says so.
pub fn position_from_editor(raw: &str) -> Option<Position> {
    let unwrapped = serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string());
    serde_json::from_str::<Option<Position>>(&unwrapped)
        .ok()
        .flatten()
}

/// The element the page wraps a typed Markdown link in while it is checked.
///
/// The page puts the words in this and asks; the answer either turns it into a
/// link or unwraps it. Marking the range rather than relying on where the caret
/// is means the answer lands in the right place however long it takes and
/// wherever somebody has moved to since.
pub const LINK_PLACEHOLDER_ID: &str = "wixen-md-link";

/// What to do with a Markdown link somebody typed, and what to say about it.
///
/// A refused address leaves the words in place and says so. Quietly turning it
/// into plain text would mean somebody sends a message believing there is a
/// link in it, which is the failure guardrail 9 is about: the gap has to stay
/// visible rather than be absorbed.
pub fn resolve_markdown_link(url: &str) -> (String, String) {
    let Some(safe) = HtmlRenderer::safe_external_url(url) else {
        return (
            unwrap_link_script(),
            format!("{url} is not an address this can link to. The words are left as they are."),
        );
    };
    let script = format!(
        "(function () {{            var mark = document.getElementById({LINK_PLACEHOLDER_ID:?});            if (!mark) {{ return; }}            var anchor = document.createElement('a');            anchor.setAttribute('href', {safe:?});            while (mark.firstChild) {{ anchor.appendChild(mark.firstChild); }}            mark.parentNode.replaceChild(anchor, mark);            var after = document.createRange();            after.setStartAfter(anchor);            after.collapse(true);            var selection = window.getSelection();            selection.removeAllRanges();            selection.addRange(after);            document.getElementById({BODY_ID:?}).focus(); }})();"
    );
    (script, format!("Link to {safe}"))
}

/// Put the words back as they were, with no link on them.
fn unwrap_link_script() -> String {
    format!(
        "(function () {{            var mark = document.getElementById({LINK_PLACEHOLDER_ID:?});            if (!mark) {{ return; }}            var parent = mark.parentNode;            while (mark.firstChild) {{ parent.insertBefore(mark.firstChild, mark); }}            parent.removeChild(mark);            document.getElementById({BODY_ID:?}).focus(); }})();"
    )
}

/// Something in the composer that Alt and a letter reaches.
///
/// Every one of these is a control in the compose window whose label underlines
/// a letter, which on Windows means Alt and that letter operates it. The message
/// body is a web view, and a web view keeps every key it is given, so from
/// inside the body none of them worked: Alt+O did nothing, Alt+T did nothing,
/// and the only key that left the body was Escape, which throws the message
/// away. The page watches for these and posts the one that was pressed.
///
/// The list is the single place the letters are decided. `wx_compose` builds
/// each control from [`Reached::label`], so the underline somebody sees and the
/// key the page watches for are the same statement, and tests pin them
/// together and check that no two clash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    From,
    To,
    Cc,
    Bcc,
    Subject,
    Send,
    Undo,
    Redo,
    Format,
    Spelling,
    Attach,
    SaveDraft,
    Discard,
    Cancel,
}

impl Reached {
    /// Every one, in the order the page indexes them by.
    pub const ALL: [Reached; 14] = [
        Reached::From,
        Reached::To,
        Reached::Cc,
        Reached::Bcc,
        Reached::Subject,
        Reached::Send,
        Reached::Undo,
        Reached::Redo,
        Reached::Format,
        Reached::Spelling,
        Reached::Attach,
        Reached::SaveDraft,
        Reached::Discard,
        Reached::Cancel,
    ];

    /// The visible label, with the ampersand that marks the underlined letter.
    pub const fn label(self) -> &'static str {
        match self {
            Self::From => "&From:",
            Self::To => "&To:",
            Self::Cc => "&Cc:",
            Self::Bcc => "&Bcc:",
            Self::Subject => "&Subject:",
            Self::Send => "Se&nd",
            Self::Undo => "&Undo",
            Self::Redo => "&Redo",
            Self::Format => "F&ormat...",
            // Not "&Spelling": Subject already answers to S, and a letter that
            // lands on one of two controls depending on which was last cannot
            // be learned.
            Self::Spelling => "S&pelling",
            Self::Attach => "&Attach File...",
            Self::SaveDraft => "Save &Draft",
            Self::Discard => "D&iscard",
            Self::Cancel => "Cance&l",
        }
    }

    /// The letter held with Alt, which is the one the label underlines.
    pub const fn letter(self) -> char {
        match self {
            Self::From => 'f',
            Self::To => 't',
            Self::Cc => 'c',
            Self::Bcc => 'b',
            Self::Subject => 's',
            Self::Send => 'n',
            Self::Undo => 'u',
            Self::Redo => 'r',
            Self::Format => 'o',
            Self::Spelling => 'p',
            Self::Attach => 'a',
            Self::SaveDraft => 'd',
            Self::Discard => 'i',
            Self::Cancel => 'l',
        }
    }
}

/// The letters the page watches for, as the array it walks.
///
/// Generated from [`Reached::ALL`] so the page and the window cannot come
/// apart. The index is the position in that list, which is what comes back.
fn reach_key_table() -> String {
    let entries: Vec<String> = Reached::ALL
        .iter()
        .enumerate()
        .map(|(index, reached)| format!("{{key:{:?},index:{index}}}", reached.letter().to_string()))
        .collect();
    format!("[{}]", entries.join(","))
}

/// The formatting keys, as the array the page walks.
///
/// Generated from [`Format::ALL`] so the menu and the keyboard cannot come
/// apart. The index is the position in that list, which is what comes back in
/// the message and is how the announcement finds its words again.
fn format_key_table() -> String {
    let entries: Vec<String> = Format::ALL
        .iter()
        .enumerate()
        .map(|(index, format)| {
            let stroke = format.keystroke();
            let (command, argument) = format.as_command();
            let value = match argument {
                Some(argument) => format!("{argument:?}"),
                None => "null".to_string(),
            };
            format!(
                "{{ctrl:{},shift:{},alt:{},key:{:?},code:{:?},command:{command:?},\
                 value:{value},index:{index}}}",
                stroke.ctrl, stroke.shift, stroke.alt, stroke.key, stroke.code,
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

/// The script that reads the message back out of the page.
///
/// Returned as a string for `run_script` rather than run here, because the web
/// view is the only thing that can execute it and this module stays pure.
pub fn read_body_script() -> String {
    format!("document.getElementById({BODY_ID:?}).innerHTML")
}

/// The script that reads the message as plain text.
///
/// `innerText` rather than a conversion of the HTML, because it is what the
/// engine computed the reader to be looking at: the line breaks are where they
/// appear, the hidden things are absent, and the list markers are the ones on
/// screen. A message goes out as multipart/alternative, and this is the half
/// that everything which does not want HTML will show.
pub fn read_plain_script() -> String {
    format!("document.getElementById({BODY_ID:?}).innerText")
}

/// The script for one formatting command.
///
/// `execCommand` is deprecated and is still the only thing every engine
/// implements for rich editing in a `contenteditable`. When that changes this
/// is the one function to rewrite.
pub fn format_script(command: Format) -> String {
    let (name, value) = command.as_command();
    let value = match value {
        Some(argument) => format!("{argument:?}"),
        None => "null".to_string(),
    };
    format!(
        "window.{BLOCK_FUNCTION}(); \
         document.execCommand({name:?}, false, {value}); \
         document.getElementById({BODY_ID:?}).focus();"
    )
}

/// The script that puts a link on the selection.
///
/// Separate because the URL comes from somebody rather than from a fixed set,
/// so it is checked before it goes into a document. `safe_external_url` is the
/// same guard the reader uses on a link a stranger sent, and it applies here
/// for a reason worth stating: a message goes to somebody else, so a
/// `javascript:` URL typed into a composer is a link posted to another person.
///
/// `None` when the URL is not one this application will put anywhere, and then
/// the caller says so rather than inserting something quietly different.
pub fn link_script(url: &str) -> Option<String> {
    let safe = HtmlRenderer::safe_external_url(url)?;
    Some(format!(
        "document.execCommand(\"createLink\", false, {safe:?}); \
         document.getElementById({BODY_ID:?}).focus();"
    ))
}

/// A key combination, in the two forms it is needed in.
///
/// `key` is `KeyboardEvent.key` lowercased: what the person's layout actually
/// produces, so it follows a keyboard rather than a diagram of one. `code` is
/// the physical key, used only as a fallback when the layout produces nothing
/// printable, because taking a key that types a character would take a
/// character away from somebody who needs it.
///
/// That fallback rule has a consequence worth stating plainly: on a layout
/// where AltGr+1 types a character, Ctrl+Alt+1 types that character instead of
/// applying Heading 1. The menu still applies it, and taking somebody's
/// superscript away to save them a menu is the wrong trade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keystroke {
    ctrl: bool,
    shift: bool,
    alt: bool,
    /// `KeyboardEvent.key`, lowercased.
    key: &'static str,
    /// `KeyboardEvent.code`, the physical key.
    code: &'static str,
}

impl std::fmt::Display for Keystroke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (held, name) in [
            (self.ctrl, "Ctrl"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
        ] {
            if held {
                write!(f, "{name}+")?;
            }
        }
        match self.key {
            " " => f.write_str("Space"),
            key => write!(f, "{}", key.to_uppercase()),
        }
    }
}

/// A formatting command the editor offers.
///
/// Headings and lists are here for the recipient rather than for the writer. A
/// message with real headings can be navigated by whoever receives it, which is
/// exactly what this application spends its time wishing other people's mail
/// had. Sending mail without it would be poor manners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Underline,
    Heading1,
    Heading2,
    Heading3,
    Paragraph,
    BulletList,
    NumberedList,
    Quote,
    RemoveFormatting,
    Undo,
    Redo,
}

impl Format {
    /// Every one, so a toolbar and a menu cover the same set.
    pub const ALL: [Format; 13] = [
        Format::Bold,
        Format::Italic,
        Format::Underline,
        Format::Heading1,
        Format::Heading2,
        Format::Heading3,
        Format::Paragraph,
        Format::BulletList,
        Format::NumberedList,
        Format::Quote,
        Format::RemoveFormatting,
        Format::Undo,
        Format::Redo,
    ];

    /// The `execCommand` name, and its argument where it takes one.
    const fn as_command(self) -> (&'static str, Option<&'static str>) {
        match self {
            Self::Bold => ("bold", None),
            Self::Italic => ("italic", None),
            Self::Underline => ("underline", None),
            Self::Heading1 => ("formatBlock", Some("<h1>")),
            Self::Heading2 => ("formatBlock", Some("<h2>")),
            Self::Heading3 => ("formatBlock", Some("<h3>")),
            Self::Paragraph => ("formatBlock", Some("<p>")),
            Self::BulletList => ("insertUnorderedList", None),
            Self::NumberedList => ("insertOrderedList", None),
            Self::Quote => ("formatBlock", Some("<blockquote>")),
            Self::RemoveFormatting => ("removeFormat", None),
            Self::Undo => ("undo", None),
            Self::Redo => ("redo", None),
        }
    }

    /// The key that applies it.
    ///
    /// The heading keys are Word's and Outlook's, which is where most people
    /// arriving here learned them. Relearning a key is a cost paid by everybody
    /// to save one person a decision.
    ///
    /// One definition, used twice: printed in the menu by [`Self::shortcut`],
    /// and compared against a key event by the page. They were two separate
    /// strings for one commit, which is one commit longer than it takes for a
    /// menu to start promising a key nothing binds.
    pub const fn keystroke(self) -> Keystroke {
        const fn ctrl(key: &'static str, code: &'static str) -> Keystroke {
            Keystroke {
                ctrl: true,
                shift: false,
                alt: false,
                key,
                code,
            }
        }
        const fn ctrl_alt(key: &'static str, code: &'static str) -> Keystroke {
            Keystroke {
                ctrl: true,
                shift: false,
                alt: true,
                key,
                code,
            }
        }
        const fn ctrl_shift(key: &'static str, code: &'static str) -> Keystroke {
            Keystroke {
                ctrl: true,
                shift: true,
                alt: false,
                key,
                code,
            }
        }
        match self {
            Self::Bold => ctrl("b", "KeyB"),
            Self::Italic => ctrl("i", "KeyI"),
            Self::Underline => ctrl("u", "KeyU"),
            Self::Heading1 => ctrl_alt("1", "Digit1"),
            Self::Heading2 => ctrl_alt("2", "Digit2"),
            Self::Heading3 => ctrl_alt("3", "Digit3"),
            Self::Paragraph => ctrl_alt("0", "Digit0"),
            Self::BulletList => ctrl_shift("l", "KeyL"),
            Self::NumberedList => ctrl_shift("o", "KeyO"),
            Self::Quote => ctrl_shift("q", "KeyQ"),
            Self::RemoveFormatting => ctrl(" ", "Space"),
            Self::Undo => ctrl("z", "KeyZ"),
            Self::Redo => ctrl("y", "KeyY"),
        }
    }

    /// The key, written the way Windows writes it.
    pub fn shortcut(self) -> String {
        self.keystroke().to_string()
    }

    /// What it is called in a menu, with its key.
    pub fn label(self) -> String {
        let name = match self {
            Self::Bold => "&Bold",
            Self::Italic => "&Italic",
            Self::Underline => "&Underline",
            Self::Heading1 => "Heading &1",
            Self::Heading2 => "Heading &2",
            Self::Heading3 => "Heading &3",
            Self::Paragraph => "&Normal Text",
            Self::BulletList => "Bulleted &List",
            Self::NumberedList => "N&umbered List",
            Self::Quote => "&Quote",
            Self::RemoveFormatting => "Remove &Formatting",
            Self::Undo => "U&ndo",
            Self::Redo => "&Redo",
        };
        format!("{name}\t{}", self.shortcut())
    }

    /// What is announced when it is applied.
    ///
    /// A formatting change nobody is told about is one that happened to
    /// somebody rather than one they made. The state is not read back from the
    /// engine, so this says what was asked for rather than what is now true,
    /// which is honest and is why it does not say "bold on".
    pub const fn spoken(self) -> &'static str {
        match self {
            Self::Bold => "Bold applied",
            Self::Italic => "Italic applied",
            Self::Underline => "Underline applied",
            Self::Heading1 => "Heading level 1",
            Self::Heading2 => "Heading level 2",
            Self::Heading3 => "Heading level 3",
            Self::Paragraph => "Normal text",
            Self::BulletList => "Bulleted list",
            Self::NumberedList => "Numbered list",
            Self::Quote => "Quote",
            Self::RemoveFormatting => "Formatting removed",
            Self::Undo => "Undone",
            Self::Redo => "Redone",
        }
    }
}

/// What the page asked the application to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorMessage {
    Send,
    Save,
    Cancel,
    /// A formatting key was pressed, and the page has already applied it.
    ///
    /// Applied there rather than here because a round trip in the middle of
    /// typing is latency somebody feels. Reported back so the announcement
    /// still goes through the queue that bounds and coalesces every other one.
    Formatted(Format),
    /// A Markdown inline marker was closed, and the page has already applied it.
    Styled(&'static crate::presentation::markdown_input::InlineStyle),
    /// A Markdown link was typed, and its address needs checking before it
    /// becomes a link.
    ///
    /// The page cannot decide this on its own. The address ends up in a
    /// document that goes to another person, so the rule that says which
    /// addresses this application will carry stays on the Rust side, where it
    /// is the same rule the reader applies to a stranger's mail.
    Link(String),
    /// Tab in the last cell of a table made another row.
    TableRowAdded,
    /// F7: walk the spelling of the message.
    CheckSpelling,
    /// A word was finished, and wants checking.
    WordFinished(String),
    /// Alt and a letter were pressed, naming a control in the compose window.
    ///
    /// The page cannot operate the window it sits in, so it says which control
    /// was asked for and the Rust side focuses it or runs it.
    Reached(Reached),
    /// Tab, so focus leaves the body. `back` is Shift+Tab.
    Leaving {
        back: bool,
    },
}

/// Read one message posted by the page.
///
/// `None` for anything unrecognised. The page is ours, so an unknown message is
/// a bug rather than an attack, and doing nothing beats guessing which command
/// somebody meant.
pub fn parse_message(raw: &str) -> Option<EditorMessage> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    match value.get("kind")?.as_str()? {
        "send" => Some(EditorMessage::Send),
        "save" => Some(EditorMessage::Save),
        "cancel" => Some(EditorMessage::Cancel),
        "format" => {
            let index = usize::try_from(value.get("index")?.as_u64()?).ok()?;
            Format::ALL
                .get(index)
                .copied()
                .map(EditorMessage::Formatted)
        }
        "style" => {
            let index = usize::try_from(value.get("index")?.as_u64()?).ok()?;
            crate::presentation::markdown_input::inline_style(index).map(EditorMessage::Styled)
        }
        "link" => Some(EditorMessage::Link(value.get("url")?.as_str()?.to_string())),
        "row" => Some(EditorMessage::TableRowAdded),
        "spelling" => Some(EditorMessage::CheckSpelling),
        "word" => Some(EditorMessage::WordFinished(
            value.get("text")?.as_str()?.to_string(),
        )),
        "reach" => {
            let index = usize::try_from(value.get("index")?.as_u64()?).ok()?;
            Reached::ALL.get(index).copied().map(EditorMessage::Reached)
        }
        "leave" => Some(EditorMessage::Leaving {
            back: value.get("back")?.as_bool()?,
        }),
        _ => None,
    }
}

/// Take the plain text back out of what the page returned.
///
/// Not sanitised, because it is text rather than markup and there is nothing in
/// it left to execute. It is unwrapped for the same reason the HTML is: some
/// backends hand a script result back as a JSON string.
pub fn plain_from_editor(raw: &str) -> String {
    serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string())
}

/// Take the body back out of what the page returned.
///
/// Sanitised on the way out as well as on the way in. Anything can be pasted
/// into an editor, and what comes out of this one is sent to another person, so
/// this is the last point at which it is ours to check.
pub fn body_from_editor(raw: &str) -> String {
    // `run_script` returns the value as a JSON string on some backends and
    // bare on others, so a quoted result is unwrapped before anything else
    // looks at it.
    let unquoted = serde_json::from_str::<String>(raw).unwrap_or_else(|_| raw.to_string());
    HtmlRenderer::new().sanitize_html(&unquoted)
}

/// A language tag safe to put in an attribute.
///
/// It comes from a settings file, which is a file somebody can edit. A tag with
/// a quote in it would close the attribute and start writing markup.
fn safe_language(tag: &str) -> String {
    let cleaned: String = tag
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(35)
        .collect();
    if cleaned.is_empty() {
        "en".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new message: nothing to quote and nothing to decide about.
    fn blank() -> MessageBody {
        MessageBody::Plain(String::new())
    }

    /// The opening `<html ...>` tag, which is where an injected attribute
    /// would have to land.
    fn first_tag(page: &str) -> &str {
        page.split_once("<html")
            .and_then(|(_, rest)| rest.split_once('>'))
            .map(|(tag, _)| tag)
            .unwrap_or("")
    }

    #[test]
    fn test_the_editor_asks_the_engine_to_check_spelling() {
        // The whole reason the composer is a web view. Without this the engine
        // marks nothing and no screen reader announces anything, and the
        // control swap bought nothing at all.
        let page = editor_document(&blank(), "en-GB", true);

        assert!(page.contains(r#"spellcheck="true""#), "{page}");
    }

    #[test]
    fn test_the_language_reaches_the_engine() {
        // The engine picks its dictionary from lang. Getting this wrong does
        // not fail loudly, it marks the wrong words, which is worse.
        let page = editor_document(&blank(), "fr-FR", true);

        assert!(page.contains(r#"<html lang="fr-FR""#), "{page}");
    }

    #[test]
    fn test_a_language_from_a_file_cannot_close_the_attribute() {
        // It comes from a settings file, which somebody can edit. A quote in
        // it would end the attribute and start writing markup.
        let page = editor_document(&blank(), r#"en" onload="alert(1)"#, true);

        // The letters survive, harmlessly, as part of the value: the tag reads
        // lang="enonloadalert1". What must not survive is anything that could
        // end the attribute and begin another, so the test is for an injected
        // attribute rather than for the word.
        assert!(
            !page.contains(" onload="),
            "an attribute was injected: {page}"
        );
        assert!(
            page.lines().next().is_some() && !first_tag(&page).contains('\''),
            "a quote reached the tag: {page}"
        );
        assert!(page.contains(r#"<html lang="enonloadalert1""#), "{page}");
    }

    #[test]
    fn test_an_empty_language_still_produces_a_valid_document() {
        let page = editor_document(&blank(), "", true);

        assert!(page.contains(r#"<html lang="en""#), "{page}");
    }

    #[test]
    fn test_a_quoted_reply_cannot_bring_a_script_into_the_editor() {
        // The security point. A reply quotes a stranger's message, and this
        // puts it in a live DOM in a real browser engine. The reader got away
        // with less care because a text control renders nothing; this cannot.
        let page = editor_document(
            &MessageBody::Html(
                "<p>Original</p><script>alert('x')</script><img src=x onerror=alert(1)>".into(),
            ),
            "en",
            true,
        );

        assert!(!page.contains("alert('x')"), "script survived: {page}");
        assert!(!page.contains("onerror"), "handler survived: {page}");
        assert!(page.contains("Original"), "the quoted message was lost");
    }

    #[test]
    fn test_a_plain_text_original_keeps_the_words_inside_angle_brackets() {
        // The sanitiser deletes anything shaped like a tag, and in a message
        // with no HTML part a bare address is shaped like a tag. So replying
        // to one sent the middle of the sentence away with nothing said. The
        // answer cannot be guessed from the string, because "if a < b and c >
        // d" is prose that every guess calls markup; it has to be carried.
        let page = editor_document(
            &MessageBody::Plain("Please reply to <ada@example.com>.".into()),
            "en",
            false,
        );

        assert!(page.contains("&lt;ada@example.com&gt;"), "{page}");
    }

    #[test]
    fn test_a_plain_text_original_keeps_its_line_breaks() {
        // The half of this that needs no angle bracket to bite. A div collapses
        // newlines, and `innerText` reads what is rendered rather than what is
        // in the tree, so every plain-text reply and forward went out as one
        // unbroken line in both halves of the message.
        //
        // For somebody reading it back with a screen reader this is the part
        // least likely to be noticed before Send, because it is read as
        // continuous prose with nothing to say the breaks are gone.
        let page = editor_document(
            &MessageBody::Plain("Hi Bob,\n\nRegards\nAda".into()),
            "en",
            false,
        );

        assert!(page.contains("Hi Bob,<br><br>Regards<br>Ada"), "{page}");
    }

    #[test]
    fn test_a_carriage_return_does_not_survive_as_a_stray_character() {
        // Mail is full of CRLF, and a \r left behind after the \n became a
        // <br> is a character the recipient's client may or may not swallow.
        let page = editor_document(&MessageBody::Plain("one\r\ntwo".into()), "en", false);

        assert!(page.contains("one<br>two"), "{page}");
        assert!(!page.contains('\r'), "stray carriage return: {page}");
    }

    #[test]
    fn test_the_editor_has_a_name_and_says_it_takes_more_than_one_line() {
        // A contenteditable is a div until it is told otherwise. Without these
        // it reports as a group with no name, which is what a custom control
        // always does until somebody remembers.
        let page = editor_document(&blank(), "en", true);

        assert!(page.contains(r#"role="textbox""#), "{page}");
        assert!(page.contains(r#"aria-multiline="true""#), "{page}");
        assert!(page.contains(r#"aria-label="Message body""#), "{page}");
    }

    #[test]
    fn test_the_three_keys_that_have_to_escape_are_bound() {
        // A web view swallows keys once it has focus, which is what trapped
        // people in the preview pane. The page is ours, so the ones that have
        // to leave are bound in it.
        let page = editor_document(&blank(), "en", true);

        assert!(page.contains("'Escape'"), "{page}");
        assert!(page.contains("event.ctrlKey"), "{page}");
        for kind in ["'cancel'", "'send'", "'save'"] {
            assert!(page.contains(kind), "{kind} not posted: {page}");
        }
    }

    #[test]
    fn test_a_list_can_be_started_on_a_message_with_nothing_in_it_yet() {
        // Measured against the running composer: on an empty message
        // `insertUnorderedList` does nothing at all, while `insertOrderedList`
        // makes a list. So Ctrl+Shift+L on a blank message announced "Bulleted
        // list" and left plain text, and every Enter after that behaved like
        // plain text too, which read as a list that would not end.
        //
        // The engine needs a block to put the list around, and an empty
        // contenteditable has none until something is typed into it.
        let page = editor_document(&blank(), "en", true);

        assert!(page.contains("function block()"), "{page}");
        assert!(page.contains("<div><br></div>"), "{page}");
        assert!(
            page.contains("block();\n      document.execCommand(f.command"),
            "the guard has to run before the command, not after: {page}"
        );
    }

    #[test]
    fn test_the_menu_gets_the_same_guard_as_the_key() {
        // The toolbar and the menu do not go through the keydown handler: they
        // run a script from the Rust side. An empty message is empty whichever
        // way the command arrives, so both ways need the block first, and the
        // page has to hand the function out for the script to reach it.
        let script = format_script(Format::BulletList);

        assert!(script.starts_with("window.wixenBlock();"), "{script}");
        assert!(
            editor_document(&blank(), "en", true).contains("window.wixenBlock = block;"),
            "the script calls a function the page never exposes"
        );
    }

    #[test]
    fn test_every_underlined_letter_in_the_composer_reaches_it_from_the_body() {
        // Alt and a letter is how a Windows dialog is worked without a mouse,
        // and the message body is a web view, which keeps every key it is
        // given. So Alt+O did nothing, and so did Alt+T, and the only way out
        // of the body was Escape, which throws the message away.
        let page = editor_document(&blank(), "en", true);

        for (index, reached) in Reached::ALL.iter().enumerate() {
            let entry = format!("{{key:{:?},index:{index}}}", reached.letter().to_string());
            assert!(
                page.contains(&entry),
                "Alt+{} does not leave the body, so {} cannot be reached: {page}",
                reached.letter().to_ascii_uppercase(),
                reached.label(),
            );
        }
    }

    #[test]
    fn test_the_letter_is_the_one_underlined_in_the_label() {
        // The letter and the underline are two statements about the same key.
        // A label promising Alt+A while the code watches for I is worse than
        // no mnemonic: it is a key somebody will press and nothing will happen.
        for reached in Reached::ALL {
            let marked = reached
                .label()
                .split('&')
                .nth(1)
                .and_then(|rest| rest.chars().next())
                .map(|letter| letter.to_ascii_lowercase());
            assert_eq!(
                marked,
                Some(reached.letter()),
                "{:?} watches for {} and its label underlines something else: {}",
                reached,
                reached.letter(),
                reached.label(),
            );
        }
    }

    #[test]
    fn test_no_two_things_in_the_composer_answer_to_the_same_letter() {
        // wxWidgets cycles between controls that share a mnemonic, so a clash
        // is not a crash. It is worse than that: Alt+S lands on Subject or on
        // Spelling depending on where the last one left off, which is a key
        // that cannot be learned.
        let mut seen = std::collections::HashMap::new();
        for reached in Reached::ALL {
            if let Some(other) = seen.insert(reached.letter(), reached) {
                panic!(
                    "Alt+{} is both {} and {}",
                    reached.letter().to_ascii_uppercase(),
                    other.label(),
                    reached.label(),
                );
            }
        }
    }

    #[test]
    fn test_tab_leaves_the_body_in_both_directions() {
        // Without this the body is a trap. Tab did nothing outside a table, so
        // the only ways out were the mouse and Escape, and Escape discards.
        let page = editor_document(&blank(), "en", true);

        assert!(page.contains("'leave'"), "{page}");
        assert!(page.contains("back: event.shiftKey"), "{page}");
    }

    #[test]
    fn test_a_message_from_the_page_is_understood() {
        assert_eq!(
            parse_message(r#"{"kind":"send"}"#),
            Some(EditorMessage::Send)
        );
        assert_eq!(
            parse_message(r#"{"kind":"save"}"#),
            Some(EditorMessage::Save)
        );
        assert_eq!(
            parse_message(r#"{"kind":"cancel"}"#),
            Some(EditorMessage::Cancel)
        );
        assert_eq!(
            parse_message(r#"{"kind":"reach","index":0}"#),
            Some(EditorMessage::Reached(Reached::ALL[0]))
        );
        assert_eq!(
            parse_message(r#"{"kind":"leave","back":true}"#),
            Some(EditorMessage::Leaving { back: true })
        );
        assert_eq!(
            parse_message(r#"{"kind":"leave","back":false}"#),
            Some(EditorMessage::Leaving { back: false })
        );
    }

    #[test]
    fn test_a_letter_nothing_answers_to_does_nothing() {
        // The index comes back from the page, and a page and a list that have
        // come apart must not reach whichever command happens to sit there.
        assert_eq!(parse_message(r#"{"kind":"reach","index":999}"#), None);
    }

    #[test]
    fn test_a_message_nobody_recognises_does_nothing() {
        // Doing nothing beats guessing which command somebody meant, and every
        // one of these is a command that would change or discard a message.
        for raw in [
            "",
            "not json",
            r#"{"kind":"delete-everything"}"#,
            r#"{"nope":1}"#,
            r#"{"kind":42}"#,
        ] {
            assert_eq!(parse_message(raw), None, "for {raw:?}");
        }
    }

    #[test]
    fn test_what_comes_out_of_the_editor_is_checked_too() {
        // Anything can be pasted into an editor, and what leaves this one is
        // sent to another person. This is the last point at which it is ours.
        let out = body_from_editor("<p>Hello</p><script>alert(1)</script>");

        assert!(!out.contains("<script"), "{out}");
        assert!(out.contains("Hello"), "{out}");
    }

    #[test]
    fn test_a_quoted_result_is_unwrapped_before_it_is_read() {
        // run_script returns the value as a JSON string on some backends and
        // bare on others. Reading the quoted form as markup would send a
        // message full of backslash-escaped quotes.
        let out = body_from_editor(r#""<p>Hello</p>""#);

        assert!(out.contains("Hello"), "{out}");
        assert!(!out.contains("\\\""), "the escaping survived: {out}");
    }

    #[test]
    fn test_the_message_can_be_read_as_text_as_well_as_markup() {
        // A message goes out as multipart/alternative, and this is the half
        // everything that does not want HTML shows. Without it the markup
        // itself is what a text-only reader sees, tags and all, which is what
        // swapping the control to a web view broke until this existed.
        assert!(read_plain_script().contains("innerText"));
        assert!(read_body_script().contains("innerHTML"));
    }

    #[test]
    fn test_plain_text_is_not_sanitised_because_it_is_not_markup() {
        // Nothing in it is left to execute, and running it through the
        // sanitiser would escape the angle brackets somebody deliberately
        // typed into a message about markup.
        let out = plain_from_editor("if a < b && c > d, see <notes>");

        assert_eq!(out, "if a < b && c > d, see <notes>");
    }

    #[test]
    fn test_the_plain_text_is_unwrapped_like_the_markup_is() {
        assert_eq!(plain_from_editor(r#""Hello there""#), "Hello there");
    }

    #[test]
    fn test_every_formatting_command_names_itself_when_applied() {
        // A formatting change nobody is told about is one that happened to
        // somebody rather than one they made.
        for format in Format::ALL {
            let (name, argument) = format.as_command();
            let script = format_script(format);
            assert!(!format.spoken().is_empty(), "{format:?}");
            assert!(script.contains(name), "{format:?}");
            if let Some(argument) = argument {
                assert!(script.contains(argument), "{format:?}");
            }
        }
    }

    #[test]
    fn test_no_two_formatting_commands_sound_alike() {
        // Announcements are how somebody knows which one they got. Two that
        // say the same thing are worse than one, because the person now has to
        // check what happened rather than trust what they heard.
        let mut spoken: Vec<&str> = Format::ALL.iter().map(|f| f.spoken()).collect();
        spoken.sort_unstable();
        let before = spoken.len();
        spoken.dedup();
        assert_eq!(spoken.len(), before, "two commands announce the same thing");
    }

    #[test]
    fn test_every_command_carries_its_key_into_the_menu() {
        // A key nobody can discover is a key nobody has. The menu is where
        // somebody working by keyboard finds out these exist at all, so the
        // key belongs on the item rather than only in the documentation.
        for format in Format::ALL {
            let label = format.label();
            assert!(label.contains('\t'), "{format:?} has no key column");
            assert!(label.contains(&format.shortcut()), "{format:?}");
            assert!(label.contains('&'), "{format:?} has no access key");
        }
    }

    #[test]
    fn test_every_key_the_menu_promises_is_bound_in_the_page() {
        // The defect this exists to stop: a menu item reading "Heading 1
        // Ctrl+Alt+1" where nothing binds Ctrl+Alt+1. A key that is advertised
        // and does nothing is worse than no key, because somebody stops looking
        // for another way to do it.
        //
        // A popup menu does not register accelerators the way a menu bar does,
        // and the body is a web view, which swallows keys once it has focus.
        // So the page is the only place these can live.
        let page = editor_document(&blank(), "en-US", true);
        for format in Format::ALL {
            let stroke = format.keystroke();
            assert!(
                page.contains(&format!("{:?}", stroke.key)),
                "{format:?}: nothing in the page reacts to {}",
                format.shortcut()
            );
        }
    }

    #[test]
    fn test_the_key_in_the_menu_is_the_key_the_page_watches_for() {
        // One definition, printed one way and compared another. Two strings
        // drift; this cannot.
        assert_eq!(Format::Heading1.shortcut(), "Ctrl+Alt+1");
        assert_eq!(Format::Bold.shortcut(), "Ctrl+B");
        assert_eq!(Format::BulletList.shortcut(), "Ctrl+Shift+L");
        assert_eq!(Format::RemoveFormatting.shortcut(), "Ctrl+Space");
    }

    #[test]
    fn test_a_key_applied_in_the_page_is_reported_so_it_can_be_announced() {
        // The page applies it, because a round trip to Rust and back is latency
        // in the middle of typing. It says what it did so the announcement
        // still goes through the queue with every other one.
        assert_eq!(
            parse_message(r#"{"kind":"format","index":0}"#),
            Some(EditorMessage::Formatted(Format::Bold))
        );
        assert_eq!(
            parse_message(r#"{"kind":"format","index":3}"#),
            Some(EditorMessage::Formatted(Format::Heading1))
        );
    }

    #[test]
    fn test_a_format_index_from_nowhere_is_ignored() {
        assert_eq!(parse_message(r#"{"kind":"format","index":99}"#), None);
        assert_eq!(parse_message(r#"{"kind":"format"}"#), None);
    }

    #[test]
    fn test_the_markdown_a_person_types_is_recognised_by_the_page() {
        // Typing "## " and carrying on is the only way to put a heading in a
        // message without leaving the sentence. It is how a lot of people who
        // cannot see a toolbar already write, so it is the primary path rather
        // than a shortcut for the menu.
        let page = editor_document(&blank(), "en-US", true);
        for rule in &crate::presentation::markdown_input::BLOCK_RULES {
            assert!(
                page.contains(&format!("{:?}", rule.marker)),
                "the page does not watch for {:?}",
                rule.marker
            );
        }
        for style in &crate::presentation::markdown_input::INLINE_STYLES {
            assert!(
                page.contains(&format!("{:?}", style.delimiter)),
                "the page does not watch for {:?}",
                style.delimiter
            );
        }
    }

    #[test]
    fn test_an_inline_style_is_reported_so_it_can_be_announced() {
        let Some(EditorMessage::Styled(style)) = parse_message(r#"{"kind":"style","index":0}"#)
        else {
            panic!("a style message should parse");
        };
        assert_eq!(style.spoken, "Bold");
    }

    #[test]
    fn test_a_style_index_from_nowhere_is_ignored() {
        assert_eq!(parse_message(r#"{"kind":"style","index":99}"#), None);
    }

    #[test]
    fn test_a_typed_link_comes_here_to_be_checked() {
        // The page cannot decide this. The address goes into a document and
        // then to another person, so the check that says which addresses this
        // application will carry lives on one side of the bridge.
        assert_eq!(
            parse_message(r#"{"kind":"link","url":"https://example.com"}"#),
            Some(EditorMessage::Link("https://example.com".to_string()))
        );
    }

    #[test]
    fn test_a_checked_link_replaces_the_text_that_was_typed() {
        let (script, spoken) = resolve_markdown_link("https://example.com/agenda");
        assert!(script.contains("https://example.com/agenda"));
        assert!(script.contains(LINK_PLACEHOLDER_ID));
        assert!(spoken.contains("Link"), "{spoken}");
    }

    #[test]
    fn test_a_link_this_will_not_carry_leaves_the_words_and_says_so() {
        // Silently making it plain text would be the wrong end of guardrail 9:
        // somebody would send a message believing it had a link in it.
        let (script, spoken) = resolve_markdown_link("javascript:alert(1)");
        assert!(!script.contains("javascript:"), "{script}");
        assert!(script.contains(LINK_PLACEHOLDER_ID));
        assert!(
            spoken.to_lowercase().contains("not"),
            "the refusal has to say so: {spoken}"
        );
    }

    #[test]
    fn test_a_table_is_built_with_headers_the_recipient_can_navigate() {
        // The whole point of putting a table in a message rather than laying
        // columns out with spaces. `scope` is what lets the person receiving it
        // hear "Total, column 3" instead of a wall of numbers.
        let markup = table_markup(2, 3, true);
        assert_eq!(markup.matches("<th scope=\"col\"").count(), 3);
        assert_eq!(
            markup.matches("<tr>").count(),
            3,
            "a header row and two more"
        );
        assert!(insert_table_script(2, 3, true).is_some());
    }

    #[test]
    fn test_a_table_without_a_header_row_has_no_header_cells() {
        let markup = table_markup(2, 2, false);
        assert!(!markup.contains("<th"), "{markup}");
        assert_eq!(markup.matches("<td").count(), 4);
    }

    #[test]
    fn test_the_header_cells_survive_the_way_out() {
        // The message is sanitised on the way out, so a header that the
        // sanitiser strips is a table that arrives unnavigable. This is the
        // test that says whether the accessibility of it actually ships.
        let cleaned = HtmlRenderer::new().sanitize_html(&table_markup(2, 2, true));
        assert!(
            cleaned.contains("scope=\"col\""),
            "the sanitiser removed the column headers: {cleaned}"
        );
        assert!(cleaned.contains("<th"), "{cleaned}");
    }

    #[test]
    fn test_a_table_with_no_rows_or_no_columns_is_not_a_table() {
        assert_eq!(insert_table_script(0, 3, true), None);
        assert_eq!(insert_table_script(3, 0, true), None);
    }

    #[test]
    fn test_a_table_too_big_to_read_is_refused() {
        // Not an arbitrary limit: a table wider than this cannot be held in
        // your head while you move across it, and a message is not a
        // spreadsheet. Refusing it out loud beats building something unusable.
        assert_eq!(insert_table_script(1, MAX_TABLE_COLUMNS + 1, true), None);
        assert_eq!(insert_table_script(MAX_TABLE_ROWS + 1, 1, true), None);
        assert!(insert_table_script(MAX_TABLE_ROWS, MAX_TABLE_COLUMNS, true).is_some());
    }

    #[test]
    fn test_the_caret_lands_in_the_first_cell() {
        // Otherwise the table is inserted and the caret is somewhere else, and
        // somebody who cannot see it has to go looking for a thing that was
        // just created for them to type into.
        let script = insert_table_script(2, 2, true).expect("2 by 2 is a table");
        assert!(script.contains(TABLE_FIRST_CELL_ID), "{script}");
    }

    #[test]
    fn test_the_table_says_its_shape_and_how_to_move_through_it() {
        let said = table_spoken(4, 3, true);
        assert!(said.contains('4') && said.contains('3'), "{said}");
        assert!(said.to_lowercase().contains("header"), "{said}");
        assert!(said.contains("Tab"), "{said}");
        assert!(!table_spoken(2, 2, false).to_lowercase().contains("header"));
    }

    #[test]
    fn test_a_row_added_by_tabbing_says_so() {
        assert_eq!(
            parse_message(r#"{"kind":"row"}"#),
            Some(EditorMessage::TableRowAdded)
        );
    }

    #[test]
    fn test_turning_the_marking_off_turns_it_off_in_the_page() {
        // The defect this exists to stop is the one two spell-check checkboxes
        // already had here: ticked, saved, and never read back by anything.
        let on = editor_document(&blank(), "en-US", true);
        let off = editor_document(&blank(), "en-US", false);
        assert!(on.contains(r#"spellcheck="true""#), "{on}");
        assert!(off.contains(r#"spellcheck="false""#));
    }

    #[test]
    fn test_the_sound_at_the_end_of_a_word_goes_with_the_marking() {
        // One setting, both halves. Two would let somebody end up with the
        // sound on and the marking off, which is a noise with no explanation.
        let off = editor_document(&blank(), "en-US", false);
        assert!(!off.contains("wordFinished(at);"), "{off}");
        let on = editor_document(&blank(), "en-US", true);
        assert!(on.contains("wordFinished(at);"));
        // An apostrophe is inside a word rather than after it. Without this
        // every contraction sounds wrong halfway through being typed: "don" is
        // checked the moment the apostrophe lands, and "don" is not a word.
        assert!(on.contains("'’"), "the apostrophe ends a word: {on}");
    }

    #[test]
    fn test_a_finished_word_comes_back_to_be_checked() {
        assert_eq!(
            parse_message(r#"{"kind":"word","text":"wrold"}"#),
            Some(EditorMessage::WordFinished("wrold".to_string()))
        );
    }

    #[test]
    fn test_an_editor_that_did_not_answer_is_not_an_empty_message() {
        // The distinction the old code threw away. An empty answer is a
        // message with nothing in it; no answer is a script that failed, and
        // treating the second as the first overwrote the draft with nothing.
        assert_eq!(message_from_editor(None, Some("\"text\"".into())), None);
        assert_eq!(message_from_editor(Some("\"<p>x</p>\"".into()), None), None);
        assert_eq!(message_from_editor(None, None), None);
    }

    #[test]
    fn test_an_empty_message_is_still_a_message() {
        // The other side of it. Somebody who has cleared the body and saved
        // should get an empty draft, not a refusal.
        assert_eq!(
            message_from_editor(Some(r#""""#.into()), Some(r#""""#.into())),
            Some((String::new(), String::new()))
        );
    }

    #[test]
    fn test_the_text_comes_back_with_its_blocks() {
        assert_eq!(
            text_from_editor(r#"[{"text":"one","block":0},{"text":"two","block":1}]"#),
            vec![
                TextNode {
                    text: "one".to_string(),
                    block: 0
                },
                TextNode {
                    text: "two".to_string(),
                    block: 1
                },
            ]
        );
    }

    #[test]
    fn test_the_text_survives_a_backend_that_wraps_its_answer() {
        // Some backends hand a script result back as a JSON string containing
        // the JSON, which is the same trap the body and the plain text hit.
        assert_eq!(
            text_from_editor(r#""[{\"text\":\"one\",\"block\":0}]""#),
            vec![TextNode {
                text: "one".to_string(),
                block: 0
            }]
        );
    }

    #[test]
    fn test_an_answer_that_is_not_the_text_is_no_text() {
        // Rather than a panic in the middle of checking somebody's message.
        assert!(text_from_editor("null").is_empty());
        assert!(text_from_editor("").is_empty());
    }

    #[test]
    fn test_where_a_replacement_left_the_caret_comes_back() {
        assert_eq!(
            position_from_editor(r#"{"node":2,"offset":7}"#),
            Some(Position { node: 2, offset: 7 })
        );
    }

    #[test]
    fn test_a_page_that_cannot_say_where_the_caret_went_says_nothing() {
        // Which stops the pass rather than letting it carry on from a guess.
        // Carrying on from the wrong place corrects a word nobody was asked
        // about, and that is worse than a check that ends early and says so.
        assert_eq!(position_from_editor("null"), None);
        assert_eq!(position_from_editor(""), None);
        assert_eq!(position_from_editor(r#"{"node":1}"#), None);
        assert_eq!(position_from_editor("true"), None);
    }

    #[test]
    fn test_the_page_can_be_asked_for_its_text_and_told_what_to_select() {
        let page = editor_document(&blank(), "en-US", true);
        for entry in ["wixenText", "wixenSelectWord", "wixenReplaceWord"] {
            assert!(page.contains(entry), "the page has no {entry}");
        }
        assert!(text_script().contains("wixenText"));

        let at = Position { node: 1, offset: 4 };
        let select = select_word_script(at, 9);
        assert!(select.contains("1, 4, 9"), "{select}");
        let replace = replace_word_script(at, 9, "world");
        assert!(replace.contains("1, 4, 9"), "{replace}");
        assert!(replace.contains("world"), "{replace}");
    }

    #[test]
    fn test_the_page_no_longer_decides_where_the_words_are() {
        // The rule was a run of letter and mark characters, which made "3rd"
        // the word "rd" and a Japanese paragraph one word. Enumerating lives
        // in `application::words` now, where it can be executed.
        //
        // The keystroke check keeps a character class, and should: it asks
        // "did that character end a word" on every key press, which is one
        // comparison and cannot wait for a round trip. It decides when to
        // sound a tone, never where a word is or what gets replaced.
        let page = editor_document(&blank(), "en-US", true);

        assert!(!page.contains("wixenWords"), "the enumerator is back");
        assert!(page.contains("wixenText"), "nothing sends the text out");
    }

    #[test]
    fn test_a_replacement_cannot_smuggle_script_into_the_page() {
        // It comes from a dictionary rather than a stranger, and it is still
        // put into a script by string, which is the shape of the mistake that
        // does not announce itself.
        let script = replace_word_script(Position { node: 0, offset: 0 }, 5, "\"); alert(1); //");
        assert!(
            script.contains(r#"\""#),
            "the quote is not escaped: {script}"
        );
        assert_eq!(script.matches("wixenReplaceWord").count(), 1, "{script}");
    }

    #[test]
    fn test_the_headings_and_lists_are_offered() {
        // These are the ones that decide whether the person receiving the
        // message can navigate it, so their absence would be the defect.
        for format in [
            Format::Heading1,
            Format::Heading2,
            Format::Heading3,
            Format::BulletList,
            Format::NumberedList,
        ] {
            assert!(Format::ALL.contains(&format), "{format:?} is not offered");
        }
    }

    #[test]
    fn test_every_formatting_script_puts_the_caret_back_in_the_body() {
        // Half of what has to happen, and worth being exact about which half.
        //
        // This asserts the script moves the DOM active element. It cannot
        // assert that the keyboard came back, because when the command was run
        // from a toolbar button the Win32 focus is on the button and no amount
        // of JavaScript inside the page moves it. That is the caller's job and
        // is done by `wx_compose::run_in_editor`; the earlier version of this
        // test read as though it covered both, and it never did.
        //
        // Every command rather than Bold alone: one of thirteen forgetting is
        // exactly the shape this would take.
        for format in Format::ALL {
            let script = format_script(format);
            assert!(
                script.contains("focus()"),
                "{format:?} leaves the caret wherever the command put it: {script}"
            );
        }
    }
}
