## MODIFIED Requirements

### Requirement: Panel lifecycle
Closing the main window SHALL retain the panel entry while a usable system-panel integration exists. Closing a popover SHALL leave the desktop application usable. An explicit Quit SHALL terminate the desktop client without stopping the daemon or changing task ownership. Window closure without a usable tray SHALL NOT strand an inaccessible background client. Removing or reinstalling the panel SHALL preserve a panel-started owner's availability and identity.

#### Scenario: Close and reopen main window
- **WHEN** a user closes the main window while its system-panel entry is usable and then chooses Open Window
- **THEN** the existing desktop client opens its main window for the selected database

#### Scenario: Quit with running tasks
- **WHEN** the user quits the desktop application while tasks hold slots
- **THEN** the daemon continues serving clients and those tasks retain their slots

#### Scenario: Close restored window after host loss
- **WHEN** the system-panel host disappears and a user closes the restored focused main window while the host remains absent
- **THEN** the desktop client exits and the same daemon preserves the queue and owned slots

#### Scenario: Remove and reinstall a macOS or Windows panel
- **WHEN** a panel-started owner is serving an existing queue and the user quits, removes and reinstalls the panel
- **THEN** the same owner runs from a retained build outside the removable installation and serves queue requests throughout removal without changing task ownership or saved preferences
