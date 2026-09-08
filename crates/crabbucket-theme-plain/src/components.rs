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
//! The same component vocabulary as the default design system, answered
//! differently.
//!
//! A page's `:::callout` names a component, so a design system that wants to
//! render somebody else's content has to have one.  What it may not do is
//! *look* like the other one: a callout here is an indented note, cards are a
//! list, and tabs are simply every panel one after another, because this
//! design system does not do tabs and saying so plainly beats doing them
//! badly.
//!
//! The props are the same types by name and shape, deliberately.  Content
//! written against one design system's attributes has to keep working against
//! another's, or "swap the theme" means "rewrite the site".

use crabbucket::directive::Directives;
use crabbucket::style::Style;
use maud::{Markup, html};
use serde::Deserialize;

use crate::NS;

/// The marks a document makes: callouts, lists of links, panels, steps.
pub const MARKS: Style = Style::new(NS, "marks", include_str!("styles/marks.css"));

/// Every component style in this module.
pub const STYLES: &[Style] = &[MARKS];

/// The kind of a callout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Neutral information.
    #[default]
    Note,
    /// Something the reader can get wrong.
    Warn,
}

impl Kind {
    fn slug(self) -> &'static str {
        match self {
            Kind::Note => "note",
            Kind::Warn => "warn",
        }
    }
}

/// `:::callout{kind = "warn", title = "Careful"}`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Callout {
    /// Which accent to use.
    #[serde(default)]
    pub kind: Kind,
    /// An optional heading above the body.
    #[serde(default)]
    pub title: Option<String>,
}

/// `:::card{title = "Routing", href = "routing/"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// The card's heading.
    pub title: String,
    /// Where it goes.
    pub href: String,
}

/// `:::tabs`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tabs {
    /// Ignored: Plain shows every panel, so there is no choice to remember.
    #[serde(default)]
    pub group: Option<String>,
}

/// `:::tab{label = "macOS"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    /// The label, which becomes the panel's heading.
    pub label: String,
    /// Ignored: every panel is shown, so none of them is the selected one.
    #[serde(default)]
    pub selected: bool,
}

/// `:::step{title = "Install"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// The step's heading.  The number comes from a CSS counter.
    pub title: String,
}

/// A directive taking no attributes at all.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bare {}

/// A label above a block, in small capitals.
fn label(text: &str) -> Markup {
    html! { p class=(MARKS.element("label")) { (text) } }
}

/// Registers the component vocabulary.
pub fn directives() -> Directives {
    let mut directives = Directives::new();

    directives
        .add("callout", |props: Callout, body| {
            html! {
                aside class=(format!("{} {}--{}",
                    MARKS.element("callout"),
                    MARKS.element("callout"),
                    props.kind.slug())) {
                    @if let Some(title) = &props.title { (label(title)) }
                    (body)
                }
            }
        })
        .add("cards", |_: Bare, body| {
            html! { ul class=(MARKS.element("cards")) { (body) } }
        })
        .add("card", |props: Card, body| {
            html! {
                li class=(MARKS.element("card")) {
                    a href=(props.href) { (props.title) }
                    " — "
                    (body)
                }
            }
        })
        .add("tabs", |_: Tabs, body| {
            // Every panel, one after another.  A design system with no
            // scripting cannot hide one behind another, and pretending
            // otherwise would leave the reader unable to see half the page.
            html! { div class=(MARKS.element("panels")) { (body) } }
        })
        .add("tab", |props: Tab, body| {
            html! {
                div class=(MARKS.element("panel")) {
                    (label(&props.label))
                    (body)
                }
            }
        })
        .add("steps", |_: Bare, body| {
            html! { div class=(MARKS.element("steps")) { (body) } }
        })
        .add("step", |props: Step, body| {
            html! {
                div class=(MARKS.element("step")) {
                    (label(&props.title))
                    (body)
                }
            }
        });

    directives
}

#[cfg(test)]
mod tests {
    use crabbucket::markdown;

    use super::directives;

    /// A context for a directive that reads neither the configuration nor any
    /// data, which is all of these.
    fn render(source: &str) -> String {
        let config = crabbucket::Config::blank();
        let data = crabbucket::directive::Data::default();
        let context = crabbucket::directive::Context::new(&config, &data);

        markdown::render(source, &directives(), &context)
            .unwrap_or_else(|fault| panic!("line {}: {}", fault.line, fault.message))
            .html
    }

    #[test]
    fn the_vocabulary_matches_the_other_design_systems() {
        // Content names components.  A theme that answers to fewer names than
        // the content uses is not a theme you can swap in.
        assert_eq!(
            directives().names(),
            ["callout", "card", "cards", "step", "steps", "tab", "tabs"]
        );
    }

    #[test]
    fn a_callout_is_an_indented_note_rather_than_a_coloured_box() {
        let html = render(":::callout{kind = \"warn\", title = \"Careful\"}\nMind the gap.\n:::\n");

        assert!(
            html.contains("pl-marks__callout pl-marks__callout--warn"),
            "got {html}"
        );
        assert!(html.contains("Careful"));
        assert!(html.contains("<p>Mind the gap.</p>"));
    }

    #[test]
    fn every_tab_panel_is_shown_because_this_design_system_has_no_script() {
        let html = render(
            ":::tabs\n:::tab{label = \"macOS\"}\nbrew\n:::\n:::tab{label = \"Linux\"}\napt\n:::\n:::\n",
        );

        assert!(
            html.contains("brew") && html.contains("apt"),
            "a panel was hidden: {html}"
        );
        assert!(!html.contains("<input"), "no radios, no script: {html}");
        assert!(
            html.contains("macOS") && html.contains("Linux"),
            "labels are lost: {html}"
        );
    }

    #[test]
    fn a_card_is_a_list_item_and_still_a_real_link() {
        let html = render(":::cards\n:::card{title = \"A\", href = \"a/\"}\nabout A\n:::\n:::\n");

        assert!(
            html.contains("<ul class=\"pl-marks__cards\">"),
            "got {html}"
        );
        assert!(
            html.contains("href=\"a/\""),
            "the link checker needs to see this: {html}"
        );
    }

    #[test]
    fn the_same_attributes_are_accepted_as_the_other_design_system() {
        // `selected` and `group` do nothing here, but rejecting them would
        // mean content could not move between design systems.
        render(":::tabs{group = \"os\"}\n:::tab{label = \"a\", selected = true}\nx\n:::\n:::\n");
    }

    #[test]
    fn an_attribute_neither_design_system_has_still_fails() {
        let config = crabbucket::Config::blank();
        let data = crabbucket::directive::Data::default();
        let context = crabbucket::directive::Context::new(&config, &data);

        let err = markdown::render(
            ":::callout{knid = \"warn\"}\nx\n:::\n",
            &directives(),
            &context,
        )
        .expect_err("should fail");

        assert!(
            err.message.contains("unknown field `knid`"),
            "got {}",
            err.message
        );
    }
}
