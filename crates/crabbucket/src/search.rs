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
//! The search index.
//!
//! For a corpus of tens of pages, an inverted index is the wrong shape: a
//! linear scan over a few tens of kilobytes of JSON is genuinely faster than
//! parsing an index would be, and it is a tenth of the code.  So the index is
//! just the pages, and the client reads them.
//!
//! That trade has a size beyond which it stops being right.  The build says so
//! rather than quietly shipping a slow page -- see [`SIZE_WARNING`].

use std::fmt::Write as _;

use crate::markdown::Heading;

/// The size past which this design is the wrong one.
///
/// Well under a second to fetch and scan on a slow connection is the bar.  Past
/// this, the answer is a real index, not a bigger download; the build says so
/// rather than deciding on the reader's behalf.
pub const SIZE_WARNING: usize = 300 * 1024;

/// Says so if an index has outgrown this design.
///
/// A separate function because the alternative is a fixture site large enough
/// to trip the threshold, which would be three hundred kilobytes of test data
/// to check one comparison.
pub fn outgrown(size: usize) -> Option<String> {
    (size > SIZE_WARNING).then(|| {
        format!(
            "the search index is {}KB, past the {}KB this design suits; \
             consider a real index rather than a bigger download",
            size / 1024,
            SIZE_WARNING / 1024
        )
    })
}

/// One page, as the search client sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document<'a> {
    /// Where the page is served.
    pub url: &'a str,
    /// Its title.
    pub title: &'a str,
    /// Its headings, which rank above body text.
    pub headings: &'a [Heading],
    /// Its body, with the markup taken out.
    pub text: String,
}

/// Builds the index.
///
/// The JSON is written by hand rather than by a serializer: it is three string
/// fields and an array, the escaping is fifteen lines and tested, and a
/// dependency to avoid writing them would be a poor trade.
pub fn index(documents: &[Document<'_>]) -> String {
    let mut out = String::from("[");

    for (at, document) in documents.iter().enumerate() {
        if at > 0 {
            out.push(',');
        }

        let _ = write!(
            out,
            "{{\"u\":{},\"t\":{},\"h\":[",
            quote(document.url),
            quote(document.title)
        );

        for (at, heading) in document.headings.iter().enumerate() {
            if at > 0 {
                out.push(',');
            }
            let _ = write!(out, "[{},{}]", quote(&heading.text), quote(&heading.id));
        }

        let _ = write!(out, "],\"b\":{}}}", quote(&document.text));
    }

    out.push(']');
    out
}

/// A JSON string literal.
///
/// Only `"`, `\` and the C0 control characters have to be escaped; everything
/// else may be UTF-8 as it stands, which is what the specification says and
/// what every parser has done for twenty years.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');

    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }

    out.push('"');
    out
}

/// The class on the permalink beside every heading.
///
/// Its text is a `#`, which is not a word anybody searches for, so it is
/// dropped along with the markup.
const PERMALINK: &str = "heading-anchor";

/// Tags that do not separate words.
///
/// This matters more than it looks.  Syntax highlighting wraps every token in
/// a `span`, so treating every tag as a word boundary turns
/// `serde::Deserialize` into three words and makes it unfindable.  A block
/// tag does separate words: `<p>a</p><p>b</p>` is two of them.
const INLINE: &[&str] = &[
    "a", "abbr", "b", "code", "del", "em", "i", "ins", "kbd", "mark", "s", "samp", "small", "span",
    "strong", "sub", "sup", "u", "var",
];

/// Strips markup out of rendered HTML, leaving searchable words.
///
/// This runs over this crate's own output, where tags are well formed and
/// there is no scripting, so a scanner is enough and a parser would be a
/// dependency bought for nothing.
pub fn plain(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut rest = html;

    while let Some(at) = rest.find('<') {
        out.push_str(&decode(&rest[..at]));

        let Some(end) = rest[at..].find('>') else {
            return collapse(&out);
        };

        let tag = &rest[at..at + end + 1];
        rest = &rest[at + end + 1..];

        if !is_inline(tag) {
            out.push(' ');
        }

        // Skip the permalink's contents, not just its tags.
        if tag.contains(PERMALINK) {
            rest = match rest.find("</a>") {
                Some(close) => &rest[close + 4..],
                None => "",
            };
        }
    }

    out.push_str(&decode(rest));
    collapse(&out)
}

/// Whether a tag is one that does not separate words.
fn is_inline(tag: &str) -> bool {
    let name: String = tag
        .trim_start_matches(['<', '/'])
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();

    INLINE.contains(&name.as_str())
}

/// Collapses every run of whitespace to one space, and trims.
fn collapse(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for word in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }

    out
}

/// Decodes the entities `maud` and `pulldown-cmark` actually emit.
fn decode(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        // Last, so that `&amp;lt;` does not become `<`.
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::{Document, index, plain, quote};
    use crate::markdown::Heading;

    #[test]
    fn markup_comes_out_and_words_stay_in() {
        let html = "<h1 id=\"a\">Title</h1>\n<p>Some <em>emphasised</em> words.</p>";
        assert_eq!(plain(html), "Title Some emphasised words.");
    }

    #[test]
    fn entities_are_decoded_and_not_twice() {
        assert_eq!(plain("<p>a &lt; b &amp;&amp; c</p>"), "a < b && c");
        assert_eq!(plain("<p>&amp;lt;</p>"), "&lt;", "decoding ran twice");
    }

    #[test]
    fn inline_markup_does_not_break_a_word_in_two() {
        // Syntax highlighting wraps every token in a span.  Treating those as
        // word boundaries makes `serde::Deserialize` unfindable.
        let html = "<code><span class=\"tok-a\">serde</span>\
                    <span class=\"tok-b\">::</span>\
                    <span class=\"tok-c\">Deserialize</span></code>";

        assert_eq!(plain(html), "serde::Deserialize");
    }

    #[test]
    fn block_markup_does_separate_words() {
        assert_eq!(plain("<p>one</p><p>two</p>"), "one two");
        assert_eq!(plain("<li>a</li><li>b</li>"), "a b");
    }

    #[test]
    fn the_permalink_beside_a_heading_is_not_a_searchable_word() {
        let html =
            "<h2 id=\"a\">Title<a class=\"heading-anchor\" href=\"#a\">#</a></h2><p>body</p>";
        assert_eq!(plain(html), "Title body", "the permalink reached the index");
    }

    #[test]
    fn an_unclosed_tag_does_not_run_away_with_the_rest() {
        assert_eq!(plain("<p>before<"), "before");
    }

    #[test]
    fn whitespace_collapses_to_single_spaces() {
        assert_eq!(plain("<p>a\n\n  b\t\tc</p>"), "a b c");
    }

    #[test]
    fn quoting_covers_what_json_requires_and_leaves_the_rest_alone() {
        assert_eq!(quote("plain"), "\"plain\"");
        assert_eq!(quote("a \"quoted\" word"), "\"a \\\"quoted\\\" word\"");
        assert_eq!(quote("back\\slash"), "\"back\\\\slash\"");
        assert_eq!(quote("line\nbreak"), "\"line\\nbreak\"");
        assert_eq!(quote("bell\u{7}"), "\"bell\\u0007\"");
        assert_eq!(
            quote("crab 🦀 é"),
            "\"crab 🦀 é\"",
            "UTF-8 needs no escaping"
        );
    }

    #[test]
    fn the_index_is_the_shape_the_client_reads() {
        let headings = [Heading {
            id: "one".into(),
            level: 2,
            text: "One".into(),
        }];
        let documents = [Document {
            url: "/repo/docs/",
            title: "Docs",
            headings: &headings,
            text: "body words".into(),
        }];

        assert_eq!(
            index(&documents),
            r#"[{"u":"/repo/docs/","t":"Docs","h":[["One","one"]],"b":"body words"}]"#
        );
    }

    #[test]
    fn an_index_that_outgrows_this_design_says_so() {
        use super::{SIZE_WARNING, outgrown};

        assert!(
            outgrown(SIZE_WARNING).is_none(),
            "the threshold itself is fine"
        );
        assert!(outgrown(0).is_none());

        let complaint = outgrown(SIZE_WARNING + 1).expect("one byte over should warn");
        assert!(complaint.contains("300KB"), "got {complaint}");
        assert!(complaint.contains("a real index"), "got {complaint}");
    }

    #[test]
    fn an_empty_site_is_an_empty_array() {
        assert_eq!(index(&[]), "[]");
    }
}
