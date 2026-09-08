+++
title = "One implementation of one rule"
date = 2026-09-07
+++

# One implementation of one rule

Route derivation used to live in `crabbucket`. Generating the enum needed the
same answer at build time, and copying it would have left two implementations
of the rule the whole routing model rests on.

So it moved to `crabbucket-routes`, which both call.
