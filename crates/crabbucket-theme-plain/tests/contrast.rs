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
//! Plain's palette, measured rather than admired.
//!
//! Written second, and shorter than the first design system's by exactly the
//! forty lines that moved into `crabbucket-tokens` when it turned out both
//! themes needed them.

use std::collections::BTreeMap;

use crabbucket_theme_plain::tok;
use crabbucket_tokens::contrast::{AA, AA_LARGE, ratio};

/// Every token in a scheme, by custom property name.
fn scheme(dark: bool) -> BTreeMap<&'static str, &'static str> {
    let mut tokens: BTreeMap<_, _> = tok::BASE.iter().copied().collect();

    if dark {
        tokens.extend(tok::OVERRIDES.iter().copied());
    }

    tokens
}

fn assert_contrast(
    tokens: &BTreeMap<&str, &str>,
    scheme: &str,
    foreground: &str,
    background: &str,
    least: f64,
) {
    let fg = tokens
        .get(foreground)
        .unwrap_or_else(|| panic!("no token {foreground}"));
    let bg = tokens
        .get(background)
        .unwrap_or_else(|| panic!("no token {background}"));
    let found = ratio(fg, bg);

    assert!(
        found >= least,
        "{scheme}: {foreground} ({fg}) on {background} ({bg}) is {found:.2}:1, wanted {least}:1"
    );
}

#[test]
fn text_meets_aa_in_both_schemes() {
    for (name, dark) in [("light", false), ("dark", true)] {
        let tokens = scheme(dark);

        assert_contrast(&tokens, name, "--color-ink", "--color-paper", AA);
        assert_contrast(&tokens, name, "--color-faded", "--color-paper", AA);
        assert_contrast(&tokens, name, "--color-accent", "--color-paper", AA);
    }
}

#[test]
fn syntax_colours_stay_legible_in_both_schemes() {
    for (name, dark) in [("light", false), ("dark", true)] {
        let tokens = scheme(dark);

        for (token, _) in tokens
            .iter()
            .filter(|(token, _)| token.starts_with("--syntax-"))
        {
            assert_contrast(&tokens, name, token, "--color-paper", AA_LARGE);
        }
    }
}

#[test]
fn the_rule_is_visible_without_competing_with_the_text() {
    for (name, dark) in [("light", false), ("dark", true)] {
        let tokens = scheme(dark);
        let found = ratio(tokens["--color-rule"], tokens["--color-paper"]);

        assert!(
            found >= 1.15,
            "{name}: the rule is invisible at {found:.2}:1"
        );
        assert!(
            found <= 4.0,
            "{name}: the rule is louder than the text at {found:.2}:1"
        );
    }
}

#[test]
fn the_base_palette_is_the_light_one() {
    assert_eq!(tok::SCHEME, "light");

    // A document is light, and the dark palette is the override.  If that
    // ever inverts, the media query and the `data-theme` guard invert with it,
    // and this is the assertion that notices.
    let light = scheme(false);
    let dark = scheme(true);

    assert!(
        ratio(light["--color-paper"], "#000000") > ratio(dark["--color-paper"], "#000000"),
        "the base palette is darker than the override"
    );
}

#[test]
fn every_colour_has_a_value_in_both_schemes() {
    let overridden: BTreeMap<_, _> = tok::OVERRIDES.iter().copied().collect();

    let missing: Vec<&str> = tok::BASE
        .iter()
        .map(|(token, _)| *token)
        .filter(|token| token.starts_with("--color-") || token.starts_with("--syntax-"))
        .filter(|token| !overridden.contains_key(token))
        .collect();

    assert!(
        missing.is_empty(),
        "these colours have no dark value: {missing:?}"
    );
}
