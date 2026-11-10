# Proposal

## Why

One job combines lint, builds, and integration; the default branch is unprotected. The user requested separate CI jobs and no merging before they succeed.

## What Changes

- Split Linux-only lint from native macOS/Windows/Linux build-and-unit-tests/integration-tests.
- Add aggregate CI passed requiring every category's success, including failure/cancellation/skipping handling.
- Protect the default branch: PRs, current required CI passed, administrators included, no force/deletion, resolved discussions.
- Store protection configuration and verify actual settings through GitHub API.

## Capabilities

### New Capabilities

- `repository-delivery`: CI categories and default-branch protection against bypassing CI.

### Modified Capabilities

No baseline specs; platform checks are already described in completed support-windows-ci and remain unchanged.

## Impact

GitHub Actions workflow, branch-protection configuration, and GitHub settings. Queue runtime/data are unchanged.
