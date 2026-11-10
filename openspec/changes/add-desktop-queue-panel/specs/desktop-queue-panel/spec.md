# Spec Delta

## Purpose

Let users observe one local anyq queue through a small desktop window and a
system-panel entry, with explicit database selection and visible freshness.

## ADDED Requirements

### Requirement: Topic browsing
The panel SHALL list every topic in the observed database, including empty topics, with its name, occupied slots, capacity, and queued-task count. Users SHALL be able to search topic names and select a topic. Selection and searching SHALL preserve queue state and resource ownership.

#### Scenario: Empty topic
- **WHEN** a topic has capacity two and no current tasks
- **THEN** it remains visible with zero occupied slots and zero queued tasks

#### Scenario: Find and select a topic
- **WHEN** a user searches for an existing topic and selects it
- **THEN** the panel shows that topic's current queue without changing any tasks or topics

### Requirement: Current task inspection
A selected topic SHALL show only queued and running tasks in registration order. Users SHALL be able to search current task names and inspect a task's canonical identity, state, complete topic membership, saved autostart default, and explicit stop status. Running SHALL mean acquired resource slots, without claiming external execution progress or success.

#### Scenario: Task requires several topics
- **WHEN** a user selects a running task linked to cpu and repo
- **THEN** details show both topics and acquired ownership without implying that an external process is alive

#### Scenario: Stopped predecessor
- **WHEN** a stopped queued task precedes another task
- **THEN** the queue preserves registration order and details identify its explicit stop status

#### Scenario: Task leaves current queue
- **WHEN** a selected task is completed by another client and a fresh snapshot arrives
- **THEN** it leaves the current queue and its previous details are cleared or explicitly identified as no longer current

### Requirement: Observation-only controls
The panel SHALL expose browsing, search, refresh, window navigation, and panel preferences. It SHALL NOT expose task/topic creation, linking, start, completion, capacity/autostart edits, queue pause, daemon stop, or execution controls. Opening, closing, searching, refreshing, and switching databases SHALL NOT mutate queue entities or release ownership.

#### Scenario: Inspect with autocreate enabled
- **WHEN** shared task.autocreate and topic.autocreate are enabled and the panel browses or refreshes
- **THEN** no missing task or topic is created and no queued task is admitted by observation

### Requirement: Main window and compact summary
The desktop application SHALL provide a main window for browsing and a compact system-panel summary where supported. The summary SHALL show connection state, distinct running/queued task totals, a bounded topic summary, and an action to open the main window. Opening a topic from the summary SHALL select the same topic in the main window.

#### Scenario: Open topic from summary
- **WHEN** a user selects cpu in the compact summary
- **THEN** the main window opens or becomes focused with cpu selected in the same database

#### Scenario: Summary has limited room
- **WHEN** more topics exist than the compact summary can display
- **THEN** it provides access to the complete main-window list and does not present the visible subset as all topics

### Requirement: Native system-panel integration
macOS SHALL provide a menu-bar icon opening a popover-style panel. Windows SHALL provide a notification-area icon opening a nearby compact flyout. Linux SHALL integrate with an available StatusNotifierItem/AppIndicator host using its supported menu or activation behavior. The main window SHALL remain launchable and usable when no usable tray host is available.

#### Scenario: macOS menu-bar access
- **WHEN** the user clicks the menu-bar icon while the main window is closed
- **THEN** the compact panel opens and offers access to the main window

#### Scenario: Windows tray access
- **WHEN** the user activates the notification-area icon
- **THEN** the compact flyout opens near that icon within the usable display area

#### Scenario: Linux host supports a menu
- **WHEN** the Linux desktop provides a compatible status-item host with menu activation
- **THEN** the icon offers a brief queue summary and a working action to open the main window through the host's menu

#### Scenario: Linux without a tray host
- **WHEN** a user launches the application in a desktop session with no usable tray host
- **THEN** the main window opens and supports the same queue inspection and settings

### Requirement: Panel lifecycle
Closing the main window SHALL retain the panel entry while a usable system-panel integration exists. Closing a popover SHALL leave the desktop application usable. An explicit Quit SHALL terminate the desktop client without stopping the daemon or changing task ownership. Window closure without a usable tray SHALL NOT strand an inaccessible background client.

#### Scenario: Close and reopen main window
- **WHEN** a user closes the main window while its system-panel entry is usable and then chooses Open Window
- **THEN** the existing desktop client opens its main window for the selected database

#### Scenario: Quit with running tasks
- **WHEN** the user quits the desktop application while tasks hold slots
- **THEN** the daemon continues serving clients and those tasks retain their slots

### Requirement: Database selection precedence
The panel SHALL observe one database at a time. It SHALL select the path in the order panel-local override, inherited ANYQ_DB, shared db.path, platform default. The local override SHALL act like CLI --db for the panel. Settings SHALL expose automatic selection, a file-path override, and the effective path and source. Resetting the override SHALL restore automatic selection.

#### Scenario: Every source is configured
- **WHEN** a panel override, ANYQ_DB, shared db.path, and platform default are available
- **THEN** the panel observes only its override and settings identify that source

#### Scenario: Automatic selection
- **WHEN** the panel override is absent and ANYQ_DB is available
- **THEN** the panel uses ANYQ_DB instead of shared db.path or the platform default

#### Scenario: Shared configuration and default fallback
- **WHEN** automatic selection runs without ANYQ_DB
- **THEN** the panel uses shared db.path if configured, otherwise the same platform default as CLI

#### Scenario: Reset panel override
- **WHEN** the user returns database settings to automatic selection
- **THEN** the effective path is resolved again from the remaining sources and the displayed source is updated

### Requirement: Panel-local preference persistence
The database override SHALL persist across desktop launches in panel-local preferences, independently of shared queue configuration. Changing or resetting it SHALL NOT edit shared db.path, environment variables, or CLI invocation behavior. Failed preference persistence SHALL preserve the prior saved selection and report an error.

#### Scenario: Restart after selecting a file
- **WHEN** the user saves a panel database override and restarts the desktop application
- **THEN** the same database is selected and the shared CLI configuration is unchanged

#### Scenario: Preference write fails
- **WHEN** saving a new override fails before persistence completes
- **THEN** the application reports the failure and retains its previous saved and active selection

### Requirement: Isolated database switching
Applying a database selection SHALL invalidate previous in-flight reads and cached selections before displaying the new queue. Late responses from a previous database SHALL NOT update either desktop surface. An unavailable, missing, or incompatible selected database SHALL produce an explicit error without creating it or silently selecting a different database.

#### Scenario: Late response after switching
- **WHEN** the user switches from database A to B before an observation of A returns
- **THEN** the response from A is discarded and no A data appears under B's identity

#### Scenario: Bad selected path
- **WHEN** a saved override references an absent or inaccessible database
- **THEN** the panel shows that path and a connection error without creating a database or falling back to the default

### Requirement: Refresh and visible freshness
The panel SHALL refresh automatically while running and allow an immediate refresh request. Both surfaces SHALL use the same accepted snapshot. Loading, empty, disconnected, stale, and error states SHALL be distinguishable. Failed refreshes SHALL mark retained data as stale and expose its last successful update time; a successful refresh SHALL clear the stale state.

#### Scenario: External completion
- **WHEN** another client completes a task and the next refresh succeeds
- **THEN** both surfaces show the newly committed task counts, queues, and slot usage together

#### Scenario: Owner becomes unavailable
- **WHEN** the current daemon cannot be reached after a successful refresh
- **THEN** retained values are visibly stale and the panel indicates the connection failure instead of showing an empty live queue

### Requirement: Localized accessible presentation
The panel SHALL support English and Russian, light and dark themes, keyboard navigation, visible focus, and accessible names for controls and status icons. Resizing to a small window SHALL retain access to topic search, queue contents, task details, and settings. Status SHALL be communicated with text as well as visual styling.

#### Scenario: Russian dark theme
- **WHEN** the panel is displayed in Russian with the dark theme
- **THEN** labels, empty/error states, and settings are localized and controls and queue states remain readable

#### Scenario: Keyboard in a small window
- **WHEN** a keyboard user narrows the main window
- **THEN** search, topic selection, task details, and settings remain reachable with a visible focus indicator

### Requirement: Native integration acceptance
Desktop UI changes SHALL have automated integration scenarios against the real native application, daemon and queue on Linux, macOS and Windows. The stable CI passed gate SHALL require these scenarios. Builds, controller tests, mocks and ad hoc screenshots SHALL NOT replace them.

#### Scenario: Native cross-platform acceptance
- **WHEN** CI validates a desktop implementation PR
- **THEN** each supported operating system runs real application/daemon integration scenarios
- **AND** the stable CI passed check requires those scenarios to succeed

### Requirement: Reviewed screenshot regression baseline
Desktop UI changes SHALL compare representative native screenshots with reviewed platform-specific PNG goldens tracked in the repository. Missing goldens, unexpected dimensions and visual regressions SHALL fail the required CI gate and retain actual/difference artifacts. Comparison jobs SHALL NOT automatically update or accept golden files.

#### Scenario: Representative platform coverage
- **WHEN** CI validates the desktop presentation
- **THEN** it compares window, compact, settings, localized/theme, narrow and connection-state screenshots on Linux, macOS and Windows

#### Scenario: Missing or changed golden
- **WHEN** a required screenshot has no reviewed golden or differs beyond the documented tolerance
- **THEN** CI fails and retains the actual image and visual difference for review without silently accepting a new baseline
