---
id: 6
title: Check fragment links against heading ids
type: feature
status: done
milestone: v0.1
depends_on:
- 5
created: 2026-09-08
updated: 2026-09-08
priority: p1
effort: s
area: routing
---

## Problem

Link checking strips the fragment and checks only the page. So
`../routing/#link-checking` passes even when that heading has been renamed —
which is the *more* common failure, because headings get reworded far more
often than pages get renamed.

## Proposal

With heading ids available, extend the checker: when a resolved internal link
carries a fragment, look the fragment up in the target page's heading set.

Same-page links (`#the-thesis`) are checked against the current page.

Report it in the same batch and the same format:

    content/docs/routing.md: ../layouts/#the-trait -> /docs/layouts/ has no
      anchor `the-trait`

Do not check fragments on external links. Do not check `#` alone — it is a
convention for "no destination", not a broken link.

## Acceptance criteria

- [x] A fragment naming a missing heading fails the build
- [x] Same-page fragments are checked
- [x] Bare `#` is ignored
- [x] Fragments on external links are ignored
- [ ] Fixture coverage in the integration suite

## 2026-09-08

Done. resolve() now returns Link { path, fragment } and the route set became links::Routes = BTreeMap<route, BTreeSet<anchor>>. Reason enum distinguishes NoSuchRoute / NoSuchAsset / NoSuchAnchor so the diagnostic says which. Bare '#' is ignored; a bare '#frag' resolves against its own page.

## 2026-09-08

Fixture coverage deferred to 0002, which builds the harness. Six unit tests cover the behaviour meanwhile.
