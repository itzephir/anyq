# Design

## Context

See proposal.md. Runtime is already native and SQLite is bundled in the binary. The portfolio has a project registry and shared Docker gateway. OpenDesign is available through a local CLI with existing SSH access.

## Goals / Non-Goals

**Goals:** anyq owns page, installer, and delivery sources; archives and platforms are verified before publication; the layout uses the same page.

**Non-Goals:** queue changes, a task GUI, copying the portfolio shell, or automatic PR merging.

## Decisions

- Static HTML/CSS/JS with self-hosted IBM Plex and itzephir semantic tokens. The page explains atomic admission and explicit complete. Small JS persists themes and copies commands; primary content works without JS.
- Shell for Unix and PowerShell 5.1+ for Windows. Archives have fixed Rust-target names; SHA256SUMS comes from the same version. GitHub redirects resolve latest; a version argument supports pinning. Unix defaults to /usr/local/bin and Windows to Program Files/anyq with system PATH. --user and -Scope User select ~/.local/bin and LOCALAPPDATA/Programs/anyq without elevation. A user directory may be specified explicitly.
- Download, SHA-256, and --version run before requesting elevated permissions. When needed, sudo/UAC only places the verified binary and updates system PATH. A temporary file is created beside the destination and its hash checked again before atomic replacement; permission refusal preserves the old version. Windows rolls back replacement on PATH-write failure. UAC arguments use JSON inside EncodedCommand without interpolating paths into code. Native CI runners test actual system directories and restore PATH afterward.
- Linux musl for portability, native macOS targets, and Windows x64 MSVC. CI first tests local artifact installation through HTTP fixtures; releases build the same format. A release is published only after collecting every artifact and checking the tag against the Cargo version.
- Project nginx ingress in the existing gateway network selects dev/prod by Host. Two static containers read their own current from read-only mounts in the private anyq network. CI sends the verified site ZIP over SSH; a restricted receiver accepts only public files, atomically switches current, and checks HTTP. Failure restores the old symlink. Separate dev/prod SSH keys use forced commands without arbitrary shell access. Production environment is restricted to main. Addresses and credentials live only in GitHub secrets/operator configuration.
- OpenDesign gets a separate anyq project with designSystemId user:itzephir. Upload the allowlisted build rather than the checkout. Verify matching files.

## Risks / Trade-offs

- No first release yet → installer reports a GitHub error; documentation distinguishes implemented code from a published version.
- Checksums from the same HTTPS source are not an independent signature → SHA-256 protects download integrity; GitHub Releases remain the trusted source.
- macOS binaries lack notarization → distribute CLI archives over HTTPS without claiming Apple signing.
- Linux ARM builds → use a separate native ARM runner and musl toolchain.

## Migration Plan

PRs run required CI, installer fixtures, and container checks; success publishes the ZIP to dev.anyq.itzephir.com. Main publishes production after merge. An operator sets up project containers/receiver once and registers anyq-gateway:8080 in the existing locator; the itzephir owner enables the DNS/TLS namespace. A separate version tag releases binaries. Website rollback restores previous current; installers can pin older versions. CLI data is not migrated.
