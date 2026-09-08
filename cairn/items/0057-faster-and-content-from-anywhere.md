---
id: 57
key: v1.2
title: Faster, and content from anywhere
type: milestone
status: backlog
created: 2026-09-08
updated: 2026-09-08
priority: p2
---

Two constraints that are currently baked in and should not be.

A build rebuilds everything, every time. At 521 pages that is 250ms and
nobody minds; at five thousand it is two and a half seconds and a dev loop
that stutters.

And content must be a directory of Markdown on this machine. That is the
right default and a poor limit: the type is the schema, so there is no reason
the bytes have to come from a file.
