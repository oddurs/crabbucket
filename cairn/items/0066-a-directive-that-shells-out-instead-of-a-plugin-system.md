---
id: 66
title: A directive that shells out, instead of a plugin system
type: docs
status: backlog
milestone: v1.1
created: 2026-09-08
updated: 2026-09-08
priority: p2
effort: m
area: content
---

## Problem

crabbucket refuses a plugin system, and the refusal is right: a dynamic
extension point would trade the type checking that is the entire product for
configurability nobody asked for.

But the refusal answers a question nobody was asking. What people actually
want is a diagram in a page, or a formula, and the answer today is "write the
SVG by hand".

mdBook solves this with a protocol rather than an ABI: `trait Preprocessor {
fn run(&self, ctx, book) -> Result<Book>; fn supports_renderer(&self, r) ->
Result<bool>; }`, and the real extension point is out-of-process —
`parse_input<R: Read>(reader)` reads `(PreprocessorContext, Book)` as JSON
from stdin, and the preprocessor writes a transformed `Book` to stdout. Any
executable in any language can be one.

It is a good design and it is the wrong one here, for a specific reason: an
mdBook preprocessor transforms the *whole book* as untyped JSON, before
anything is typed. Adopting it would put an untyped transformation upstream of
a pipeline whose value is that everything downstream is typed.

## Proposal

Do not build a plugin system. Document the pattern that already works, and
make it pleasant.

A directive is a Rust function that receives typed attributes. Nothing stops
it invoking `mermaid`, `dot`, `d2` or `katex` and caching the result — and the
build already has the machinery: the social-card cache in `.crabbucket/` is
exactly this, keyed by content.

What is missing is not capability but three affordances:

- **A cache helper.** `Context` should offer "give me the bytes for this key,
  or run this closure and remember them" so a directive does not reimplement
  content-addressed caching. `Page` already carries `cache: &Path`; directives
  do not.
- **An honest failure.** A directive whose external tool is not installed
  should fail the build naming the tool and how to install it, not produce an
  empty `<figure>`. And the same site must build in CI, so the cached output
  needs to be committable — meaning the cache key has to be stable across
  machines, which means hashing the *input*, not the tool's output.
- **A worked example.** One directive in the repository that shells out —
  mermaid is the obvious one — compiled and tested, so the pattern is
  demonstrated rather than described.

The result is extensibility with no ABI, no dynamic linking, no protocol
version, and no untyped stage: the extension is a crate that implements a
trait, which is what the design already said.

## Acceptance criteria

- [ ] A content-addressed cache helper on `Context`
- [ ] A missing external tool fails the build naming the tool
- [ ] The cache key is the input, so output is reproducible and committable
- [ ] One real shelling-out directive in the tree, tested
- [ ] `doc/DESIGN` explains why this rather than a plugin protocol, and cites
      mdBook's as the alternative considered
