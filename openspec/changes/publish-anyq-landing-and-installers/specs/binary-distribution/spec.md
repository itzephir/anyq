# Spec Delta

## Purpose

Install anyq from published native binaries without a compiler or changes to user queue data.

## ADDED Requirements

### Requirement: Native releases
A release SHALL contain macOS arm64/x86_64, Linux arm64/x86_64, and Windows x86_64 binaries plus SHA-256 for each archive. An incomplete set SHALL NOT become the latest release.

#### Scenario: Version release
- **WHEN** all native builds for a version tag are successfully verified
- **THEN** GitHub Release publishes the full archive/checksum set

### Requirement: Verified installation
The installer SHALL detect OS/architecture, fetch latest or an explicit version, and verify SHA-256 and binary version before replacement. Download/verification failures or unsupported platforms SHALL preserve the installed binary. Settings/databases SHALL be retained.

#### Scenario: Corrupt download
- **WHEN** the downloaded archive does not match SHA-256
- **THEN** installation fails and the existing binary is preserved

#### Scenario: Windows without Unix shell
- **WHEN** a Windows user runs the PowerShell installer
- **THEN** anyq.exe installs without Bash, WSL, or Rust

### Requirement: System scope by default
The installer SHALL default to /usr/local/bin on macOS/Linux and Program Files/anyq on Windows. Explicit --user or -Scope User SHALL select ~/.local/bin or LOCALAPPDATA/Programs/anyq without elevation. An explicit directory SHALL override the mode's default.

#### Scenario: Install for everyone
- **WHEN** a user runs the installer without selecting a directory or mode
- **THEN** the binary is placed in the system directory and available to all users

#### Scenario: User mode
- **WHEN** --user or -Scope User is selected
- **THEN** the installer uses the user directory without requesting sudo/UAC

### Requirement: Limited elevation
The installer SHALL download/verify the binary before sudo/UAC. Elevated permissions SHALL be used only for placement/system PATH; the copy SHALL be verified before atomic replacement. Permission refusal or verification failure SHALL preserve the installed version.

#### Scenario: Elevation refused
- **WHEN** the user declines sudo/UAC
- **THEN** the installer reports an error, preserving the previous binary and PATH

### Requirement: Windows PATH scope
The Windows installer SHALL add the directory without duplicates to system PATH in System mode and user PATH in User mode. PATH-write failures SHALL roll back binary replacement. -NoPathUpdate SHALL disable PATH changes.

#### Scenario: Windows system installation
- **WHEN** System-mode installation succeeds
- **THEN** system PATH contains the directory and user PATH is preserved

#### Scenario: PATH error
- **WHEN** PATH writing fails after binary placement
- **THEN** the previous binary is restored
