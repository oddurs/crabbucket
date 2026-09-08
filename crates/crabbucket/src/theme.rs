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
//!
//! The trait carries an associated [`Theme::Layout`] type, which is what keeps
//! layouts out of the string world.  A theme declares the layouts it offers as
//! an enum; `layout = "docs"` in a page's frontmatter deserializes into it, and
//! a layout the theme does not have fails the build naming the file, the line
//! and the layouts that do exist.

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::directive::Directives;
use crate::url::Url;

/// The frontmatter every page has, whatever else it has.
///
/// `L` is the theme's layout type.  A site with richer frontmatter loads its
/// own type through [`crate::Collection`]; this is the shape the built-in
/// pipeline needs in order to render a page without the site writing any Rust.
#[derive(Debug, Clone, Deserialize)]
pub struct PageMeta<L> {
    /// The page title.
    pub title: String,

    /// A one-line description, used for `<meta name="description">`.
    #[serde(default)]
    pub description: Option<String>,

    /// Which of the theme's layouts to render this page with.
    #[serde(default)]
    pub layout: L,

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

impl<L> PageMeta<L> {
    /// The label this page should carry in navigation.
    pub fn label(&self) -> &str {
        self.nav_label.as_deref().unwrap_or(&self.title)
    }
}

/// One entry in a navigation list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItem {
    /// The visible label.
    pub label: String,
    /// Where it goes.
    pub href: Url,
    /// Whether it is the page currently being rendered.
    pub current: bool,
}

/// What a page needs to know about every other page.
///
/// A theme is handed this rather than a pre-built navigation list, because the
/// navigation a docs site wants -- a masthead across the top and a sidebar of
/// the pages under `docs/` -- is two different views of the same set, and only
/// the theme knows which it needs.
#[derive(Debug, Clone, Default)]
pub struct SiteIndex {
    pages: Vec<PageRef>,
}

/// One page, as seen from another page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRef {
    /// The page's route.
    pub route: String,
    /// The label it would like to be called by.
    pub label: String,
    /// Its position in the primary navigation, if it asked for one.
    pub nav_order: Option<u32>,
}

impl SiteIndex {
    /// Builds an index from the pages that will be written.
    pub fn new(pages: Vec<PageRef>) -> Self {
        SiteIndex { pages }
    }

    /// Every page, in route order.
    pub fn pages(&self) -> &[PageRef] {
        &self.pages
    }

    /// Whether a route exists.  This is what link checking is built on.
    pub fn contains(&self, route: &str) -> bool {
        let route = route.trim_matches('/');
        self.pages.iter().any(|page| page.route == route)
    }

    /// The primary navigation: the pages that asked for a `nav_order`, in that
    /// order, with `current` set on the one being rendered.
    pub fn nav(&self, config: &Config, current: &str) -> Vec<NavItem> {
        let mut listed: Vec<&PageRef> = self
            .pages
            .iter()
            .filter(|page| page.nav_order.is_some())
            .collect();

        listed.sort_by(|a, b| {
            a.nav_order
                .cmp(&b.nav_order)
                .then_with(|| a.label.cmp(&b.label))
        });
        listed
            .into_iter()
            .map(|page| self.item(config, page, current))
            .collect()
    }

    /// Every page below `prefix`, in route order -- a docs sidebar.
    ///
    /// The section's own index page is excluded, since it is usually the thing
    /// the sidebar hangs off rather than an entry in it.
    pub fn under(&self, config: &Config, prefix: &str, current: &str) -> Vec<NavItem> {
        let prefix = prefix.trim_matches('/');
        self.pages
            .iter()
            .filter(|page| page.route != prefix && page.route.starts_with(prefix))
            .map(|page| self.item(config, page, current))
            .collect()
    }

    fn item(&self, config: &Config, page: &PageRef, current: &str) -> NavItem {
        NavItem {
            label: page.label.clone(),
            href: Url::new(config, &page.route),
            current: page.route == current.trim_matches('/'),
        }
    }
}

/// Everything a theme is given in order to render one page.
pub struct Page<'a, L> {
    /// The site's configuration, which carries the base path.
    pub config: &'a Config,
    /// The page's frontmatter, including its layout.
    pub meta: &'a PageMeta<L>,
    /// The page's route.  The empty string is the site root.
    pub route: &'a str,
    /// The page body, already rendered from Markdown to HTML.
    pub html: &'a str,
    /// Every page in the site, for building navigation.
    pub site: &'a SiteIndex,
}

impl<L> Page<'_, L> {
    /// The primary navigation, with this page marked as current.
    pub fn nav(&self) -> Vec<NavItem> {
        self.site.nav(self.config, self.route)
    }

    /// The pages below `prefix`, with this page marked as current.
    pub fn section(&self, prefix: &str) -> Vec<NavItem> {
        self.site.under(self.config, prefix, self.route)
    }
}

/// A design system.
pub trait Theme {
    /// The layouts this design system offers.
    ///
    /// Making this an associated type rather than a string is the whole reason
    /// a misspelled layout cannot reach the output: `serde` rejects it while
    /// the page's frontmatter is being read.
    type Layout: DeserializeOwned + Default;

    /// Renders a complete HTML document.
    fn render(&self, page: &Page<'_, Self::Layout>) -> String;

    /// The stylesheet written to `site.css`.
    fn stylesheet(&self) -> String;

    /// The directives this design system offers to Markdown.
    ///
    /// A theme with no components to reach from content says nothing, and a
    /// page that writes `:::anything` then fails the build saying so.
    fn directives(&self) -> Directives {
        Directives::new()
    }

    /// The client router written to `router.js`, when the site asks for one.
    ///
    /// A `String` rather than a `&str` because a theme's own class names go
    /// into it, and those are not known until the theme is written.
    fn router_js(&self) -> String;
}
