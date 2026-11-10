# Design

## Context

PR #1 already passed a combined CI job on macOS/Windows/Linux. Default branch is main; user access is ADMIN; no protection/rulesets yet. Existing check runs confirm GitHub Actions app id 15368.

## Goals / Non-Goals

**Goals:** independent check categories, stable required status, and actually enforced main protection.

**Non-Goals:** unsolicited PR merging, mandatory third-party review for a personal project, releases, or automatic deployment.

## Decisions

- One Linux lint job (Rustfmt, Clippy, AST, Cargo graph, Ruff, OpenSpec) and two native macOS/Windows/Linux matrices: build-and-unit-tests (cargo build workspace, cargo test --lib --bins excluding SQLite adapter suite), integration-tests (SQLite adapter suite, CLI workflows, portable demo). Do not repeat common linters on other operating systems.
- CI passed uses always(), needs all three categories, and explicitly checks every needs.*.result == success. Ordinary dependency skipping is insufficient: GitHub treats skipped required checks as successful.
- Run workflow on PRs/main pushes. Open-PR branch pushes use synchronize without duplicate push/PR runs. Concurrency cancels obsolete branch/PR runs.
- Classic protection for verified default main: CI passed only from GitHub Actions, strict up-to-date, enforce_admins, PRs with zero required approvals, conversation resolution, no force/deletion, linear history. Keep squash/rebase available. Do not require someone else's review of the user's personal project.
- Store exact REST update in .github/branch-protection.json; GET verifies applied settings. Protection is not itself a workflow; payload/documentation do not prove enforcement.

## Risks / Trade-offs

- [Plan/permissions prohibit private branch protection] → inspect actual API error, retain ready configuration, explicitly report missing enforcement; do not change visibility/payment automatically.
- [Aggregate job renamed] → CI passed is a stable protection contract; update both together.
- [Owner removes protection in settings] → administrators control policy; current policy forbids ordinary merge/push bypass.
