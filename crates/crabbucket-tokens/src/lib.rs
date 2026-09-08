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
//! Generates a design system's token module from its token file.
//!
//! This is a build-time crate, used from a theme's `build.rs`.  It is separate
//! from `crabbucket` for one reason: a build dependency is compiled before
//! anything else, and pulling a Markdown parser and a syntax highlighter into
//! that phase to read one TOML file would be absurd.  Its only dependency is
//! `toml`.
//!
//! ```
//! use crabbucket_tokens::{Scheme, generate};
//!
//! let source = r##"
//!     [color]
//!     surface = "#0b0d10"
//!
//!     [color.light]
//!     surface = "#ffffff"
//! "##;
//!
//! let module = generate(source, Scheme::Dark).unwrap();
//!
//! assert!(module.contains("pub const SURFACE: &str = \"var(--color-surface, #0b0d10)\";"));
//! assert!(module.contains("prefers-color-scheme: light"));
//! ```
//!
//! Both halves of every token come out of this one pass: the Rust constant a
//! component refers to, and the CSS custom property the browser resolves.  A
//! renamed token becomes a compile error at every use site, and the two can
//! never disagree because nothing emits one without the other.

use std::collections::BTreeMap;
use std::fmt;

/// Which of the two colour schemes a token file's top-level values are.
///
/// A theme declares this rather than having it inferred, because a token file
/// with no overrides at all has nothing to infer from and would otherwise get
/// a `color-scheme` that is simply a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// The base palette is light; a `dark` sub-table overrides it.
    Light,
    /// The base palette is dark; a `light` sub-table overrides it.
    Dark,
}

impl Scheme {
    /// The name this scheme has in CSS.
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }

    /// The other one, which is the only sub-table name a token group may have.
    pub fn other(self) -> Scheme {
        match self {
            Scheme::Light => Scheme::Dark,
            Scheme::Dark => Scheme::Light,
        }
    }
}

/// Something wrong with a token file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// One token: its CSS custom property name and its value.
type Token = (String, String);

/// Generates the token module.
///
/// # Errors
///
/// Fails if the file is not valid TOML, if a group carries a sub-table that is
/// not the other scheme, if an override names a token the base does not have,
/// or if any value is not a string.
pub fn generate(source: &str, base: Scheme) -> Result<String, Error> {
    let table: toml::Table = source.parse().map_err(|err| Error(format!("{err}")))?;

    let groups: BTreeMap<&String, &toml::Table> = table
        .iter()
        .filter_map(|(name, value)| value.as_table().map(|table| (name, table)))
        .collect();

    let mut rust = String::new();
    let mut base_tokens: Vec<Token> = Vec::new();
    let mut overrides: Vec<Token> = Vec::new();

    for (group, entries) in &groups {
        rust.push_str(&format!(
            "/// The `{group}` tokens, generated from the design token file.\npub mod {} {{\n",
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
            base_tokens.push((property, value.to_string()));
        }

        rust.push_str("}\n\n");
        overrides.extend(scheme(group, entries, base)?);
    }

    rust.push_str(&format!(
        "/// Every token's custom property and its value in the base palette.\n\
         pub const BASE: &[(&str, &str)] = &{};\n\n\
         /// The tokens the other scheme overrides, and their values.\n\
         pub const OVERRIDES: &[(&str, &str)] = &{};\n\n\
         /// Which scheme the base palette is.\n\
         pub const SCHEME: &str = {:?};\n\n\
         /// Every token, as CSS: the base palette, then the overrides.\n\
         pub const CSS: &str = {:?};\n",
        pairs(&base_tokens),
        pairs(&overrides),
        base.name(),
        css(&base_tokens, &overrides, base)
    ));

    Ok(rust)
}

/// A group's overrides for the other scheme, checked against the group itself.
fn scheme(group: &str, entries: &toml::Table, base: Scheme) -> Result<Vec<Token>, Error> {
    let other = base.other().name();
    let mut found = Vec::new();

    for (name, value) in entries.iter() {
        let Some(overrides) = value.as_table() else {
            continue;
        };

        if name != other {
            return Err(Error(format!(
                "{group}.{name} is not a scheme; the base palette is {}, \
                 so the only sub-table it may have is `{other}`",
                base.name()
            )));
        }

        for (token, value) in overrides.iter() {
            if !entries.contains_key(token) {
                return Err(Error(format!(
                    "{group}.{other}.{token} has no `{group}.{token}` to override"
                )));
            }

            let value = value
                .as_str()
                .ok_or_else(|| Error(format!("{group}.{other}.{token} must be a string")))?;

            found.push((format!("--{group}-{token}"), value.to_string()));
        }
    }

    Ok(found)
}

/// The stylesheet: the base palette, then the two ways of asking for the other
/// scheme.
///
/// The media query is guarded so an explicit `data-theme` for the base scheme
/// beats the system preference, and the attribute selector comes last so an
/// explicit choice of the other one beats a system preference for the base.
fn css(base_tokens: &[Token], overrides: &[Token], base: Scheme) -> String {
    let mut out = String::new();

    out.push_str(&format!(":root{{color-scheme:{};", base.name()));
    out.push_str(&declarations(base_tokens));
    out.push('}');

    if overrides.is_empty() {
        return out;
    }

    let other = base.other().name();
    let block = format!("color-scheme:{other};{}", declarations(overrides));

    out.push_str(&format!(
        "@media (prefers-color-scheme: {other}){{:root:not([data-theme=\"{}\"]){{{block}}}}}",
        base.name()
    ));
    out.push_str(&format!(":root[data-theme=\"{other}\"]{{{block}}}"));
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

/// Contrast, for a design system that wants to assert its palette rather than
/// judge it.
///
/// Here because the second design system needed the same forty lines the
/// first one had, which is the signal that they belong to neither.
pub mod contrast {
    /// The WCAG AA ratio for body text.
    pub const AA: f64 = 4.5;

    /// The WCAG AA ratio for large text, and for anything decorative that
    /// still has to be legible.
    pub const AA_LARGE: f64 = 3.0;

    /// The relative luminance of an `#rrggbb` colour, per WCAG 2.1.
    ///
    /// # Panics
    ///
    /// Panics if `hex` is not six hexadecimal digits, with or without a `#`.
    /// This is for tests, where a malformed colour is a bug to be shouted
    /// about rather than a case to be handled.
    pub fn luminance(hex: &str) -> f64 {
        let hex = hex.trim_start_matches('#');
        assert_eq!(hex.len(), 6, "not a six-digit hex colour: {hex}");

        let channel = |at: usize| {
            let value = u8::from_str_radix(&hex[at..at + 2], 16)
                .unwrap_or_else(|_| panic!("not hex: {hex}")) as f64
                / 255.0;

            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
    }

    /// The contrast ratio between two colours, from 1.0 to 21.0.
    pub fn ratio(a: &str, b: &str) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        let (lighter, darker) = if a > b { (a, b) } else { (b, a) };

        (lighter + 0.05) / (darker + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::{Scheme, generate};

    const DARK_FIRST: &str = r##"
        [color]
        surface = "#0b0d10"
        text = "#e7eaf0"

        [color.light]
        surface = "#ffffff"
        text = "#1a1f26"

        [space]
        md = "16px"
    "##;

    #[test]
    fn a_constant_is_a_var_reference_with_the_base_value_as_its_fallback() {
        let module = generate(DARK_FIRST, Scheme::Dark).unwrap();

        assert!(module.contains("pub mod color {"));
        assert!(
            module.contains("pub const SURFACE: &str = \"var(--color-surface, #0b0d10)\";"),
            "got {module}"
        );
        assert!(module.contains("pub const MD: &str = \"var(--space-md, 16px)\";"));
    }

    #[test]
    fn the_overrides_land_in_both_places_that_ask_for_them() {
        let module = generate(DARK_FIRST, Scheme::Dark).unwrap();

        assert!(module.contains("@media (prefers-color-scheme: light)"));
        assert!(
            module.contains(":root:not([data-theme=\\\"dark\\\"])"),
            "got {module}"
        );
        assert!(
            module.contains(":root[data-theme=\\\"light\\\"]"),
            "got {module}"
        );
        assert!(
            module.contains("color-scheme:dark"),
            "the base declares its own scheme"
        );
    }

    #[test]
    fn a_light_first_palette_works_the_same_way_round() {
        let source = r##"
            [color]
            paper = "#fdfdfb"

            [color.dark]
            paper = "#16161a"
        "##;

        let module = generate(source, Scheme::Light).unwrap();

        assert!(
            module.contains("var(--color-paper, #fdfdfb)"),
            "got {module}"
        );
        assert!(module.contains("@media (prefers-color-scheme: dark)"));
        assert!(
            module.contains(":root:not([data-theme=\\\"light\\\"])"),
            "got {module}"
        );
        assert!(
            module.contains(":root[data-theme=\\\"dark\\\"]"),
            "got {module}"
        );
    }

    #[test]
    fn a_palette_with_no_overrides_emits_no_scheme_blocks() {
        let module = generate("[color]\nink = \"#000\"\n", Scheme::Light).unwrap();

        assert!(module.contains("var(--color-ink, #000)"));
        assert!(!module.contains("prefers-color-scheme"), "got {module}");
    }

    #[test]
    fn an_override_of_a_token_that_does_not_exist_is_an_error() {
        let source = "[color]\nink = \"#000\"\n\n[color.dark]\npaper = \"#fff\"\n";
        let err = generate(source, Scheme::Light).unwrap_err();

        assert!(err.to_string().contains("color.dark.paper"), "got {err}");
        assert!(err.to_string().contains("no `color.paper`"), "got {err}");
    }

    #[test]
    fn a_sub_table_that_is_not_the_other_scheme_is_an_error() {
        let source = "[color]\nink = \"#000\"\n\n[color.sepia]\nink = \"#333\"\n";
        let err = generate(source, Scheme::Light).unwrap_err();

        assert!(err.to_string().contains("not a scheme"), "got {err}");
        assert!(
            err.to_string().contains("`dark`"),
            "it names the one that is allowed: {err}"
        );
    }

    #[test]
    fn overriding_in_the_same_scheme_as_the_base_is_an_error() {
        // `[color.light]` under a light base is almost certainly a mistake,
        // and silently emitting a block that can never apply would hide it.
        let source = "[color]\nink = \"#000\"\n\n[color.light]\nink = \"#333\"\n";
        assert!(generate(source, Scheme::Light).is_err());
    }

    #[test]
    fn a_token_name_becomes_a_usable_identifier() {
        let source = "[size]\nstep--1 = \"0.875rem\"\nstep-0 = \"1rem\"\n";
        let module = generate(source, Scheme::Light).unwrap();

        assert!(module.contains("pub const STEP__1"), "got {module}");
        assert!(module.contains("pub const STEP_0"), "got {module}");
    }

    #[test]
    fn the_tables_a_contrast_test_reads_are_emitted() {
        let module = generate(DARK_FIRST, Scheme::Dark).unwrap();

        assert!(module.contains("pub const BASE: &[(&str, &str)]"));
        assert!(module.contains("pub const OVERRIDES: &[(&str, &str)]"));
        assert!(module.contains("pub const SCHEME: &str = \"dark\""));
    }

    #[test]
    fn a_file_that_is_not_toml_says_so_rather_than_panicking() {
        assert!(generate("[color", Scheme::Light).is_err());
    }

    #[test]
    fn the_contrast_maths_is_right() {
        use super::contrast::ratio;

        assert!((ratio("#000000", "#ffffff") - 21.0).abs() < 0.01);
        assert!((ratio("#ffffff", "#ffffff") - 1.0).abs() < 0.01);

        // #767676 is the lightest grey that clears AA on white, and #777777
        // is the first that does not.  If the maths drifts, that moves.
        assert!(
            ratio("#767676", "#ffffff") >= 4.5,
            "the boundary moved down"
        );
        assert!(ratio("#777777", "#ffffff") < 4.5, "the boundary moved up");
    }
}
