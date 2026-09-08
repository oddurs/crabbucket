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
//! Scanning a real directory, and what happens when it changes.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crabbucket_routes::{generate, scan, watch};

/// A content directory of its own, with the given files in it.
fn content(files: &[&str]) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let root = std::env::temp_dir().join(format!(
        "crabbucket-routes-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let _ = fs::remove_dir_all(&root);
    add(&root, files);
    root
}

fn add(root: &Path, files: &[&str]) {
    for file in files {
        let path = root.join(file);
        fs::create_dir_all(path.parent().expect("no parent")).expect("cannot create");
        fs::write(&path, "+++\ntitle = \"t\"\n+++\n").expect("cannot write");
    }
}

fn routes(root: &Path) -> Vec<String> {
    scan(root)
        .expect("cannot scan")
        .into_iter()
        .map(|(route, _)| route)
        .collect()
}

#[test]
fn a_directory_becomes_a_sorted_route_set() {
    let root = content(&["index.md", "about.md", "docs/index.md", "docs/routing.md"]);

    assert_eq!(routes(&root), ["", "about", "docs", "docs/routing"]);
}

#[test]
fn only_markdown_counts() {
    let root = content(&["index.md"]);
    fs::write(root.join("notes.txt"), "not content").expect("cannot write");
    fs::write(root.join(".DS_Store"), "not content").expect("cannot write");

    assert_eq!(routes(&root), [""]);
}

#[test]
fn a_page_added_after_the_first_scan_is_found_by_the_second() {
    // The build script watches the directory as well as the files precisely
    // so that this is the case cargo notices.
    let root = content(&["index.md"]);
    assert_eq!(routes(&root), [""]);

    add(&root, &["docs/new.md"]);
    assert_eq!(routes(&root), ["", "docs/new"]);
}

#[test]
fn a_page_added_after_the_first_scan_appears_in_the_enum() {
    let root = content(&["index.md"]);
    let before = generate(&routes(&root)).expect("cannot generate");
    assert!(!before.contains("DocsNew"), "got {before}");

    add(&root, &["docs/new.md"]);
    let after = generate(&routes(&root)).expect("cannot generate");
    assert!(after.contains("    DocsNew,"), "got {after}");
}

#[test]
fn a_page_that_is_renamed_takes_its_variant_with_it() {
    // This is the promise: a link written as `Route::DocsRouting` stops
    // compiling, because the variant it names stops existing.
    let root = content(&["index.md", "docs/routing.md"]);
    assert!(
        generate(&routes(&root))
            .expect("cannot generate")
            .contains("DocsRouting")
    );

    fs::rename(
        root.join("docs/routing.md"),
        root.join("docs/route-table.md"),
    )
    .expect("rename");

    let after = generate(&routes(&root)).expect("cannot generate");
    assert!(
        !after.contains("DocsRouting"),
        "the old variant survived: {after}"
    );
    assert!(after.contains("DocsRouteTable"), "got {after}");
}

#[test]
fn a_missing_content_directory_is_an_empty_site_rather_than_a_crash() {
    let root = std::env::temp_dir().join("crabbucket-routes-nothing-here");
    let _ = fs::remove_dir_all(&root);

    assert!(routes(&root).is_empty());
}

#[test]
fn every_file_found_is_watched() {
    let root = content(&["index.md", "docs/routing.md"]);
    let found = scan(&root).expect("cannot scan");
    let lines = watch(&root, &found);

    assert_eq!(
        lines.lines().count(),
        3,
        "the directory and both files: {lines}"
    );
    assert!(lines.contains("docs/routing.md"), "got {lines}");
}
