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
//! Build failures, and how they are shown.
//!
//! Two rules shape this module.  Every variant carries the path it applies to,
//! so an error that does not name its file cannot be constructed.  And an error
//! that knows a line shows that line, with a caret under the part that is
//! wrong, because rustc taught everyone what a good diagnostic looks like and
//! it is not a sentence about a line number.

use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

use crate::links::DeadLink;

/// A piece of a source file, pointed at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    /// The line, 1-based and absolute within the file the reader will open --
    /// not relative to the frontmatter block it came from.
    pub line: usize,
    /// The column, 1-based.
    pub column: usize,
    /// The text of that line, without its newline.
    pub text: String,
    /// How many columns the caret should span.
    pub width: usize,
}

impl Snippet {
    /// Locates `span` within `source`.
    ///
    /// `line_offset` is how many lines of the file come before `source`, which
    /// for frontmatter is the opening `+++`.  Getting this wrong is the bug
    /// every frontmatter parser ships with, so it is a parameter rather than an
    /// assumption.
    pub fn new(source: &str, span: Range<usize>, line_offset: usize) -> Self {
        let start = span.start.min(source.len());
        let before = &source[..start];
        let line_index = before.matches('\n').count();
        let line_start = before.rfind('\n').map_or(0, |at| at + 1);

        let text = source[line_start..]
            .split('\n')
            .next()
            .unwrap_or("")
            .trim_end_matches('\r');

        let column = start - line_start;
        let width = span
            .len()
            .max(1)
            .min(text.len().saturating_sub(column).max(1));

        Snippet {
            line: line_index + 1 + line_offset,
            column: column + 1,
            text: text.to_string(),
            width,
        }
    }

    /// Points at a whole line, for a failure that knows a line but not a span.
    pub fn at_line(source: &str, line: usize, line_offset: usize) -> Self {
        let text = source.lines().nth(line.saturating_sub(1)).unwrap_or("");

        Snippet {
            line: line + line_offset,
            column: 1,
            text: text.to_string(),
            width: text.len().max(1),
        }
    }

    fn write(&self, out: &mut String, color: bool) {
        let gutter = " ".repeat(self.line.to_string().len());
        let caret = "^".repeat(self.width);
        let (red, dim, off) = palette(color);

        out.push_str(&format!("\n{dim}{gutter} |{off}"));
        out.push_str(&format!("\n{dim}{} |{off} {}", self.line, self.text));
        out.push_str(&format!(
            "\n{dim}{gutter} |{off} {}{red}{caret}{off}",
            " ".repeat(self.column.saturating_sub(1))
        ));
    }
}

/// A build failure, always attributed to a file.
#[derive(Debug)]
pub enum Error {
    /// The file could not be read or written.
    Io {
        /// The file the operation was on.
        path: PathBuf,
        /// The underlying failure.
        source: std::io::Error,
    },

    /// A content file did not begin with a `+++` frontmatter block.
    MissingFrontmatter {
        /// The offending content file.
        path: PathBuf,
    },

    /// A content file opened a frontmatter block but never closed it.
    UnterminatedFrontmatter {
        /// The offending content file.
        path: PathBuf,
    },

    /// Frontmatter, or configuration, that did not fit its type.  This is the
    /// error a missing `title` produces, and the one a layout the theme does
    /// not have produces.
    Schema {
        /// The offending file.
        path: PathBuf,
        /// What was wrong, from serde.
        message: String,
        /// Where it was wrong, when the deserializer said.
        snippet: Option<Snippet>,
    },

    /// A page in a collection a feed carries has no date.
    ///
    /// Configuring a feed is how a site says its pages are dated; this is the
    /// build holding it to that.
    Undated {
        /// The page with no date.
        path: PathBuf,
        /// The collection whose feed needs one.
        collection: String,
    },

    /// One or more pages link somewhere that does not exist.
    ///
    /// Every dead link is reported at once, because fixing them one build at a
    /// time is the reason link checking gets turned off.
    DeadLinks(Vec<DeadLink>),
}

impl Error {
    /// Attaches a path to an [`std::io::Error`], which does not carry one.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    /// Builds a schema error from a `toml` failure, locating it in `source`.
    pub(crate) fn schema(
        path: impl Into<PathBuf>,
        error: &toml::de::Error,
        source: &str,
        line_offset: usize,
    ) -> Self {
        Error::Schema {
            path: path.into(),
            message: error.message().to_string(),
            snippet: error
                .span()
                .map(|span| Snippet::new(source, span, line_offset)),
        }
    }

    /// Builds a schema error from a directive failure, locating it in `source`.
    pub(crate) fn directive(
        path: impl Into<PathBuf>,
        fault: &crate::directive::Fault,
        source: &str,
        line_offset: usize,
    ) -> Self {
        Error::Schema {
            path: path.into(),
            message: fault.message.clone(),
            snippet: Some(Snippet::at_line(source, fault.line, line_offset)),
        }
    }

    /// The path this error is about, or the first one for a batch of links.
    pub fn path(&self) -> Option<&std::path::Path> {
        match self {
            Error::Io { path, .. }
            | Error::MissingFrontmatter { path }
            | Error::UnterminatedFrontmatter { path }
            | Error::Schema { path, .. }
            | Error::Undated { path, .. } => Some(path),
            Error::DeadLinks(links) => links.first().map(|link| link.source.as_path()),
        }
    }

    /// Renders the error, optionally with colour.
    ///
    /// Colour is a parameter rather than a global, so the decision belongs to
    /// whoever knows whether anything is watching -- which is the executable,
    /// not the library.
    pub fn render(&self, color: bool) -> String {
        let (red, dim, off) = palette(color);

        match self {
            Error::Io { path, source } => format!("{}: {source}", path.display()),

            Error::MissingFrontmatter { path } => format!(
                "{}: expected a `+++` frontmatter block at the top of the file",
                path.display()
            ),

            Error::UnterminatedFrontmatter { path } => {
                format!("{}: unterminated `+++` frontmatter block", path.display())
            }

            Error::Schema {
                path,
                message,
                snippet,
            } => {
                let mut out = match snippet {
                    Some(at) => {
                        format!("{}:{}:{}: {message}", path.display(), at.line, at.column)
                    }
                    None => format!("{}: {message}", path.display()),
                };

                if let Some(at) = snippet {
                    at.write(&mut out, color);
                }

                out
            }

            Error::Undated { path, collection } => format!(
                "{}: a feed is configured for `{collection}`, so this page needs a `date`",
                path.display()
            ),

            Error::DeadLinks(links) => {
                let mut out = format!(
                    "{red}{} dead internal link{}{off}:",
                    links.len(),
                    if links.len() == 1 { "" } else { "s" }
                );

                for link in links {
                    out.push_str(&format!(
                        "\n  {dim}{}{off}: {}",
                        link.source.display(),
                        link.describe()
                    ));
                }

                out
            }
        }
    }
}

/// The escape sequences, or nothing at all.
fn palette(color: bool) -> (&'static str, &'static str, &'static str) {
    if color {
        ("\x1b[31m", "\x1b[2m", "\x1b[0m")
    } else {
        ("", "", "")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render(false))
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// The result type used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Error, Snippet};

    #[test]
    fn a_snippet_points_at_the_right_column() {
        let source = "title = \"Hi\"\norder = nope\n";
        let at = Snippet::new(source, 21..25, 0);
        assert_eq!(at.line, 2);
        assert_eq!(at.column, 9);
        assert_eq!(at.text, "order = nope");
        assert_eq!(at.width, 4);
    }

    #[test]
    fn line_numbers_are_file_absolute_not_frontmatter_relative() {
        // The `+++` line is line 1 of the file, so frontmatter line 1 is line 2.
        let source = "ttile = \"Hi\"\n";
        let at = Snippet::new(source, 0..5, 1);
        assert_eq!(at.line, 2);
    }

    #[test]
    fn the_frame_puts_the_caret_under_the_span() {
        let err = Error::Schema {
            path: PathBuf::from("content/index.md"),
            message: "unknown field `ttile`".to_string(),
            snippet: Some(Snippet::new("ttile = \"Hi\"\n", 0..5, 1)),
        };

        let rendered = err.render(false);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines[0], "content/index.md:2:1: unknown field `ttile`");
        assert_eq!(lines[1], "  |");
        assert_eq!(lines[2], "2 | ttile = \"Hi\"");
        assert_eq!(lines[3], "  | ^^^^^");
    }

    #[test]
    fn colour_is_a_parameter_and_is_off_by_default() {
        let err = Error::DeadLinks(Vec::new());
        assert!(!err.render(false).contains('\x1b'));
        assert!(err.render(true).contains('\x1b'));
        assert_eq!(err.to_string(), err.render(false));
    }

    #[test]
    fn a_span_past_the_end_of_the_line_does_not_overrun() {
        let at = Snippet::new("a = 1\n", 0..99, 0);
        assert_eq!(at.text, "a = 1");
        assert!(at.width <= at.text.len());
    }
}
