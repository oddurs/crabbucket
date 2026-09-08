// Copyright (C) 2026 Oddur Sigurdsson
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Generates the token module from design/tokens.toml.
//
// Both halves of a design token -- the Rust constant a component refers to and
// the CSS custom property the browser resolves -- come out of this one
// generator, which is the only way to guarantee they cannot drift apart.  A
// renamed token becomes a compile error at every use site, which is the point.
//
// A group may carry a `light` sub-table.  Those values override the group's
// under `prefers-color-scheme: light` and under an explicit
// `data-theme="light"`, and the Rust constants do not change at all -- they
// are `var()` references, which is precisely why a second palette costs no
// component a single line.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// The name of the one recognised scheme override.
const LIGHT: &str = "light";

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

/// One token: its CSS custom property name and its value.
type Token = (String, String);

/// Emits one Rust module per token group, the `:root` block that defines the
/// custom properties, and the blocks that override them in light mode.
fn generate(table: &toml::Table) -> String {
    let groups: BTreeMap<&String, &toml::Table> = table
        .iter()
        .filter_map(|(name, value)| value.as_table().map(|table| (name, table)))
        .collect();

    let mut rust = String::new();
    let mut dark: Vec<Token> = Vec::new();
    let mut light: Vec<Token> = Vec::new();

    for (group, entries) in &groups {
        rust.push_str(&format!(
            "/// The `{group}` tokens, generated from `design/tokens.toml`.\npub mod {} {{\n",
            ident(group)
        ));

        for (name, value) in entries.iter() {
            let Some(value) = value.as_str() else {
                continue; // A sub-table; handled below.
            };

            let property = format!("--{group}-{name}");
            rust.push_str(&format!(
                "    /// `{value}` -- also available to CSS as `var({property})`.\n\
                 \x20   pub const {}: &str = \"var({property}, {value})\";\n",
                konst(name)
            ));
            dark.push((property, value.to_string()));
        }

        rust.push_str("}\n\n");
        light.extend(scheme(group, entries));
    }

    rust.push_str(&format!(
        "/// Every token's custom property and its value in the base palette.\n\
         pub const DARK: &[(&str, &str)] = &{};\n\n\
         /// The tokens a light palette overrides, and their values.\n\
         pub const LIGHT: &[(&str, &str)] = &{};\n\n\
         /// Every token, as CSS: the base palette, then the light overrides.\n\
         pub const CSS: &str = {:?};\n",
        pairs(&dark),
        pairs(&light),
        css(&dark, &light)
    ));

    rust
}

/// A group's `light` overrides, checked against the group itself.
fn scheme(group: &str, entries: &toml::Table) -> Vec<Token> {
    let mut found = Vec::new();

    for (name, value) in entries.iter() {
        let Some(overrides) = value.as_table() else {
            continue;
        };

        assert_eq!(
            name, LIGHT,
            "design/tokens.toml: {group}.{name} is not a scheme; only `{LIGHT}` is"
        );

        for (token, value) in overrides.iter() {
            assert!(
                entries.contains_key(token),
                "design/tokens.toml: {group}.{LIGHT}.{token} has no `{group}.{token}` to override"
            );

            let value = value.as_str().unwrap_or_else(|| {
                panic!("design/tokens.toml: {group}.{LIGHT}.{token} must be a string")
            });

            found.push((format!("--{group}-{token}"), value.to_string()));
        }
    }

    found
}

/// The stylesheet: the base palette, then the two ways of asking for light.
///
/// The media query is guarded so that an explicit `data-theme="dark"` beats
/// the system preference, and the attribute selector comes last so that an
/// explicit `data-theme="light"` beats a system preference for dark.
fn css(dark: &[Token], light: &[Token]) -> String {
    let mut out = String::new();

    out.push_str(":root{color-scheme:dark;");
    out.push_str(&declarations(dark));
    out.push('}');

    if light.is_empty() {
        return out;
    }

    let overrides = format!("color-scheme:light;{}", declarations(light));

    out.push_str(&format!(
        "@media (prefers-color-scheme: light){{:root:not([data-theme=\"dark\"]){{{overrides}}}}}"
    ));
    out.push_str(&format!(":root[data-theme=\"light\"]{{{overrides}}}"));
    out
}

fn declarations(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(|(name, value)| format!("{name}:{value};"))
        .collect()
}

/// A `&[(&str, &str)]` literal.
fn pairs(tokens: &[Token]) -> String {
    let mut out = String::from("[");
    for (name, value) in tokens {
        out.push_str(&format!("({name:?}, {value:?}), "));
    }
    out.push(']');
    out
}

/// Turns a token group name into a module identifier.
fn ident(name: &str) -> String {
    name.replace('-', "_")
}

/// Turns a token name into a constant identifier: `step--1` becomes `STEP__1`.
fn konst(name: &str) -> String {
    name.to_uppercase().replace(['-', '.'], "_")
}
