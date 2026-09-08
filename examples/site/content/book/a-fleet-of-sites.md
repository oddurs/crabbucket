+++
title = "A fleet of sites"
description = "Twelve repositories, one house style, and what a version bump does and does not break."
layout = "docs"
order = 6
+++

# 6. A fleet of sites

This chapter is the reason the rest of the book is shaped the way it is.

You have a dozen repositories. Each has a small site: a landing page, a page
of documentation, maybe a changelog. Each was made at a different time, and
they look it. You want them to look like one thing, and you want to keep
wanting that in a year without a dozen pull requests every time you change
your mind about a heading.

[Chapter 5](../your-own-design-system/) built the design system. This chapter
is about depending on it from twelve places at once.

:::callout{kind = "note", title = "Written first"}
This chapter was drafted before the other eight, on the theory that the place
a design is weakest is the place its largest promise is made. It was: two of
the things below are annoying, and they are here because nobody else writes
them down.
:::

## What a site in a fleet looks like

A site that depends on a design system is a crate. It has to be: `crab build`
renders with the design system it was compiled with, and yours was not.

That is a smaller change than it sounds. The site is still a `content/`
directory of Markdown and a `site.toml`; it just gains three files.

```toml
# Cargo.toml
[package]
name = "ferrite-docs"
edition = "2024"

[dependencies]
crabbucket = "0.1"
house-style = { git = "https://github.com/me/house-style", tag = "v2.1.0" }
```

```rust
pub fn main() -> ExitCode {
    match crabbucket::build(Path::new("."), &Ferrite) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
```

That is the whole of `src/main.rs`. Twelve repositories contain that file and
differ only in what is in `content/`.

`cargo run` builds the site. `crab new --theme <url>` scaffolds all three
files for you, so in practice you write none of it.

## The version bump

Here is the claim, stated plainly so it can be checked:

> Restyling twelve sites is a version bump in twelve `Cargo.toml` files.

It is true, and it is worth being precise about why. The sites do not contain
colours, spacing, fonts, class names, or HTML. They contain Markdown and
frontmatter. Everything visual crossed the boundary as a *dependency*, so
moving it moves everything downstream of it.

What the sites do contain is two kinds of name:

- **layout names** — `layout = "docs"` in a page's frontmatter
- **component names** — `:::callout` in a page's body

Those two are your design system's public surface. Tokens, class names,
element structure and CSS are not: no site ever writes them down. That
asymmetry is the whole of what follows.

## What is free

**Adding a layout.** The enum grows a variant nothing names yet.

**Adding a component.** Same.

**Adding, removing or renaming a token.** Sites never name tokens. Renaming
one breaks *your* components, at compile time, in your own crate:

```
error[E0425]: cannot find value `ACCENT` in module `tok::color`
```

which is a build failure in the design system, before anything is published.

**Changing every colour, every font, the whole page shell.** This is the
change you actually wanted, and it is free by construction.

## What breaks

**Removing a layout.** Every page that names it fails:

```
error: unknown layout `gallery'
  --> content/press/index.md:3:10
   |
 3 | layout = "gallery"
   |          ^^^^^^^^^ expected one of: page, docs, landing
```

The file, the line, and the alternatives. That is the good failure — nothing
renders wrong, it simply does not render — but it is a breaking change and it
belongs behind a major version.

Most of the time you do not have to make it. Keep the old name as an alias
for whatever replaced it:

```rust
    /// Prose with the rest of its section listed beside it.
    #[serde(alias = "landing")]
    Docs,
```

One line, and twelve sites need no edits at all.

**Removing a component.** Same shape: the page names a directive nothing is
registered under, and the build stops with the file and the line. The same
escape hatch does not quite exist — a directive is registered by string, so
you register the old name too, pointing at the new handler.

**Renaming a component's class.** This is the one with no guarantee. If a
site wrote its own CSS against `.hs-callout`, nothing in the build knows, and
the site keeps building while looking wrong. Treat class names as public API,
or make it a rule that sites do not write CSS. This is stated as a limitation
rather than defended: it is the one place the design has nothing to offer.

## Migrating twelve sites

Not at once.

1. Bump the dependency in **one** site — the smallest one, or the one you
   know best.
2. Build it. Every failure is a layout name or a component name, each with a
   file and a line.
3. Fix them. Or, better, go back to the design system and add the aliases
   that would have made them not fail, then start again at step 1 and watch
   it pass.
4. Only then bump the other eleven.

Step 3 is the one people skip and should not. If a site needed edits, that is
evidence about the change, not about the site — and eleven more sites are
about to need the same edits.

## Previews, and the base path

A fleet has a second problem the single site does not: the same content is
served from more than one URL. A GitHub Pages project site lives at
`/repository/`; a pull-request preview lives at the root, or under a hash, or
somewhere a bot decided.

crabbucket takes the base path as a build input rather than as content, so
one site can do both:

```rust
pub fn build_at(site: &Path, base: Option<String>) -> Result<Report> {
    crabbucket::build_with(site, &Ferrite, Options::new().maybe_base(base))
}
```

Every link, asset and stylesheet URL is rewritten from that one value. No
page knows where it is being served from, which is why nothing has to be
edited when the answer changes. [Chapter 3](../routes-and-links/) is the long
version.

## The two annoying parts

**One: the design system's own version numbers become a thing you think
about.** With one site you can move fast and fix it. With twelve, a breaking
change costs twelve bumps and twelve reviews, so you start to hesitate — and
hesitating about design changes is exactly what having a design system was
supposed to stop. The answer is to use aliases aggressively so that fewer
changes are breaking. It is a real cost and it does not go away.

**Two: `git` dependencies do not resolve to a lockfile you can read at a
glance.** Twelve `Cargo.lock` files pinning twelve commits of the same theme
is not obviously twelve different versions until you look. Publish the theme
to a registry if you can — a version number in twelve manifests is legible
in a way a commit hash is not.

## What you have now

A design system in one crate, twelve sites depending on it, and a build that
fails loudly when a page names something the design system no longer has.

The next three chapters are the boring, necessary ones: [the dev
loop](../the-dev-loop/), [deploying](../deploying/), and a
[reference](../reference/) to keep beside you.
