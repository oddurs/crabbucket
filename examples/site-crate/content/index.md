+++
title = "Typed routes"
nav_order = 1
nav_label = "Home"
+++

# Typed routes

This site is a crate. Its `build.rs` scans `content/` and generates a `Route`
enum, so a link written in Rust does not compile if the page moves.

The navigation you are looking at is built from that enum, not from strings.
