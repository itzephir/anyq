# Spec Delta

## REMOVED Requirements

### Requirement: Explicit command and payload
**Reason**: anyq admits named tasks; the caller executes external commands.
**Migration**: After successful task wait, run your command directly, then task complete. Passing argv/commands in an anyq payload is no longer required.

### Requirement: Execution and ownership renewal
**Reason**: Persistent slot ownership until complete replaces process management and heartbeat leases.
**Migration**: Complete explicitly after external work. After a crash, inspect that work and complete manually.

### Requirement: Worker follow-up stage
**Reason**: anyq no longer decides command outcomes or automatically publishes follow-up stages.
**Migration**: The caller registers and coordinates follow-up tasks itself.
