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
//! content as a typed collection, hand each page to the theme, and write the
//! result as directories of `index.html` so that every route ends in a slash
//! and works on a plain static host.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::content::Collection;
use crate::error::{Error, Result};
use crate::theme::{NavItem, Page, PageMeta, Theme};
use crate::url::Url;

/// What a build produced, for the benefit of whoever asked for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The routes written, in the order they were written.
    pub routes: Vec<String>,
    /// The directory the site was written to.
    pub out_dir: PathBuf,
    /// Pages skipped because they were marked as drafts.
    pub drafts: usize,
}

/// Builds the site rooted at `site_dir` into `site_dir/dist`.
///
/// # Errors
///
/// Fails if the configuration or any content file is unreadable or invalid, or
/// if the output cannot be written.
pub fn build(site_dir: &Path, theme: &dyn Theme) -> Result<Report> {
    let config = Config::load(site_dir)?;
    let content = Collection::<PageMeta>::load(&site_dir.join("content"))?;
    let out_dir = site_dir.join("dist");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }

    let nav = navigation(&config, &content);
    let mut routes = Vec::new();
    let mut drafts = 0;

    for entry in &content {
        if entry.meta.draft {
            drafts += 1;
            continue;
        }

        let nav = mark_current(&config, &nav, &entry.route);
        let page = Page {
            config: &config,
            meta: &entry.meta,
            route: &entry.route,
            html: &entry.html,
            nav: &nav,
        };

        write(&page_path(&out_dir, &entry.route), &theme.render(&page))?;
        routes.push(entry.route.clone());
    }

    write(&out_dir.join("site.css"), &theme.stylesheet())?;
    if config.router {
        write(&out_dir.join("router.js"), theme.router_js())?;
    }

    // GitHub Pages otherwise runs the output through Jekyll, which drops any
    // file or directory whose name begins with an underscore.
    write(&out_dir.join(".nojekyll"), "")?;

    copy_tree(&site_dir.join("static"), &out_dir)?;

    Ok(Report {
        routes,
        out_dir,
        drafts,
    })
}

/// Collects the pages that asked to be in the primary navigation.
fn navigation(config: &Config, content: &Collection<PageMeta>) -> Vec<NavItem> {
    let mut listed: Vec<(u32, NavItem)> = content
        .entries()
        .iter()
        .filter(|entry| !entry.meta.draft)
        .filter_map(|entry| {
            entry.meta.nav_order.map(|order| {
                (
                    order,
                    NavItem {
                        label: entry.meta.label().to_string(),
                        href: Url::new(config, &entry.route),
                        current: false,
                    },
                )
            })
        })
        .collect();

    listed.sort_by(|(a, left), (b, right)| a.cmp(b).then_with(|| left.label.cmp(&right.label)));
    listed.into_iter().map(|(_, item)| item).collect()
}

/// Copies the navigation with the entry for `route` marked as current.
///
/// The comparison is between two [`Url`]s rather than between strings, so the
/// base path is applied to both sides or to neither.
fn mark_current(config: &Config, nav: &[NavItem], route: &str) -> Vec<NavItem> {
    let here = Url::new(config, route);
    nav.iter()
        .map(|item| NavItem {
            label: item.label.clone(),
            href: item.href.clone(),
            current: item.href == here,
        })
        .collect()
}

/// Where a route's `index.html` goes.
fn page_path(out_dir: &Path, route: &str) -> PathBuf {
    if route.is_empty() {
        out_dir.join("index.html")
    } else {
        out_dir.join(route).join("index.html")
    }
}

/// Writes a file, creating its parent directories.
fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    fs::write(path, contents).map_err(|source| Error::io(path, source))
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

    use super::page_path;

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
}
