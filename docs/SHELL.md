# Shell plugin

Enable the packaged mode explicitly, before admitting tasks with shell metadata:

```sh
anyq plugin enable shell /absolute/path/to/anyq plugin-runtime shell
anyq plugin inspect shell
```

Only the **task's committed running snapshot** supplies commands. Topic metadata
never supplies or merges commands. Missing `shell` does nothing; an invalid plan
records a visible validation error in `plugin state shell` and retains ownership.

```json
{
  "shell": {
    "sh": ["echo first", "echo second"],
    "autocomplete": false
  }
}
```

Use exactly one nonempty array of nonempty strings, `sh` or `src`. `autocomplete`
is boolean and defaults to false. Unknown options, null bytes and invalid types
are rejected before launch. Plans contain at most 128 entries and must leave room
within the 1 MiB execution receipt for bounded output.

`sh` runs sequential command text through `/bin/sh -c` on Unix and the native
`SystemRoot/System32/cmd.exe /D /S /C` on Windows. Command text follows that
interpreter's syntax. `interpreter` can explicitly select a program string or a
nonempty `[program, argument, ...]` array; it replaces the complete default.

`src` executes exact program paths, resolving relative paths against the captured
working directory. It does not expand `~`, wildcards, variables or file extensions.
Unix scripts need executable permissions and a shebang, or an explicit interpreter
such as `["/bin/sh"]`. Windows `.cmd`/`.bat` files require an explicit cmd boundary;
use native `.exe` programs directly. An unavailable interpreter or non-executable
source produces a recorded launch failure, with no fallback execution mode.

The default working directory is the creating CLI's absolute directory, captured
before admission even if shell metadata is added later. `cwd` overrides it and
must be absolute. Historical tasks without a captured origin need an explicit
`cwd` while still registered/queued. The daemon's own directory does not decide
execution context. Native `src` paths must resolve to Unicode executable paths;
unrepresentable paths fail before launch.

Children inherit the daemon account's environment. Optional `env` provides string
overrides; empty values are supported, invalid keys are rejected. The full inherited
environment is never saved. Inspection records override **keys**, excluding values;
ordinary plugin status excludes state, scripts and output. Explicit metadata,
`plugin state`, and `plugin inspect` can contain user-provided sensitive content.

Entries run one at a time. The first nonzero exit, launch failure or cancellation
prevents later entries. Both output pipes drain concurrently, even after the saved
output budget fills. Inspection retains bounded stdout/stderr, truncation flags,
entry index, exit code, resolved command/context and planned/running/exited/
cancelled/uncertain status after daemon restart. Each stream captures at most
64 KiB per entry and the action retains a bounded aggregate history.

With `autocomplete: false`, both success and failure leave the task running.
With true, final confirmed exit, including nonzero exit or launch failure with no
process, requests ordinary idempotent completion. Completed means resource ownership
was released; the separate execution outcome still records failure. A completion
race or lost response cannot create another logical completion or rerun work.

## Native containment and recovery

Before spawn, the plugin durably registers managed work and records the prepared
plan. Native Unix process groups are established before child execution. The adapter
keeps the leader waitable with `waitid(WNOWAIT)`, scans live native group members
and waits for both pipes to reach EOF before reporting stopped. A leader's exit
alone never proves a descendant stopped; its PID is not reaped while group
identity is needed. Linux and macOS use the safe rustix/sysinfo adapters.

On Windows, the safe [windows-spawn API](https://docs.rs/windows-spawn/0.1.0/windows_spawn/)
creates a suspended process and its kill-on-close Job Object. A separate query Job
is assigned before resume using [win32job](https://docs.rs/win32job/2.0.3/win32job/).
An empty job member list, root exit and drained pipes are required for stopped.
Job assignment/query failure is an error, never evidence of cleanup.

Manual completion, timeout, plugin disable and daemon shutdown request the same
cancellation barrier. Unix requests termination, then forces the retained group
after a bounded grace period; Windows closes the private kill-on-close Job.
Resources remain held until the whole managed group/job is confirmed stopped.
Cooperating descendants stay within this native containment; deliberate Unix
session/group escape is unsupported and must not be used for resource-owning work.

A plugin/owner crash around spawn leaves a durable uncertain action. Recovery
never launches that action again and never kills a saved PID that could have been
reused. Inspect it, independently clean up the external work, then explicitly use:

```sh
anyq plugin resolve shell ACTION_ID --confirmed-cleanup
anyq task complete TASK
```

Disable keeps execution history. A failed or uncertain cleanup holds every slot.
Core caller-managed workflows still run no commands unless shell was explicitly
enabled with valid task metadata. Run `python examples/shell.py` after building the
ordinary binary for a platform-native example with durable outcomes.
