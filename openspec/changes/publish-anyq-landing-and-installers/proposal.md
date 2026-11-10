# Proposal

## Why

anyq needs a dedicated introductory landing page and prebuilt binary installation without a Rust toolchain. The project should appear in the itzephir.com catalog, with an editable layout in the existing OpenDesign.

## What Changes

- Static anyq.itzephir.com landing page in this repository, using the itzephir foundation, two themes, and a mobile layout.
- Shell and PowerShell installers with platform detection, SHA-256, and default system installation; explicit user mode remains available.
- Native release builds for macOS/Linux/Windows and complete publication through GitHub Releases.
- Reproducible OpenDesign package and landing container for the existing project gateway.
- anyq catalog entry through /dev without portfolio code changes.
- Automatic dev.anyq.itzephir.com deployment from PRs and production from main; project ingress independently selects the environment by Host.

## Capabilities

### New Capabilities

- `binary-distribution`: release and verified installation of prebuilt binaries.
- `project-website`: landing page, editable layout, and publication through the existing gateway.

### Modified Capabilities

None. Queue domain contracts and required CI remain unchanged.

## Impact

New website/, install/, release/deploy tooling, and GitHub workflows. anyq runtime gains no dependencies. Design uses licensed self-hosted IBM Plex Sans/Mono. Public files contain no operator addresses or credentials. PR merging and version releases remain separate explicit actions.
