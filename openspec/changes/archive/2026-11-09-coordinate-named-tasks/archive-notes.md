# Completing the transition and cleaning the active contract

The named-task transition was accepted in PR #3. PR #5 removed the version 0.2/message-database warning from README. On completion, delta specs were synced into baseline specs, the `command-worker` capability and its Purpose were removed, and historical explanations were moved from active files into this archive.

## Preserved history

- `superseded-specs/` contains baseline specs before synchronization: message queue, consumer groups, leases, command execution, and process-tree management.
- `examples/demo.sh` contains the previous shell demo with `group`, `publish`, `worker`, and `jobs`. It is historical; the current root `examples/demo.py` uses named tasks.
- Proposal, design, tasks, and delta specs preserve accepted decisions and the transition plan, including data format and pilot.

## Removed transition explanations

README described replacing `task add` with `task create`, replacing `task.autoadd`/`topic.autoadd` with `task.autocreate`/`topic.autocreate`, and selecting a separate schema 2 database file. AGENTS.md and current rules no longer compare against old messages, consumer groups, worker, or leases. `--db` help no longer mentions a schema version.

SQLite previously handled `user_version = 1` separately with this message:

```text
schema version 1 contains legacy messages; choose a separate database with --db; existing data was not migrated
```

The special branch was replaced by a generic incompatible-version error recommending another file through `--db`. Validation runs before changing data or journal mode. Unsupported versions and nonempty unversioned files remain rejected without changing their content.

## Checks after cleanup

The `the_legacy_cli_is_removed` check and separate negative old-syntax examples were removed. Unknown config-key checks use neutral `task.unknown` and `topic.unknown`. CLI/SQLite incompatible-database tests use an `unrelated` table, verifying the error code, unchanged bytes, and no journal files. Current command, atomicity, and interruption coverage remains.

The historical durability/old-database-transition requirement is now phrased as task durability and arbitrary-database compatibility checking in the active spec. Current specs describe only the active contract; archived deltas retain the reasons for removing previous requirements.
