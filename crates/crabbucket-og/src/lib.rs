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
//! OpenGraph cards, drawn from a design system's own tokens.
//!
//! A link to a page shared anywhere renders as a grey rectangle without one,
//! which for a project whose pitch is that things should look considered is
//! the most visible unconsidered surface it has.
//!
//! This is a separate crate because it costs 76 transitive crates and a few
//! seconds of compilation, and a site with no interest in social previews
//! should pay neither.  A design system that wants cards depends on it and
//! implements [`crabbucket::Theme::og_image`]; `crabbucket` itself does not
//! know it exists.
//!
//! The font is bundled rather than found on the machine.  An image generated
//! during a build has to look the same everywhere the build runs, and a font
//! the build merely hopes to find does not give that.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree, fontdb};

/// The size every social platform expects.
pub const WIDTH: u32 = 1200;

/// The height that goes with it.
pub const HEIGHT: u32 = 630;

/// Public Sans, under the SIL Open Font License 1.1.  See `fonts/OFL.txt`.
///
/// Two static instances rather than the variable family: resvg does not apply
/// a variable font's weight axis, so the variable file rendered every title at
/// Thin whatever `font-weight` said.
const REGULAR: &[u8] = include_bytes!("../fonts/PublicSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../fonts/PublicSans-Bold.ttf");

/// The family name the SVG asks for.
const FAMILY: &str = "Public Sans";

/// Something went wrong drawing a card.
#[derive(Debug)]
pub enum Error {
    /// The SVG this crate composed was not valid, which is this crate's bug.
    Svg(resvg::usvg::Error),
    /// The image could not be encoded.
    Encode(png::EncodingError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Svg(err) => write!(f, "the card is not valid SVG: {err}"),
            Error::Encode(err) => write!(f, "the card could not be encoded: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// The colours a card is drawn in, which come from the design system's tokens
/// rather than from here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Palette<'a> {
    /// The card's background.
    pub background: &'a str,
    /// The page title.
    pub foreground: &'a str,
    /// The site title, and anything secondary.
    pub muted: &'a str,
    /// The rule down the side.
    pub accent: &'a str,
}

/// What a card says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card<'a> {
    /// The page's title, which is the point of the card.
    pub title: &'a str,
    /// The site's title, smaller, above it.
    pub site: &'a str,
    /// The colours.
    pub palette: Palette<'a>,
}

/// Draws a card and encodes it as a PNG.
///
/// # Errors
///
/// Fails only if the SVG this crate composes is invalid, or if encoding fails.
/// Both are bugs here rather than in the caller.
pub fn render(card: &Card<'_>) -> Result<Vec<u8>, Error> {
    let svg = compose(card);

    let mut fonts = fontdb::Database::new();
    fonts.load_font_data(REGULAR.to_vec());
    fonts.load_font_data(BOLD.to_vec());

    let options = Options {
        fontdb: Arc::new(fonts),
        ..Options::default()
    };
    let tree = Tree::from_str(&svg, &options).map_err(Error::Svg)?;

    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).expect("the card's size is not zero");
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());

    pixmap.encode_png().map_err(Error::Encode)
}

/// Draws a card, or reads it back from `cache` if it has been drawn before.
///
/// A card costs about 65ms.  A thirteen-page site is most of a second, which
/// on a build that otherwise takes a tenth of one is the difference between a
/// dev loop and a wait.  The key covers everything that changes the image, so
/// a hit is only ever the same picture.
///
/// A cache that cannot be read or written is not an error: the card is drawn
/// instead.  The worst case is the speed it had before.
pub fn cached(cache: &Path, card: &Card<'_>) -> Result<Vec<u8>, Error> {
    let path = cache.join(format!("{}.png", key(card)));

    if let Ok(found) = fs::read(&path) {
        return Ok(found);
    }

    let png = render(card)?;

    if fs::create_dir_all(cache).is_ok() {
        let _ = fs::write(&path, &png);
    }

    Ok(png)
}

/// What a cached card is filed under.
///
/// Everything that changes the picture: the words, the colours, the size, and
/// the layout, which is versioned by hand because a change to `compose` is not
/// otherwise visible from here.
fn key(card: &Card<'_>) -> String {
    /// Bump when `compose` changes what it draws.
    const LAYOUT: u32 = 1;

    let mut hasher = DefaultHasher::new();

    LAYOUT.hash(&mut hasher);
    (WIDTH, HEIGHT).hash(&mut hasher);
    card.title.hash(&mut hasher);
    card.site.hash(&mut hasher);
    card.palette.hash(&mut hasher);

    format!("{:016x}", hasher.finish())
}

/// The SVG a card is, before it is a PNG.
///
/// Public so that a design system can look at what it is about to get, and so
/// that the layout is testable without rasterising anything.
pub fn compose(card: &Card<'_>) -> String {
    /// The left margin, and where the accent rule ends.
    const LEFT: i32 = 96;
    /// The baseline of the last line of the title.
    ///
    /// The block is bottom-aligned rather than centred, so a one-line title
    /// and a three-line one share an edge instead of drifting apart.
    const FLOOR: i32 = 500;

    let title = fit(card.title);
    let first = FLOOR - (title.lines.len() as i32 - 1) * title.leading;

    let mut lines = String::new();
    for (index, line) in title.lines.iter().enumerate() {
        let y = first + index as i32 * title.leading;
        lines.push_str(&format!(
            "<text x=\"{LEFT}\" y=\"{y}\" font-family=\"{FAMILY}\" font-size=\"{}\" \
             font-weight=\"bold\" fill=\"{}\">{}</text>",
            title.size,
            card.palette.foreground,
            escape(line)
        ));
    }

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{HEIGHT}\" \
         viewBox=\"0 0 {WIDTH} {HEIGHT}\">\
         <rect width=\"{WIDTH}\" height=\"{HEIGHT}\" fill=\"{}\"/>\
         <rect x=\"0\" y=\"0\" width=\"12\" height=\"{HEIGHT}\" fill=\"{}\"/>\
         <text x=\"{LEFT}\" y=\"120\" font-family=\"{FAMILY}\" font-size=\"26\" \
         fill=\"{}\" letter-spacing=\"3\">{}</text>\
         <rect x=\"{LEFT}\" y=\"552\" width=\"64\" height=\"5\" fill=\"{}\"/>\
         {lines}\
         </svg>",
        card.palette.background,
        card.palette.accent,
        card.palette.muted,
        escape(card.site),
        card.palette.accent,
    )
}

/// A title, broken into lines that fit, at a size that lets them.
#[derive(Debug, PartialEq, Eq)]
struct Fitted {
    lines: Vec<String>,
    size: u32,
    leading: i32,
}

/// The sizes to try, largest first, and how many lines each may take.
const SIZES: [(u32, usize); 3] = [(76, 2), (62, 3), (50, 4)];

/// Breaks a title into lines, shrinking the type until it fits.
///
/// The width is estimated from the character count rather than measured.  A
/// card is one short line of display type in one known font, and measuring it
/// properly would mean shaping the text twice -- once to decide, once to draw.
/// The estimate is deliberately cautious, so a title breaks early rather than
/// running off the edge.
fn fit(title: &str) -> Fitted {
    /// The usable width, in pixels, inside the margins.
    const USABLE: f32 = 990.0;
    /// Public Sans at weight 700 averages about this fraction of its size per
    /// character, measured over the Latin lowercase alphabet.
    const ADVANCE: f32 = 0.55;

    for (size, limit) in SIZES {
        let per_line = (USABLE / (size as f32 * ADVANCE)) as usize;
        let lines = wrap(title, per_line.max(1));

        if lines.len() <= limit {
            return Fitted {
                lines,
                size,
                leading: (size as f32 * 1.2) as i32,
            };
        }
    }

    let (size, limit) = SIZES[SIZES.len() - 1];
    let per_line = (USABLE / (size as f32 * ADVANCE)) as usize;
    let mut lines = wrap(title, per_line.max(1));

    // A title too long for the card is cut rather than allowed to overflow it.
    lines.truncate(limit);
    if let Some(last) = lines.last_mut() {
        last.push('…');
    }

    Fitted {
        lines,
        size,
        leading: (size as f32 * 1.2) as i32,
    }
}

/// Greedy word wrapping.
fn wrap(text: &str, per_line: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let would_be = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if !current.is_empty() && would_be > per_line {
            lines.push(std::mem::take(&mut current));
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Escapes text for XML.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Card, HEIGHT, Palette, WIDTH, compose, fit, render, wrap};

    fn card(title: &str) -> Card<'_> {
        Card {
            title,
            site: "crabbucket",
            palette: Palette {
                background: "#0b0d10",
                foreground: "#e7eaf0",
                muted: "#98a2b3",
                accent: "#e8623c",
            },
        }
    }

    #[test]
    fn a_card_is_the_size_every_platform_expects() {
        let png = render(&card("Routing")).expect("cannot render");

        // PNG width and height live at bytes 16..24, big-endian.
        let width = u32::from_be_bytes(png[16..20].try_into().expect("short png"));
        let height = u32::from_be_bytes(png[20..24].try_into().expect("short png"));

        assert_eq!((width, height), (WIDTH, HEIGHT));
        assert_eq!(&png[1..4], b"PNG");
    }

    #[test]
    fn the_same_card_renders_the_same_bytes_every_time() {
        // A build that produced a different image each run would churn every
        // deploy and every cache for no reason.
        assert_eq!(
            render(&card("Routing")).unwrap(),
            render(&card("Routing")).unwrap()
        );
    }

    #[test]
    fn the_colours_come_from_the_palette_and_nowhere_else() {
        let svg = compose(&card("Routing"));

        assert!(svg.contains("fill=\"#0b0d10\""), "got {svg}");
        assert!(svg.contains("fill=\"#e8623c\""), "the accent rule: {svg}");
        assert!(svg.contains("fill=\"#e7eaf0\""), "the title: {svg}");
    }

    #[test]
    fn a_title_is_escaped_rather_than_breaking_the_svg() {
        let svg = compose(&card("Rust & <you>"));

        assert!(svg.contains("Rust &amp; &lt;you&gt;"), "got {svg}");
        render(&card("Rust & <you>")).expect("an escaped title still renders");
    }

    #[test]
    fn a_short_title_gets_the_largest_type_and_one_line() {
        let fitted = fit("Routing");

        assert_eq!(fitted.lines, ["Routing"]);
        assert_eq!(fitted.size, 76);
    }

    #[test]
    fn a_longer_title_wraps_before_it_shrinks() {
        let fitted = fit("Why routes became a type and what that cost");

        assert!(fitted.lines.len() > 1, "it should wrap: {fitted:?}");
        assert!(
            fitted.lines.len() <= 3,
            "it should not need four lines: {fitted:?}"
        );
    }

    #[test]
    fn a_title_too_long_for_the_card_is_cut_rather_than_overflowing_it() {
        let fitted = fit(&"word ".repeat(200));

        assert_eq!(
            fitted.lines.len(),
            4,
            "the last size allows four: {fitted:?}"
        );
        assert!(fitted.lines.last().expect("no lines").ends_with('…'));
    }

    #[test]
    fn wrapping_never_splits_a_word() {
        let lines = wrap("alpha beta gamma delta", 11);

        for line in &lines {
            assert!(
                line.len() <= 11 || !line.contains(' '),
                "{line:?} was split badly"
            );
        }
        assert_eq!(lines.join(" "), "alpha beta gamma delta", "a word was lost");
    }

    #[test]
    fn a_word_longer_than_a_line_is_left_whole() {
        // Better one overflowing line than a title nobody can read.
        let lines = wrap("supercalifragilistic", 5);
        assert_eq!(lines, ["supercalifragilistic"]);
    }

    #[test]
    fn a_cached_card_is_the_same_card() {
        use super::cached;

        let dir = std::env::temp_dir().join(format!("crabbucket-og-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let drawn = cached(&dir, &card("Routing")).expect("cannot draw");
        let read_back = cached(&dir, &card("Routing")).expect("cannot read back");

        assert_eq!(drawn, read_back);
        assert_eq!(std::fs::read_dir(&dir).expect("no cache").count(), 1);
    }

    #[test]
    fn a_different_card_is_a_different_cache_entry() {
        use super::cached;

        let dir = std::env::temp_dir().join(format!("crabbucket-og-two-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        cached(&dir, &card("Routing")).expect("cannot draw");
        cached(&dir, &card("Layouts")).expect("cannot draw");

        let mut recoloured = card("Routing");
        recoloured.palette.accent = "#00ff00";
        cached(&dir, &recoloured).expect("cannot draw");

        assert_eq!(
            std::fs::read_dir(&dir).expect("no cache").count(),
            3,
            "a colour change has to miss, or the card would be stale"
        );
    }

    #[test]
    fn a_cache_that_cannot_be_written_is_not_an_error() {
        use super::cached;

        // /dev/null is not a directory, so creating it fails and writing to it
        // fails.  The card is still drawn.
        let png = cached(Path::new("/dev/null/nope"), &card("Routing")).expect("cannot draw");
        assert_eq!(&png[1..4], b"PNG");
    }

    #[test]
    fn an_empty_title_still_renders() {
        render(&card("")).expect("cannot render");
    }
}
