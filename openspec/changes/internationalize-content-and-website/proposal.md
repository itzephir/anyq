# Proposal

## Why

Russian-only documentation and the landing page limit international use. Repository content needs a consistent English baseline while the user interface remains available in English and Russian.

## What Changes

- Translate README files, AGENTS.md, project guides, and all current, active, and archived OpenSpec artifacts into English without changing requirement meaning or task completion status.
- Require English repository content and English/Russian user interfaces in AGENTS.md and OpenSpec context. Localization resources and language-specific test fixtures may contain Russian.
- Add complete English/Russian landing-page translations, a persistent language switch, browser-language detection, and a URL override.
- Expose the active language through HTML `lang` and a language-change event so layout and accessibility use the same resolved language.
- Retain the existing static delivery format, theme controls, installation commands, and English content without JavaScript.

## Capabilities

### New Capabilities

- `website-localization`: bilingual landing content, language selection, persistence, and layout/accessibility language state.

### Modified Capabilities

- `repository-delivery`: English repository-content policy with explicit localization/test exceptions and bilingual interface requirements.

## Impact

Documentation, OpenSpec context and artifacts, website HTML/JavaScript/CSS, and website CI checks. No queue behavior, database schema, runtime dependency, or hosting receiver changes. The branch includes merged PR #10 before translation; its synchronized admission specs and archive placement are retained.
