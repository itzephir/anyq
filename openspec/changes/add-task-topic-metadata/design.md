# Design

## Context

See [proposal.md](proposal.md). This stage builds on `introduce-queue-daemon`.
Task and Topic are queue domain aggregates, TaskRecords/TopicRecords belong to
dispatch, and SQLite restores canonical identities. Config is a separate feature
with strict known typed keys; that policy must not be reused for arbitrary metadata.

## Goals / Non-Goals

**Goals:** deterministic value semantics and atomic metadata changes that survive
transport/persistence round trips and are available before automatic admission.

**Non-Goals:** plugin validation, inherited topic defaults, metadata-triggered
execution, generic config keys, YAML objects/tags, TOML dates or hot updates to
an already running task's execution inputs.

## Decisions

### Format-independent domain value

Add Metadata/MetadataValue types in queue API, one type per file. Use ordered
string-key objects, arrays, strings, booleans, null and lossless JSON-compatible
numeric representations with finite/bounded validation. Equality must survive
JSON/YAML/TOML imports and persistence. Structural limits (initially 1 MiB and
64 levels) are shared policy, not duplicated parser heuristics. Empty object is
the default. Unknown namespaces are stored without plugin lookup.

Serialization/parsing belongs to outer adapters and translates to this value;
using serde_json::Value inside the domain would couple its contracts to one
format. All parsers must detect duplicate object keys and reject unsupported
TOML/YAML values rather than stringifying them silently.

### Focused typed use cases

Provide separate task/topic metadata read, set-key, unset-key and replace
contracts with their own requests/interactors. Shared domain helpers implement
dotted paths, missing-path behavior, type conflicts and task-state edit rules.
Missing intermediate objects are created; empty path segments and crossing a
scalar/array are errors. Removing a missing key is an idempotent no-op; reading
one is an error. A file replacement replaces the whole object. Valid JSON scalar
CLI values retain type; other input becomes a literal string, so `2h` stays text.
Literal dots in object keys remain supported through whole-object import/export;
the dotted-path editing syntax does not add a second escaping language.

Use the existing UnitOfWork to reread and patch current values under the same
transaction, preventing lost updates. Task edits call domain validation before
save. Topic edits never mutate a task or recompute a captured execution snapshot.

### Client-side imports and atomic creation

CLI reads the chosen local file, selects its parser by .json/.yaml/.yml/.toml,
validates the complete object and sends typed data rather than a daemon-local
filename. Creation loads one file, then repeated --meta pairs in order, before
RegisterTask/CreateTopic are called. The registration input gains initial
metadata so the existing all-links-before-admission transaction also covers it.
Capture the caller directory as context in the protocol; shell behavior itself
remains for its owning stage.

CLI normalization gains meta/metafile in both orderings. The domain naming
inventory adds those reserved bare names while canonical restore stays unchanged;
existing command-like names remain addressable using backticks. Invalid flags,
files and values fail before database/daemon startup whenever entity state is
not required. New messages/help use the presentation localization boundary.

### Compatible storage upgrade

Add checked metadata JSON columns to tasks/topics and advance schema in one
IMMEDIATE transaction. Use a single explicit serializer at the adapter boundary.
Retain schema 2 -> 3 -> metadata upgrade in the same initialization transaction,
with no admission on open. Update protocol DTOs and typed client adapters together;
incompatible protocol peers are rejected before operations.

## Risks / Trade-offs

- [Large or recursive input] -> limits before allocation where possible and
  again before domain persistence; bounded YAML aliases and no custom tags.
- [JSON number loss] -> lossless conversion and fixtures at numeric boundaries.
- [Autostart races separate meta command] -> creation-time data is the supported
  atomic path; running edits are rejected rather than racing plugin launch.
- [Reserved names now collide] -> quoted identity remains valid and migration
  never reparses stored names.

## Migration Plan

Apply only after the daemon stage. Upgrade existing databases atomically with
empty metadata. Verify interrupted migration and metadata/autocreate rollback.
An older binary cannot open the new schema; rollback uses a backup made before
upgrade or the new binary to export data, never destructive downgrade. Keep the
metadata capability as a delta until implementation checks pass.
