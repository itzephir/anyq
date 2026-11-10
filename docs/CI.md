# CI and default branch protection

The complete check suite can also be triggered explicitly through workflow_dispatch for
a selected branch; manual execution does not weaken the final CI passed gate.

Workflow [.github/workflows/ci.yml](../.github/workflows/ci.yml) runs on PRs and pushes to `main`. Updates to an open PR branch are checked through synchronize events; pushing that branch does not trigger a duplicate run. A new commit cancels older runs for the same PR.

| Job | Platforms | Coverage |
| --- | --- | --- |
| Lint | Linux only | Rustfmt, Clippy --all-features with warnings denied, Rust AST linter, Cargo/API boundary checker, Ruff, OpenSpec |
| Build & unit tests | Linux, macOS, Windows | Workspace build; unit tests excluding SQLite/file adapters and CLI integration suites |
| Integration tests | Linux, macOS, Windows | SQLite/file adapter suites; CLI aliases/names/config/autocreate, concurrency, demo; separate required Native Ctrl+C integration tests step |
| Native distribution | macOS ARM64/Intel, Linux ARM64/x64, Windows x64 | Release build, archive, latest/pinned installation, and download/checksum failure rejection |
| Desktop packages | Linux, macOS ARM64/Intel, Windows | Feature-enabled controller tests and optional native packages; manual GUI acceptance remains separate |
| Website | Linux | EN/RU selection and interaction tests; static container, HTTP, installers, preserving the working version when an update fails |
| Deploy dev / prod website | Linux, after CI passed | Same-repository PRs update dev.anyq.itzephir.com; main pushes update anyq.itzephir.com. Separate environment keys and forced receiver; HTTPS revision checked after publication |
| CI passed | Aggregate of all categories | Passes only when Lint, both build/test matrices, distribution, website, and desktop packaging succeed |

`CI passed` uses `always()` and checks specific dependency results. Failure, cancelled, and skipped do not pass the gate. Matrices use `fail-fast: false` so an error on one OS does not hide others' results. Only this stable final status is linked to branch protection; renaming it requires updating protection at the same time.

The native suite runs with integration-test-hooks and sends real SIGINT on Unix or CTRL_C_EVENT in a separate console on Windows. It checks commits, intermediate writes, chains, autocreate, config replacement, actual lock waits, and full stdout/stderr pipes, without skip/ignore. Workflow/file suites reject unknown TOML sections and tables instead of values without rewriting the file. Ordinary cargo test without the feature does not replace the required native target run. See the outcome table in [INTERRUPTIONS.md](INTERRUPTIONS.md).

The default branch `main` is protected on GitHub. The exact configuration body is stored in [.github/branch-protection.json](../.github/branch-protection.json):

- All changes require a PR, including administrator changes.
- `CI passed` is required from GitHub Actions app 15368, verified against this repository's check runs.
- The PR branch must be up to date with `main` (`strict: true`).
- Force pushes and deletion are forbidden.
- Resolved discussions and linear history are required; squash/rebase are available.
- Required third-party approvals: 0; an author may merge their own verified PR.

Protection is applied through the API and checked with an actual GET, rather than inferred from a JSON file in the repository. To reapply with administrator permissions:

```bash
gh api --method PUT repos/itzephir/anyq/branches/main/protection --input .github/branch-protection.json
gh api repos/itzephir/anyq/branches/main/protection
```

If the default branch changes, move the rule to the new default branch and update the workflow push filter. The workflow contains no administrator tokens and does not modify its own protection.

The Native distribution matrix verifies system installation into /usr/local/bin and
Program Files/anyq, updates, and the correct Windows PATH scope. These checks
run only on disposable CI runners, remove their installed binary, and
restore PATH. User mode, elevation refusal, and rollback on
PATH errors are checked separately.
