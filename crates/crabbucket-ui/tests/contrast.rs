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
//! Contrast, checked rather than eyeballed.
//!
//! A second palette is the test of whether the token mechanism is real: if a
//! light scheme cannot be expressed in the token file, the token file is not
//! the source of truth it claims to be.  And a palette nobody measured is a
//! palette that fails somebody.
//!
//! The ratios are WCAG 2.1: AA wants 4.5 for body text and 3.0 for large text
//! and for anything decorative that still has to be legible.

use std::collections::BTreeMap;

use crabbucket_ui::tok;

use crabbucket_tokens::contrast::{AA, AA_LARGE, ratio};

/// Every token in a scheme, by custom property name.
fn scheme(light: bool) -> BTreeMap<&'static str, &'static str> {
    let mut tokens: BTreeMap<_, _> = tok::BASE.iter().copied().collect();

    if light {
        tokens.extend(tok::OVERRIDES.iter().copied());
    }

    tokens
}

/// Asserts a ratio, naming the scheme and both tokens when it fails.
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
fn the_luminance_maths_is_right() {
    // Anchors from the specification: black on white is the maximum.
    assert!((ratio("#000000", "#ffffff") - 21.0).abs() < 0.01);
    assert!((ratio("#ffffff", "#ffffff") - 1.0).abs() < 0.01);

    // #767676 is the lightest grey that clears AA on white, and #777777 is
    // the first one that does not.  If the maths drifts, that boundary moves.
    assert!(
        ratio("#767676", "#ffffff") >= 4.5,
        "the boundary moved down"
    );
    assert!(ratio("#777777", "#ffffff") < 4.5, "the boundary moved up");
}

#[test]
fn text_meets_aa_in_both_schemes() {
    for (name, light) in [("dark", false), ("light", true)] {
        let tokens = scheme(light);

        for surface in ["--color-surface", "--color-surface-raised"] {
            assert_contrast(&tokens, name, "--color-text", surface, AA);
            assert_contrast(&tokens, name, "--color-text-muted", surface, AA);
            assert_contrast(&tokens, name, "--color-link", surface, AA);
            assert_contrast(&tokens, name, "--color-accent", surface, AA);
        }
    }
}

#[test]
fn syntax_colours_stay_legible_in_both_schemes() {
    for (name, light) in [("dark", false), ("light", true)] {
        let tokens = scheme(light);

        for (token, _) in tokens
            .iter()
            .filter(|(token, _)| token.starts_with("--syntax-"))
        {
            assert_contrast(&tokens, name, token, "--color-surface-raised", AA_LARGE);
        }
    }
}

#[test]
fn a_border_is_visible_without_shouting() {
    for (name, light) in [("dark", false), ("light", true)] {
        let tokens = scheme(light);
        let found = ratio(tokens["--color-border"], tokens["--color-surface"]);

        assert!(
            found >= 1.2,
            "{name}: the border is invisible at {found:.2}:1"
        );
        assert!(
            found <= 4.0,
            "{name}: the border is louder than the text at {found:.2}:1"
        );
    }
}

#[test]
fn every_light_override_replaces_something_that_exists() {
    let dark: BTreeMap<_, _> = tok::BASE.iter().copied().collect();

    for (token, _) in tok::OVERRIDES {
        assert!(
            dark.contains_key(token),
            "{token} exists only in the light palette"
        );
    }
}

#[test]
fn the_light_palette_actually_covers_the_colours() {
    let overridden: BTreeMap<_, _> = tok::OVERRIDES.iter().copied().collect();

    let missing: Vec<&str> = tok::BASE
        .iter()
        .map(|(token, _)| *token)
        .filter(|token| token.starts_with("--color-") || token.starts_with("--syntax-"))
        .filter(|token| !overridden.contains_key(token))
        .collect();

    assert!(
        missing.is_empty(),
        "these colours have no light value: {missing:?}"
    );
}
