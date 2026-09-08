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

//! Building a whole site.
//!
//! This is the pipeline `crab build` runs: read the configuration, load the
//! content as a typed collection, hand each page to the theme, write the
//! result as directories of `index.html`, and refuse to finish if any page
//! links somewhere that does not exist.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use toml::value::Datetime;

use crate::config::Config;
use crate::content::{Collection, Entry};
use crate::directive::Directives;
use crate::error::{Error, Result};
use crate::feed::{self, Item};
use crate::links::{self, Rendered};
use crate::search;
use crate::theme::{FeedLink, Page, PageMeta, PageRef, SiteIndex, Theme};
use crate::url::Url;

/// Reads the pages a site declared, pairs each with the body the site
/// rendered, and turns them into entries indistinguishable from content.
///
/// The metadata is deserialized into the design system's own types, so a
/// declared page gets the same frontmatter checking a written one does: an
/// unknown layout fails, a missing required field fails, and both name
/// `site.toml`.
fn declared_pages<T: Theme>(
    site_dir: &Path,
    config: &Config,
    pages: &BTreeMap<String, String>,
) -> Result<Vec<Written<T>>> {
    let manifest = site_dir.join("site.toml");
    let mut entries = Vec::new();
    let mut routes = BTreeSet::new();

    for table in &config.pages {
        let route = table
            .get("route")
            .and_then(|route| route.as_str())
            .unwrap_or_default()
            .trim_matches('/')
            .to_string();

        let mut table = table.clone();
        table.remove("route");

        let meta: PageMeta<T::Layout, T::Extra> =
            toml::Value::Table(table)
                .try_into()
                .map_err(|err: toml::de::Error| Error::Schema {
                    path: manifest.clone(),
                    message: err.message().to_string(),
                    snippet: None,
                })?;

        let Some(html) = pages.get(&route) else {
            return Err(Error::Unrendered {
                path: manifest,
                route,
            });
        };

        routes.insert(route.clone());
        entries.push(Entry {
            meta,
            route,
            // Diagnostics about a declared page point at the file that
            // declared it, since there is no other file to point at.
            path: manifest.clone(),
            html: html.clone(),
            // A rendered body is not Markdown, so nothing collected headings
            // from it.  A declared page gets no table of contents, and no
            // fragment of it can be linked to and checked.
            headings: Vec::new(),
        });
    }

    for route in pages.keys() {
        if !routes.contains(route.trim_matches('/')) {
            return Err(Error::Undeclared {
                path: manifest,
                route: route.clone(),
            });
        }
    }

    Ok(entries)
}

/// Where a design system may keep expensive things between builds.
///
/// Beside the site rather than inside `dist/`, which is deleted every build.
pub const CACHE: &str = ".crabbucket";

/// Where a page's social card is written.
///
/// Derived rather than passed around, so the build and the design system's
/// `<meta>` tag cannot disagree about it.
pub fn card_path(route: &str) -> String {
    match route.trim_matches('/') {
        "" => "og/index.png".to_string(),
        route => format!("og/{route}.png"),
    }
}

/// The route reserved for the error page.
///
/// GitHub Pages, and most static hosts, serve `404.html` from the site root in
/// place of anything they cannot find.
const ERROR_ROUTE: &str = "404";

/// The pages of a site, borrowed from the collection that owns them.
type Written<T> = Entry<PageMeta<<T as Theme>::Layout, <T as Theme>::Extra>>;

/// One page of a site, however it came to be: read from `content/` or
/// declared in `site.toml` and rendered by the site itself.
type Pages<'a, T> = Vec<&'a Written<T>>;

/// A page that has been rendered but not yet written: the entry it came from,
/// the URL it will be served at, and its HTML.
type Rendering<'a, T> = (
    &'a Entry<PageMeta<<T as Theme>::Layout, <T as Theme>::Extra>>,
    Url,
    String,
);

/// Where a build spent its time.
///
/// Four passes, measured on a monotonic clock, that between them account for
/// nearly all of a build: everything else is a handful of `String`s.  They are
/// reported rather than logged because a build budget belongs in the caller's
/// CI, not in a flag on this crate.
///
/// The four do not sum exactly to wall-clock time -- the small amount of work
/// between them is not attributed to anything -- so [`total`](Timings::total)
/// is the sum of the parts and not a measurement of the whole.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Timings {
    /// Reading the configuration, the data files and the content, including
    /// parsing every page's Markdown and highlighting its code.
    pub read: Duration,
    /// Handing each page to the design system, and drawing any social cards.
    pub render: Duration,
    /// Resolving every internal link against the finished set of routes.
    pub check: Duration,
    /// Writing the pages, the assets, the feeds and the static tree.
    pub write: Duration,
}

impl Timings {
    /// The sum of the four passes.
    #[must_use]
    pub fn total(&self) -> Duration {
        self.read + self.render + self.check + self.write
    }
}

/// What a build produced, for the benefit of whoever asked for it.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The routes written, in the order they were written.
    pub routes: Vec<String>,
    /// The directory the site was written to.
    pub out_dir: PathBuf,
    /// Pages skipped because they were marked as drafts.
    pub drafts: usize,
    /// How many internal links were resolved and found to exist.
    pub links: usize,
    /// Anything the build wants the reader to know but not to stop for.
    pub warnings: Vec<String>,
    /// Where the build spent its time.
    pub timings: Timings,
}

impl Report {
    /// The one line worth printing when everything went well.
    ///
    /// Pluralisation lives here rather than in whatever printed it, because
    /// otherwise every caller reimplements it and one of them gets it wrong.
    pub fn headline(&self) -> String {
        format!(
            "{} {}, {} {} checked -> {}",
            self.routes.len(),
            plural(self.routes.len(), "page", "pages"),
            self.links,
            plural(self.links, "link", "links"),
            self.out_dir.display()
        )
    }

    /// The `drafts` line, if there were any.
    pub fn drafts_line(&self) -> Option<String> {
        (self.drafts > 0).then(|| {
            format!(
                "{} {} skipped",
                self.drafts,
                plural(self.drafts, "draft", "drafts")
            )
        })
    }
}

/// Everything the build has to say, in the order it should be said.
///
/// `Display` is complete on purpose.  A report carries warnings and a draft
/// count, and both are easy to have and easy to forget to print -- so
/// `println!("{report}")`, which is what anybody writes first, says all of it.
/// A caller that wants the warnings on stderr can still walk the fields.
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.headline())?;

        if let Some(line) = self.drafts_line() {
            write!(f, "\n{line}")?;
        }

        for warning in &self.warnings {
            write!(f, "\nwarning: {warning}")?;
        }

        Ok(())
    }
}

fn plural<'a>(count: usize, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

/// Overrides for one invocation, which the configuration file does not know
/// about and should not be edited to express.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Options {
    /// Write somewhere other than `dist/`.
    pub out_dir: Option<PathBuf>,
    /// Serve from somewhere other than the configured base path.
    pub base: Option<String>,
    /// Bodies for the pages the site renders itself, by route.
    ///
    /// A site declares such a page in `site.toml` under `[[page]]` -- which is
    /// what gives it a title, a layout and a place in the navigation, and what
    /// lets `build.rs` put it in the generated `Route` enum -- and supplies
    /// its body here.
    ///
    /// Declared and not supplied is an error, and so is the reverse: a page
    /// half-added is worse than one not added.
    pub pages: BTreeMap<String, String>,
    /// Directives the site registers itself.
    ///
    /// A component usually belongs to a design system, but not always: a
    /// `terminal` that reads recordings the site generates belongs to the
    /// site, and no design system should have to know about it.
    ///
    /// A name the design system already uses is an error, because a site
    /// silently replacing a component would change every page that uses it
    /// without a word.
    pub directives: Directives,
}

impl Options {
    /// No overrides, which is what `build` uses.
    pub fn new() -> Self {
        Options::default()
    }

    /// Write the site somewhere other than `dist/`.
    pub fn out_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.out_dir = Some(dir.into());
        self
    }

    /// Serve from somewhere other than the configured base path.
    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    /// Serve from an override that may not be there, which is the shape a
    /// command line argument arrives in.
    pub fn maybe_base(mut self, base: Option<impl Into<String>>) -> Self {
        self.base = base.map(Into::into);
        self
    }

    /// Supply every declared page's body at once.
    pub fn pages(mut self, pages: BTreeMap<String, String>) -> Self {
        self.pages = pages;
        self
    }

    /// Supply the body of a page the site declared in `site.toml`.
    pub fn page(mut self, route: impl Into<String>, body: impl Into<String>) -> Self {
        self.pages.insert(route.into(), body.into());
        self
    }

    /// Register directives of the site's own, alongside its design system's.
    pub fn directives(mut self, directives: Directives) -> Self {
        self.directives = directives;
        self
    }
}

/// Builds the site rooted at `site_dir` into `site_dir/dist`.
///
/// # Errors
///
/// Fails if the configuration or any content file is unreadable or invalid, if
/// any page links somewhere that does not exist, or if the output cannot be
/// written.
pub fn build<T: Theme>(site_dir: &Path, theme: &T) -> Result<Report> {
    build_with(site_dir, theme, Options::default())
}

/// Builds the site, with overrides.
///
/// # Errors
///
/// As [`build`].
pub fn build_with<T: Theme>(site_dir: &Path, theme: &T, options: Options) -> Result<Report> {
    let mut timings = Timings::default();
    let clock = Instant::now();

    let mut config = Config::load(site_dir)?;
    if let Some(base) = &options.base {
        config.set_base(base);
    }

    // Read before any content, because a directive may ask for it while a page
    // is being loaded.
    let data = crate::directive::Data::load(&site_dir.join("data"))?;
    let context = crate::directive::Context::new(&config, &data);

    let mut directives = theme.directives();
    directives
        .merge(options.directives)
        .map_err(|message| Error::Schema {
            path: site_dir.join("site.toml"),
            message,
            snippet: None,
        })?;

    let content = Collection::<PageMeta<T::Layout, T::Extra>>::load(
        &site_dir.join("content"),
        &directives,
        &context,
    )?;
    let out_dir = match &options.out_dir {
        Some(dir) => dir.clone(),
        None => site_dir.join("dist"),
    };

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }

    // Pages the site rendered itself, checked against what it declared and
    // then treated exactly like content.
    let declared = declared_pages::<T>(site_dir, &config, &options.pages)?;
    timings.read = clock.elapsed();

    // A route claimed twice is one of them silently winning, which is exactly
    // the class of failure this project exists to refuse.
    for page in &declared {
        if content
            .entries()
            .iter()
            .any(|entry| entry.route == page.route)
        {
            return Err(Error::Collides {
                path: site_dir.join("site.toml"),
                route: page.route.clone(),
            });
        }
    }

    let published: Pages<'_, T> = content
        .entries()
        .iter()
        .chain(declared.iter())
        .filter(|entry| !entry.meta.draft)
        .collect();

    let drafts = content.len() + declared.len() - published.len();

    // The error page is a page, but it is not a route: nothing may link to it,
    // it is not in the navigation, and it is written as a file rather than as
    // a directory, because that is what a static host looks for.
    let (error_page, live): (Pages<'_, T>, Pages<'_, T>) = published
        .into_iter()
        .partition(|entry| entry.route == ERROR_ROUTE);

    let index = SiteIndex::new(
        live.iter()
            .map(|entry| PageRef {
                route: entry.route.clone(),
                label: entry.meta.label().to_string(),
                nav_order: entry.meta.nav_order,
                order: entry.meta.order,
            })
            .collect(),
    );

    // Rendering and checking are separate passes: a link is only dead relative
    // to the finished set of routes, so nothing can be judged until every page
    // is known.
    let mut warnings = Vec::new();

    // Feeds need absolute URLs, so a site with no `url` gets none and is told
    // rather than handed a feed full of relative links.
    let feeds: Vec<FeedLink> = match &config.url {
        Some(_) => config
            .feeds
            .iter()
            .map(|feed| {
                let (rss, atom) = feed.paths();
                FeedLink {
                    title: feed.title.clone().unwrap_or_else(|| config.title.clone()),
                    rss: Url::asset(&config, &rss),
                    atom: Url::asset(&config, &atom),
                }
            })
            .collect(),
        None => {
            if !config.feeds.is_empty() {
                warnings.push(
                    "site.toml configures a feed but has no url; \
                     a feed needs absolute URLs, so none was written"
                        .to_string(),
                );
            }
            Vec::new()
        }
    };

    // Configuring a feed is how a site says a collection is dated.  This is
    // the build holding it to that, before anything is written.
    if !feeds.is_empty() {
        for entry in &live {
            let covered = config.feeds.iter().find(|feed| feed.covers(&entry.route));

            if let Some(feed) = covered
                && entry.meta.date.is_none()
            {
                return Err(Error::Undated {
                    path: entry.path.clone(),
                    collection: feed.collection.clone(),
                });
            }
        }
    }

    let cache = site_dir.join(CACHE);
    let clock = Instant::now();
    let mut cards: Vec<(String, Vec<u8>)> = Vec::new();
    let mut rendered: Vec<Rendering<'_, T>> = Vec::with_capacity(live.len() + error_page.len());
    for entry in live.iter().chain(error_page.iter()) {
        // A card is only ever referenced from an absolute URL, so a site
        // without one is not asked for any.
        let card_url = config
            .url
            .as_ref()
            .map(|_| Url::asset(&config, &card_path(&entry.route)));

        let page = Page {
            config: &config,
            meta: &entry.meta,
            route: &entry.route,
            html: &entry.html,
            headings: &entry.headings,
            site: &index,
            feeds: &feeds,
            cache: &cache,
            card: card_url.as_ref(),
        };

        if card_url.is_some()
            && let Some(png) = theme.og_image(&page)
        {
            cards.push((card_path(&entry.route), png));
        }

        // The error page is served in place of any path, so it is checked as
        // though it sat at the site root.
        let url = if entry.route == ERROR_ROUTE {
            Url::new(&config, "")
        } else {
            Url::new(&config, &entry.route)
        };

        // `~/` means the site root, whatever the base is.  Resolved here, on
        // the finished page, so it works in content and in components alike
        // and nothing downstream has to know the convention exists.
        rendered.push((
            *entry,
            url,
            links::absolutize(&theme.render(&page), &config),
        ));
    }

    timings.render = clock.elapsed();

    // What the site asked for, and what the design system actually has.  A
    // mismatch is worth saying and not worth stopping for: the page still
    // works, it just has less in it than the configuration implies.
    let router = declined(config.router, theme.router_js(), "router", &mut warnings);
    let search = declined(config.search, theme.search_js(), "search", &mut warnings);

    let mut assets: BTreeSet<String> = BTreeSet::new();
    assets.insert("site.css".to_string());
    if router.is_some() {
        assets.insert("router.js".to_string());
    }
    if config.url.is_some() {
        assets.insert("sitemap.xml".to_string());
        assets.insert("robots.txt".to_string());
    }
    if search.is_some() {
        assets.insert("search.json".to_string());
        assets.insert("search.js".to_string());
    }
    for declared in &config.feeds {
        if !feeds.is_empty() {
            let (rss, atom) = declared.paths();
            assets.insert(rss);
            assets.insert(atom);
        }
    }

    for (path, _) in &cards {
        assets.insert(path.clone());
    }

    assets.extend(tree(&site_dir.join("static"))?);

    // Route to heading ids, so a fragment link can be checked against the page
    // it points at rather than merely against that page existing.
    let routes: links::Routes = live
        .iter()
        .map(|entry| (entry.route.trim_matches('/').to_string(), entry.anchors()))
        .collect();

    let anchors: Vec<BTreeSet<String>> = rendered
        .iter()
        .map(|(entry, _, _)| entry.anchors())
        .collect();

    let pages: Vec<Rendered<'_>> = rendered
        .iter()
        .zip(&anchors)
        .map(|((entry, url, html), anchors)| Rendered {
            source: &entry.path,
            url: url.as_str(),
            anchors,
            error_page: entry.route == ERROR_ROUTE,
            html,
        })
        .collect();

    let clock = Instant::now();
    let checked = links::check(&config, &pages, &routes, &assets);
    timings.check = clock.elapsed();

    if !checked.dead.is_empty() {
        return Err(Error::DeadLinks(checked.dead));
    }

    // A client the build writes and no page loads is the opposite of a dead
    // link, and just as broken: the feature is configured, the file is
    // shipped, and nothing happens.  It is a warning rather than an error
    // because a design system is allowed to load its own scripts some other
    // way -- but the silence is worth breaking.
    warnings.extend(unloaded("router.js", router.is_some(), &checked.referenced));
    warnings.extend(unloaded("search.js", search.is_some(), &checked.referenced));

    let clock = Instant::now();
    for (entry, _, html) in &rendered {
        write(&page_path(&out_dir, &entry.route), html)?;
    }

    for (path, png) in &cards {
        write_bytes(&out_dir.join(path), png)?;
    }

    write(&out_dir.join("site.css"), &theme.stylesheet())?;
    if let Some(client) = &router {
        write(&out_dir.join("router.js"), client)?;
    }

    if let Some(client) = &search {
        let documents: Vec<search::Document<'_>> = rendered
            .iter()
            .filter(|(entry, _, _)| entry.route != ERROR_ROUTE)
            .map(|(entry, url, _)| search::Document {
                url: url.as_str(),
                title: entry.meta.title.as_str(),
                headings: &entry.headings,
                text: search::plain(&entry.html),
            })
            .collect();

        let index = search::index(&documents);

        warnings.extend(search::outgrown(index.len()));

        write(&out_dir.join("search.json"), &index)?;
        write(&out_dir.join("search.js"), client)?;
    }

    if let Some(url) = &config.url {
        write(&out_dir.join("sitemap.xml"), &sitemap(url, &config, &live))?;
        write(&out_dir.join("robots.txt"), &robots(url, &config))?;

        for declared in &config.feeds {
            let (rss_path, atom_path) = declared.paths();
            let title = declared
                .title
                .clone()
                .unwrap_or_else(|| config.title.clone());
            let home = origin(url).to_string() + &relative(&config, "");

            let mut items: Vec<(&Datetime, Item<'_>)> = live
                .iter()
                .filter(|entry| declared.covers(&entry.route))
                .filter_map(|entry| {
                    entry.meta.date.as_ref().map(|date| {
                        (
                            date,
                            Item {
                                title: &entry.meta.title,
                                url: origin(url).to_string() + &relative(&config, &entry.route),
                                date,
                                html: &entry.html,
                            },
                        )
                    })
                })
                .collect();

            // Newest first, and by route where two pages share a date, so a
            // rebuild produces the same file.
            items.sort_by(|(a, left), (b, right)| {
                b.to_string()
                    .cmp(&a.to_string())
                    .then_with(|| left.url.cmp(&right.url))
            });

            let items: Vec<Item<'_>> = items
                .into_iter()
                .map(|(_, item)| item)
                .take(declared.limit)
                .collect();

            let self_rss = origin(url).to_string() + &relative_asset(&config, &rss_path);
            let self_atom = origin(url).to_string() + &relative_asset(&config, &atom_path);

            write(
                &out_dir.join(&rss_path),
                &feed::rss(&title, &config.description, &home, &self_rss, &items),
            )?;
            write(
                &out_dir.join(&atom_path),
                &feed::atom(&title, &home, &self_atom, &items),
            )?;
        }
    }

    // GitHub Pages otherwise runs the output through Jekyll, which drops any
    // file or directory whose name begins with an underscore.
    write(&out_dir.join(".nojekyll"), "")?;

    copy_tree(&site_dir.join("static"), &out_dir)?;
    timings.write = clock.elapsed();

    Ok(Report {
        routes: live.iter().map(|entry| entry.route.clone()).collect(),
        out_dir,
        drafts,
        links: checked.examined,
        warnings,
        timings,
    })
}

/// Says so if a client was written and no page loads it.
fn unloaded(
    file: &str,
    written: bool,
    referenced: &std::collections::BTreeSet<String>,
) -> Option<String> {
    (written && !referenced.contains(file)).then(|| {
        format!(
            "{file} was written but no page loads it; \
             the design system's document template is missing its script tag"
        )
    })
}

/// Reconciles what the site asked for with what the design system has.
///
/// Returns the client to write, if there is one to write, and records a
/// warning when the site asked for something its design system does not offer.
fn declined(
    asked: bool,
    offered: Option<String>,
    what: &str,
    warnings: &mut Vec<String>,
) -> Option<String> {
    match (asked, offered) {
        (true, Some(client)) => Some(client),
        (true, None) => {
            warnings.push(format!(
                "site.toml asks for the {what}, but this design system has none;                  the site is built without it"
            ));
            None
        }
        (false, _) => None,
    }
}

/// The site's absolute root, without a trailing slash.
///
/// `url` is the absolute URL of the site root, base path included, so a
/// project site's is `https://you.github.io/repo/`.  Everything absolute is
/// built by appending a site-relative path to it, which is why the base must
/// not be added a second time.
fn origin(url: &str) -> &str {
    url.trim_end_matches('/')
}

/// A site-relative asset path, with the base stripped off.
fn relative_asset(config: &Config, path: &str) -> String {
    let url = Url::asset(config, path);
    let base = config.base.trim_end_matches('/');
    url.as_str()
        .strip_prefix(base)
        .unwrap_or(url.as_str())
        .to_string()
}

/// A site-relative path, with the base stripped off.
fn relative(config: &Config, route: &str) -> String {
    let url = Url::new(config, route);
    let base = config.base.trim_end_matches('/');
    url.as_str()
        .strip_prefix(base)
        .unwrap_or(url.as_str())
        .to_string()
}

/// Every live route, as absolute URLs.
///
/// No `lastmod`: there is no honest source for one yet, and a fabricated
/// timestamp is worse than an absent field.
fn sitemap<L, E>(url: &str, config: &Config, live: &[&Entry<PageMeta<L, E>>]) -> String {
    let origin = origin(url);
    let mut out = String::new();

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    for entry in live {
        out.push_str(&format!(
            "  <url><loc>{origin}{}</loc></url>\n",
            relative(config, &entry.route)
        ));
    }

    out.push_str("</urlset>\n");
    out
}

/// Allow everything, and say where the sitemap is.
fn robots(url: &str, config: &Config) -> String {
    let _ = config;
    format!(
        "User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n",
        origin(url)
    )
}

/// Where a route's output goes.
///
/// Every route is a directory holding an `index.html`, so every URL ends in a
/// slash.  The error page is the one exception, because a static host looks
/// for `404.html` by that name.
fn page_path(out_dir: &Path, route: &str) -> PathBuf {
    match route {
        "" => out_dir.join("index.html"),
        ERROR_ROUTE => out_dir.join("404.html"),
        route => out_dir.join(route).join("index.html"),
    }
}

/// Writes a text file, creating its parent directories.
///
/// Line endings are normalised to line feeds.  A design system's stylesheet
/// and clients arrive here through `include_str!`, so on a checkout with CRLF
/// they carry carriage returns and the built site differs byte for byte from
/// the same site built anywhere else.  A build should produce the same output
/// on every machine, and this is most of what that costs.
fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }

    let normalised;
    let contents = if contents.contains('\r') {
        normalised = contents.replace("\r\n", "\n");
        normalised.as_str()
    } else {
        contents
    };

    fs::write(path, contents).map_err(|source| Error::io(path, source))
}

/// Writes bytes, creating parent directories.
fn write_bytes(path: &Path, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    fs::write(path, contents).map_err(|source| Error::io(path, source))
}

/// Every file below `dir`, as site-relative paths.  An absent directory is not
/// an error: a site is allowed to have no static assets.
fn tree(dir: &Path) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|err| {
            let path = err.path().unwrap_or(dir).to_path_buf();
            Error::io(path, err.into())
        })?;

        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(dir).unwrap_or(entry.path());
            found.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    Ok(found)
}

/// Copies `from` into `into`, doing nothing if `from` does not exist.
fn copy_tree(from: &Path, into: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }

    for found in walkdir::WalkDir::new(from) {
        let found = found.map_err(|err| {
            let path = err.path().unwrap_or(from).to_path_buf();
            Error::io(path, err.into())
        })?;

        let relative = found.path().strip_prefix(from).unwrap_or(found.path());
        let target = into.join(relative);

        if found.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|source| Error::io(&target, source))?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
            }
            fs::copy(found.path(), &target).map_err(|source| Error::io(&target, source))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{card_path, page_path, relative, robots};
    use crate::config::Config;

    fn config(base: &str) -> Config {
        {
            let mut config = Config::blank();
            config.title = "t".into();
            config.description = String::new();
            config.url = Some("https://example.com/".into());
            config.base = base.into();
            config
        }
    }

    #[test]
    fn a_card_sits_where_both_halves_can_find_it() {
        assert_eq!(card_path(""), "og/index.png");
        assert_eq!(card_path("docs/routing"), "og/docs/routing.png");
    }

    #[test]
    fn every_route_becomes_a_directory_with_an_index() {
        let out = Path::new("dist");
        assert_eq!(page_path(out, ""), Path::new("dist/index.html"));
        assert_eq!(page_path(out, "about"), Path::new("dist/about/index.html"));
        assert_eq!(
            page_path(out, "docs/intro"),
            Path::new("dist/docs/intro/index.html")
        );
    }

    #[test]
    fn the_error_page_is_a_file_because_that_is_what_hosts_look_for() {
        assert_eq!(
            page_path(Path::new("dist"), "404"),
            Path::new("dist/404.html")
        );
    }

    #[test]
    fn robots_points_at_the_sitemap_beside_the_site_root() {
        let text = robots("https://example.com/repo/", &config("/repo/"));
        assert!(
            text.contains("Sitemap: https://example.com/repo/sitemap.xml"),
            "got {text}"
        );
    }

    #[test]
    fn absolute_paths_do_not_repeat_the_base() {
        // `url` already contains the base, so appending it again is the bug
        // this test exists to prevent.
        let config = config("/repo/");
        assert_eq!(relative(&config, ""), "/");
        assert_eq!(relative(&config, "docs/intro"), "/docs/intro/");
    }
}
