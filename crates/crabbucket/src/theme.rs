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

//! The seam between a site and its design system.
//!
//! A design system is a crate implementing [`Theme`].  `crabbucket-ui` is the
//! default one; a private crate shared across a fleet of small sites is the
//! interesting one, because restyling all of them is then a version bump.

use serde::Deserialize;

use crate::config::Config;
use crate::url::Url;

/// The frontmatter every page has, whatever else it has.
///
/// A site with richer frontmatter loads its own type through
/// [`crate::Collection`]; this is the shape the built-in pipeline needs in
/// order to render a page without the site writing any Rust at all.
#[derive(Debug, Clone, Deserialize)]
pub struct PageMeta {
    /// The page title.
    pub title: String,

    /// A one-line description, used for `<meta name="description">`.
    #[serde(default)]
    pub description: Option<String>,

    /// Where the page sits in the primary navigation.  Pages without it are
    /// reachable but not listed.
    #[serde(default)]
    pub nav_order: Option<u32>,

    /// The label to use in navigation, if it should differ from the title.
    #[serde(default)]
    pub nav_label: Option<String>,

    /// Whether to skip the page entirely.
    #[serde(default)]
    pub draft: bool,
}

impl PageMeta {
    /// The label this page should carry in navigation.
    pub fn label(&self) -> &str {
        self.nav_label.as_deref().unwrap_or(&self.title)
    }
}

/// One entry in the site's primary navigation.
#[derive(Debug, Clone)]
pub struct NavItem {
    /// The visible label.
    pub label: String,
    /// Where it goes.
    pub href: Url,
    /// Whether it is the page currently being rendered.
    pub current: bool,
}

/// Everything a theme is given in order to render one page.
pub struct Page<'a> {
    /// The site's configuration, which carries the base path.
    pub config: &'a Config,
    /// The page's frontmatter.
    pub meta: &'a PageMeta,
    /// The page's route, for the site's own reference.
    pub route: &'a str,
    /// The page body, already rendered from Markdown to HTML.
    pub html: &'a str,
    /// The primary navigation, with the current page marked.
    pub nav: &'a [NavItem],
}

/// A design system.
pub trait Theme {
    /// Renders a complete HTML document.
    fn render(&self, page: &Page<'_>) -> String;

    /// The stylesheet written to `site.css`.
    fn stylesheet(&self) -> String;

    /// The client router written to `router.js`, when the site asks for one.
    fn router_js(&self) -> &str;
}
