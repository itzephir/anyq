# Design

## Context

See [proposal.md](proposal.md). This stage uses the preceding event/plugin
contracts, durable timers and managed-execution barrier. Current queue ownership
has no expiry; timeout is a separately enabled client of explicit completion.

## Goals / Non-Goals

**Goals:** deterministic deadline resolution, persisted original timing, replay-safe
completion and cancellation-compatible behavior before shell is implemented.

**Non-Goals:** leases/heartbeats, admission time limits, automatic failure inference,
stopping unmanaged caller work, or an exactly-on-time guarantee during downtime.

## Decisions

### Pure duration and precedence policy

Add a timeout feature with separate API/policy and protocol adapter. Parse one
positive integer plus ms/s/m/h/d using checked arithmetic. Resolve the minimum
of task.timeout and every configured topic.timeout; preserve source identity in
the plugin's inspectable state. Missing configuration is a no-op. Invalid values
fail that delivery visibly without changing ownership. Explicit shortest-duration
policy avoids topic-order dependence; generic metadata merging is not introduced.

### One persisted timer per running identity

On task.running, derive deadline from occurred_at and effective duration. Return
the state/timer acknowledgement through the host's atomic acknowledgement contract.
Use the running event ID as the logical key, so redelivery and restart cannot
extend the deadline or create another timer. On task.completed, retire any timer.
The timer callback rereads task state and submits an idempotent completion action.
Already completed is a successful resolved outcome; registered/queued are errors,
never an excuse to pass force. Missed deadlines become eligible on recovery.

### Shared cancellation path

Timeout requests completion through the runtime barrier introduced in stage 3.
A fixture managed worker demonstrates refusal/pending/stopped behavior now; the
shell plugin later implements that same contract. Cancellation failure leaves
the action unresolved and visible with all slots held. For caller-managed work,
there is no managed receipt to cancel, so an explicit timeout releases ownership
without claiming to stop the caller's process. Document this before activation.

### First-party packaging

Expose `anyq plugin-runtime timeout` as the executable plugin mode, registered
through the ordinary enable command (or an equivalent documented convenience
that expands to that registration). English-only CLI status uses plugin state;
queue admission and SQL conversion gain no timeout branches. Pure policy tests
use a fake clock; recovery tests persist real timers with short durations.

## Risks / Trade-offs

- [Delayed event processing] -> original persisted timestamp, immediately due
  timer when already expired, no restart-relative deadline.
- [Manual completion races expiry] -> transactional action receipt and current
  task validation; no duplicate successor events.
- [Timeout releases unmanaged resource too early] -> opt-in policy and explicit
  distinction between resource ownership and external process termination.
- [Malformed metadata after admission] -> visible validation failure with held
  resources; caller can manually complete rather than silently accept a fallback.

## Migration Plan

No queue schema change is needed beyond the plugin foundation. Ship disabled;
enable explicitly before creating tasks that should receive timeout handling.
Disable pauses existing timers without discarding their original deadlines.
Verify timer/restart/cancellation behavior natively and update the ownership spec
only after those checks pass. Removing the plugin does not reset task states.
