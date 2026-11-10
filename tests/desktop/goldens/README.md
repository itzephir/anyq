# Reviewed native screenshot goldens

Linux and Windows each contain 15 reviewed PNGs generated from `58edbf5` by
[the native generator matrix](https://github.com/itzephir/anyq/actions/runs/42882733857).
Both native jobs passed real application/daemon integration and two independent
clean launches of every configured screenshot state before creating these files.
The downloaded ZIP digests matched the GitHub artifact digests.

macOS goldens remain pending its independent-render acceptance. Do not copy
another platform's files or accept the currently rejected macOS text rendering.

The fixture, clock/timezone, preferences, viewport, selection, scroll/focus,
cursor, capture readiness and comparison settings are owned by
[`../config.json`](../config.json). See the [runner contract](../README.md) for
explicit `verify` and `generate` modes and native antialias handling.
