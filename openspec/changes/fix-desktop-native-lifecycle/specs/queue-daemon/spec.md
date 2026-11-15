## ADDED Requirements

### Requirement: Readiness-driven response transfer
Daemon and client stream waits SHALL resume when their native connection becomes ready, retain bounded cancellation and transfer deadlines, and preserve complete large query results under background scheduling.

#### Scenario: Large output with macOS background policy
- **WHEN** real CLI and daemon processes run under macOS background policy and a client consumes a large task list
- **THEN** the client returns the entire ordered list within the existing integration deadline and running task ownership remains unchanged

#### Scenario: Closed output with macOS background policy
- **WHEN** a real CLI receives a large daemon response under macOS background policy after its stdout consumer closes the pipe
- **THEN** it reports the output error within the existing integration deadline without releasing existing task ownership
