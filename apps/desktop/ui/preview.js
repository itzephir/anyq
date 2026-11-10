export function previewState(mode = "live", theme = "dark", language = "en") {
  const topics = [
    { name: "cpu", capacity: 4, running: 2, queued: 1, autostart: null },
    { name: "repo", capacity: 1, running: 1, queued: 2, autostart: null },
    { name: "emulator", capacity: 1, running: 0, queued: 1, autostart: false },
    { name: "cloud", capacity: 2, running: 0, queued: 1, autostart: null },
  ];
  const tasks = [
    {
      sequence: 1,
      name: "build-ui",
      state: "running",
      topics: ["cpu", "repo"],
      autostart_default: true,
      stopped: false,
    },
    {
      sequence: 2,
      name: "lint-tests",
      state: "running",
      topics: ["cpu"],
      autostart_default: true,
      stopped: false,
    },
    {
      sequence: 3,
      name: "release-preview",
      state: "queued",
      topics: ["cpu", "repo"],
      autostart_default: true,
      stopped: false,
    },
    {
      sequence: 4,
      name: "manual-check",
      state: "queued",
      topics: ["emulator"],
      autostart_default: true,
      stopped: true,
    },
    {
      sequence: 5,
      name: "sync-assets",
      state: "queued",
      topics: ["cloud", "repo"],
      autostart_default: true,
      stopped: false,
    },
  ];

  return {
    generation: 1,
    status: mode === "stale" ? "stale" : mode === "error" ? "error" : "live",
    path: "/Users/ephir/.local/share/anyq/queue.db",
    source: "default",
    preferences: { database_override: null, theme, language },
    last_success:
      mode === "error"
        ? null
        : mode === "stale"
          ? Date.now() - 120000
          : Date.now(),
    snapshot:
      mode === "error"
        ? null
        : mode === "empty"
          ? { topics: [], tasks: [], running: 0, queued: 0 }
          : { topics, tasks, running: 2, queued: 3 },
    selected_topic: "cpu",
    show_settings: false,
  };
}
