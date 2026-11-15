# Design

## Context

The Wayland test sends Alt+F4 through the real input system after detecting visibility, but keyboard focus is asynchronous. Its failure traces contain no final close event. Windows already copies the panel executable to a content-addressed owner cache; macOS currently starts an owner inside the removable app. Intel package CI fails the post-removal owner assertion, while its logs omit the observed status.

## Goals / Non-Goals

Preserve the independent owner and resource lifetime, require an observable native input target, and retain enough diagnostics to identify a failure stage. Do not relax deadlines, bypass acceptance, change pixels or accept new goldens automatically.

## Decisions

- Require the main window to be visible and focused before native close input. This matches the real keyboard operation and does not call the product's close command or synthesize success.
- Reuse the existing hash-verified immutable owner cache on macOS and Windows. macOS copies receive private executable permissions before publication and use an extensionless name. Retaining the standalone early daemon mode avoids dependence on bundle removal or replacement. A source-injection helper permits isolated executable-copy tests without changing the running test binary.
- Inspect the running macOS owner through libproc and Windows through the existing native API. Production package tests assert that the owner path is the retained build, check its identity separately after client quit and removal, and perform a real queue query while the package is absent. Existing two-cycle install/remove/reinstall and queue/preference fingerprints remain required.
- Verify existing reviewed screenshots on Linux/macOS/Windows because presentation is unchanged. Use required native GNOME/Wayland and both macOS package jobs to confirm the lifecycle fixes.

## Risks / Trade-offs

- Retained executable copies consume disk space per panel build. Preserve them while an owner may still execute; cache cleanup is outside this fix.
- Native focus or process inspection can be denied. Acceptance fails with observed diagnostics rather than skipping or simulating those operations.
- The Intel failure's exact kernel-level termination reason is not in the saved artifacts. Assert the stronger invariant that a panel-started owner never executes from the removable installation, and verify the real native lifecycle in CI.

## Migration Plan

New panel-started owners use retained executables. Existing owners keep their lifetime and are not replaced automatically. No database or protocol migration is required.
