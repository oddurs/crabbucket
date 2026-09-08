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
//!
//! A page is first split into Markdown and [directives][crate::directive], and
//! each run of Markdown is parsed on its own.  One consequence is worth
//! knowing: a link reference definition or a footnote is visible only within
//! the run that defines it, because CommonMark scopes both to a document and
//! each run is a document.  Directives are for components rather than for
//! prose, so this has not bitten yet; if it does, the fix is to hoist
//! definitions across runs before parsing rather than to merge the runs.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use maud::PreEscaped;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::directive::{Block, Context, Directives, Fault, scan};

/// The prefix on every class emitted by the highlighter.
const CLASS_PREFIX: &str = "tok-";

/// The fence flag that keeps a block out of the search index.
///
/// Inline code stays indexed regardless, so that `serde::Deserialize` remains
/// findable.  This is for blocks whose content is not made of words anybody
/// would search for: a terminal capture, a diagram, ASCII art.
const NO_SEARCH: &str = "no-search";

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

/// Renders a page body: Markdown, directives, and the headings they contain.
///
/// # Errors
///
/// Fails if a directive is unterminated, unknown to `directives`, or carries
/// attributes that do not fit the type it was registered with.
pub fn render(source: &str, directives: &Directives, context: &Context<'_>) -> Result<Body, Fault> {
    let blocks = scan(source)?;
    let mut pass = Pass {
        directives,
        context,
        headings: Vec::new(),
        seen: BTreeMap::new(),
    };
    let html = pass.blocks(&blocks)?;

    Ok(Body {
        html,
        headings: pass.headings,
    })
}

/// One rendering pass over a page.
///
/// Heading ids have to be unique across the whole page, not within one run of
/// Markdown, so the `seen` counter lives here rather than in the function that
/// assigns them.
struct Pass<'a> {
    directives: &'a Directives,
    context: &'a Context<'a>,
    headings: Vec<Heading>,
    seen: BTreeMap<String, usize>,
}

impl Pass<'_> {
    fn blocks(&mut self, blocks: &[Block]) -> Result<String, Fault> {
        let mut out = String::new();

        for block in blocks {
            match block {
                Block::Text { source, .. } => out.push_str(&self.markdown(source)),
                Block::Directive {
                    line,
                    name,
                    attrs,
                    body,
                } => {
                    let inner = PreEscaped(self.blocks(body)?);
                    let rendered =
                        self.directives
                            .render(name, attrs, inner, *line, self.context)?;
                    out.push_str(&rendered.into_string());
                }
            }
        }

        Ok(out)
    }

    fn markdown(&mut self, source: &str) -> String {
        let mut events: Vec<Event<'_>> = Parser::new_ext(source, options()).collect();

        self.headings
            .extend(anchor_headings(&mut events, &mut self.seen));
        highlight_code(&mut events);

        let mut html = String::with_capacity(source.len() * 3 / 2);
        html::push_html(&mut html, events.into_iter());
        html
    }
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
fn anchor_headings(
    events: &mut Vec<Event<'_>>,
    seen: &mut BTreeMap<String, usize>,
) -> Vec<Heading> {
    let mut headings = Vec::new();
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

                let id = explicit.unwrap_or_else(|| unique(&slug(&text), seen));

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
    let mut open: Option<(usize, Info)> = None;
    let mut code = String::new();

    for index in 0..events.len() {
        match &events[index] {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                open = Some((index, Info::read(info)));
                code.clear();
            }
            Event::Text(chunk) if open.is_some() => code.push_str(chunk),
            Event::End(TagEnd::CodeBlock) => {
                if let Some((start, info)) = open.take()
                    && let Some(html) = fence(&code, &info)
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

/// A fence's info string: a language, and any flags after it.
#[derive(Debug, Default, PartialEq, Eq)]
struct Info {
    language: String,
    excluded: bool,
}

impl Info {
    fn read(info: &str) -> Self {
        let mut parsed = Info::default();

        for word in info.split_whitespace() {
            if word == NO_SEARCH {
                parsed.excluded = true;
            } else if parsed.language.is_empty() {
                parsed.language = word.to_string();
            }
        }

        parsed
    }

    /// The attributes this fence's `<pre>` carries.
    fn attributes(&self) -> String {
        let mut out = String::new();

        if !self.language.is_empty() {
            out.push_str(&format!(" data-language=\"{}\"", escape(&self.language)));
        }

        if self.excluded {
            out.push_str(" data-search=\"off\"");
        }

        out
    }
}

/// Renders one fenced block, or returns `None` to leave it to the default
/// rendering.
///
/// A block is taken over when there is something to add: highlighting, or an
/// attribute that changes what the build does with it.
fn fence(code: &str, info: &Info) -> Option<String> {
    let highlighted = highlight(code, &info.language);

    if highlighted.is_none() && !info.excluded {
        return None;
    }

    let body = highlighted.unwrap_or_else(|| escape_text(code));

    Some(format!(
        "<pre class=\"code\"{}><code>{body}</code></pre>",
        info.attributes()
    ))
}

/// Highlights one block's contents, or returns `None` if the language is not
/// known.
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

    Some(generator.finalize())
}

/// Escapes text going into an element rather than an attribute.
fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes the few characters that matter inside an attribute value.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::{Body, render, slug};
    use crate::directive::Directives;

    /// Renders with no directives registered, which is what most of these
    /// tests are about.
    fn plain(source: &str) -> Body {
        let config = crate::config::Config::blank();
        let data = crate::directive::Data::default();
        let context = crate::directive::Context::new(&config, &data);

        render(source, &Directives::new(), &context).expect("no directives, so nothing to fail")
    }

    fn to_html(source: &str) -> String {
        plain(source).html
    }

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
        let out = plain("## Getting started\n\n### Install\n");
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
        let out = plain("## Getting started {#start}\n");
        assert_eq!(out.headings[0].id, "start");
        assert!(out.html.contains("<h2 id=\"start\">"));
    }

    #[test]
    fn repeated_headings_get_distinct_ids() {
        let out = plain("## Notes\n\n## Notes\n\n## Notes\n");
        let ids: Vec<&str> = out.headings.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, ["notes", "notes-2", "notes-3"]);
    }

    #[test]
    fn heading_text_survives_inline_markup() {
        let out = plain("## What `Url` *does*\n");
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
    fn a_fence_can_take_itself_out_of_the_search_index() {
        let html = to_html("```console no-search\nnot words\n```\n");

        assert!(html.contains("data-search=\"off\""), "got {html}");
        assert!(
            html.contains("data-language=\"console\""),
            "the language survives: {html}"
        );
    }

    #[test]
    fn the_flag_works_without_a_language_too() {
        let html = to_html("``` no-search\nnot words\n```\n");

        assert!(html.contains("data-search=\"off\""), "got {html}");
        assert!(
            !html.contains("data-language"),
            "no-search is not a language: {html}"
        );
        assert!(
            html.contains("not words"),
            "the content is still rendered: {html}"
        );
    }

    #[test]
    fn an_excluded_block_still_escapes_its_content() {
        let html = to_html("``` no-search\n<script>&\n```\n");

        assert!(html.contains("&lt;script&gt;&amp;"), "got {html}");
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
        let out = plain("## One\n\n## Two\n");
        assert!(out.has_anchor("one"));
        assert!(!out.has_anchor("three"));
    }
}
