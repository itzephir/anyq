# Timeout plugin

Timeout is opt-in. Enable it **before** admitting the tasks it should observe:

```sh
anyq plugin enable timeout /absolute/path/to/anyq plugin-runtime timeout
anyq topic create cpu --meta timeout 30m
anyq task create build -t cpu --meta timeout 2h
anyq plugin status timeout
anyq plugin state timeout
```

The shortest timeout from the committed task and separate topic snapshots wins:
this example expires after 30 minutes. A duration is a positive integer followed
by `ms`, `s`, `m`, `h`, or `d`. Zero, negative, fractional, missing-unit, non-string
and overflowing values are invalid. Missing metadata does nothing. Invalid values
leave ownership held and are reported with their source and original value in
`plugin state timeout` as `last_error`; metadata itself is unchanged. Saved state uses the public
metadata wire representation (objects are key/value pairs, numbers are exact text).
The explicit `plugin state NAME` command includes plugin-owned state; treat that output like
the task metadata it contains.

One durable timer uses the running event ID. Its deadline is the **original
committed running timestamp** plus the shortest duration. Delivery delay, retry,
topic edits, disable/re-enable and daemon restart do not grant more time. State
lists active deadlines, their source, original duration and task. The host's
128-timer and 1 MiB state limits apply; an exceeded limit is a visible delivery
failure and never releases resources.

At expiry, the plugin reads the task and requests ordinary idempotent completion.
It never forces registered or queued tasks. An already completed task is resolved
without another completion event. Completion events retire outstanding timers.
Resource release and admission of successors remain one queue transaction.

Managed workers must confirm stopped before timeout can release **any** attached
resource. A pending or uncertain worker keeps every slot; the unchanged overdue
timer remains pending and state explains why. Inspect managed actions with
`anyq plugin inspect WORKER`. After independently confirming cleanup, an operator
can use `plugin resolve WORKER ACTION --confirmed-cleanup`; this never guesses
whether an external process has stopped.

For ordinary caller-managed work there is no managed receipt to cancel. Timeout
releases queue ownership and **does not stop or claim to stop the caller's
process**. Enable this policy only where that resource-release behavior is wanted.
Without the enabled policy, elapsed time never changes ownership.

`anyq plugin disable timeout` pauses actions and retains original deadlines.
`anyq daemon stop` prevents processing while stopped. Start the daemon and enable
the same executable/arguments to resume; an overdue timer is eligible immediately.
First enable observes future events only, so enabling after a task already started
does not add a retrospective timeout. A malformed task can be manually completed.

Run `python examples/timeout.py` after `cargo build -p anyq` for an isolated real
expiry. Native integration tests additionally control the clock and verify overdue
restart, snapshot precedence, manual races and managed cancellation on every
supported operating system.
