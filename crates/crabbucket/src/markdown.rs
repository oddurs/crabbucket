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
//! Two things happen here beyond turning CommonMark into HTML, and both exist
//! so that something downstream can be checked rather than hoped for.
//!
//! Headings get stable ids, and the ids come back out alongside the HTML, so
//! the table of contents and the fragment checker both read the same list
//! rather than parsing the rendered HTML back into one.
//!
//! Code fences are highlighted here, at build time, into `<span>`s carrying
//! scope classes.  The colours are the theme's business; the classes are this
//! module's.  Nothing is shipped to the browser to make code coloured, because
//! the text was already static when it was written.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// The prefix on every class emitted by the highlighter.
const CLASS_PREFIX: &str = "tok-";

/// One heading, as the table of contents and the link checker need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// The id, which is what a fragment link points at.
    pub id: String,
    /// The heading level: 1 for `h1`, 6 for `h6`.
    pub level: u8,
    /// The heading text, with any inline markup flattened away.
    pub text: String,
}

/// A rendered page body, and everything derived from rendering it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Body {
    /// The HTML.
    pub html: String,
    /// Every heading, in document order.
    pub headings: Vec<Heading>,
}

impl Body {
    /// Whether the body contains a heading with this id.
    pub fn has_anchor(&self, id: &str) -> bool {
        self.headings.iter().any(|heading| heading.id == id)
    }
}

/// Renders Markdown to a fragment of HTML.
pub fn render(source: &str) -> Body {
    let parser = Parser::new_ext(source, options());
    let mut events: Vec<Event<'_>> = parser.collect();

    let headings = anchor_headings(&mut events);
    highlight_code(&mut events);

    let mut html = String::with_capacity(source.len() * 3 / 2);
    html::push_html(&mut html, events.into_iter());

    Body { html, headings }
}

/// Renders Markdown to a fragment of HTML, discarding what was derived.
pub fn to_html(source: &str) -> String {
    render(source).html
}

fn options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options
}

/// Gives every heading an id and a permalink, and collects them.
///
/// An explicit `{#id}` written in the source wins over the derived slug, since
/// someone who wrote one down is holding a link to it.
#[allow(
    clippy::needless_range_loop,
    reason = "the loop rewrites events by index"
)]
fn anchor_headings(events: &mut Vec<Event<'_>>) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut inserts: Vec<(usize, Event<'_>)> = Vec::new();
    let mut open: Option<(usize, u8, Option<String>)> = None;
    let mut text = String::new();

    for index in 0..events.len() {
        match &events[index] {
            Event::Start(Tag::Heading { level, id, .. }) => {
                open = Some((
                    index,
                    level_of(*level),
                    id.as_ref().map(|id| id.to_string()),
                ));
                text.clear();
            }
            Event::Text(chunk) | Event::Code(chunk) if open.is_some() => {
                text.push_str(chunk);
            }
            Event::End(TagEnd::Heading(_)) => {
                let Some((start, level, explicit)) = open.take() else {
                    continue;
                };

                let id = explicit.unwrap_or_else(|| unique(&slug(&text), &mut seen));

                // Rewrite the opening tag so the id is on the element itself,
                // then put the permalink at the end of the heading's content.
                if let Event::Start(Tag::Heading { id: slot, .. }) = &mut events[start] {
                    *slot = Some(id.clone().into());
                }

                inserts.push((index, Event::Html(permalink(&id).into())));
                headings.push(Heading {
                    id,
                    level,
                    text: text.trim().to_string(),
                });
                text.clear();
            }
            _ => {}
        }
    }

    // Late to early, so earlier indices stay valid.
    for (index, event) in inserts.into_iter().rev() {
        events.insert(index, event);
    }

    headings
}

/// The anchor that makes a heading linkable.
///
/// It is in the document rather than added by script, and it is a real link
/// with a label, so it is reachable by keyboard and announced by a screen
/// reader.  Hiding it until hover is the stylesheet's business.
fn permalink(id: &str) -> String {
    format!("<a class=\"heading-anchor\" href=\"#{id}\" aria-label=\"Link to this section\">#</a>")
}

/// Turns heading text into a slug.
fn slug(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut hyphen = false;

    for character in text.trim().chars() {
        if character.is_alphanumeric() {
            out.extend(character.to_lowercase());
            hyphen = false;
        } else if !out.is_empty() && !hyphen {
            out.push('-');
            hyphen = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}

/// Disambiguates a slug that has already been used on this page.
fn unique(slug: &str, seen: &mut BTreeMap<String, usize>) -> String {
    let count = seen.entry(slug.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        slug.to_string()
    } else {
        format!("{slug}-{}", *count)
    }
}

fn level_of(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// The syntax set, loaded once per process rather than once per fence.
fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Replaces every fenced code block whose language is known with highlighted
/// markup.
///
/// A fence with no language, or with one nothing is known about, is left
/// exactly as it was: unhighlighted code is a fine outcome, and a build that
/// fails over a language tag would be an absurd one.
#[allow(
    clippy::needless_range_loop,
    reason = "the loop rewrites events by index"
)]
fn highlight_code(events: &mut Vec<Event<'_>>) {
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();
    let mut open: Option<(usize, String)> = None;
    let mut code = String::new();

    for index in 0..events.len() {
        match &events[index] {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) => {
                // The info string may carry more than a language.
                let language = language.split_whitespace().next().unwrap_or("").to_string();
                open = Some((index, language));
                code.clear();
            }
            Event::Text(chunk) if open.is_some() => code.push_str(chunk),
            Event::End(TagEnd::CodeBlock) => {
                if let Some((start, language)) = open.take()
                    && let Some(html) = highlight(&code, &language)
                {
                    replacements.push((start, index, html));
                }
                code.clear();
            }
            _ => {}
        }
    }

    for (start, end, html) in replacements.into_iter().rev() {
        events.splice(start..=end, [Event::Html(html.into())]);
    }
}

/// Highlights one block, or returns `None` if the language is not known.
fn highlight(code: &str, language: &str) -> Option<String> {
    if language.is_empty() {
        return None;
    }

    let syntaxes = syntaxes();
    let syntax = syntaxes
        .find_syntax_by_token(language)
        .or_else(|| syntaxes.find_syntax_by_extension(language))?;

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        syntaxes,
        ClassStyle::SpacedPrefixed {
            prefix: CLASS_PREFIX,
        },
    );

    for line in LinesWithEndings::from(code) {
        generator
            .parse_html_for_line_which_includes_newline(line)
            .ok()?;
    }

    Some(format!(
        "<pre class=\"code\" data-language=\"{}\"><code>{}</code></pre>",
        escape(language),
        generator.finalize()
    ))
}

/// Escapes the few characters that matter inside an attribute value.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{render, slug, to_html};

    #[test]
    fn headings_and_emphasis_render() {
        assert!(to_html("# Hi\n").contains("<h1 id=\"hi\">Hi"));
        assert!(to_html("*loud*").contains("<em>loud</em>"));
    }

    #[test]
    fn tables_are_enabled() {
        let html = to_html("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<table>"), "got {html}");
    }

    #[test]
    fn slugs_are_what_you_would_have_typed() {
        assert_eq!(slug("Getting started"), "getting-started");
        assert_eq!(slug("What `Url` does"), "what-url-does");
        assert_eq!(slug("Why not .astro files?"), "why-not-astro-files");
        assert_eq!(slug("  spaced  out  "), "spaced-out");
        assert_eq!(slug("!!!"), "section");
    }

    #[test]
    fn every_heading_gets_an_id_and_a_permalink() {
        let out = render("## Getting started\n\n### Install\n");
        assert_eq!(out.headings.len(), 2);
        assert_eq!(out.headings[0].id, "getting-started");
        assert_eq!(out.headings[0].level, 2);
        assert_eq!(out.headings[1].id, "install");
        assert!(out.html.contains("<h2 id=\"getting-started\">"));
        assert!(out.html.contains("href=\"#getting-started\""));
        assert!(out.html.contains("aria-label=\"Link to this section\""));
    }

    #[test]
    fn an_explicit_id_wins() {
        let out = render("## Getting started {#start}\n");
        assert_eq!(out.headings[0].id, "start");
        assert!(out.html.contains("<h2 id=\"start\">"));
    }

    #[test]
    fn repeated_headings_get_distinct_ids() {
        let out = render("## Notes\n\n## Notes\n\n## Notes\n");
        let ids: Vec<&str> = out.headings.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, ["notes", "notes-2", "notes-3"]);
    }

    #[test]
    fn heading_text_survives_inline_markup() {
        let out = render("## What `Url` *does*\n");
        assert_eq!(out.headings[0].text, "What Url does");
        assert_eq!(out.headings[0].id, "what-url-does");
    }

    #[test]
    fn a_known_language_is_highlighted() {
        let html = to_html("```rust\nfn main() {}\n```\n");
        assert!(html.contains("data-language=\"rust\""), "got {html}");
        assert!(html.contains("tok-"), "no scope classes in {html}");
    }

    #[test]
    fn an_unknown_or_absent_language_is_left_alone() {
        for source in ["```\nplain\n```\n", "```wingdings\nplain\n```\n"] {
            let html = to_html(source);
            assert!(html.contains("<pre><code"), "got {html}");
            assert!(!html.contains("tok-"), "unexpectedly highlighted: {html}");
        }
    }

    #[test]
    fn highlighting_does_not_disturb_the_prose_around_it() {
        let html = to_html("Before\n\n```rust\nlet x = 1;\n```\n\nAfter\n");
        assert!(html.contains("<p>Before</p>"));
        assert!(html.contains("<p>After</p>"));
    }

    #[test]
    fn has_anchor_answers_the_fragment_checkers_question() {
        let out = render("## One\n\n## Two\n");
        assert!(out.has_anchor("one"));
        assert!(!out.has_anchor("three"));
    }
}
