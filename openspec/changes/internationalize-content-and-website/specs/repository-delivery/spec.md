# Spec Delta

## ADDED Requirements

### Requirement: Repository and interface language policy
Repository documentation, specifications, instructions, and code comments SHALL be written in English. User-interface work SHALL support English (`en`) and Russian (`ru`). Localization resources and language-specific test fixtures SHALL be allowed to contain Russian. AGENTS.md and OpenSpec context SHALL carry the same policy.

#### Scenario: Contributor writes new content
- **WHEN** a contributor adds documentation, specifications, instructions, comments, or interface content
- **THEN** repository prose is English and interface content supports EN/RU, with Russian confined to localization or language-specific fixtures
