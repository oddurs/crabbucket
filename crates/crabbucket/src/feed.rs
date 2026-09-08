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
//! Feeds, in both of the formats readers disagree about.
//!
//! A feed needs dates, and most pages do not have one, so `date` is optional
//! in a page's frontmatter -- and required of any page in a collection a feed
//! is configured for.  A dated collection with an undated page in it fails the
//! build naming the page, which is the same bargain as everything else here:
//! say what you want and the build holds you to it.
//!
//! TOML has real dates, so `date = 2026-09-08` in frontmatter is a date rather
//! than a string that looks like one, and a malformed one fails at the file
//! and the line without this module doing anything.

use std::fmt::Write as _;

use serde::Deserialize;
use toml::value::Datetime;

/// A feed declared in `site.toml`.
///
/// ```toml
/// [[feed]]
/// collection = "notes"
/// title = "Notes"
/// limit = 20
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feed {
    /// The route prefix whose pages this feed carries.  The empty string is
    /// the whole site.
    pub collection: String,

    /// The feed's title.  Defaults to the site's.
    #[serde(default)]
    pub title: Option<String>,

    /// How many entries to carry, newest first.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

impl Feed {
    /// Whether a route belongs to this feed.
    pub fn covers(&self, route: &str) -> bool {
        let collection = self.collection.trim_matches('/');
        let route = route.trim_matches('/');

        collection.is_empty() || (route.starts_with(collection) && route != collection)
    }

    /// Where this feed's files go, relative to the site root.
    pub fn paths(&self) -> (String, String) {
        let prefix = self.collection.trim_matches('/');

        if prefix.is_empty() {
            ("feed.xml".to_string(), "atom.xml".to_string())
        } else {
            (format!("{prefix}/feed.xml"), format!("{prefix}/atom.xml"))
        }
    }
}

/// One entry, ready to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<'a> {
    /// The page's title.
    pub title: &'a str,
    /// Its absolute URL.
    pub url: String,
    /// Its date.
    pub date: &'a Datetime,
    /// Its rendered body.
    pub html: &'a str,
}

/// Renders RSS 2.0.
pub fn rss(title: &str, description: &str, home: &str, url: &str, items: &[Item<'_>]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n");

    let _ = writeln!(out, "  <title>{}</title>", escape(title));
    let _ = writeln!(out, "  <link>{}</link>", escape(home));
    let _ = writeln!(out, "  <description>{}</description>", escape(description));
    let _ = writeln!(
        out,
        "  <atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/>",
        escape(url)
    );

    for item in items {
        out.push_str("  <item>\n");
        let _ = writeln!(out, "    <title>{}</title>", escape(item.title));
        let _ = writeln!(out, "    <link>{}</link>", escape(&item.url));
        let _ = writeln!(
            out,
            "    <guid isPermaLink=\"true\">{}</guid>",
            escape(&item.url)
        );
        let _ = writeln!(out, "    <pubDate>{}</pubDate>", rfc2822(item.date));
        let _ = writeln!(out, "    <description>{}</description>", escape(item.html));
        out.push_str("  </item>\n");
    }

    out.push_str("</channel>\n</rss>\n");
    out
}

/// Renders Atom.
pub fn atom(title: &str, home: &str, url: &str, items: &[Item<'_>]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");

    let _ = writeln!(out, "  <title>{}</title>", escape(title));
    let _ = writeln!(out, "  <id>{}</id>", escape(url));
    let _ = writeln!(out, "  <link href=\"{}\"/>", escape(home));
    let _ = writeln!(out, "  <link href=\"{}\" rel=\"self\"/>", escape(url));

    // A feed's own timestamp is its newest entry's.  Inventing "now" would
    // make every build differ from the last one for no reason.
    if let Some(newest) = items.first() {
        let _ = writeln!(out, "  <updated>{}</updated>", rfc3339(newest.date));
    }

    for item in items {
        out.push_str("  <entry>\n");
        let _ = writeln!(out, "    <title>{}</title>", escape(item.title));
        let _ = writeln!(out, "    <id>{}</id>", escape(&item.url));
        let _ = writeln!(out, "    <link href=\"{}\"/>", escape(&item.url));
        let _ = writeln!(out, "    <updated>{}</updated>", rfc3339(item.date));
        let _ = writeln!(
            out,
            "    <content type=\"html\">{}</content>",
            escape(item.html)
        );
        out.push_str("  </entry>\n");
    }

    out.push_str("</feed>\n");
    out
}

/// A date as Atom wants it: RFC 3339.
///
/// A frontmatter date with no time is midnight UTC.  Guessing a time zone from
/// the machine doing the build would make the same content produce different
/// feeds on different machines.
fn rfc3339(date: &Datetime) -> String {
    let day = date
        .date
        .map(|d| (d.year, d.month, d.day))
        .unwrap_or((1970, 1, 1));
    let time = date
        .time
        .map(|t| (t.hour, t.minute, t.second))
        .unwrap_or((0, 0, 0));

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        day.0, day.1, day.2, time.0, time.1, time.2
    )
}

/// A date as RSS wants it: RFC 822, as amended by RFC 2822.
fn rfc2822(date: &Datetime) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let (year, month, day) = date
        .date
        .map(|d| (d.year, d.month, d.day))
        .unwrap_or((1970, 1, 1));
    let time = date
        .time
        .map(|t| (t.hour, t.minute, t.second))
        .unwrap_or((0, 0, 0));

    let weekday = DAYS[weekday(year as i32, month as u32, day as u32)];
    let month_name = MONTHS[(month as usize).clamp(1, 12) - 1];

    format!(
        "{weekday}, {day:02} {month_name} {year:04} {:02}:{:02}:{:02} +0000",
        time.0, time.1, time.2
    )
}

/// The day of the week, 0 for Sunday, by Sakamoto's method.
fn weekday(year: i32, month: u32, day: u32) -> usize {
    const OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

    let month = month.clamp(1, 12) as usize;
    let year = if month < 3 { year - 1 } else { year };

    let index =
        (year + year / 4 - year / 100 + year / 400 + OFFSETS[month - 1] + day as i32).rem_euclid(7);

    index as usize
}

/// Escapes text for XML.
///
/// `'` is escaped as `&apos;` rather than left alone because the same function
/// is used for attribute values, and an apostrophe in a title is common.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use toml::value::Datetime;

    use super::{Feed, Item, atom, escape, rfc2822, rfc3339, rss, weekday};

    fn date(text: &str) -> Datetime {
        text.parse().expect("not a date")
    }

    fn feed(collection: &str) -> Feed {
        Feed {
            collection: collection.to_string(),
            title: None,
            limit: 20,
        }
    }

    #[test]
    fn a_feed_covers_the_pages_below_its_collection() {
        let notes = feed("notes");

        assert!(notes.covers("notes/one"));
        assert!(notes.covers("notes/deep/two"));
        assert!(
            !notes.covers("notes"),
            "the index is what the feed is for, not in"
        );
        assert!(!notes.covers("docs/one"));
        assert!(!notes.covers(""));
    }

    #[test]
    fn a_feed_for_the_whole_site_covers_everything() {
        let all = feed("");

        assert!(all.covers("anything"));
        assert!(all.covers(""));
    }

    #[test]
    fn a_feeds_files_sit_beside_the_collection_they_carry() {
        assert_eq!(
            feed("notes").paths(),
            ("notes/feed.xml".into(), "notes/atom.xml".into())
        );
        assert_eq!(feed("").paths(), ("feed.xml".into(), "atom.xml".into()));
    }

    #[test]
    fn the_weekday_maths_is_right() {
        // Anchors: the day this was written, an epoch, and a leap day.
        assert_eq!(weekday(2026, 9, 8), 2, "2026-09-08 is a Tuesday");
        assert_eq!(weekday(1970, 1, 1), 4, "the epoch is a Thursday");
        assert_eq!(weekday(2024, 2, 29), 4, "2024-02-29 is a Thursday");
        assert_eq!(weekday(2000, 1, 1), 6, "2000-01-01 is a Saturday");
        assert_eq!(weekday(1900, 1, 1), 1, "1900 is not a leap year: a Monday");
    }

    #[test]
    fn a_date_with_no_time_is_midnight_utc() {
        // Guessing a time zone from the build machine would make the same
        // content produce different feeds on different machines.
        assert_eq!(rfc3339(&date("2026-09-08")), "2026-09-08T00:00:00Z");
        assert_eq!(
            rfc2822(&date("2026-09-08")),
            "Tue, 08 Sep 2026 00:00:00 +0000"
        );
    }

    #[test]
    fn a_date_with_a_time_keeps_it() {
        assert_eq!(
            rfc3339(&date("2026-09-08T14:30:00Z")),
            "2026-09-08T14:30:00Z"
        );
        assert_eq!(
            rfc2822(&date("2026-09-08T14:30:00Z")),
            "Tue, 08 Sep 2026 14:30:00 +0000"
        );
    }

    #[test]
    fn xml_escaping_covers_what_a_title_can_contain() {
        assert_eq!(escape("Rust & <you>"), "Rust &amp; &lt;you&gt;");
        assert_eq!(escape("it's \"quoted\""), "it&apos;s &quot;quoted&quot;");
    }

    #[test]
    fn both_feeds_carry_the_entries_and_say_where_they_are() {
        let when = date("2026-09-08");
        let items = [Item {
            title: "A note & a half",
            url: "https://example.com/notes/one/".into(),
            date: &when,
            html: "<p>body</p>",
        }];

        let feed = rss(
            "Notes",
            "Things",
            "https://example.com/",
            "https://example.com/notes/feed.xml",
            &items,
        );

        assert!(feed.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(
            feed.contains("<title>A note &amp; a half</title>"),
            "got {feed}"
        );
        assert!(feed.contains("<pubDate>Tue, 08 Sep 2026 00:00:00 +0000</pubDate>"));
        assert!(
            feed.contains("&lt;p&gt;body&lt;/p&gt;"),
            "the body is escaped: {feed}"
        );
        assert!(feed.contains("rel=\"self\""));

        let feed = atom(
            "Notes",
            "https://example.com/",
            "https://example.com/notes/atom.xml",
            &items,
        );

        assert!(
            feed.contains("<updated>2026-09-08T00:00:00Z</updated>"),
            "got {feed}"
        );
        assert!(feed.contains("<id>https://example.com/notes/one/</id>"));
        assert!(
            feed.matches("<updated>").count() == 2,
            "the feed and the entry: {feed}"
        );
    }

    #[test]
    fn an_empty_feed_is_still_well_formed() {
        let feed = atom(
            "Notes",
            "https://example.com/",
            "https://example.com/atom.xml",
            &[],
        );

        assert!(feed.contains("<feed"), "got {feed}");
        assert!(feed.ends_with("</feed>\n"));
        assert!(
            !feed.contains("<updated>"),
            "no entries, so no timestamp to invent"
        );
    }
}
