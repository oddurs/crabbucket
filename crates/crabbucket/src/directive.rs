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
//! Directives: the bridge from Markdown to typed components.
//!
//! ```text
//! :::callout{kind = "warn"}
//! crabbucket needs Rust 1.85 or newer.
//! :::
//! ```
//!
//! A theme registers a handler under a name, together with the type its
//! attributes deserialize into.  An attribute that does not fit that type, or
//! a name nothing is registered under, fails the build with the file and the
//! line -- which is the whole point, and the reason this exists instead of
//! letting people write raw HTML into Markdown.
//!
//! Attributes are a TOML inline table, so quoting, escaping and nesting are
//! TOML's problem rather than this module's, and the syntax is the one the
//! rest of the project already uses.
//!
//! The scanner works on lines rather than on `pulldown_cmark`'s event stream,
//! for one reason: a directive written *inside* a fenced code block must be
//! left alone, and these very docs do that.  Tracking fences over lines is a
//! dozen lines of code; recovering the original text from an event stream is
//! not.

use std::collections::BTreeMap;
use std::fmt;

use maud::Markup;
use serde::de::DeserializeOwned;

/// Something wrong with a directive, at a line within the page body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The line the directive opened on, counted from the start of the body.
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl Fault {
    fn at(line: usize, message: impl Into<String>) -> Self {
        Fault {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// A directive's attributes, before they are given a type.
pub type Attributes = toml::Table;

type Handler = Box<dyn Fn(&Attributes, Markup) -> Result<Markup, String> + Send + Sync>;

/// The directives a theme offers.
///
/// Registration is by name and by type together, so the type is not something
/// a handler has to remember to check:
///
/// ```
/// use crabbucket::directive::Directives;
/// use maud::html;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Aside {
///     title: String,
/// }
///
/// let mut directives = Directives::new();
/// directives.add("aside", |props: Aside, body| {
///     html! { aside { h4 { (props.title) } (body) } }
/// });
///
/// assert_eq!(directives.names(), ["aside"]);
/// ```
#[derive(Default)]
pub struct Directives {
    handlers: BTreeMap<String, Handler>,
}

impl fmt::Debug for Directives {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directives")
            .field("names", &self.names())
            .finish()
    }
}

impl Directives {
    /// No directives at all, which is what a theme gets if it says nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a handler, with the type its attributes deserialize into.
    ///
    /// `P` is deserialized from the attribute table before the handler runs,
    /// so a handler never sees an attribute it did not ask for and never has
    /// to check one it did.
    pub fn add<P, F>(&mut self, name: &str, render: F) -> &mut Self
    where
        P: DeserializeOwned,
        F: Fn(P, Markup) -> Markup + Send + Sync + 'static,
    {
        let handler: Handler = Box::new(move |attrs, body| {
            let props: P = toml::Value::Table(attrs.clone())
                .try_into()
                .map_err(|err: toml::de::Error| err.message().to_string())?;
            Ok(render(props, body))
        });

        self.handlers.insert(name.to_string(), handler);
        self
    }

    /// Every registered name, in order.
    pub fn names(&self) -> Vec<&str> {
        self.handlers.keys().map(String::as_str).collect()
    }

    /// Whether anything is registered.
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Renders one directive.
    pub(crate) fn render(
        &self,
        name: &str,
        attrs: &Attributes,
        body: Markup,
        line: usize,
    ) -> Result<Markup, Fault> {
        let Some(handler) = self.handlers.get(name) else {
            return Err(Fault::at(line, self.unknown(name)));
        };

        handler(attrs, body).map_err(|message| Fault::at(line, format!("`{name}`: {message}")))
    }

    fn unknown(&self, name: &str) -> String {
        if self.handlers.is_empty() {
            format!("unknown directive `{name}`; this design system has none")
        } else {
            format!(
                "unknown directive `{name}`, expected one of {}",
                self.names()
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

/// A run of the page: either Markdown, or a directive wrapping more of both.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Block {
    /// Markdown, verbatim.
    Text {
        /// The line it starts on, counted from the start of the body.
        line: usize,
        /// The Markdown.
        source: String,
    },
    /// A directive.
    Directive {
        /// The line the directive opened on.
        line: usize,
        /// Its name.
        name: String,
        /// Its attributes.
        attrs: Attributes,
        /// What it contains, which may be more directives.
        body: Vec<Block>,
    },
}

/// One open directive, and what has accumulated inside it.
struct Frame {
    open: Option<(usize, String, Attributes)>,
    blocks: Vec<Block>,
    text: Vec<String>,
    text_line: usize,
}

impl Frame {
    fn root() -> Self {
        Frame {
            open: None,
            blocks: Vec::new(),
            text: Vec::new(),
            text_line: 1,
        }
    }

    /// Turns whatever text has accumulated into a block.
    fn flush(&mut self) {
        if self.text.iter().any(|line| !line.trim().is_empty()) {
            self.blocks.push(Block::Text {
                line: self.text_line,
                source: self.text.join("\n"),
            });
        }
        self.text.clear();
    }

    fn push(&mut self, line: &str, number: usize) {
        if self.text.is_empty() {
            self.text_line = number;
        }
        self.text.push(line.to_string());
    }
}

/// Splits a page body into Markdown and directives.
///
/// # Errors
///
/// Fails if a directive is never closed, or if its attributes are not a TOML
/// inline table.
pub(crate) fn scan(source: &str) -> Result<Vec<Block>, Fault> {
    let mut stack = vec![Frame::root()];
    let mut fence: Option<(char, usize)> = None;

    for (index, line) in source.lines().enumerate() {
        let number = index + 1;

        if let Some(open) = fence {
            if closes_fence(line, open) {
                fence = None;
            }
            top(&mut stack).push(line, number);
            continue;
        }

        if let Some(open) = opens_fence(line) {
            fence = Some(open);
            top(&mut stack).push(line, number);
            continue;
        }

        match directive_line(line, number)? {
            Some(Marker::Open { name, attrs }) => {
                top(&mut stack).flush();
                stack.push(Frame {
                    open: Some((number, name, attrs)),
                    blocks: Vec::new(),
                    text: Vec::new(),
                    text_line: number + 1,
                });
            }
            Some(Marker::Close) if stack.len() > 1 => {
                let mut frame = stack.pop().expect("the root frame is never popped");
                frame.flush();

                let (line, name, attrs) = frame.open.expect("only the root frame has no opening");

                top(&mut stack).blocks.push(Block::Directive {
                    line,
                    name,
                    attrs,
                    body: frame.blocks,
                });
            }
            // A `:::` with nothing open is ordinary text, not an error.  It is
            // valid Markdown, and guessing that someone meant a directive is
            // how a tool starts refusing documents it should have rendered.
            _ => top(&mut stack).push(line, number),
        }
    }

    if stack.len() > 1 {
        let frame = stack.pop().expect("checked");
        let (line, name, _) = frame.open.expect("only the root frame has no opening");
        return Err(Fault::at(line, format!("unterminated directive `{name}`")));
    }

    let mut root = stack.pop().expect("the root frame is never popped");
    root.flush();
    Ok(root.blocks)
}

fn top(stack: &mut [Frame]) -> &mut Frame {
    stack.last_mut().expect("the root frame is never popped")
}

/// What a `:::` line means.
enum Marker {
    Open { name: String, attrs: Attributes },
    Close,
}

/// Reads a `:::` line, if the line is one.
fn directive_line(line: &str, number: usize) -> Result<Option<Marker>, Fault> {
    let Some(rest) = line.strip_prefix(":::") else {
        return Ok(None);
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return Ok(Some(Marker::Close));
    }

    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();

    if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Ok(None);
    }

    let after = rest[name.len()..].trim();

    let attrs = match after
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        Some(inner) => attributes(inner, number)?,
        None if after.is_empty() => Attributes::new(),
        None => return Ok(None),
    };

    Ok(Some(Marker::Open { name, attrs }))
}

/// Parses a directive's attributes as a TOML inline table.
fn attributes(inner: &str, line: usize) -> Result<Attributes, Fault> {
    if inner.trim().is_empty() {
        return Ok(Attributes::new());
    }

    let document: Attributes = format!("attrs = {{ {inner} }}")
        .parse()
        .map_err(|err: toml::de::Error| Fault::at(line, err.message().to_string()))?;

    match document.get("attrs") {
        Some(toml::Value::Table(table)) => Ok(table.clone()),
        _ => Err(Fault::at(line, "attributes must be a TOML inline table")),
    }
}

/// Whether a line opens a fenced code block, and with what.
fn opens_fence(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let width = trimmed.chars().take_while(|c| *c == marker).count();

    (width >= 3).then_some((marker, width))
}

/// Whether a line closes the fence that is open.
fn closes_fence(line: &str, (marker, width): (char, usize)) -> bool {
    let trimmed = line.trim_start();
    let found = trimmed.chars().take_while(|c| *c == marker).count();

    found >= width && trimmed[found..].trim().is_empty()
}

#[cfg(test)]
mod tests {
    use maud::html;
    use serde::Deserialize;

    use super::{Attributes, Block, Directives, scan};

    #[derive(Deserialize)]
    struct Props {
        kind: String,
    }

    fn directives() -> Directives {
        let mut directives = Directives::new();
        directives.add("callout", |props: Props, body| {
            html! { aside class=(props.kind) { (body) } }
        });
        directives
    }

    fn text(block: &Block) -> &str {
        match block {
            Block::Text { source, .. } => source,
            Block::Directive { name, .. } => panic!("expected text, got directive `{name}`"),
        }
    }

    #[test]
    fn a_page_with_no_directives_is_one_block() {
        let blocks = scan("# Hi\n\nBody.\n").unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(text(&blocks[0]), "# Hi\n\nBody.");
    }

    #[test]
    fn a_directive_becomes_its_own_block() {
        let blocks = scan("Before\n\n:::callout{kind = \"warn\"}\nInside\n:::\n\nAfter\n").unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(text(&blocks[0]).trim(), "Before");

        let Block::Directive {
            name,
            attrs,
            body,
            line,
        } = &blocks[1]
        else {
            panic!("expected a directive");
        };
        assert_eq!(name, "callout");
        assert_eq!(attrs.get("kind").and_then(|v| v.as_str()), Some("warn"));
        assert_eq!(*line, 3);
        assert_eq!(text(&body[0]).trim(), "Inside");

        assert_eq!(text(&blocks[2]).trim(), "After");
    }

    #[test]
    fn a_directive_inside_a_code_fence_is_left_alone() {
        let source = "```markdown\n:::callout{kind = \"warn\"}\nnot a directive\n:::\n```\n";
        let blocks = scan(source).unwrap();
        assert_eq!(blocks.len(), 1, "the fence was not respected: {blocks:?}");
        assert!(text(&blocks[0]).contains(":::callout"));
    }

    #[test]
    fn tildes_fence_too_and_a_longer_fence_is_needed_to_close_one() {
        let source = "~~~~\n```\n:::callout\n~~~~\n";
        let blocks = scan(source).unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn directives_nest() {
        let source = ":::callout{kind = \"note\"}\n:::callout{kind = \"warn\"}\ndeep\n:::\n:::\n";
        let blocks = scan(source).unwrap();

        let Block::Directive { body, .. } = &blocks[0] else {
            panic!("expected a directive")
        };
        let Block::Directive { name, .. } = &body[0] else {
            panic!("expected a nested one")
        };
        assert_eq!(name, "callout");
    }

    #[test]
    fn a_directive_with_no_attributes_needs_no_braces() {
        let blocks = scan(":::callout\nhi\n:::\n").unwrap();
        let Block::Directive { attrs, .. } = &blocks[0] else {
            panic!("expected a directive")
        };
        assert_eq!(attrs, &Attributes::new());
    }

    #[test]
    fn an_unterminated_directive_names_the_line_it_opened_on() {
        let fault = scan("Before\n\n:::callout\nhi\n").unwrap_err();
        assert_eq!(fault.line, 3);
        assert!(
            fault.message.contains("unterminated"),
            "got {}",
            fault.message
        );
    }

    #[test]
    fn a_stray_close_is_ordinary_text() {
        let blocks = scan("Before\n\n:::\n\nAfter\n").unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(text(&blocks[0]).contains(":::"));
    }

    #[test]
    fn something_that_is_not_a_directive_is_left_as_text() {
        for source in [
            ":::1bad\nx\n:::\n",
            "::: not a name\n",
            ":::callout trailing\n",
        ] {
            let blocks = scan(source).unwrap();
            assert!(
                matches!(blocks.first(), Some(Block::Text { .. })),
                "{source:?} was taken for a directive"
            );
        }
    }

    #[test]
    fn bad_attributes_fail_at_the_directives_line() {
        let fault = scan("\n\n:::callout{kind = }\nx\n:::\n").unwrap_err();
        assert_eq!(fault.line, 3);
    }

    #[test]
    fn a_registered_directive_gets_its_props_typed() {
        let out = directives()
            .render("callout", &attrs("warn"), html! { p { "hi" } }, 1)
            .unwrap()
            .into_string();

        assert_eq!(out, "<aside class=\"warn\"><p>hi</p></aside>");
    }

    #[test]
    fn a_missing_attribute_is_reported_as_serde_sees_it() {
        let fault = directives()
            .render("callout", &Attributes::new(), html! {}, 7)
            .unwrap_err();

        assert_eq!(fault.line, 7);
        assert!(
            fault.message.contains("missing field `kind`"),
            "got {}",
            fault.message
        );
    }

    #[test]
    fn an_unknown_directive_lists_the_ones_that_exist() {
        let fault = directives()
            .render("callou", &Attributes::new(), html! {}, 3)
            .unwrap_err();
        assert!(fault.message.contains("`callou`"), "got {}", fault.message);
        assert!(
            fault.message.contains("expected one of `callout`"),
            "got {}",
            fault.message
        );
    }

    #[test]
    fn a_theme_with_no_directives_says_so() {
        let fault = Directives::new()
            .render("callout", &Attributes::new(), html! {}, 1)
            .unwrap_err();
        assert!(fault.message.contains("has none"), "got {}", fault.message);
    }

    fn attrs(kind: &str) -> Attributes {
        let mut attrs = Attributes::new();
        attrs.insert("kind".into(), toml::Value::String(kind.into()));
        attrs
    }
}
