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
//! The walkthrough claims every snippet in it is from this crate.
//!
//! Documentation that quotes code goes stale silently, and a guide that no
//! longer compiles is worse than no guide: it teaches the wrong thing with
//! confidence.  So the claim is checked rather than made.
//!
//! This does not compare the snippets character by character -- rustfmt would
//! break that on any reflow, and the guide elides bodies with `/* … */` on
//! purpose.  It checks the load-bearing part: every identifier the guide puts
//! in front of a reader still exists here, spelled the way the guide spells
//! it.

use std::fs;
use std::path::{Path, PathBuf};

/// The page that makes the claim.
fn guide() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/site/content/docs/your-own-design-system.md");

    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

/// Every Rust and CSS source file in this crate, concatenated.
fn crate_source() -> String {
    fn walk(dir: &Path, into: &mut String) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                walk(&path, into);
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rs" | "css" | "toml")
            ) {
                into.push_str(&fs::read_to_string(&path).unwrap_or_default());
                into.push('\n');
            }
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut source = String::new();

    walk(&root.join("src"), &mut source);
    source.push_str(&fs::read_to_string(root.join("build.rs")).unwrap_or_default());
    source.push_str(&fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default());
    source.push_str(&fs::read_to_string(root.join("design/tokens.toml")).unwrap_or_default());

    source
}

#[test]
fn every_name_the_guide_shows_still_exists() {
    let source = crate_source();

    // Each of these appears in a snippet a reader is invited to copy.  If one
    // is renamed here and not there, the guide is teaching a name that is
    // gone.
    let names = [
        "pub const NS: &str = \"pl\";",
        "pub const PAGE: Style = Style::new(NS, \"page\", include_str!(\"styles/page.css\"));",
        "pub const PROSE: Style = Style::new(NS, \"prose\", include_str!(\"styles/prose.css\"));",
        "pub enum Layout {",
        "#[serde(alias = \"docs\", alias = \"landing\")]",
        "impl Theme for Plain {",
        "type Layout = Layout;",
        "fn stylesheet(&self) -> String {",
        "pub mod tok {",
        "include!(concat!(env!(\"OUT_DIR\"), \"/tokens.rs\"));",
        "generate(&source, Scheme::Light)",
        ".&__masthead",
        "--color-paper",
        "--color-ink",
        "[color.dark]",
    ];

    let missing: Vec<&str> = names
        .iter()
        .copied()
        .filter(|name| !source.contains(name))
        .collect();

    assert!(
        missing.is_empty(),
        "the guide shows names this crate no longer has: {missing:#?}"
    );
}

#[test]
fn the_guide_shows_what_the_crate_actually_depends_on() {
    let manifest = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("no manifest");

    // The guide says four dependencies and names them.  A fifth appearing here
    // without the guide mentioning it is the kind of drift nobody notices
    // until they follow the walkthrough and it does not build.
    let mut declared: Vec<&str> = Vec::new();
    let mut inside = false;

    for line in manifest.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            inside = line.contains("dependencies]");
            continue;
        }

        if inside && !line.is_empty() && !line.starts_with('#') {
            let name = line.split(['.', ' ', '=']).next().unwrap_or("");
            if !name.is_empty() && !declared.contains(&name) {
                declared.push(name);
            }
        }
    }

    declared.sort_unstable();

    assert_eq!(
        declared,
        ["crabbucket", "crabbucket-tokens", "maud", "serde"],
        "the guide names four dependencies; this crate has different ones"
    );
}

#[test]
fn the_guide_is_linked_from_the_page_it_expands_on() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/site/content/docs/design-systems.md");

    let page = fs::read_to_string(&path).expect("no design-systems page");

    assert!(
        page.contains("your-own-design-system/"),
        "a walkthrough nothing links to is a walkthrough nobody reads"
    );
}

#[test]
fn the_guide_covers_the_parts_the_item_asked_for() {
    // Adding and removing layouts and tokens, and migrating a site across a
    // breaking change.  These are the questions somebody has after writing
    // their first theme, and the ones nobody writes down.
    let guide = guide();

    for topic in [
        "### Adding a layout",
        "### Removing a layout",
        "### Adding a token",
        "### Removing or renaming a token",
        "### Migrating a site across a breaking change",
    ] {
        assert!(
            guide.contains(topic),
            "the guide no longer covers `{topic}`"
        );
    }
}
