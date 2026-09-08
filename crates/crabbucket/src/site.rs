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
use crate::content::Collection;
use crate::error::{Error, Result};
use crate::links::{self, Rendered};
use crate::theme::{Page, PageMeta, PageRef, SiteIndex, Theme};
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
    /// How many internal links were resolved and found to exist.
    pub links: usize,
}

/// Builds the site rooted at `site_dir` into `site_dir/dist`.
///
/// # Errors
///
/// Fails if the configuration or any content file is unreadable or invalid, if
/// any page links somewhere that does not exist, or if the output cannot be
/// written.
pub fn build<T: Theme>(site_dir: &Path, theme: &T) -> Result<Report> {
    let config = Config::load(site_dir)?;
    let content = Collection::<PageMeta<T::Layout>>::load(&site_dir.join("content"))?;
    let out_dir = site_dir.join("dist");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }

    let live: Vec<_> = content
        .entries()
        .iter()
        .filter(|entry| !entry.meta.draft)
        .collect();
    let drafts = content.len() - live.len();

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
    let mut rendered = Vec::with_capacity(live.len());
    for entry in &live {
        let page = Page {
            config: &config,
            meta: &entry.meta,
            route: &entry.route,
            html: &entry.html,
            site: &index,
        };

        rendered.push((entry, Url::new(&config, &entry.route), theme.render(&page)));
    }

    let mut assets: BTreeSet<String> = BTreeSet::new();
    assets.insert("site.css".to_string());
    if config.router {
        assets.insert("router.js".to_string());
    }
    assets.extend(tree(&site_dir.join("static"))?);

    let routes: BTreeSet<String> = live
        .iter()
        .map(|entry| entry.route.trim_matches('/').to_string())
        .collect();

    let pages: Vec<Rendered<'_>> = rendered
        .iter()
        .map(|(entry, url, html)| Rendered {
            source: &entry.path,
            url: url.as_str(),
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

    // GitHub Pages otherwise runs the output through Jekyll, which drops any
    // file or directory whose name begins with an underscore.
    write(&out_dir.join(".nojekyll"), "")?;

    copy_tree(&site_dir.join("static"), &out_dir)?;

    Ok(Report {
        routes: rendered
            .iter()
            .map(|(entry, _, _)| entry.route.clone())
            .collect(),
        out_dir,
        drafts,
        links: checked.examined,
    })
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
