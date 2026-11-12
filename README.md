# anyq

A local resource admission queue for named tasks: builds, agents, people, and emulators. A bundled Rust daemon owns the selected anyq queue database; CLI commands communicate with that daemon. Queue commands start the owner automatically, and it keeps running after clients exit.

A task requires several topics at once. When autostart is enabled, the queue automatically acquires one slot from every topic atomically. Wait only waits for running and succeeds immediately if the task is already running. The caller performs the work and releases ownership with complete.

## Installation

Prebuilt binaries for macOS, Linux, and Windows are installed with short
[Shell/PowerShell scripts](install/README.md), without Rust and with SHA-256 verification.
Until the first GitHub Release, use the source installation below.
The landing page and its delivery workflow live in [website/](website/README.md).
The website supports English and Russian, detects browser language, and remembers
your EN/RU choice. Use `?lang=en` or `?lang=ru` for an explicit language override.
Repository documentation, specifications, and comments are maintained in English;
the CLI is English-only, while website interfaces support both languages.

Rust 1.85+ and C build tools for bundled SQLite are required. Linux, macOS, and Windows MSVC with Visual Studio Build Tools C++ are supported.

```bash
cargo install --path apps/cli --locked
anyq --help
```

Database precedence: --db PATH > ANYQ_DB > db.path in config > platform default. Default: $XDG_DATA_HOME/anyq/queue.db or ~/.local/share/anyq/queue.db on Unix; %LOCALAPPDATA%\anyq\queue.db on Windows.

## Commands

| Command | Action |
| --- | --- |
| `anyq daemon start` / `run` | Start the background owner or serve in the foreground |
| `anyq daemon status` / `stop` | Inspect without starting, or stop without releasing task slots |
| `anyq task NAME meta [KEY [VALUE]]` / `topic TOPIC meta [KEY [VALUE]]` | Read JSON metadata or patch one key; `--unset KEY` removes it |
| `anyq task NAME metafile FILE` / `topic TOPIC metafile FILE` | Replace metadata from local JSON, YAML or TOML |
| `anyq topic list` / `topic --list` / `topic -l` | List topics |
| `anyq topic create TOPIC [-c N]` | Create a topic; -c = --capacity, default 1 |
| `anyq topic TOPIC` | Current running/queued tasks in registration order and capacity |
| `anyq topic capacity TOPIC [N]` | Read capacity or change it to N |
| `anyq topic TOPIC autostart [on/off]` | Override topic autostart; on when omitted |
| `anyq topic link TOPIC NAME... [-w] [-c]` | Atomically link tasks, then optionally wait/complete in registration order |
| `anyq task list` / `task --list` / `task -l` | All names alphabetically, including completed tasks |
| `anyq task NAME` | State and stored topics |
| `anyq task create NAME [-t TOPIC] [-w] [-c]` | Register with initial links and follow-up steps |
| `anyq task link NAME TOPIC... [-w] [-c]` | Atomically link a task to all topics |
| `anyq task start NAME [-w] [-c]` | Start now; -w waits for admission, -c completes afterward |
| `anyq task wait NAME [-c]` | Observe running; -c completes afterward |
| `anyq task complete NAME [-f]` | Complete; -f = --force |
| `anyq config KEY [VALUE]` | Read or write a setting |
| `anyq config --get KEY` / `--unset KEY` | Read or reset a setting |
| `anyq config list` / `--list` / `-l` | Effective settings |
| `anyq help [command]` | Help, for example help task link |

-v = --version, -h = --help, -w = --wait, -c = --complete for tasks. --db PATH is available at every level. The CLI is English-only.

The daemon uses a private Unix socket or Windows named pipe under the current account,
with one OS lifetime lock per canonical database path. Config/help/version remain
independent of the owner. See [daemon lifecycle and protocol](docs/DAEMON.md).

Tasks/topics have independent metadata objects. Creation accepts `--metafile FILE`
and repeated `--meta KEY VALUE` patches before admission in the same transaction.
Task metadata freezes at running; use creation flags or `--stop` while preparing
inputs. See [metadata formats, editing and limits](docs/METADATA.md).

All actions support a name-first form: topic link cpu job = topic cpu link job; task link job cpu = task job link cpu; topic cpu capacity [N]; topic cpu create [-c N]; task job create/wait/complete. Normalization preserves names and global options.

Create accepts repeated --topic/-t. The task and all initial links are saved in one transaction. Wait requires at least one topic; complete requires wait. Complete on link also requires wait. Invalid flag combinations are rejected before opening the database. All batch links are saved before checking admission; any participant's error rolls back the entire batch, including autocreate. For topic link -wc, tasks are awaited and completed in registration order, regardless of argument order.

```bash
anyq task create build-001 -t itzdesign-commands -t itzdesign-repo-lock
# running when autostart and resources allow; otherwise queued without ownership
anyq task complete build-001 -f

anyq task create build-002 -tw itzdesign-commands -t itzdesign-repo-lock
# running after admission, until explicit complete
npm run build
anyq task complete build-002

anyq task create probe-001 -twc pixel-10-emulator
# queued → running → completed
```

-twc TOPIC = --topic TOPIC --wait --complete. Each step in the chain has a separate commit. An error stops subsequent steps. Complete does not run an external command between admission and completion.

## Names

Tasks and topics share domain naming rules. A bare name starts with Latin a-z/A-Z; remaining characters are unrestricted, including spaces, Unicode, and backslashes. The bare words list/create/capacity/link/add/start/autostart/wait/complete/config/help are reserved. A leading hyphen requires a wrapper.

One literal outer pair of backticks allows any content: an empty string, Unicode, command words, leading hyphens, and nested backticks. It is removed exactly once: build and `build` identify the same entity. Inner escapes are not interpreted. Output provides a reversible spelling of the canonical name.

The shell must pass backticks as literal text. Examples for bash/zsh:

```bash
anyq topic create '`list`'
anyq task create '`--task`' -twc '`list`'
anyq task create build\ with\ spaces -t '`list`'
```

PowerShell also accepts the literal wrapper inside single quotes. Shell quoting and literal backticks are separate layers. The -- separator alone does not bypass domain restrictions.

## Configuration

File: ~/.anyq/config.toml. Home is $HOME on Unix and %USERPROFILE% on Windows. Without a file, both autocreate settings are off and task.autostart=true; reading defaults does not create the file.

```bash
anyq config task.autostart false
anyq config task.autocreate true
anyq config topic.autocreate true
anyq config db.path '~/queues/tasks.db'
anyq config list
anyq config --get task.autocreate
anyq config --unset db.path
```

task.autocreate and topic.autocreate accept true/false, default false. task.autostart accepts true/false, default true; its value is saved as the default when creating a new task (including autocreate). Changing config does not open the queue or alter existing tasks. db.path is a nonempty string; ~/ expands relative to home, and a relative path resolves against the working directory. Unset restores the default. Unknown keys and invalid values are errors. list is a subcommand and cannot be a key.

Autocreate creates a missing named object inside the main operation's transaction; a new topic has capacity 1. Link can create both entities and their link together. A successful lookup may also create a missing entity. Errors roll back automatic creations: ordinary task complete missing reports was not run and leaves no missing task; --force may successfully create and complete it. Existing states and naming rules are preserved.

Config writes are atomic and protected by an OS file lock. Concurrent changes reread current values. Config/help/version do not open the database; help/version work with damaged config. Reading an absent db.path returns 1 without a message; other errors return 2.

Empty task/topic/db sections are valid and use defaults. A table instead of a terminal bool/string, an unknown section, or list = {} is an error; reads and set/unset preserve the original file bytes.

## Registration and admission

```bash
export ANYQ_DB="$HOME/.local/share/anyq/tasks-v2.db"
anyq topic create itzdesign-commands
anyq topic create itzdesign-repo-lock
anyq topic create pixel-10-emulator
anyq topic list

anyq task create build-revision-001
anyq task link build-revision-001 itzdesign-commands itzdesign-repo-lock
anyq task wait build-revision-001
npm run build
anyq task complete build-revision-001
anyq task build-revision-001
```

In PowerShell, set the path through $env:ANYQ_DB. Wait exit code 0 means running is already persisted. Waiting appears when resources are busy. The caller decides how to handle the external command's result and when to release ownership.

## Start control

Without an explicit override, a topic inherits task.autostart saved when the task was created. `topic TOPIC autostart on/off` sets its own value, taking precedence over config. For automatic admission of a task with multiple topics, every topic must allow autostart. Changing a topic setting does not stop running tasks.

On create, `--stop` prevents automatic admission of this task until explicit start, even after resources are released or topic autostart is enabled. `--start` does nothing extra when autostart is allowed; when disabled, it requires immediate full admission or rolls back the entire registration. Short `-s` inverts effective autostart: it acts as --stop when on and --start when off. These three flags are mutually exclusive.

```bash
anyq task create staged -t cpu --stop
anyq task staged start              # error if admission is unavailable now
anyq task staged start -w           # wait and start
anyq task staged start -wc          # wait, start, and complete
anyq topic cpu autostart off
anyq task create manual -t cpu --start
```

`task start` respects global order and all resource constraints. Without -w, busy resources or queue order cause an error without changes; with -w, full admission is retried. A running task reports `is already running` and succeeds. Start is invalid for registered tasks without topics and completed tasks. On start, -c requires -w; wait continues to observe only, including with --stop.

All short create flags can be combined: `anyq task create NAME -tswc TOPIC`. With autostart on, -s leaves the task queued, -w waits for explicit start from another invocation, then -c completes it. With autostart off, -s requests immediate admission. For several resources, use repeated -t or one batch task link: membership cannot change after admission. A stopped queued task keeps its place and may delay later participants.

## States, ordering, and errors

| State | Meaning |
| --- | --- |
| registered | Name exists, no topics |
| queued | Has topics, not admitted |
| running | One slot from every topic |
| completed | Ownership released, history retained |

Names are unique and cannot be reused after complete. Repeating a queued link is idempotent. Running/completed tasks cannot change membership. Ordinary complete is valid only for running; force also completes registered/queued. Repeated and concurrent waits for running succeed without changing ownership. Wait for completed and repeated completion are errors.

```bash
anyq task create unfinished
anyq task wait unfinished
# error: 'unfinished' is not registered in any topic
anyq task complete unfinished
# error: 'unfinished' was not run
anyq task complete --force unfinished
# 'unfinished' successfully completed
```

Capacity is positive, default 1; reducing it below current usage is rejected. A task has one registration sequence across all topics. Admission is possible when the task is among the first queued tasks within the number of free slots in every topic. No partial acquisition occurs. Queued tasks keep their place without an active wait. Registration, link, complete, and changes to topic capacity or autostart advance all eligible queued tasks in the same transaction. Remove an unwanted task with complete --force.

Lookups do not change existing ownership; completed tasks retain links but leave the current queue. An unknown name is an error when the corresponding autocreate setting is off.

CLI success is 0, input/state errors return 2 with stderr, and the first Ctrl+C returns 130. Committed state survives interruption before the success message.

## Persistent ownership and interruptions

Returning from wait, a caller crash, and elapsed time do not release slots. After a crash, inspect the external work, then explicitly complete the known task.

Stopping or crashing the queue daemon also preserves ownership. The next queue
command restarts it and recovers committed state. A lost mutation response is
reported as an uncertain outcome and is never automatically replayed; inspect
the named task before retrying. Hooks and execution plugins remain planned in
the [plugin roadmap](docs/PLUGIN-ROADMAP.md).

Ctrl+C before commit rolls back the current transaction; after commit it preserves the result. In a -twc chain, registration may already persist running; wait only observes it, then complete persists completed. -tswc may leave a stopped queued task. After exit 130, inspect task NAME: persisted running requires complete. The outcome table and native integration suite are in [INTERRUPTIONS.md](docs/INTERRUPTIONS.md).

The first Ctrl+C also works when stdout/stderr is full and the consumer is not reading. Output may be incomplete after such an interruption; persisted states and ownership do not change.

Database schemas 2 and 3 upgrade to 4 in one transaction: tasks, ordering, links, states and ownership are preserved, with empty metadata for existing entities. Schema 2 also receives autostart defaults without admission on open. Unsupported versions (including 1) are rejected without changing data.

History is not cleaned up automatically. Keep the database on a local disk; SQLite uses WAL, synchronous FULL, and short IMMEDIATE transactions. Use SQLite backup for a live database.

## Optional desktop panel

The [desktop panel](docs/DESKTOP.md) displays existing queue topics, current tasks
and slot usage through the daemon. It has a main window and menu-bar/tray access,
English/Russian and light/dark themes, and panel-local database settings. It is
built/packaged separately from the CLI; native acceptance still in progress is
tracked in [desktop verification](docs/DESKTOP-VERIFICATION.md).

## Checks

```bash
cargo build --locked -p anyq
python examples/demo.py
npm ci --ignore-scripts
npm run spec:check
python -m pip install -r tooling/requirements-dev.txt
python -m ruff check .
python -m ruff format --check .
python tooling/check_architecture.py
cargo run --locked -p anyq-architecture-lint
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked -p anyq --test interruptions --features integration-test-hooks
```

Python is needed for tooling/demos, and Node for pinned OpenSpec tooling. The anyq runtime requires neither.

[Code rules](docs/CODE-RULES.md), [architecture](docs/ARCHITECTURE.md), [CI and main protection](docs/CI.md). [Baseline specs](openspec/specs/), [changes](openspec/changes/), [archive](openspec/changes/archive/).

## Executable plugins

Explicitly enable trusted local observers with `anyq plugin enable NAME EXECUTABLE [ARG...]`.
Committed lifecycle events, durable acknowledgements, namespaced state/timers,
replay-safe completion and managed cancellation are described in [PLUGINS.md](docs/PLUGINS.md).
Run `python3 examples/plugins.py` after building anyq for an isolated public-protocol example.
Timeout and shell remain separate planned stages of the [stacked roadmap](docs/PLUGIN-ROADMAP.md).

### Optional timeout policy

Enable `anyq plugin-runtime timeout` through the ordinary plugin command to apply
metadata deadlines. See [TIMEOUT.md](docs/TIMEOUT.md) for snapshot precedence,
restart recovery and managed versus caller-owned work. The CLI is English-only.

### Optional shell execution

Register `anyq plugin-runtime shell` explicitly to execute task `shell.sh` or
`shell.src` plans after admission. See [SHELL.md](docs/SHELL.md) for native syntax,
captured directories, autocomplete, output inspection and uncertain recovery.
