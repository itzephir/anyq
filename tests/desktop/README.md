# Native desktop integration and screenshot tests

Every UI behavior change requires real native integration and screenshot checks
on Linux, macOS and Windows. The required CI matrix is in
`.github/workflows/desktop.yml`, included in the stable `CI passed` gate.

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
Dynamic source paths and clock labels are masked; assertions separately verify
the real source identity. Other content remains visible and comparable. The
comparison permits at most 50 changed pixels with a channel delta above 8; it
rejects missing goldens, dimension changes and larger visual regressions. Actual,
difference and JSON evidence are uploaded even when CI fails. No comparison job
writes or accepts golden changes.

To run locally after installing `tooling/requirements-desktop.txt`:

```sh
cargo build --locked -p anyq -p anyq-desktop --features anyq-desktop/integration-test-hooks,anyq-desktop/custom-protocol
python tooling/test_desktop.py --cli target/debug/anyq --panel target/debug/anyq-panel
```

Use `.exe` binaries on Windows. Linux needs a graphical X11 session; CI uses
Xvfb, a real window manager and D-Bus. The ordinary-window/no-tray fallback is
exercised when no compatible host exists. Real GNOME/KDE/Wayland host behavior
remains additional native acceptance rather than being inferred from Xvfb.
macOS capture requires the test runner's screen-capture permission.

To update goldens intentionally, use `--update-goldens` on the corresponding
native platform, inspect every changed PNG and commit the result in the PR.
The initial hosted-runner images may be collected from CI artifacts, inspected,
and committed. A missing initial baseline deliberately fails comparison until
that review exists. Never fabricate one platform's images from another platform.
