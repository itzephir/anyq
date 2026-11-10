# Spec Delta

## Purpose

Introduce anyq, its admission model, and installation publicly, with a separate editable layout in personal OpenDesign.

## ADDED Requirements

### Requirement: anyq landing page
The landing page SHALL live in the anyq repository, explain atomic acquisition across all topics and explicit complete, provide installation commands for three operating systems and source links. It SHALL use the itzephir visual foundation with two themes, keyboard navigation, and mobile layouts from 320 px.

#### Scenario: Narrow screen
- **WHEN** the page opens at 320 px in either theme
- **THEN** content is readable without page-level horizontal overflow and commands scroll inside their blocks

### Requirement: Layout and catalog
An editable landing page SHALL be uploaded as a separate anyq project to existing OpenDesign with the itzephir design system. The itzephir.com catalog SHALL include anyq, its future anyq.itzephir.com address, and truthful readiness.

#### Scenario: Before launch
- **WHEN** the landing page is not yet published
- **THEN** the catalog shows design-in-progress without a broken launch button

### Requirement: Independent publication
The landing page SHALL build independently of the queue application and publish at anyq.itzephir.com through the existing project gateway. Failed updates SHALL preserve the previous working version. Credentials/private addresses SHALL NOT enter artifacts.

#### Scenario: Update verification
- **WHEN** a new container version fails its HTTP health check
- **THEN** the operator script restores the previous container

### Requirement: Project dev and production
Successful same-repository PRs SHALL publish the verified landing page at dev.anyq.itzephir.com. Successful main pushes SHALL publish at anyq.itzephir.com. The project SHALL select environments by public Host, preserve the previous version on errors, and use separate restricted dev/prod credentials.

#### Scenario: Preview without merge
- **WHEN** the current PR commit passes required CI
- **THEN** dev serves this version over HTTPS while production stays unchanged

#### Scenario: Merge
- **WHEN** a verified change is merged into main and its CI succeeds
- **THEN** production publishes this version through the same project router
