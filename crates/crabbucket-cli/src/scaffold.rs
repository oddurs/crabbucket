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
//! There are two shapes, because there are two kinds of site.  A site with no
//! design system of its own is a content directory that `crab build` renders.
//! A site with one is a crate: a design system is a crate you depend on, so a
//! site that depends on one is a program that calls `crabbucket::build`.
//! `--theme` chooses the second.
//!
//! Getting that wrong is not cosmetic.  The two shapes need different build
//! commands and different watch inputs, and a content-shaped `turborust.toml`
//! in a crate-shaped site rebuilds nothing when the Rust changes.
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
    /// The design system this site depends on, as a crate name or a git URL.
    ///
    /// Naming one makes the site a crate rather than a content directory,
    /// because a site that depends on a design system has to call the build
    /// itself.
    pub theme: Option<String>,
}

/// A design system a scaffolded site depends on.
#[derive(Debug, PartialEq, Eq)]
struct Theme {
    /// The crate name.
    name: String,
    /// The dependency line for `Cargo.toml`.
    dependency: String,
    /// What to say about pinning it.
    pin: &'static str,
}

impl Theme {
    /// Reads a `--theme` argument.
    ///
    /// Anything with a scheme is a git dependency; anything else is a crate
    /// name.  A git dependency needs no version, and a crates.io one is left
    /// unpinned with a comment saying so -- guessing a version that does not
    /// exist would be worse than saying nothing.
    fn parse(given: &str) -> Self {
        let given = given.trim();

        if given.contains("://") {
            let name = given
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(given)
                .trim_end_matches(".git")
                .to_string();

            return Theme {
                dependency: format!("{name} = {{ git = \"{given}\" }}"),
                name,
                pin: "Pin it to a tag or a rev before you rely on it.",
            };
        }

        Theme {
            name: given.to_string(),
            dependency: format!("{given} = \"*\""),
            pin: "Replace `*` with a real version requirement.",
        }
    }

    /// The crate name as a Rust identifier.
    fn ident(&self) -> String {
        self.name.replace('-', "_")
    }
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

    let theme = request.theme.as_deref().map(Theme::parse);

    let mut files: Vec<(&str, String)> = vec![
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
    ];

    files.extend(match &theme {
        Some(theme) => crate_files(&name, theme),
        None => content_files(),
    });

    for (path, contents) in &files {
        put(&dir.join(path), contents)?;
    }

    Ok(files.len())
}

/// What a content-only site needs beyond its content.
fn content_files() -> Vec<(&'static str, String)> {
    vec![
        (
            "turborust.toml",
            include_str!("templates/turborust.toml").to_string(),
        ),
        (
            ".github/workflows/pages.yml",
            include_str!("templates/pages.yml").to_string(),
        ),
        (".gitignore", "/dist\n".to_string()),
    ]
}

/// What a site with its own design system needs instead.
///
/// It is a crate, so it has a manifest, a `main` that calls the build, a
/// `turborust.toml` that watches Rust as well as content, and a workflow that
/// runs `cargo run` rather than installing `crab`.
fn crate_files(name: &str, theme: &Theme) -> Vec<(&'static str, String)> {
    vec![
        (
            "Cargo.toml",
            include_str!("templates/crate-cargo.toml")
                .replace("@NAME@", name)
                .replace("@THEME_DEP@", &theme.dependency)
                .replace("@PIN@", theme.pin),
        ),
        (
            "src/main.rs",
            include_str!("templates/crate-main.rs")
                .replace("@NAME@", name)
                .replace("@THEME_IDENT@", &theme.ident()),
        ),
        (
            "turborust.toml",
            include_str!("templates/crate-turborust.toml").replace("@NAME@", name),
        ),
        (
            ".github/workflows/pages.yml",
            include_str!("templates/crate-pages.yml").to_string(),
        ),
        (".gitignore", "/dist\n/target\n".to_string()),
    ]
}

fn site_toml(name: &str, base: &str, router: bool) -> String {
    include_str!("templates/site.toml")
        .replace("@NAME@", name)
        .replace("@BASE@", base)
        .replace("@ROUTER@", if router { "true" } else { "false" })
}

fn put(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }

    fs::write(path, contents).map_err(|err| format!("{}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn a_bare_name_is_a_crates_io_dependency() {
        let theme = Theme::parse("my-house-style");

        assert_eq!(theme.name, "my-house-style");
        assert_eq!(theme.dependency, "my-house-style = \"*\"");
        assert_eq!(
            theme.ident(),
            "my_house_style",
            "a dash is not an identifier"
        );
        assert!(theme.pin.contains('*'), "the reader is told to pin it");
    }

    #[test]
    fn anything_with_a_scheme_is_a_git_dependency() {
        let theme = Theme::parse("https://github.com/me/house-style.git");

        assert_eq!(theme.name, "house-style", ".git is not part of the name");
        assert_eq!(
            theme.dependency,
            "house-style = { git = \"https://github.com/me/house-style.git\" }"
        );
        assert!(theme.pin.contains("rev"), "the reader is told to pin it");
    }

    #[test]
    fn a_url_without_the_git_suffix_or_with_a_trailing_slash_works_too() {
        for url in [
            "https://github.com/me/house-style",
            "https://github.com/me/house-style/",
            "ssh://git@example.com/me/house-style.git",
        ] {
            assert_eq!(Theme::parse(url).name, "house-style", "for {url}");
        }
    }

    #[test]
    fn surrounding_whitespace_is_not_part_of_the_name() {
        assert_eq!(Theme::parse("  house-style  ").name, "house-style");
    }
}
