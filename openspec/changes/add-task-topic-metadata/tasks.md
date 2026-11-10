# Tasks

## 1. Metadata domain and use cases

- [ ] 1.1 Add format-independent Metadata/MetadataValue and dotted-path policy in queue API; verify pure tests for value round trips, finite numbers, limits, scalar conflicts and missing-key behavior.
- [ ] 1.2 Add separate typed task/topic read, set, unset and replace contracts/interactors; verify unknown namespaces, no generic inheritance, edit-state rules, atomic patches and autocreate rollback without IO.
- [ ] 1.3 Extend creation inputs/aggregates with initial metadata before admission; verify an immediately running task already has the complete object and invalid creation leaves no entity or links.
- [ ] 1.4 Document value/path/file semantics and task/topic snapshot boundaries; verify examples match pure policy fixtures.

## 2. Persistence and transport

- [ ] 2.1 Add atomic SQLite metadata migration and adapter conversion; verify empty metadata for existing entities, schema interruption, rollback and unchanged order/ownership with real databases.
- [ ] 2.2 Extend daemon DTOs and focused remote adapters; verify nested values and canonical command-like names survive client/server/SQLite round trips.
- [ ] 2.3 Test independent concurrent key patches through real clients; verify both committed keys survive and no metadata operation allocates resources.
- [ ] 2.4 Update architecture docs/checks for parser ownership and schema compatibility; verify inner crates import no format or IO libraries.

## 3. CLI imports and atomic creation

- [ ] 3.1 Add meta/metafile in both command orderings, read/unset/typed scalar handling and shared reserved names; verify aliases, quoted old names and en/ru error/help output.
- [ ] 3.2 Add bounded duplicate-aware JSON/YAML/TOML file parsing in outer adapters; verify root/type/tag/date/alias/depth/size errors preserve existing data and file paths resolve at the client.
- [ ] 3.3 Add creation --metafile and repeated --meta pairs with documented precedence; verify CLI registration, links and metadata roll back together at native interruption gates.
- [ ] 3.4 Add runnable task/topic examples and document whole-object replacement versus key patches; verify examples on temporary databases without installed plugins.

## 4. Integration acceptance

- [ ] 4.1 Run the complete mandatory local checks including the native interruption suite; verify metadata/migration coverage and record actual results.
- [ ] 4.2 Open the metadata-only implementation PR against its predecessor and verify current CI passed on all native matrices and required categories.
- [ ] 4.3 Sync only verified metadata behavior into baseline specs and verify strict OpenSpec validation with every later plugin task still unchecked.

## Workflow follow-up

- This is stage 2 and depends on introduce-queue-daemon.
- Do not merge without the user's instruction. Archive after merge and verified baseline sync.
