## ADDED Requirements

### Requirement: Separate admission from observation
Admission decisions SHALL belong to the domain and be invoked by queue mutations inside their transactions. Blocking wait SHALL depend only on the narrow running-check contract. Storage SHALL execute exact use-case filters, including queued tasks unrestricted by a single topic, without deciding admission.

#### Scenario: Observe without mutation
- **WHEN** a running check is performed for an existing queued or running task
- **THEN** it returns the result without writing a state transition or changing ownership
