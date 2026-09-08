//! Renders a few cards to /tmp so a human can look at them.

use crabbucket_og::{Card, Palette, render};

fn main() {
    let palette = Palette {
        background: "#0b0d10",
        foreground: "#e7eaf0",
        muted: "#98a2b3",
        accent: "#e8623c",
    };

    let titles = [
        ("short", "Routing"),
        ("medium", "Why routes became a type"),
        (
            "long",
            "A site is a typed value, and the build either fails or the site is correct",
        ),
    ];

    let started = std::time::Instant::now();
    for (name, title) in titles {
        let png = render(&Card {
            title,
            site: "crabbucket",
            palette,
        })
        .expect("cannot render");
        std::fs::write(format!("/tmp/og-{name}.png"), &png).expect("cannot write");
        println!("{name}: {} bytes", png.len());
    }
    println!("three cards in {:?}", started.elapsed());
}
