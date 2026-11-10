# Spec Delta

## MODIFIED Requirements

### Requirement: Native coordination interface
Task and topic commands SHALL work natively through the local background queue daemon. The caller SHALL remain responsible for external work unless it explicitly enables plugin-managed execution through task metadata. Optional shell execution SHALL belong to the plugin, not queue domain or admission use cases. Successful operations SHALL return 0 with a clear message; state and input errors SHALL return nonzero.

#### Scenario: External build
- **WHEN** the caller registers a task, links it to topics, obtains wait, performs a build, and calls complete
- **THEN** anyq coordinates slots while the caller launches and manages the build process
