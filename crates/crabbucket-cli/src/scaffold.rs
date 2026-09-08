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
//! Scaffolding a new site.
//!
//! What `crab new` writes is a site that builds, passes link checking and
//! deploys, with nothing left as an exercise.  A scaffold that needs edits
//! before its first build is a scaffold that has moved the documentation into
//! the filesystem.
//!
//! The templates are `include_str!`-ed from this crate rather than read from a
//! data directory at runtime, so an installed `crab` is one file.

use std::fs;
use std::path::{Path, PathBuf};

/// What was asked for.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Request {
    /// Where to write it.
    pub dir: PathBuf,
    /// The path the site will be served from.  Defaults to `/NAME/`, because a
    /// project site is the common case and a user site is the one worth typing
    /// a flag for.
    pub base: Option<String>,
    /// Whether to ship the client router.
    pub router: bool,
    /// Whether to write into a directory that is not empty.
    pub force: bool,
}

/// Writes the scaffold, returning how many files were created.
///
/// # Errors
///
/// Refuses a directory that already has anything in it unless `force` is set.
/// Scaffolding over someone's files is the kind of thing a tool gets one
/// chance at.
pub fn write(request: &Request) -> Result<usize, String> {
    let dir = &request.dir;

    if dir.is_dir()
        && !request.force
        && fs::read_dir(dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
    {
        return Err(format!(
            "{}: directory is not empty; pass --force to scaffold into it anyway",
            dir.display()
        ));
    }

    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty() && name != ".")
        .unwrap_or_else(|| "site".to_string());

    let base = request.base.clone().unwrap_or_else(|| format!("/{name}/"));
    let router = request.router;

    let files: Vec<(&str, String)> = vec![
        ("site.toml", site_toml(&name, &base, router)),
        (
            "content/index.md",
            include_str!("templates/index.md").to_string(),
        ),
        (
            "content/docs/index.md",
            include_str!("templates/docs.md").to_string(),
        ),
        (
            "content/404.md",
            include_str!("templates/404.md").to_string(),
        ),
        ("static/.gitkeep", String::new()),
        (
            "turborust.toml",
            include_str!("templates/turborust.toml").to_string(),
        ),
        (".github/workflows/pages.yml", pages_workflow()),
        (".gitignore", "/dist\n".to_string()),
    ];

    for (path, contents) in &files {
        put(&dir.join(path), contents)?;
    }

    Ok(files.len())
}

fn site_toml(name: &str, base: &str, router: bool) -> String {
    include_str!("templates/site.toml")
        .replace("@NAME@", name)
        .replace("@BASE@", base)
        .replace("@ROUTER@", if router { "true" } else { "false" })
}

fn pages_workflow() -> String {
    include_str!("templates/pages.yml").to_string()
}

fn put(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }

    fs::write(path, contents).map_err(|err| format!("{}: {err}", path.display()))
}
