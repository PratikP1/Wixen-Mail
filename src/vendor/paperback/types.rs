// Vendored from Paperback (https://github.com/trypsynth/paperback).
//
// MIT License
//
// Copyright (c) 2025-2026 Quin Gillespie
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Local changes: import paths point at this module rather than paperback-core;
// the `t!` translation macro routes through Wixen Mail's own i18n registry;
// parsers and types this application does not //! The positional metadata the converter records alongside its text.
//!
//! This is what makes a plain text body navigable: every heading, link, list
//! and table comes back with the offset it sits at, so a reader can jump by
//! structure instead of reading from the top.

#[derive(Debug, Clone)]
pub struct HeadingInfo {
    pub offset: usize,
    pub level: i32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct LinkInfo {
    pub offset: usize,
    pub text: String,
    pub reference: String,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub offset: usize,
    pub alt_text: String,
}

#[derive(Debug, Clone)]
pub struct ListInfo {
    pub offset: usize,
    pub item_count: i32,
    /// Display-unit span of the list, from its start offset to the end of its content.
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct ListItemInfo {
    pub offset: usize,
    pub level: i32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub offset: usize,
    pub text: String,
    pub html_content: String,
    /// Display-unit span of the emitted table text.
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct SeparatorInfo {
    pub offset: usize,
    pub length: usize,
}

/// What a converter exposes about the structure it found.
///
/// Taken from upstream's `parser` module, which this application does not
/// otherwise use. Keeping the trait means the converter can be swapped for
/// another one later without the reader knowing.
pub trait ConverterOutput {
    fn get_headings(&self) -> &[HeadingInfo];
    fn get_links(&self) -> &[LinkInfo];
    fn get_images(&self) -> &[ImageInfo];
    fn get_figures(&self) -> &[ImageInfo];
    fn get_tables(&self) -> &[TableInfo];
    fn get_separators(&self) -> &[SeparatorInfo];
    fn get_lists(&self) -> &[ListInfo];
    fn get_list_items(&self) -> &[ListItemInfo];
    fn get_bolds(&self) -> &[FormatInfo];
    fn get_italics(&self) -> &[FormatInfo];
    fn get_underlines(&self) -> &[FormatInfo];
}
