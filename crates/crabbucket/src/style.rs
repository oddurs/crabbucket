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
//! Component styles, declared beside the components they style.
//!
//! `doc/DESIGN` section 10.  The problem this solves is not stylesheet size --
//! at the scale these sites run at, a few kilobytes of unused CSS is
//! optimising the wrong number.  The problem is **collision**: two component
//! crates that both style `.card`, where whichever sheet loads last wins and
//! nothing says so.
//!
//! So a [`Style`] carries a namespace, and its CSS is written with `&` standing
//! in for the component's own class root:
//!
//! ```
//! use crabbucket::style::Style;
//!
//! const CALLOUT: Style = Style::new("cb", "callout", "
//!     .& { border-left: 3px solid var(--color-accent); }
//!     .&--warn { border-left-color: var(--color-accent); }
//!     .&__body > :last-child { margin-bottom: 0; }
//! ");
//!
//! assert_eq!(CALLOUT.class(), "cb-callout");
//! assert_eq!(CALLOUT.modifier("warn"), "cb-callout--warn");
//! assert!(CALLOUT.render().contains(".cb-callout--warn"));
//! ```
//!
//! That is BEM with the block name factored out, and the block name namespaced
//! in exactly one place.  There is no macro and no build step, because neither
//! is needed: the substitution is a string replacement and the guarantee comes
//! from the namespace being impossible to forget.
//!
//! A small, fixed set of class names belongs to the framework rather than to
//! any theme -- `heading-anchor`, `code`, and the `tok-` scope classes the
//! highlighter emits.  Those are documented here so a theme knows what it does
//! not own.

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The class names emitted by the framework itself, which every theme styles
/// and no theme may rename.
pub const FRAMEWORK_CLASSES: &[&str] = &["heading-anchor", "code"];

/// The prefix on every class the syntax highlighter emits.
pub const HIGHLIGHT_PREFIX: &str = "tok-";

/// One component's styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Style {
    /// A short namespace, conventionally an abbreviation of the crate that
    /// owns the component.
    pub namespace: &'static str,
    /// The component's name, unique within the namespace.
    pub name: &'static str,
    /// The CSS, with `&` standing in for the component's class root.
    pub css: &'static str,
}

impl Style {
    /// Declares a component's styles.
    pub const fn new(namespace: &'static str, name: &'static str, css: &'static str) -> Self {
        Style {
            namespace,
            name,
            css,
        }
    }

    /// The component's own class: `cb-callout`.
    pub fn class(&self) -> String {
        format!("{}-{}", self.namespace, self.name)
    }

    /// A BEM element of the component: `cb-callout__body`.
    pub fn element(&self, element: &str) -> String {
        format!("{}__{element}", self.class())
    }

    /// A BEM modifier of the component: `cb-callout--warn`.
    ///
    /// Modifiers are additional to the block class, not a replacement for it,
    /// so markup carries both.
    pub fn modifier(&self, modifier: &str) -> String {
        format!("{}--{modifier}", self.class())
    }

    /// The block class and one modifier, in the order a `class` attribute
    /// wants them.
    pub fn with(&self, modifier: &str) -> String {
        format!("{} {}", self.class(), self.modifier(modifier))
    }

    /// The CSS, with `&` resolved.
    pub fn render(&self) -> String {
        self.css.replace('&', &self.class())
    }
}

/// The styles a theme has composed, in declaration order and without repeats.
#[derive(Debug, Default, Clone)]
pub struct StyleSheet {
    styles: Vec<Style>,
    seen: BTreeSet<(&'static str, &'static str)>,
}

impl StyleSheet {
    /// An empty sheet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a component's styles, ignoring a component already added.
    ///
    /// Deduplication is by namespace and name together, so two crates may both
    /// have a `card` and both survive.
    pub fn add(&mut self, style: Style) -> &mut Self {
        if self.seen.insert((style.namespace, style.name)) {
            self.styles.push(style);
        }
        self
    }

    /// Adds several at once.
    pub fn extend(&mut self, styles: impl IntoIterator<Item = Style>) -> &mut Self {
        for style in styles {
            self.add(style);
        }
        self
    }

    /// The whole sheet, each component labelled with a comment so a browser's
    /// inspector says where a rule came from.
    pub fn render(&self) -> String {
        let mut out = String::new();

        for style in &self.styles {
            let _ = write!(
                out,
                "\n/* {}/{} */\n{}",
                style.namespace,
                style.name,
                style.render()
            );
        }

        out
    }

    /// How many components are in the sheet.
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Whether the sheet is empty.
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{Style, StyleSheet};

    const CALLOUT: Style = Style::new(
        "cb",
        "callout",
        ".& { color: red; }\n.&--warn { color: orange; }\n.&__body { margin: 0; }\n",
    );

    #[test]
    fn a_class_carries_its_namespace() {
        assert_eq!(CALLOUT.class(), "cb-callout");
        assert_eq!(CALLOUT.element("body"), "cb-callout__body");
        assert_eq!(CALLOUT.modifier("warn"), "cb-callout--warn");
        assert_eq!(CALLOUT.with("warn"), "cb-callout cb-callout--warn");
    }

    #[test]
    fn the_ampersand_resolves_everywhere_it_appears() {
        let css = CALLOUT.render();
        assert!(css.contains(".cb-callout {"));
        assert!(css.contains(".cb-callout--warn {"));
        assert!(css.contains(".cb-callout__body {"));
        assert!(!css.contains('&'));
    }

    #[test]
    fn two_crates_may_both_have_a_card() {
        let theirs = Style::new("xy", "callout", ".& { color: blue; }");
        let mut sheet = StyleSheet::new();
        sheet.add(CALLOUT).add(theirs);

        assert_eq!(sheet.len(), 2);
        let css = sheet.render();
        assert!(css.contains(".cb-callout {"));
        assert!(css.contains(".xy-callout {"));
    }

    #[test]
    fn the_same_component_added_twice_appears_once() {
        let mut sheet = StyleSheet::new();
        sheet.add(CALLOUT).add(CALLOUT).add(CALLOUT);
        assert_eq!(sheet.len(), 1);
    }

    #[test]
    fn each_component_is_labelled_in_the_output() {
        let mut sheet = StyleSheet::new();
        sheet.add(CALLOUT);
        assert!(sheet.render().contains("/* cb/callout */"));
    }
}
