# Queue observation

The optional desktop panel uses the narrow `ObserveQueue` queue contract through
the local owner. This capability requires daemon protocol version 2. A client
connected to an incompatible owner reports an error instead of opening SQLite or
presenting an empty live queue. Existing queue command forms and admission rules
are unchanged.

## Contract

`anyq_daemon_impl::observe(path, cancelled)` connects an observation-only session
and returns only `ObserveQueue`. The selected database must already exist.
Discovery can start the bundled owner, but the owner opens this session's file
without SQLite's create flag. A missing file, including one removed between
discovery and opening, cannot be replaced by an empty database.

The client executable must support the internal `daemon run --db PATH` launch
form, as the CLI does. The desktop composition root supplies that internal mode
before starting its graphical event loop. Ordinary CLI startup and creation
behavior is unchanged.

A successful snapshot contains all topics, including empty ones, and distinct
queued/running tasks with complete memberships, registration sequence, saved
autostart defaults, explicit stop status, and nullable topic overrides. It is
read in one short transaction and fully materialized before network output.
Completed/registered tasks do not appear in the current view. Topic counts derive
from task states and links; global counts count each task once.

Running describes acquired slots, not external process liveness or outcome.
Observation does not run admission, accept autocreate policy, or expose task/topic
mutations. The owner rejects mutation requests on an observation session even
when shared configuration enables autocreate. Cancellation and disconnect leave
existing resource ownership intact.

## Bounds and compatibility

`ObserveQueueRequest::default()` permits up to 10,000 combined topics, current
tasks, and current membership links, and up to 1 MiB of their name text. Smaller
positive limits are supported. Requests above these ceilings are rejected.
SQLite checks the exact requested states and size inside the same read snapshot
before collecting strings. Oversized results fail explicitly; no successful
response silently omits records. The existing 16 MiB protocol frame bound and
bounded/cancellable transport still apply.

The response carries the canonical database identity, and the client verifies it
before restoring domain entities. Canonical names are restored without applying
CLI quote parsing again. Unusual names, including empty strings and literal
backticks, retain their identity. Renderers must display names as text.

## Verification

- Queue API tests cover distinct counts, order, canonical identities, saved start
  preferences, and rejection of non-current/inconsistent snapshots.
- SQLite tests cover empty topics, exact state inclusion, size limits,
  non-creation of missing files, and concurrent completion/admission snapshots.
- CLI workflow tests use real owner processes to exercise observation sessions,
  canonical-name round trips, shared autocreate settings, denied mutations,
  explicit size errors, refresh after completion, and disappearance of a target
  after discovery.
- Existing native cancellation, malformed/slow-client, large-response, and
  owner-recovery suites remain required on Linux, macOS, and Windows.
