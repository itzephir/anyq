# Tasks

## 1. Landing page

- [x] 1.1 Create a static landing page and design documentation; check both themes on desktop/mobile and without JavaScript.
- [x] 1.2 Upload an editable layout to OpenDesign; check files and the anyq entry in the /dev catalog.

## 2. Installation

- [x] 2.1 Implement Shell/PowerShell installers and documentation; verify successful installation and preservation of the old binary on errors.
- [x] 2.2 Add a native release matrix and complete-set packaging; verify packaging and smoke tests on each OS's CI runner.

## 3. Delivery

- [x] 3.1 Add a container, workflow, and website rollback script; verify build and HTTP health check.
- [x] 3.2 Run required project checks and open a PR; verify GitHub Actions results.
- [x] 3.3 Add project ingress and an atomic receiver; verify Host selection, unsafe archive rejection, and rollback; document operations.
- [x] 3.4 Configure separate dev deployment from PRs and prod from main with restricted keys; verify CI and environment isolation.
- [x] 3.5 Enable the anyq namespace through the itzephir owner, deploy the current PR, and verify HTTPS dev.anyq.itzephir.com with a matching revision.

## 4. System installation

- [x] 4.1 Make /usr/local/bin the macOS/Linux default; check user mode, atomic replacement, and sudo refusal without losing the installed binary.
- [x] 4.2 Make Program Files/anyq the Windows default; check separate elevation, PATH scope, special-character forwarding, and rollback.
- [x] 4.3 Verify the real system directory, updates, and PATH on all five native CI targets with safe test-file cleanup.
- [x] 4.4 Update instructions, landing page, and OpenDesign layout; run required checks and open a separate PR with successful CI/dev deployment.

## Workflow follow-up

- After an authorized merge, verify production and publish a version tag; verify public installation from GitHub Releases.
- Archive the change after publication has been fully verified.
