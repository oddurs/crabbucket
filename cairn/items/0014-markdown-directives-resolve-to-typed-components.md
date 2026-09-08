---
id: 14
title: Markdown directives resolve to typed components
type: feature
status: done
milestone: v0.2
labels:
- design
created: 2026-09-08
updated: 2026-09-08
priority: p0
effort: l
area: content
---

## Problem

`doc/DESIGN` section 5 promises this and it is the largest unbuilt piece of
the design. Without it, a Markdown page can use exactly the elements
CommonMark defines — so a callout, a card, a tabbed example or anything else
the design system offers is unreachable from the place where content is
actually written.

The alternative everyone reaches for is raw HTML in Markdown, which throws
away every guarantee this project is built on at the exact moment someone
wants something to look good.

## Proposal

Generic directive syntax, the CommonMark community's proposal, which is
already what MyST and remark-directive use:

    :::callout{kind="warn"}
    crabbucket needs Rust 1.85 or newer.
    :::

Resolution:

1. A theme registers directives by name against a handler.
2. The attribute block parses as a TOML inline table.
3. Attributes deserialize into the handler's own props type, so `kind="wrn"`
   fails with serde's "unknown variant" message and the file and line.
4. The body is rendered as Markdown and passed in as `Markup`.
5. An unregistered name fails the build listing the registered ones.

Implementation note: pulldown-cmark has no directive support, so this is a
pass over its event stream — recognise the fence, capture until the closing
one, recurse for the body. Do not preprocess the source text with string
replacement; that breaks the moment a directive appears inside a code block,
which is exactly what these very docs do.

Inline directives (`:name[text]{attrs}`) are out of scope until the block
form has been used in anger.

## Acceptance criteria

- [x] `:::name{attrs}` resolves to a registered component
- [x] Attributes deserialize into a typed props struct
- [x] Unknown directive fails the build, listing what is registered
- [x] Bad attributes fail with the file, the line, and serde's message
- [x] A directive written inside a fenced code block is left alone
- [x] Directive bodies may contain Markdown, including nested directives
- [x] `doc/DESIGN` section 5 matches what was built

## 2026-09-08

Done. The scanner works on lines rather than on pulldown-cmark's event stream, for one reason: a directive written inside a fenced code block must be left alone, and the components documentation page does exactly that. Tracking fences over lines is a dozen lines; recovering original text from an event stream is not.

Attributes are a TOML inline table, so quoting, escaping and nesting are TOML's problem. Directives::add takes the props type as a parameter and deserializes before calling the handler, so a handler never sees an attribute it did not ask for.

One consequence worth knowing and now documented in the module: each run of Markdown between directives is parsed as its own document, so a link reference definition or footnote is visible only within its run. Directives are for components rather than prose, so it has not bitten; the fix if it does is to hoist definitions, not to merge runs.
