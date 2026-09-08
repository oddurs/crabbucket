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
//! The components a documentation site actually needs, and no more.
//!
//! Each is a function taking typed props, and each is registered as a
//! [directive][crabbucket::directive] so a Markdown page can reach it without
//! leaving Markdown.  A page that misspells an attribute fails the build with
//! the file and the line; the alternative -- raw HTML in Markdown -- throws
//! away every guarantee this project is built on at the exact moment someone
//! wants something to look good.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crabbucket::directive::Directives;
use crabbucket::style::Style;
use maud::{Markup, html};
use serde::Deserialize;

use crate::NS;

/// The callout component.
pub const CALLOUT: Style = Style::new(NS, "callout", include_str!("styles/callout.css"));

/// A grid of linked cards.
pub const CARDS: Style = Style::new(NS, "cards", include_str!("styles/cards.css"));

/// Tabbed panels, which work with scripting disabled.
pub const TABS: Style = Style::new(NS, "tabs", include_str!("styles/tabs.css"));

/// A numbered walkthrough.
pub const STEPS: Style = Style::new(NS, "steps", include_str!("styles/steps.css"));

/// Every component style in this module, for the theme to compose.
pub const STYLES: &[Style] = &[CALLOUT, CARDS, TABS, STEPS];

/// The kind of a callout, which decides its accent.
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
    /// The modifier suffix used in the class name.
    pub fn slug(self) -> &'static str {
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
    /// Which accent to use.  Defaults to a note.
    #[serde(default)]
    pub kind: Kind,
    /// An optional heading above the body.
    #[serde(default)]
    pub title: Option<String>,
}

/// An aside that stands apart from the prose around it.
pub fn callout(props: &Callout, body: Markup) -> Markup {
    html! {
        aside class=(CALLOUT.with(props.kind.slug())) {
            @if let Some(title) = &props.title {
                p class=(CALLOUT.element("title")) { (title) }
            }
            div class=(CALLOUT.element("body")) { (body) }
        }
    }
}

/// `:::cards`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cards {}

/// A grid holding cards.
pub fn cards(body: Markup) -> Markup {
    html! { div class=(CARDS.class()) { (body) } }
}

/// `:::card{title = "Routing", href = "routing/"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// The card's heading.
    pub title: String,
    /// Where it goes.  Checked by the link checker like any other link,
    /// because by the time the build looks at it, it is any other link.
    pub href: String,
}

/// One card in a grid.
pub fn card(props: &Card, body: Markup) -> Markup {
    html! {
        a class=(CARDS.element("card")) href=(props.href) {
            span class=(CARDS.element("title")) { (props.title) }
            div class=(CARDS.element("body")) { (body) }
        }
    }
}

/// `:::tabs`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tabs {
    /// A name for the group, used to remember the reader's choice.  Two tab
    /// groups sharing a name move together, which is what you want for
    /// "macOS / Linux / Windows" repeated down a page.
    #[serde(default)]
    pub group: Option<String>,
}

/// A row of tabs with their panels.
pub fn tabs(props: &Tabs, body: Markup) -> Markup {
    let group = props.group.clone().unwrap_or_else(|| "tabs".to_string());

    html! {
        div class=(TABS.class()) data-tab-group=(group) { (body) }
    }
}

/// `:::tab{label = "macOS"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    /// The label on the tab itself.
    pub label: String,
    /// Whether this tab is the one shown first.  With none selected the first
    /// tab wins, so this is only needed to choose a different one.
    #[serde(default)]
    pub selected: bool,
}

/// One tab: a radio, its label, and its panel.
///
/// The three are siblings so that CSS alone can show the right panel, and so
/// that the component needs to know nothing about its neighbours.  See
/// `styles/tabs.css`.
///
/// `id` is unique across the page and `index` is the tab's position within its
/// group, which is all that is needed to derive the shared radio name: the
/// group is named after its first tab, and the first tab is `id - index`.
pub fn tab(props: &Tab, body: Markup, id: usize, index: usize) -> Markup {
    let element = format!("{NS}-tab-{id}");
    let group = format!("{NS}-tabs-{}", id - index);

    html! {
        input type="radio"
              class=(TABS.element("radio"))
              name=(group)
              id=(element)
              data-tab-label=(props.label)
              checked[props.selected || index == 0];
        label class=(TABS.element("label")) for=(element) { (props.label) }
        div class=(TABS.element("panel")) role="tabpanel" { (body) }
    }
}

/// `:::steps`
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Steps {}

/// A numbered walkthrough.
pub fn steps(body: Markup) -> Markup {
    html! { div class=(STEPS.class()) { (body) } }
}

/// `:::step{title = "Install"}`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// The step's heading.
    pub title: String,
}

/// One step in a walkthrough.  The number comes from CSS, not from the author.
pub fn step(props: &Step, body: Markup) -> Markup {
    html! {
        div class=(STEPS.element("step")) {
            p class=(STEPS.element("title")) { (props.title) }
            div class=(STEPS.element("body")) { (body) }
        }
    }
}

/// Registers every component as a directive.
pub fn directives() -> Directives {
    let mut directives = Directives::new();

    directives
        .add("callout", |props: Callout, body| callout(&props, body))
        .add("cards", |_: Cards, body| cards(body))
        .add("card", |props: Card, body| card(&props, body))
        .add("steps", |_: Steps, body| steps(body))
        .add("step", |props: Step, body| step(&props, body));

    // A tab needs two numbers it cannot work out for itself: an id no other
    // tab on the page shares, and its position within its own group.
    //
    // Both come from counters, which works because directives render depth
    // first -- every `tab` runs before the `tabs` that contains it -- so
    // `tabs` resetting the position on its way out leaves the counter correct
    // for the next group.  Pages are rendered in a deterministic order, so the
    // ids are stable across builds; they end up in the DOM, and an id that
    // moves between builds is an id nothing can link to.
    let ids = Arc::new(AtomicUsize::new(0));
    let position = Arc::new(AtomicUsize::new(0));

    let tab_position = Arc::clone(&position);
    directives.add("tab", move |props: Tab, body| {
        let id = ids.fetch_add(1, Ordering::Relaxed);
        let index = tab_position.fetch_add(1, Ordering::Relaxed);
        tab(&props, body, id, index)
    });

    directives.add("tabs", move |props: Tabs, body| {
        position.store(0, Ordering::Relaxed);
        tabs(&props, body)
    });

    directives
}

#[cfg(test)]
mod tests {
    use maud::html;

    use super::{Callout, Card, Kind, Step, Tab, callout, card, step, tab};

    #[test]
    fn a_callout_carries_its_kind_and_its_title() {
        let props = Callout {
            kind: Kind::Warn,
            title: Some("Careful".into()),
        };
        let html = callout(&props, html! { p { "body" } }).into_string();

        assert!(html.contains("cb-callout cb-callout--warn"));
        assert!(html.contains("cb-callout__title\">Careful"));
        assert!(html.contains("<p>body</p>"));
    }

    #[test]
    fn a_callout_without_a_title_has_no_empty_heading() {
        let html = callout(&Callout::default(), html! {}).into_string();
        assert!(!html.contains("cb-callout__title"));
        assert!(
            html.contains("cb-callout--note"),
            "the default kind is a note"
        );
    }

    #[test]
    fn a_card_is_a_link_so_the_link_checker_sees_it() {
        let props = Card {
            title: "Routing".into(),
            href: "routing/".into(),
        };
        let html = card(&props, html! {}).into_string();

        assert!(
            html.contains("<a class=\"cb-cards__card\" href=\"routing/\""),
            "got {html}"
        );
    }

    #[test]
    fn a_tab_is_a_radio_a_label_and_a_panel_in_that_order() {
        let props = Tab {
            label: "macOS".into(),
            selected: false,
        };
        let html = tab(&props, html! { p { "brew" } }, 3, 0).into_string();

        let radio = html.find("type=\"radio\"").expect("no radio");
        let label = html.find("<label").expect("no label");
        let panel = html.find("cb-tabs__panel").expect("no panel");

        assert!(
            radio < label && label < panel,
            "the CSS depends on this order: {html}"
        );
        assert!(html.contains("id=\"cb-tab-3\""));
        assert!(html.contains("for=\"cb-tab-3\""));
        assert!(
            html.contains("checked"),
            "the first tab of a group is shown"
        );
    }

    #[test]
    fn every_tab_in_a_group_shares_one_radio_name() {
        let props = Tab {
            label: "x".into(),
            selected: false,
        };

        // Ids 7, 8, 9 are positions 0, 1, 2 of one group, so the group is
        // named after id 7 in all three.
        for (id, index) in [(7, 0), (8, 1), (9, 2)] {
            let html = tab(&props, html! {}, id, index).into_string();
            assert!(
                html.contains("name=\"cb-tabs-7\""),
                "id {id} escaped its group: {html}"
            );
        }
    }

    #[test]
    fn only_the_first_tab_of_a_group_is_shown_unless_one_says_otherwise() {
        let plain = Tab {
            label: "x".into(),
            selected: false,
        };
        assert!(
            !tab(&plain, html! {}, 1, 1)
                .into_string()
                .contains("checked")
        );

        let chosen = Tab {
            label: "x".into(),
            selected: true,
        };
        assert!(
            tab(&chosen, html! {}, 1, 1)
                .into_string()
                .contains("checked")
        );
    }

    #[test]
    fn a_step_numbers_itself_from_css_and_not_from_the_author() {
        let html = step(
            &Step {
                title: "Install".into(),
            },
            html! { p { "x" } },
        )
        .into_string();
        assert!(html.contains("cb-steps__step"));
        assert!(html.contains("cb-steps__title\">Install"));
        assert!(!html.contains('1'), "a number reached the markup: {html}");
    }
}
