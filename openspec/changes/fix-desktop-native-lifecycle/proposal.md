# Proposal

## Why

Required desktop acceptance fails when the Wayland host-loss fixture closes a restored window before it has keyboard focus, and when the macOS Intel owner does not remain available after removing its app bundle. The PR must verify reliable window input and an owner lifetime independent of the installed panel.

## What Changes

- Wait for the restored main window's actual focus before sending native keyboard closure in host-loss acceptance.
- Launch macOS owners from a verified retained executable outside the removable app, extending the existing Windows strategy.
- Verify the retained owner's actual process path, availability after client exit and package removal, unchanged queue data, and successful reinstall on both macOS architectures.
- Keep required integration and screenshot verification on all supported systems.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `desktop-queue-panel`: explicit host-loss closure and installed-package-independent owner lifetime acceptance.

## Impact

Desktop composition and retained owner file handling; native macOS process inspection; Wayland input fixtures; production package acceptance and documentation. Queue state, public protocols, presentation and reviewed PNGs remain compatible.
