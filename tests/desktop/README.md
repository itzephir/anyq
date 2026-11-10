# Native desktop integration and screenshot tests

Every UI behavior change requires real native integration and screenshot checks
on Linux, macOS and Windows. The required CI matrix is in
`.github/workflows/desktop.yml`, included in the stable `CI passed` gate.

The modes are explicit: `verify` compares with committed goldens and fails on a
regression; `generate` writes new goldens for review. Both modes launch every
visual case twice from independent clean processes and databases, and reject
even one unstable pixel before comparing or generating anything.

`config.json` owns each state's CLI seed commands, expected complete snapshot,
fixed clock/timezone, source, preferences, viewport/decorations, selected topic/task,
scroll, focus, masks and comparison thresholds. Cases do not inherit earlier scenarios.
Only machine-specific source-path text is masked and its actual identity is
asserted separately. The display clock and timezone are injected only into the
feature-gated test build. Timers still use real bounded deadlines; tests wait for
observable states rather than asserting arbitrary elapsed delays.

`tooling/test_desktop.py` launches the real CLI, owner and Tauri webviews using an
isolated home/database. `integration-test-hooks` adds a private file transport
for driving the actual renderer; it is absent from shipping packages and does
not replace the daemon or its queue. Screenshots capture native window pixels,
not a browser preview or a DOM re-render. Queue fingerprints prove that viewing,
searching, refresh, settings and client exit do not change business state.

Scenarios cover current/empty queues, full membership, text escaping, search,
shared window/compact state, en/ru, both themes, narrow layout, missing/incompatible
sources, stale recovery, owner restart, concurrent CLI completion, oversized
results, source generations, preference restart/reset and close/reopen/quit.

PNG goldens are tracked in `goldens/linux`, `goldens/macos` and `goldens/windows`.
Only configured source-path regions are masked; the fixed update time remains
visible and comparable. Other content remains visible and comparable. The
visual configuration disables native window decorations to exclude the macOS
translucent rounded frame and its dependency on desktop pixels. Integration
scenarios retain ordinary decorated windows; the production window is unchanged.
The configured comparison requires exact pixels and rejects even one changed
channel, missing goldens and dimension changes. Actual,
difference and JSON evidence are uploaded even when CI fails. No comparison job
writes or accepts golden changes.

To run locally after installing `tooling/requirements-desktop.txt`:

```sh
cargo build --locked -p anyq -p anyq-desktop --features anyq-desktop/integration-test-hooks,anyq-desktop/custom-protocol
python tooling/test_desktop.py --cli target/debug/anyq --panel target/debug/anyq-panel --mode verify
```

Use `.exe` binaries on Windows. Linux needs a graphical X11 session; CI uses
Xvfb, a real window manager and D-Bus. The ordinary-window/no-tray fallback is
exercised when no compatible host exists. Real GNOME/KDE/Wayland host behavior
remains additional native acceptance rather than being inferred from Xvfb.
macOS capture requires the test runner's screen-capture permission.

To update goldens intentionally, use `--mode generate` on the corresponding
native platform, inspect every changed PNG and commit the result in the PR.
The native UI workflow also exposes `verify`/`generate` through workflow_dispatch.
Before that workflow reaches main, invoke the existing CI workflow on the PR
branch with `desktop_goldens=true`; its separate generator job uploads candidates
while the required comparison job still runs in verify mode. Generation never
commits files or grants a CI pass for missing/unreviewed goldens.
The initial hosted-runner images may be collected from CI artifacts, inspected,
and committed. A missing initial baseline deliberately fails comparison until
that review exists. Never fabricate one platform's images from another platform.
