# Proposal

## Why

anyq should coordinate admission of external work to local resources. The implemented message/command-worker flow and temporary Python launcher replaced this responsibility: the user expects named tasks, several topics, and explicit wait/complete.

## What Changes

- **BREAKING:** replace messages, consumer groups, leases, retries, and worker with named tasks; do not retain the previous API in parallel.
- Add native `anyq task create NAME [--topic TOPIC]`, `anyq topic link TOPIC NAME`, equivalent `anyq task link NAME TOPIC`, `anyq task wait NAME`, `anyq task complete NAME`, `anyq task complete --force NAME`, and `anyq topic --list`. Change capacity through `anyq topic capacity TOPIC N`; creation retains `--capacity N`.
- Support short flags `-t` for repeated `--topic`, `-l` for `--list`, `-c` for `--capacity` on topic creation, and `-f` for `--force`. Registration with initial topics is atomic and does not acquire slots.
- Add `task create NAME -t TOPIC --wait/-w --complete/-c`, including `-twc TOPIC`: wait requires topic; complete requires wait. Without complete, ownership stays running. `-v`/`-h` equal --version/--help.
- Add `task list/--list/-l` for all names, `task NAME` for state/topics, `topic TOPIC` for current queued/running registration order, and `topic capacity TOPIC` to read capacity. Lookups do not change existing ownership; autocreate may create missing names.
- Define shared domain task/topic names: bare starts with a-z/A-Z, differs from list/create/capacity/link/add/wait/complete/config/help, and has an arbitrary tail. A literal backtick wrapper permits any content, including command words, and is stripped once: build and `build` share identity. CLI does not duplicate business naming rules.
- Add command-first/name-first forms, list/help, and dependent link/wait chains with complete.
- Add git-style config list/--list/-l in ~/.anyq/config.toml: task.autocreate/topic.autocreate default false, db.path. List is not a valid key. Autocreate is transactional and rolls back with the main use case. Path precedence: --db, ANYQ_DB, db.path, existing platform default.
- One task may require several topic slots. `wait` atomically obtains admission across all of them and returns; the caller executes external commands.
- Retain ownership until explicit completion. CLI exit, caller crash, and elapsed time do not release slots.
- Reject `wait` without topics and ordinary `complete` before admission. `--force` can finish registered/queued tasks without executing them.
- Persist history/state locally. Do not automatically convert or clear the old message database: the new model uses a new schema version and requires a separate file when an old database is detected.
- Replace examples/checks with register → link topics → await admission → perform external work → complete.

## Capabilities

### New Capabilities

- `local-config`: typed settings, TOML persistence, git-style CLI, database path resolution, and autocreate policy.

### Modified Capabilities

- `task-queue`: named tasks, multiple topics, atomic admission, and persistent ownership replace messages/group deliveries.
- `command-worker`: remove the capability; command execution, heartbeat, and automatic follow-up are no longer anyq responsibilities.
- `platform-support`: retain native coordination CLI on three operating systems; remove external process-tree management from the contract.
- `architecture`: independent checks use task admission instead of obsolete lease scenarios; boundary/DI rules remain.

## Impact

Main changes affect `features/queue/{api,impl}`, CLI, and SQLite schema. Four worker crates and dependencies are removed. Architecture boundaries, one type per file, and separate typed use cases remain; update the allowed Cargo graph, documentation, AGENTS.md, and OpenSpec context. Retain Linux lint and both native CI matrices. After implementation/verification, move the temporary itzdesign pilot to native task CLI on a separate database; do not overwrite its old binary/data.
