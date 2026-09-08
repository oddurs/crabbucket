// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Chapter 5 of the book shows this file.  It is twelve lines because
// generating a token module is `crabbucket-tokens`' job rather than every
// design system's.

use std::{env, fs, path::PathBuf};

use crabbucket_tokens::{Scheme, generate};

fn main() {
    let tokens = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("design/tokens.toml");

    println!("cargo::rerun-if-changed={}", tokens.display());
    println!("cargo::rerun-if-changed=build.rs");

    let source =
        fs::read_to_string(&tokens).unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let module = generate(&source, Scheme::Light)
        .unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("no out dir")).join("tokens.rs");
    fs::write(&out, module).unwrap_or_else(|err| panic!("{}: {err}", out.display()));
}
