# Ctrl+C and persistence boundaries

The first Ctrl+C requests cancellation; CLI reaches the current safe boundary and returns 130. A committed stage is not undone. An open transaction rolls back completely. External work is outside anyq's ownership; a CLI signal does not prove that work has stopped.

| Interruption point | Persisted result |
| --- | --- |
| Before argument parsing | Neither database nor config is created |
| Database open / before schema commit | Database is absent or can be reopened and initialized again |
| After schema commit | Empty usable task/topic database |
| Registration / before commit, including partial links | No new task or partial links |
| After registration commit | registered without topics, or queued/running with all initial topics and the stored start mode |
| Link / before commit | Complete previous membership |
| After link commit | Complete new membership and every automatic admission caused by this operation |
| Waiting for a busy resource | Task is queued; free resources are not held |
| Admission / before running commit | Task is queued without ownership |
| After running commit | One slot from every topic until explicit complete |
| Complete / before commit | Previous state and all previous ownership |
| After completed commit | Previous ownership released, eligible successors already running; history and links retained |
| Force for registered/queued | Previous state before commit; completed afterward without running the removed task; eligible successors admitted |
| Topic creation / capacity change | Previous state before commit; complete new topic/capacity and all automatic admissions afterward |
| Autocreate | New entities are part of the main use-case commit; full rollback before it |
| Lookup | Existing states and ownership retained; autocreate side effects follow commit |
| Config read | File unchanged |
| Config set/unset before replacement | Complete previous file; temporary file removed |
| Config after replacement | Complete new file |
| Waiting for config/SQLite lock | Other operation unchanged; new operation leaves no changes |
| Full stdout/stderr pipe without a reading consumer | First signal exits CLI with 130; persisted ownership and config unchanged, output may be incomplete |

Create -twc persists registration with all links and automatic admission; wait only observes state; complete then finishes the task and admits eligible successors in one transaction. With autostart on, create -tswc persists stopped/queued and waits for a separate start. Batch link saves the entire batch before admission; partial links and partial advances do not survive an interruption before commit.

Task start without -w either persists running or returns an error without changes. Start -w retries without holding free slots; interruption leaves the previous stopped/queued state before admission commit and running afterward. Start -wc performs a separate complete after admission. Read-only wait writes no transitions, even when interrupted between its transaction boundaries.

A topic autostart change is committed atomically with every resulting admission; stopped tasks and existing running ownership are preserved. Schema upgrade 2 → 3 rolls back the new columns and version before commit and persists them completely afterward. Existing tasks and ownership are not restarted on open.

After exit code 130, inspect task NAME: interruption after admission commit preserves running even if the started message was never printed.

A second Ctrl+C forcibly terminates the process. SQLite rolls back unfinished transactions during recovery, and the OS file lock is released. Forced termination before config replacement may leave a temporary file; cleanup for the first cooperative signal does not apply. config.toml.lock intentionally remains but does not itself hold a lock.

## Integration checks

apps/cli/tests/interruptions.rs launches the real CLI, holds it at a selected checkpoint, and sends a real signal: SIGINT on Unix, CTRL_C_EVENT in a separate console on Windows. It then checks exit 130, durable states, slots, file integrity, lock release, and a successful subsequent operation. SQLite/file lock waits are tested with an independent process actually holding the lock.

The suite covers boundaries and intermediate writes, including all three create -twc commits, command variants, autocreate, and config set/unset/read. It checks observable boundaries rather than every machine instruction.

Separate tests produce output larger than pipe capacity, wait for the first byte, and send one real signal. Remaining data is read only after CLI exits, so the test cannot accidentally drain the pipe and hide a hang. Checks cover stdout and stderr, running/config persistence, and successful subsequent operations. Successful large-output reads and consumer-closed pipes are tested in the workflow suite.

```bash
cargo test --locked -p anyq --test interruptions --features integration-test-hooks
```

Checkpoints are built only with integration-test-hooks. ANYQ_TEST_CHECKPOINT variables have no effect in a normal build. CI on all three operating systems explicitly enables the required native suite; omitting the feature does not replace running it.
