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

//! The error type shared by every stage of a build.
//!
//! Every variant carries the path it applies to.  A build error that does not
//! name the file it came from is a bug report waiting to happen, so the type
//! makes it impossible to construct one.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::links::DeadLink;

/// A build failure, always attributed to a file.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The file could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The file the operation was on.
        path: PathBuf,
        /// The underlying failure.
        #[source]
        source: std::io::Error,
    },

    /// A content file did not begin with a `+++` frontmatter block.
    #[error("{path}: expected a `+++` frontmatter block at the top of the file")]
    MissingFrontmatter {
        /// The offending content file.
        path: PathBuf,
    },

    /// A content file opened a frontmatter block but never closed it.
    #[error("{path}: unterminated `+++` frontmatter block")]
    UnterminatedFrontmatter {
        /// The offending content file.
        path: PathBuf,
    },

    /// The frontmatter did not deserialize into the collection's type.  This
    /// is the error a missing `title` produces.
    #[error("{path}: {source}")]
    Frontmatter {
        /// The offending content file.
        path: PathBuf,
        /// The deserialization failure, which carries its own line and column.
        #[source]
        source: toml::de::Error,
    },

    /// One or more pages link somewhere that does not exist.
    ///
    /// Every dead link is reported at once, because fixing them one build at a
    /// time is the reason link checking gets turned off.
    #[error("{}", dead_links(.0))]
    DeadLinks(Vec<DeadLink>),

    /// The site's configuration file was not valid.
    #[error("{path}: {source}")]
    Config {
        /// The configuration file.
        path: PathBuf,
        /// The deserialization failure.
        #[source]
        source: toml::de::Error,
    },
}

/// Formats a whole batch of dead links, one per line, each naming its page.
fn dead_links(links: &[DeadLink]) -> String {
    let mut out = format!(
        "{} dead internal link{}:",
        links.len(),
        if links.len() == 1 { "" } else { "s" }
    );

    for link in links {
        let _ = write!(
            out,
            "\n  {}: {} -> {} does not exist",
            link.source.display(),
            link.href,
            link.target
        );
    }

    out
}

impl Error {
    /// Attaches a path to an [`std::io::Error`], which does not carry one.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

/// The result type used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
