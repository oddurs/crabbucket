// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Generates a `Route` enum from content/, so that a link written in Rust
// stops compiling when the page it points at is renamed.

use std::{env, fs, path::PathBuf};

fn main() {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("content");

    let found = crabbucket_routes::scan(&content)
        .unwrap_or_else(|err| panic!("{}: {err}", content.display()));

    // The directory as well as the files: cargo only notices a new page
    // through the directory, and watching the files alone misses it.
    print!("{}", crabbucket_routes::watch(&content, &found));
    println!("cargo::rerun-if-changed=build.rs");

    // The site renders some of its own pages.  They are in the enum too, or a
    // link to one would be the only untyped link on the site.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("site.toml");
    println!("cargo::rerun-if-changed={}", manifest.display());

    let mut routes: Vec<String> = found.into_iter().map(|(route, _)| route).collect();
    routes.extend(
        crabbucket_routes::declared(&manifest)
            .unwrap_or_else(|err| panic!("{}: {err}", manifest.display())),
    );
    let module = crabbucket_routes::generate(&routes)
        .unwrap_or_else(|err| panic!("{}: {err}", content.display()));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("no out dir")).join("routes.rs");
    fs::write(&out, module).unwrap_or_else(|err| panic!("{}: {err}", out.display()));
}
