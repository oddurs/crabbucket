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
#[non_exhaustive]
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

    /// Whether to build a search index and ship the client that reads it.
    ///
    /// Opt-in, like the router, and for the same reason: search needs script,
    /// and a page that ships none is the thing worth defaulting to.
    #[serde(default)]
    pub search: bool,

    /// The feeds to generate.
    ///
    /// Each needs absolute URLs, so a site with no `url` gets none, and is
    /// told rather than handed a feed full of relative links.
    #[serde(default, rename = "feed")]
    pub feeds: Vec<crate::feed::Feed>,

    /// Pages the site renders itself, rather than writing as Markdown.
    ///
    /// Held raw because their `layout` and any extra fields are the design
    /// system's types, which this struct cannot name.  [`crate::build_with`]
    /// deserializes them, so a declared page gets the same frontmatter
    /// checking as a written one.
    ///
    /// ```toml
    /// [[page]]
    /// route = ""
    /// title = "A site is a typed value"
    /// layout = "landing"
    /// nav_order = 1
    /// ```
    #[serde(default, rename = "page")]
    pub pages: Vec<toml::Table>,

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

impl Config {
    /// A configuration with nothing in it.
    ///
    /// A real site's configuration is read from `site.toml`; this is for tests
    /// and for the rare caller that needs one before it has a file.  The
    /// fields are public, so anything else is set by assigning to them --
    /// which is why the struct is `non_exhaustive` and this is not a `Default`
    /// somebody would reach for in earnest.
    pub fn blank() -> Self {
        Config {
            title: String::new(),
            description: String::new(),
            url: None,
            base: "/".to_string(),
            search: false,
            feeds: Vec::new(),
            pages: Vec::new(),
            router: false,
        }
    }
}

/// Forces a base path into the `/…/` shape the rest of the crate assumes.
fn normalize_base(base: &str) -> String {
    // Squeezed as well as trimmed: `base = "/my//repo/"' is a plausible thing
    // to type, and it would otherwise put a doubled slash into every URL on
    // the site -- a different URL to a browser, the same one to a person.
    // Slashes and spaces, in any interleaving: `" / repo / "' is a thing
    // somebody types, and trimming spaces and then slashes leaves the space
    // in `"repo /"' behind.
    let trimmed = crate::url::squeeze(
        base.trim_matches(|character: char| character == '/' || character.is_whitespace()),
    );
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, normalize_base};

    #[test]
    fn a_site_that_names_no_base_is_served_from_the_root() {
        // `root()' is the serde default for `base', and a mutant that made it
        // return "" or "xyzzy" broke nothing: no test read a config without a
        // base, which is what most sites have.
        let config: Config = toml::from_str("title = \"t\"\ndescription = \"d\"\n")
            .expect("a site.toml with only a title should parse");

        assert_eq!(config.base, "/");
        assert!(!config.search);
        assert!(!config.router);
        assert!(config.url.is_none());
    }

    #[test]
    fn a_base_with_a_doubled_slash_does_not_put_one_in_every_url() {
        // Found by the property test in tests/properties.rs, which is why
        // there is a named one here: a property test says something is
        // wrong, and a named example stops it coming back quietly.
        assert_eq!(normalize_base("/my//repo/"), "/my/repo/");
        assert_eq!(normalize_base("///"), "/");
    }

    #[test]
    fn slashes_and_spaces_are_trimmed_in_any_order() {
        // Trimming spaces and then slashes leaves the space in "repo /".
        for given in [" / repo / ", "repo /", "/ repo", " //repo// "] {
            assert_eq!(normalize_base(given), "/repo/", "for input {given:?}");
        }
    }

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
