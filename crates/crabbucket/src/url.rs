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

//! Site-absolute URLs, and the one place the base path is applied.

use std::fmt;

use crate::config::Config;

/// A URL within the site.
///
/// Constructing one is the only supported way to link to a page, which is what
/// makes the base path a solved problem rather than a recurring one: it is
/// applied here, once, and nowhere else in the crate or in a site's own code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url(String);

impl Url {
    /// Builds a URL for a site-relative path such as `docs/getting-started`.
    ///
    /// The leading and trailing slashes of `path` are ignored; the result
    /// always begins with the configured base and, for a non-empty path, ends
    /// in a slash.
    pub fn new(config: &Config, path: &str) -> Self {
        let path = squeeze(path.trim_matches('/'));
        if path.is_empty() {
            Url(config.base.clone())
        } else {
            Url(format!("{}{}/", config.base, path))
        }
    }

    /// Builds a URL for an asset, which -- unlike a page -- keeps its file
    /// extension and gets no trailing slash.
    pub fn asset(config: &Config, path: &str) -> Self {
        Url(format!(
            "{}{}",
            config.base,
            squeeze(path.trim_start_matches('/'))
        ))
    }

    /// The URL as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Collapses runs of slashes, so no URL this type builds contains `//`.
///
/// A doubled slash is a different URL to a browser and the same one to a
/// person, which is the worst combination available -- and it is reachable
/// from a configuration file: a feed declared over the collection
/// `notes//archive' would otherwise put one in every feed URL on the site.
///
/// This lives here because this is the only place a URL is assembled.  A
/// caller that has to remember to tidy its own path is a caller that will
/// forget, which is the argument for the whole type.
pub(crate) fn squeeze(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut last_was_slash = false;

    for character in path.chars() {
        if character == '/' && last_was_slash {
            continue;
        }

        last_was_slash = character == '/';
        out.push(character);
    }

    out
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl maud::Render for Url {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::Url;
    use crate::config::Config;

    fn config(base: &str) -> Config {
        {
            let mut config = Config::blank();
            config.title = "t".into();
            config.description = String::new();
            config.url = None;
            config.base = base.into();
            config
        }
    }

    #[test]
    fn interpolating_a_url_into_markup_emits_the_url() {
        // The whole mechanism by which a link gets the base path: a component
        // writes `href=(Url::new(config, "docs"))' and `Render' puts the
        // string in.  cargo-mutants found that emptying `render_to' broke
        // nothing, which meant nothing tested the one thing this type is for.
        let config = config("/crabbucket/");
        let url = Url::new(&config, "docs/routing");

        let markup = maud::html! { a href=(url) { "Routing" } };

        assert_eq!(
            markup.into_string(),
            "<a href=\"/crabbucket/docs/routing/\">Routing</a>"
        );
    }

    #[test]
    fn a_doubled_slash_never_reaches_a_url() {
        // A feed declared over the collection `notes//archive' would
        // otherwise put one into every feed URL on the site.  Found by the
        // property test in tests/properties.rs.
        let config = config("/crabbucket/");

        assert_eq!(
            Url::asset(&config, "notes//archive/feed.xml").as_str(),
            "/crabbucket/notes/archive/feed.xml"
        );
        assert_eq!(
            Url::new(&config, "docs//routing").as_str(),
            "/crabbucket/docs/routing/"
        );
    }

    #[test]
    fn a_project_site_gets_its_repository_prefix() {
        let config = config("/crabbucket/");
        assert_eq!(
            Url::new(&config, "docs/intro").as_str(),
            "/crabbucket/docs/intro/"
        );
        assert_eq!(Url::new(&config, "").as_str(), "/crabbucket/");
    }

    #[test]
    fn a_user_site_gets_no_prefix() {
        let config = config("/");
        assert_eq!(Url::new(&config, "docs/intro").as_str(), "/docs/intro/");
        assert_eq!(Url::new(&config, "/").as_str(), "/");
    }

    #[test]
    fn assets_keep_their_extension_and_gain_no_slash() {
        let config = config("/crabbucket/");
        assert_eq!(
            Url::asset(&config, "site.css").as_str(),
            "/crabbucket/site.css"
        );
        assert_eq!(
            Url::asset(&config, "/site.css").as_str(),
            "/crabbucket/site.css"
        );
    }
}
