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

//! The site's own configuration, read from `site.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};

/// A site's configuration.
///
/// The only field that carries real weight is [`Config::base`].  A GitHub
/// Pages project site is served from `/repo/` and a user site from `/`, and
/// getting that wrong is the single most common way one of these sites ships
/// broken.  It is recorded once, here, and applied in exactly one place --
/// [`crate::url::Url`] -- so no page ever has to know about it.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The site's title, used in `<title>` and in feeds.
    pub title: String,

    /// A one-line description of the site.
    #[serde(default)]
    pub description: String,

    /// The site's absolute URL, needed only by the things that cannot be
    /// relative: feeds, sitemaps, and `og:` metadata.  Omitted, those are
    /// simply not emitted, rather than emitted wrong.
    #[serde(default)]
    pub url: Option<String>,

    /// The path the site is served from, with leading and trailing slashes;
    /// `/` for a user site or custom domain, `/repo/` for a project site.
    #[serde(default = "root")]
    pub base: String,

    /// Whether to emit the client-side router.  Off by default: a site that
    /// ships no JavaScript is the thing worth defaulting to.
    #[serde(default)]
    pub router: bool,
}

fn root() -> String {
    "/".to_string()
}

impl Config {
    /// Reads `site.toml` from a site directory.
    ///
    /// # Errors
    ///
    /// Fails if the file is unreadable or is not valid configuration.
    pub fn load(site_dir: &Path) -> Result<Self> {
        let path: PathBuf = site_dir.join("site.toml");
        let text = fs::read_to_string(&path).map_err(|source| Error::io(&path, source))?;
        let mut config: Config =
            toml::from_str(&text).map_err(|error| Error::schema(&path, &error, &text, 0))?;
        config.base = normalize_base(&config.base);
        Ok(config)
    }
}

impl Config {
    /// Overrides the base path, normalising it the same way the file does.
    ///
    /// Every spelling reaches the same place, so a command-line override and a
    /// configuration file cannot disagree about what `repo` means.
    pub fn set_base(&mut self, base: &str) {
        self.base = normalize_base(base);
    }
}

/// Forces a base path into the `/…/` shape the rest of the crate assumes.
fn normalize_base(base: &str) -> String {
    let trimmed = base.trim().trim_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_base;

    #[test]
    fn every_spelling_of_a_base_path_normalizes_the_same_way() {
        for given in ["repo", "/repo", "repo/", "/repo/", " /repo/ "] {
            assert_eq!(normalize_base(given), "/repo/", "for input {given:?}");
        }
    }

    #[test]
    fn an_empty_base_is_the_root() {
        for given in ["", "/", "   "] {
            assert_eq!(normalize_base(given), "/", "for input {given:?}");
        }
    }
}
