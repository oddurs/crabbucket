// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Generates the token module from design/tokens.toml.
//
// Both halves of a design token -- the Rust constant a component refers to and
// the CSS custom property the browser resolves -- come out of this one
// generator, which is the only way to guarantee they cannot drift apart.  A
// renamed token becomes a compile error at every use site, which is the point.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("no manifest dir"));
    let tokens = manifest.join("../../design/tokens.toml");

    println!("cargo::rerun-if-changed={}", tokens.display());
    println!("cargo::rerun-if-changed=build.rs");

    let text =
        fs::read_to_string(&tokens).unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));
    let table: toml::Table = text
        .parse()
        .unwrap_or_else(|err| panic!("{}: {err}", tokens.display()));

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("no out dir")).join("tokens.rs");
    fs::write(&out, generate(&table)).unwrap_or_else(|err| panic!("{}: {err}", out.display()));
}

/// Emits one Rust module per token group, plus the `:root` block that defines
/// the matching custom properties.
fn generate(table: &toml::Table) -> String {
    let groups: BTreeMap<&String, &toml::Table> = table
        .iter()
        .filter_map(|(name, value)| value.as_table().map(|table| (name, table)))
        .collect();

    let mut rust = String::new();
    let mut css = String::from(":root{");

    for (group, entries) in &groups {
        rust.push_str(&format!(
            "/// The `{group}` tokens, generated from `design/tokens.toml`.\npub mod {} {{\n",
            ident(group)
        ));

        for (name, value) in entries.iter() {
            let Some(value) = value.as_str() else {
                panic!("design/tokens.toml: {group}.{name} must be a string");
            };

            let var = format!("--{group}-{name}");
            rust.push_str(&format!(
                "    /// `{value}` -- also available to CSS as `var({var})`.\n\
                 \x20   pub const {}: &str = \"var({var}, {value})\";\n",
                konst(name)
            ));
            css.push_str(&format!("{var}:{value};"));
        }

        rust.push_str("}\n\n");
    }

    css.push('}');

    rust.push_str(&format!(
        "/// Every token as a `:root` block of CSS custom properties.\n\
         pub const CSS: &str = {:?};\n",
        css
    ));

    rust
}

/// Turns a token group name into a module identifier.
fn ident(name: &str) -> String {
    name.replace('-', "_")
}

/// Turns a token name into a constant identifier: `step--1` becomes `STEP__1`.
fn konst(name: &str) -> String {
    name.to_uppercase().replace(['-', '.'], "_")
}
