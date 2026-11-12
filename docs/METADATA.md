# Task and topic metadata

Each task and topic stores an independent JSON-compatible object, initially `{}`.
Unknown keys and plugin namespaces are preserved. Metadata does not execute
commands, interpret plugin settings, acquire slots, link topics or advance queues.
Configuration retains its separate inventory of typed known keys.

## Read and edit

Both command orderings work:

```sh
anyq task create build
anyq task build meta timeout 2h
anyq task meta build shell.autocomplete true
anyq task build meta
anyq task build meta timeout
anyq task build meta --unset shell.autocomplete

anyq topic create cpu
anyq topic cpu meta timeout 1h
anyq topic meta cpu timeout
```

Reads print compact JSON. Successful writes have no output. A valid JSON value
retains its type, including arrays and objects; other CLI text is a literal
string. Thus `2h` is a string, `true` is a boolean, `-1` is a number, and the
argument `"\"true\""` is a string containing `true`. Strings, finite decimal
numbers, booleans, null, arrays and string-key objects are supported.

A dotted key traverses objects. Missing parents are created by a set; crossing
a scalar or array is an error. Empty path segments are invalid. A missing read
fails; removing a missing path succeeds without changes. Empty parent objects
remain after removal. Literal dots in keys are supported by complete object
imports and exports; dotted editing has no additional escaping syntax.

Task metadata can change while registered or queued, including a stopped task.
Running and completed task edits fail, even for an unset of an absent key.
Topic metadata remains editable and never rewrites task metadata. Generic
inheritance and merging across task/topic objects do not occur. Later lifecycle
events capture their execution inputs separately; topic edits apply to future
snapshots.

## Creation and file replacement

```sh
anyq task build-with-inputs create -t cpu --stop \
  --metafile meta.yaml --meta timeout 2h --meta shell.autocomplete false
anyq task build-with-inputs meta
anyq task build-with-inputs metafile replacement.toml
anyq task start build-with-inputs
anyq task complete build-with-inputs
```

Creation accepts one `--metafile FILE` and repeated `--meta KEY VALUE` pairs.
The file is loaded first regardless of flag position; patches then apply in
argument order. Registration, metadata, automatic topic creation and all initial
links commit together before automatic admission. With available topics an
ordinary creation can immediately become running, so supply initial metadata
on creation or use `--stop` when preparing it in several commands.

`metafile FILE` replaces the entire object; a key patch preserves unrelated keys.
Recreating an existing topic preserves its metadata; use `meta` or `metafile`
to change that object.
File reads occur in the client, relative to its current directory. The daemon
receives typed values rather than a filename. Extensions `.json`, `.yaml`, `.yml`
and `.toml` select the parser. Syntax, duplicates and structural errors fail
before daemon startup when queue state is not needed. Files never execute.

Duplicate keys are rejected at every nesting level. YAML keys must resolve to
strings; tags, recursive aliases and multiple documents are rejected. Ordinary
aliases expand to independent values under the same structural limits. TOML
dates/times and non-finite numbers are rejected. Numeric conversion preserves
decimal digits rather than rounding through a floating-point representation;
equivalent decimal spellings normalize to one value. JSON numeric syntax,
finite-range validation and a bounded decimal exponent apply to numbers.

## Limits and persistence

Metadata files must be UTF-8 and at most 1 MiB. The decoded object has a separate
1 MiB budget: eight bytes per value, plus UTF-8 key/string bytes and normalized
decimal-number bytes. The root counts as level one; maximum depth is 64. Parsers
share the domain's incremental budget, including expanded YAML aliases. YAML
also permits at most 1,024 anchors and 1,024 alias references, with a 1 MiB
anchor-storage budget. These bounds are checked before persistence and again
when restoring transported values.

SQLite stores checked JSON-object columns in schema 4. Schema 2 and 3 databases
upgrade in one transaction, preserving canonical names, order, links, states and
every running slot, with empty metadata for old entities. Opening a database
does not admit tasks. Incompatible schemas are rejected before changing journal
mode or data. An older binary cannot open schema 4; rollback uses a pre-upgrade
backup rather than a destructive downgrade.

Stop each database's daemon before replacing the binary. Protocol version 2
adds metadata and lossless native caller-directory context; incompatible peers
are rejected. Metadata key patches reread and update the current object within
one transaction, so independent concurrent keys survive without a lost update.

Run `python examples/metadata.py` after building the CLI to exercise these forms
on an isolated temporary database. See [INTERRUPTIONS.md](INTERRUPTIONS.md) for
the pre-commit rollback and post-commit durability boundaries.
