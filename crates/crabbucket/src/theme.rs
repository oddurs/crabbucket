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

use std::path::Path;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::directive::Directives;
use crate::markdown::Heading;
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

    /// Where the page sits within its own section: the sidebar, and the
    /// previous and next links.
    ///
    /// Separate from `nav_order` because they answer different questions.
    /// `nav_order` decides whether a page is in the masthead at all; `order`
    /// decides reading order among siblings, which every documentation page
    /// has and almost none of which belong in the masthead.  Pages without it
    /// fall to the end, in route order.
    #[serde(default)]
    pub order: Option<u32>,

    /// The label to use in navigation, if it should differ from the title.
    #[serde(default)]
    pub nav_label: Option<String>,

    /// When the page is dated, if it is.
    ///
    /// Optional because most pages are not dated, and required of any page in
    /// a collection a feed is configured for -- a dated collection with an
    /// undated page in it fails the build naming the page.
    ///
    /// TOML has real dates, so `date = 2026-09-08` is a date rather than a
    /// string that looks like one, and a malformed one fails at the file and
    /// the line without anything here doing the checking.
    #[serde(default)]
    pub date: Option<toml::value::Datetime>,

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
    /// Its position within its own section, if it asked for one.
    pub order: Option<u32>,
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

    /// Every page below `prefix`, in reading order -- a docs sidebar.
    ///
    /// Reading order is `order` where a page states one and route order
    /// otherwise, which is the same rule the previous and next links use.  Two
    /// orderings that can disagree eventually will, so there is only one.
    ///
    /// The section's own index page is excluded, since it is usually the thing
    /// the sidebar hangs off rather than an entry in it.
    pub fn under(&self, config: &Config, prefix: &str, current: &str) -> Vec<NavItem> {
        let prefix = prefix.trim_matches('/');

        let mut section: Vec<&PageRef> = self
            .pages
            .iter()
            .filter(|page| page.route != prefix && page.route.starts_with(prefix))
            .collect();

        section.sort_by(|a, b| {
            a.order
                .unwrap_or(u32::MAX)
                .cmp(&b.order.unwrap_or(u32::MAX))
                .then_with(|| a.route.cmp(&b.route))
        });

        section
            .into_iter()
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
    /// The page's headings, in document order, for a table of contents.
    ///
    /// These are the same ids the link checker validates fragments against,
    /// so a table of contents cannot point at a heading that is not there.
    pub headings: &'a [Heading],
    /// Every page in the site, for building navigation.
    pub site: &'a SiteIndex,
    /// The site's feeds, for the `<link rel="alternate">` tags in the head.
    pub feeds: &'a [FeedLink],
    /// Somewhere a design system may keep expensive things between builds.
    ///
    /// Outside `dist/`, which is deleted every build, and safe to delete: a
    /// miss costs time and nothing else.
    pub cache: &'a Path,
    /// Where this page's social card will be written, if the design system
    /// draws one.  `None` when the site has no absolute URL, since a card is
    /// only ever referenced from one.
    pub card: Option<&'a Url>,
}

/// A feed, as a page's head needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedLink {
    /// The feed's title.
    pub title: String,
    /// Where the RSS is.
    pub rss: Url,
    /// Where the Atom is.
    pub atom: Url,
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

    /// The pages either side of this one within `prefix`.
    ///
    /// The ordering is [`Page::section`]'s, and deliberately so: a reader who
    /// follows "next" through a section should visit it in the order the
    /// sidebar showed them, and two orderings that can disagree eventually
    /// will.
    pub fn neighbours(&self, prefix: &str) -> (Option<NavItem>, Option<NavItem>) {
        let section = self.section(prefix);
        let Some(at) = section.iter().position(|item| item.current) else {
            return (None, None);
        };

        let previous = at.checked_sub(1).and_then(|at| section.get(at)).cloned();
        (previous, section.get(at + 1).cloned())
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

    /// The social card for a page, as PNG bytes.
    ///
    /// `None` means this design system does not draw them, which is the
    /// default.  Drawing one is expensive, so a design system that does should
    /// keep them in [`Page::cache`] -- `crabbucket-og` does that for you.
    ///
    /// Only called when the site has an absolute URL, because a card that
    /// cannot be linked to absolutely is a card nothing will ever fetch.
    fn og_image(&self, page: &Page<'_, Self::Layout>) -> Option<Vec<u8>> {
        let _ = page;
        None
    }

    /// The search client, written to `search.js` when the site asks for it.
    ///
    /// `None` means this design system has no search.  A site that asks for
    /// it anyway is told so, and gets no dead `search.js`.
    fn search_js(&self) -> Option<String> {
        None
    }

    /// The client router, written to `router.js` when the site asks for it.
    ///
    /// `None` means this design system has no router, which is a reasonable
    /// thing for a design system to be: one that renders a document rather
    /// than an application has nothing for a router to improve.
    ///
    /// A `String` rather than a `&str` because a theme's own class names go
    /// into it, and those are not known until the theme is written.
    fn router_js(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{NavItem, Page, PageMeta, PageRef, SiteIndex};
    use crate::config::Config;

    fn config() -> Config {
        Config {
            title: "t".into(),
            description: String::new(),
            url: None,
            base: "/repo/".into(),
            search: false,
            feeds: Vec::new(),
            router: false,
        }
    }

    fn page(route: &str, nav_order: Option<u32>, order: Option<u32>) -> PageRef {
        PageRef {
            route: route.into(),
            label: route.to_uppercase(),
            nav_order,
            order,
        }
    }

    fn index() -> SiteIndex {
        SiteIndex::new(vec![
            page("", Some(1), None),
            page("docs", Some(2), None),
            page("docs/zebra", None, Some(1)),
            page("docs/apple", None, Some(2)),
            page("docs/mango", None, None),
            page("about", None, None),
        ])
    }

    fn labels(items: &[NavItem]) -> Vec<&str> {
        items.iter().map(|item| item.label.as_str()).collect()
    }

    #[test]
    fn the_primary_navigation_is_only_pages_that_asked_for_it() {
        let items = index().nav(&config(), "");
        assert_eq!(labels(&items), ["", "DOCS"]);
        assert!(items[0].current, "the page being rendered is marked");
        assert!(!items[1].current);
    }

    #[test]
    fn a_section_reads_in_order_and_then_alphabetically() {
        // `order` first, then route order for whatever did not state one.
        let items = index().under(&config(), "docs", "docs/apple");
        assert_eq!(labels(&items), ["DOCS/ZEBRA", "DOCS/APPLE", "DOCS/MANGO"]);
    }

    #[test]
    fn a_section_excludes_its_own_index_page() {
        let items = index().under(&config(), "docs", "docs");
        assert!(
            !labels(&items).contains(&"DOCS"),
            "the index is the thing the sidebar hangs off"
        );
    }

    #[test]
    fn hrefs_carry_the_base_path() {
        let items = index().under(&config(), "docs", "docs/apple");
        assert_eq!(items[0].href.as_str(), "/repo/docs/zebra/");
    }

    fn meta() -> PageMeta<()> {
        PageMeta {
            title: "T".into(),
            description: None,
            layout: (),
            nav_order: None,
            order: None,
            nav_label: None,
            date: None,
            draft: false,
        }
    }

    fn at(route: &str, site: &SiteIndex, config: &Config) -> (Option<NavItem>, Option<NavItem>) {
        let meta = meta();
        let page = Page {
            config,
            meta: &meta,
            route,
            html: "",
            headings: &[],
            site,
            feeds: &[],
            cache: Path::new(""),
            card: None,
        };
        page.neighbours("docs")
    }

    #[test]
    fn neighbours_follow_the_order_the_sidebar_showed() {
        let (site, config) = (index(), config());

        let (previous, next) = at("docs/apple", &site, &config);
        assert_eq!(previous.map(|item| item.label), Some("DOCS/ZEBRA".into()));
        assert_eq!(next.map(|item| item.label), Some("DOCS/MANGO".into()));
    }

    #[test]
    fn each_end_of_a_section_omits_its_missing_side() {
        let (site, config) = (index(), config());

        let (previous, next) = at("docs/zebra", &site, &config);
        assert!(previous.is_none(), "the first page has nothing before it");
        assert!(next.is_some());

        let (previous, next) = at("docs/mango", &site, &config);
        assert!(previous.is_some());
        assert!(next.is_none(), "the last page has nothing after it");
    }

    #[test]
    fn a_page_outside_the_section_has_no_neighbours_in_it() {
        let (site, config) = (index(), config());
        let (previous, next) = at("about", &site, &config);

        assert!(previous.is_none() && next.is_none());
    }
}
