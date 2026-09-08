// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Generates the token module from this crate's design/tokens.toml.
//
// This used to be forty lines of generator.  Writing a second design system
// turned them into `crabbucket-tokens`, because the second theme's first
// draft was a copy of them.
//
// The file lives inside the crate rather than at the top of the tree because
// a build script cannot read outside its own package once the crate is
// packaged -- and because these are this design system's tokens, not the
// project's.  crabbucket-theme-plain has its own, which is the point.

use std::{env, fs, path::PathBuf};

use crabbucket_tokens::{Scheme, generate};

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = manifest.join("design/tokens.toml");

    println!("cargo::rerun-if-changed={}", tokens.display());
    println!("cargo::rerun-if-changed=build.rs");

    let source =
        fs::read_to_string(&tokens).unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let module =
        generate(&source, Scheme::Dark).unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("no out dir")).join("tokens.rs");
    fs::write(&out, module).unwrap_or_else(|err| panic!("{}: {err}", out.display()));
}
