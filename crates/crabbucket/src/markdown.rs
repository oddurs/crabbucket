// Copyright (C) 2026 Oddur Sigurdsson
//
// This file is part of crabbucket.
//
// crabbucket is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// crabbucket is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Markdown rendering.
//!
//! The directive syntax described in `doc/DESIGN` -- `:::callout{…}` resolving
//! to a component function -- is not implemented yet.  Until it is, this is
//! CommonMark with the table, footnote and strikethrough extensions, which is
//! what a documentation page needs on the first day.

use pulldown_cmark::{Options, Parser, html};

/// Renders Markdown to a fragment of HTML.
pub fn to_html(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(source, options);
    let mut out = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::to_html;

    #[test]
    fn headings_and_emphasis_render() {
        assert_eq!(to_html("# Hi\n"), "<h1>Hi</h1>\n");
        assert!(to_html("*loud*").contains("<em>loud</em>"));
    }

    #[test]
    fn tables_are_enabled() {
        let html = to_html("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<table>"), "got {html}");
    }
}
