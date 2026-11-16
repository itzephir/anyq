# User plugin catalog

Each user installs independent native plugin binaries in `~/.anyq/plugins/`.
Use the plugin name as its filename (`shell`, `timeout`, or another validated
name; add `.exe` on Windows). The invocation path is preserved, including a
symbolic link. A directory entry does not grant execution permission.

```sh
anyq plugin list
anyq plugin enable observer
anyq plugin disable observer
```

These operations manage one user-wide enabled set and require no queue database.
An explicit `--db` does not narrow their scope or start that database's owner.
`plugin list` reports the invocation path, executable availability, enablement
and policy revision. Repeating enable advances only that plugin's revision.

The hidden `.catalog.toml` stores lifecycle policy, not plugin settings. Updates
take an OS file lock and atomically replace the complete document. Missing or
non-executable binaries cannot be enabled. A removed enabled binary remains
listed as unavailable until disabled or restored. Invalid storage is an error;
it is not replaced with an empty catalog.

Plugins own the meaning of arbitrary task and topic metadata passed through the
public protocol. The catalog introduces no plugin-specific parameter schema,
command arguments or metadata-to-command conversion.

Catalog state does not replace queue-local cursors, timers, receipts or managed
work. Runtime status/state/inspect/resolve address a selected queue. Readiness
after enablement is observed through that queue's plugin status; catalog updates
are not a transaction spanning several databases.
