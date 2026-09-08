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

//! Typed content collections.
//!
//! A collection is a directory of Markdown files whose `+++` frontmatter
//! deserializes into a type the site chose.  Astro validates that shape at
//! runtime; here it is the collection's type parameter, so a page missing a
//! field fails the build with the file and the line.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::markdown;

/// The delimiter opening and closing a frontmatter block.
const FENCE: &str = "+++";

/// One content file, with its frontmatter deserialized.
#[derive(Debug, Clone)]
pub struct Entry<T> {
    /// The deserialized frontmatter.
    pub meta: T,
    /// The site-relative route, derived from the path below the collection
    /// root: `docs/intro.md` is `docs/intro`, and any `index.md` is the route
    /// of its directory.  The empty string is the site root.
    pub route: String,
    /// The file this entry was read from, for diagnostics.
    pub path: PathBuf,
    /// The body, rendered to HTML.
    pub html: String,
    /// Every heading in the body, in document order.  This is what fragment
    /// links are checked against, and what a table of contents is built from.
    pub headings: Vec<markdown::Heading>,
}

impl<T> Entry<T> {
    /// The ids of every heading in the body.
    pub fn anchors(&self) -> BTreeSet<String> {
        self.headings
            .iter()
            .map(|heading| heading.id.clone())
            .collect()
    }
}

/// Every entry in one content directory.
#[derive(Debug, Clone)]
pub struct Collection<T> {
    entries: Vec<Entry<T>>,
}

impl<T: DeserializeOwned> Collection<T> {
    /// Loads every `.md` file under `dir`, recursively.
    ///
    /// Entries come back sorted by route, so a build is reproducible regardless
    /// of the order the filesystem hands files over.  Sorting into the order a
    /// site actually wants -- by an `order` field, by date -- is the site's
    /// job, and is ordinary Rust because [`Collection::entries`] is a slice.
    ///
    /// # Errors
    ///
    /// Fails if a file is unreadable, lacks frontmatter, or has frontmatter
    /// that does not deserialize into `T`.
    pub fn load(dir: &Path) -> Result<Self> {
        let mut entries = Vec::new();

        for found in WalkDir::new(dir).sort_by_file_name() {
            let found = found.map_err(|err| {
                let path = err.path().unwrap_or(dir).to_path_buf();
                Error::io(path, err.into())
            })?;

            let path = found.path();
            if !found.file_type().is_file() || path.extension().is_none_or(|e| e != "md") {
                continue;
            }

            entries.push(Entry::load(path, dir)?);
        }

        entries.sort_by(|a, b| a.route.cmp(&b.route));
        Ok(Collection { entries })
    }
}

impl<T> Collection<T> {
    /// The entries, in slug order.
    pub fn entries(&self) -> &[Entry<T>] {
        &self.entries
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> IntoIterator for Collection<T> {
    type Item = Entry<T>;
    type IntoIter = std::vec::IntoIter<Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Collection<T> {
    type Item = &'a Entry<T>;
    type IntoIter = std::slice::Iter<'a, Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<T: DeserializeOwned> Entry<T> {
    /// Reads and parses a single content file.
    ///
    /// # Errors
    ///
    /// Fails if the file is unreadable, lacks frontmatter, or has frontmatter
    /// that does not deserialize into `T`.
    pub fn load(path: &Path, root: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        let (frontmatter, body) = split(&text, path)?;

        // The opening `+++` is line 1 of the file, so frontmatter line 1 is
        // line 2, and an editor jumping to the reported line lands on it.
        let meta: T = toml::from_str(frontmatter)
            .map_err(|error| Error::schema(path, &error, frontmatter, 1))?;

        let body = markdown::render(body);

        Ok(Entry {
            meta,
            route: route_of(path, root),
            path: path.to_path_buf(),
            html: body.html,
            headings: body.headings,
        })
    }
}

/// Derives a route from a content file's path below the collection root.
fn route_of(path: &Path, root: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path).with_extension("");
    let mut route = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    if route == "index" {
        route.clear();
    } else if let Some(parent) = route.strip_suffix("/index") {
        route = parent.to_string();
    }

    route
}

/// Splits a content file into its frontmatter and its body.
fn split<'a>(text: &'a str, path: &Path) -> Result<(&'a str, &'a str)> {
    let rest = text
        .strip_prefix(FENCE)
        .and_then(|rest| {
            rest.strip_prefix('\n')
                .or_else(|| rest.strip_prefix("\r\n"))
        })
        .ok_or_else(|| Error::MissingFrontmatter {
            path: path.to_path_buf(),
        })?;

    let end = rest
        .find(&format!("\n{FENCE}"))
        .ok_or_else(|| Error::UnterminatedFrontmatter {
            path: path.to_path_buf(),
        })?;

    let frontmatter = &rest[..end];
    let body = rest[end + 1 + FENCE.len()..].trim_start_matches(['\r', '\n']);
    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{route_of, split};
    use crate::error::Error;

    #[test]
    fn frontmatter_and_body_come_apart() {
        let text = "+++\ntitle = \"Hi\"\n+++\n\n# Heading\n";
        let (frontmatter, body) = split(text, Path::new("t.md")).unwrap();
        assert_eq!(frontmatter, "title = \"Hi\"");
        assert_eq!(body, "# Heading\n");
    }

    #[test]
    fn a_body_containing_a_fence_is_not_cut_short() {
        let text = "+++\ntitle = \"Hi\"\n+++\nbefore\n+++\nafter\n";
        let (frontmatter, body) = split(text, Path::new("t.md")).unwrap();
        assert_eq!(frontmatter, "title = \"Hi\"");
        assert_eq!(body, "before\n+++\nafter\n");
    }

    #[test]
    fn a_file_without_frontmatter_is_an_error_naming_the_file() {
        let err = split("# Heading\n", Path::new("t.md")).unwrap_err();
        assert!(matches!(err, Error::MissingFrontmatter { .. }));
        assert!(err.to_string().starts_with("t.md:"));
    }

    #[test]
    fn routes_come_from_the_path_below_the_root() {
        let root = Path::new("content");
        assert_eq!(route_of(Path::new("content/index.md"), root), "");
        assert_eq!(route_of(Path::new("content/about.md"), root), "about");
        assert_eq!(
            route_of(Path::new("content/docs/intro.md"), root),
            "docs/intro"
        );
        assert_eq!(route_of(Path::new("content/docs/index.md"), root), "docs");
    }

    #[test]
    fn an_unterminated_block_is_an_error_naming_the_file() {
        let err = split("+++\ntitle = \"Hi\"\n", Path::new("t.md")).unwrap_err();
        assert!(matches!(err, Error::UnterminatedFrontmatter { .. }));
    }
}
