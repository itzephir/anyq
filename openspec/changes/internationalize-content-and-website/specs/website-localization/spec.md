# Spec Delta

## Purpose

Make the anyq landing page accessible to English and Russian readers with consistent language selection and layout state.

## ADDED Requirements

### Requirement: Complete bilingual website
The website SHALL provide English and Russian content, metadata, accessible labels, theme labels, and clipboard feedback. Static HTML SHALL contain usable English content and installation commands without JavaScript.

#### Scenario: Russian interface
- **WHEN** Russian is selected
- **THEN** headings, explanations, controls, metadata, and feedback use Russian while commands and resource identifiers remain unchanged

#### Scenario: JavaScript unavailable
- **WHEN** JavaScript is disabled
- **THEN** English content, documentation links, and both installation disclosures remain usable

### Requirement: Deterministic language selection
The website SHALL select a valid `lang=en|ru` URL override, otherwise a valid saved choice, otherwise the first supported browser language, otherwise English. Regional browser tags SHALL resolve to their primary supported language. Unsupported values and unavailable storage SHALL fall through safely.

#### Scenario: Explicit override
- **WHEN** the URL requests English while saved and browser preferences request Russian
- **THEN** the page uses English

#### Scenario: Regional browser preference
- **WHEN** no valid override or saved choice exists and the first supported browser preference is ru-RU
- **THEN** the page uses Russian

#### Scenario: Unsupported languages and blocked storage
- **WHEN** storage is unavailable and neither the URL nor browser preferences identify a supported language
- **THEN** the page uses English and controls continue to work

### Requirement: Persistent language switch
The website SHALL offer a keyboard-accessible EN/RU switch on mobile and desktop. Switching SHALL update content, saved preference when available, and the current URL without navigation, preserving other parameters, anchors, theme, and installer state.

#### Scenario: Switch and return
- **WHEN** a visitor switches from English to Russian and later returns without an explicit language override
- **THEN** Russian is selected from the saved preference

### Requirement: Language state for layout and accessibility
The rendered language SHALL be exposed in the HTML `lang` attribute and through `anyq:languagechange` with `detail.language`. Initial rendering and every language change SHALL keep content, metadata, accessibility labels, and exposed language consistent.

#### Scenario: Layout observes a switch
- **WHEN** the visitor changes language
- **THEN** HTML language selectors and language-change listeners observe the language of the displayed content
