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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::content::{Collection, Entry};
use crate::error::{Error, Result};
use crate::links::{self, Rendered};
use crate::theme::{Page, PageMeta, PageRef, SiteIndex, Theme};
use crate::url::Url;

/// The route reserved for the error page.
///
/// GitHub Pages, and most static hosts, serve `404.html` from the site root in
/// place of anything they cannot find.
const ERROR_ROUTE: &str = "404";

/// What a build produced, for the benefit of whoever asked for it.
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
}

/// Overrides for one invocation, which the configuration file does not know
/// about and should not be edited to express.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Write somewhere other than `dist/`.
    pub out_dir: Option<PathBuf>,
    /// Serve from somewhere other than the configured base path.
    pub base: Option<String>,
}

/// Builds the site rooted at `site_dir` into `site_dir/dist`.
///
/// # Errors
///
/// Fails if the configuration or any content file is unreadable or invalid, if
/// any page links somewhere that does not exist, or if the output cannot be
/// written.
pub fn build<T: Theme>(site_dir: &Path, theme: &T) -> Result<Report> {
    build_with(site_dir, theme, &Options::default())
}

/// Builds the site, with overrides.
///
/// # Errors
///
/// As [`build`].
pub fn build_with<T: Theme>(site_dir: &Path, theme: &T, options: &Options) -> Result<Report> {
    let mut config = Config::load(site_dir)?;
    if let Some(base) = &options.base {
        config.set_base(base);
    }

    let content = Collection::<PageMeta<T::Layout>>::load(&site_dir.join("content"))?;
    let out_dir = match &options.out_dir {
        Some(dir) => dir.clone(),
        None => site_dir.join("dist"),
    };

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }

    let published: Vec<&Entry<PageMeta<T::Layout>>> = content
        .entries()
        .iter()
        .filter(|entry| !entry.meta.draft)
        .collect();
    let drafts = content.len() - published.len();

    // The error page is a page, but it is not a route: nothing may link to it,
    // it is not in the navigation, and it is written as a file rather than as
    // a directory, because that is what a static host looks for.
    let (error_page, live): (
        Vec<&Entry<PageMeta<T::Layout>>>,
        Vec<&Entry<PageMeta<T::Layout>>>,
    ) = published
        .into_iter()
        .partition(|entry| entry.route == ERROR_ROUTE);

    let index = SiteIndex::new(
        live.iter()
            .map(|entry| PageRef {
                route: entry.route.clone(),
                label: entry.meta.label().to_string(),
                nav_order: entry.meta.nav_order,
            })
            .collect(),
    );

    // Rendering and checking are separate passes: a link is only dead relative
    // to the finished set of routes, so nothing can be judged until every page
    // is known.
    let mut rendered = Vec::with_capacity(live.len() + error_page.len());
    for entry in live.iter().chain(error_page.iter()) {
        let page = Page {
            config: &config,
            meta: &entry.meta,
            route: &entry.route,
            html: &entry.html,
            site: &index,
        };

        // The error page is served in place of any path, so it is checked as
        // though it sat at the site root.
        let url = if entry.route == ERROR_ROUTE {
            Url::new(&config, "")
        } else {
            Url::new(&config, &entry.route)
        };

        rendered.push((*entry, url, theme.render(&page)));
    }

    let mut assets: BTreeSet<String> = BTreeSet::new();
    assets.insert("site.css".to_string());
    if config.router {
        assets.insert("router.js".to_string());
    }
    if config.url.is_some() {
        assets.insert("sitemap.xml".to_string());
        assets.insert("robots.txt".to_string());
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

    let checked = links::check(&config, &pages, &routes, &assets);
    if !checked.dead.is_empty() {
        return Err(Error::DeadLinks(checked.dead));
    }

    for (entry, _, html) in &rendered {
        write(&page_path(&out_dir, &entry.route), html)?;
    }

    write(&out_dir.join("site.css"), &theme.stylesheet())?;
    if config.router {
        write(&out_dir.join("router.js"), theme.router_js())?;
    }

    if let Some(url) = &config.url {
        write(&out_dir.join("sitemap.xml"), &sitemap(url, &config, &live))?;
        write(&out_dir.join("robots.txt"), &robots(url, &config))?;
    }

    // GitHub Pages otherwise runs the output through Jekyll, which drops any
    // file or directory whose name begins with an underscore.
    write(&out_dir.join(".nojekyll"), "")?;

    copy_tree(&site_dir.join("static"), &out_dir)?;

    Ok(Report {
        routes: live.iter().map(|entry| entry.route.clone()).collect(),
        out_dir,
        drafts,
        links: checked.examined,
    })
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
fn sitemap<T>(url: &str, config: &Config, live: &[&Entry<PageMeta<T>>]) -> String {
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

/// Writes a file, creating its parent directories.
fn write(path: &Path, contents: &str) -> Result<()> {
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

    use super::{page_path, relative, robots};
    use crate::config::Config;

    fn config(base: &str) -> Config {
        Config {
            title: "t".into(),
            description: String::new(),
            url: Some("https://example.com/".into()),
            base: base.into(),
            router: false,
        }
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
