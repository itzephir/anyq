# Reviewed native screenshot goldens

Each platform directory contains 15 reviewed PNGs generated from `902f968` by
[the native generator matrix](https://github.com/itzephir/anyq/actions/runs/42888007910).
All three native jobs passed real application/daemon integration and two independent
clean launches of every configured visual state before creating these 45 files.
The downloaded ZIP digests matched the GitHub artifact digests. The source PNGs
were reviewed without replacing or synthesizing the native pixels.

| Platform | Native runner | Reviewed files |
| --- | --- | --- |
| Linux | Ubuntu 24.04, X11/Xvfb and Openbox | 15 |
| macOS | macOS 15, ARM64 | 15 |
| Windows | Windows hosted runner, native WebView2 | 15 |

The fixture, clock/timezone, preferences, viewport, selection, scroll/focus,
cursor, capture readiness, kerning policy and comparison settings are owned by
[`../config.json`](../config.json). See the [runner contract](../README.md) for
explicit `verify` and `generate` modes and native antialias handling.

Missing files, unexpected dimensions, unstable independent renders and visual
regressions fail verification. Updating these images requires an explicit
native `generate` run, inspection and a repository commit.
