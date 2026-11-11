import en from "./locales/en.js";
import ru from "./locales/ru.js";
import { previewState } from "./preview.js";
import icons from "./icons.js";

const query = new URLSearchParams(location.search);
const invoke = window.__TAURI__?.core?.invoke;
const previewAllowed =
  document.body.dataset.preview === "true" ||
  (location.protocol === "http:" &&
    ["localhost", "127.0.0.1"].includes(location.hostname));
const surface = query.get("surface") || document.body.dataset.surface || "main";
let state = invoke
  ? {
      status: "loading",
      preferences: { language: "en", theme: "system" },
      snapshot: null,
    }
  : previewAllowed
    ? previewState(
        query.get("state") || document.body.dataset.state,
        query.get("theme") || document.body.dataset.theme || "dark",
        query.get("lang") || document.body.dataset.language || "en",
      )
    : {
        status: "error",
        preferences: { language: "en", theme: "system" },
        snapshot: null,
      };
let selectedTopic = null;
let selectedTask = null;
let topicFilter = "";
let taskFilter = "";
let settingsOpen = surface === "settings";
let draft = null;
let settingsError = "";
let generation = null;
let strings = en;

const icon = (name) =>
  `<svg viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.queue}</svg>`;
const escape = (value) =>
  String(value ?? "").replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        char
      ],
  );
const name = (value) => (value === "" ? "``" : value);
const sourceLabel = () =>
  strings[
    {
      override: "sourceOverride",
      environment: "sourceEnvironment",
      configuration: "sourceConfiguration",
      default: "sourceDefault",
    }[state.source] || "sourceDefault"
  ];
const pill = (status) =>
  `<span class="pill ${status}">${status === "running" ? '<span class="dot"></span>' : ""}${strings[status]}</span>`;
const iconButton = (id, type, label) =>
  `<button class="icon" id="${id}" aria-label="${escape(label)}" title="${escape(label)}">${icon(type)}</button>`;
const status = () =>
  `<span class="status ${state.status}"><span class="dot"></span>${strings[state.status === "live" ? "connected" : state.status] || strings.error}</span>`;
const banner = () =>
  state.status === "stale"
    ? `<div class="banner">${status()}<span class="spacer"></span>${strings.staleHelp}</div>`
    : "";
const empty = (title, description, actions = "") =>
  `<div class="empty">${icon("queue")}<h2>${escape(title)}</h2><p>${escape(description)}</p>${actions}</div>`;

function topicButton(topic, index, compact = false) {
  if (compact)
    return `<button class="compact-topic" data-open-topic="${index}"><span class="mono">${escape(name(topic.name))}</span><span class="muted">${topic.running}/${topic.capacity} · ${topic.queued} ${strings.queued}</span></button>`;
  return `<button class="topic-item" aria-pressed="${topic.name === selectedTopic}" data-topic="${index}" data-focus="topic:${escape(topic.name)}">
    <span class="row"><span class="topic-name">${escape(name(topic.name))}</span><span class="spacer"></span><span class="mono muted">${topic.running}/${topic.capacity}</span></span>
    <span class="row"><span class="usage spacer"><span style="width:${Math.min(100, (topic.running / topic.capacity) * 100)}%"></span></span><span class="small muted">${topic.queued} ${strings.queued}</span></span></button>`;
}

function details(task) {
  return `<section class="detail" aria-label="${strings.task}"><h2>${strings.task}</h2><div class="detail-card">
    <div class="detail-title row"><span class="mono">${escape(name(task.name))}</span><span class="spacer"></span>${pill(task.state)}</div>
    <dl class="detail-grid"><div><dt>${strings.resources}</dt><dd class="resource-tags">${task.topics.map((topic) => `<span class="pill mono">${escape(name(topic))}</span>`).join("")}</dd></div><div><dt>${strings.registration}</dt><dd class="mono">#${task.sequence}</dd></div>
    <div><dt>${strings.autoDefault}</dt><dd>${task.autostart_default ? strings.yes : strings.no}</dd></div><div><dt>${strings.stopped}</dt><dd>${task.stopped ? strings.yes : strings.no}</dd></div></dl>
    <p class="note">${strings.admission}</p></div></section>`;
}

function topicContent(topic) {
  const tasks = state.snapshot.tasks.filter((task) =>
    task.topics.includes(topic.name),
  );
  const visible = tasks.filter((task) =>
    task.name.toLocaleLowerCase().includes(taskFilter.toLocaleLowerCase()),
  );
  const selected = tasks.find((task) => task.sequence === selectedTask);
  return `<header class="content-header"><div><h1 class="mono">${escape(name(topic.name))}</h1><span class="muted small">${strings.autoTopic}: ${topic.autostart === null ? strings.inherited : topic.autostart ? strings.yes : strings.no}</span></div>
    <div class="header-actions">${iconButton("refresh", "refresh", strings.refresh)}${iconButton("settings", "settings", strings.settings)}</div></header>
    ${banner()}<div class="overview"><div class="stat"><p class="stat-label">${strings.running}</p><strong>${topic.running}<span class="muted" style="font-size:18px"> / ${topic.capacity}</span></strong><span class="unit">${strings.slots}</span></div>
    <div class="stat"><p class="stat-label">${strings.queued}</p><strong>${topic.queued}</strong></div></div>
    <div class="queue-header row"><h2>${strings.tasks}</h2><span class="pill">${tasks.length}</span><input type="search" id="task-search" aria-label="${strings.searchTasks}" placeholder="${strings.searchTasks}" value="${escape(taskFilter)}"></div>
    ${tasks.length === 0 ? empty(strings.emptyQueue, strings.emptyQueueHelp) : `<div class="task-table">${visible.map((task) => `<button class="task-row" data-task="${task.sequence}" data-focus="task:${task.sequence}" aria-pressed="${task.sequence === selectedTask}"><span class="task-icon">${icon("task")}</span><span><span class="task-name">${escape(name(task.name))}</span><span class="task-sub">${escape(task.topics.map(name).join(" · "))}${task.stopped ? ` · ${strings.stopped}` : ""}</span></span>${pill(task.state)}</button>`).join("") || empty(strings.noMatches, "")}</div>`}
    ${selected ? details(selected) : ""}`;
}

function mainWindow() {
  const topics = state.snapshot?.topics || [];
  const topic = topics.find((topic) => topic.name === selectedTopic);
  const matches = topics
    .map((topic, index) => ({ topic, index }))
    .filter(({ topic }) =>
      topic.name.toLocaleLowerCase().includes(topicFilter.toLocaleLowerCase()),
    );
  const age = state.last_success
    ? new Date(state.last_success).toLocaleTimeString(
        document.documentElement.lang,
        { hour: "2-digit", minute: "2-digit" },
      )
    : "—";
  return `<main class="workspace"><aside class="sidebar"><div class="brand"><span class="brand-mark">${icon("logo")}</span><strong>anyq</strong><span class="small muted">${strings.topics} · ${state.snapshot ? topics.length : "—"}</span></div>
    <div class="side-label row"><h2>${strings.topics}</h2></div><div class="search-wrap"><input type="search" id="topic-search" placeholder="${strings.searchTopics}" aria-label="${strings.searchTopics}" value="${escape(topicFilter)}"></div>
    <div class="topic-list" role="group" aria-label="${strings.topics}">${matches.map(({ topic, index }) => topicButton(topic, index)).join("") || `<p class="muted small" style="padding:12px">${state.snapshot ? (topics.length ? strings.noMatches : strings.emptyTopics) : strings[state.status] || strings.error}</p>`}</div></aside>
    <section class="content">${topic ? topicContent(topic) : `<div class="row"><span class="spacer"></span>${iconButton("refresh", "refresh", strings.refresh)}${iconButton("settings", "settings", strings.settings)}</div>${empty(state.status === "error" ? strings.error : state.status === "loading" ? strings.loading : strings.emptyTopics, state.status === "error" ? strings[state.error_kind] || strings.errorHelp : strings.emptyTopicsHelp)}`}</section>
    <footer class="footer">${status()}<span class="source-path mono" title="${escape(state.path)}">${escape(state.path)}</span><span class="spacer"></span><span class="updated" id="updated-time">${strings.updated} ${age}</span></footer></main>`;
}

function compactWindow() {
  const snapshot = state.snapshot;
  return `<main class="compact"><header class="compact-header row"><span class="brand-mark">${icon("logo")}</span><strong>anyq</strong><span class="spacer"></span>${status()}</header>${banner()}
    ${
      snapshot
        ? `<div class="compact-counts"><span><strong>${snapshot.running}</strong>${strings.running}</span><span><strong>${snapshot.queued}</strong>${strings.queued}</span></div><div class="compact-topics">${
            snapshot.topics
              .slice(0, 4)
              .map((topic, index) => topicButton(topic, index, true))
              .join("") || empty(strings.emptyTopics, "")
          }</div>`
        : empty(
            strings[state.status] || strings.error,
            strings[state.error_kind] || strings.errorHelp,
          )
    }
    <footer class="compact-footer"><button class="primary" id="open-main">${strings.open}</button>${iconButton("settings", "settings", strings.settings)}${iconButton("refresh", "refresh", strings.refresh)}</footer></main>`;
}

function settingsWindow() {
  draft ??= { ...state.preferences };
  const options = (values, selected) =>
    values
      .map(
        ([value, label]) =>
          `<option value="${value}" ${selected === value ? "selected" : ""}>${label}</option>`,
      )
      .join("");
  return `<main class="settings-surface"><header class="settings-header row"><h1>${strings.settings}</h1><span class="spacer"></span>${iconButton("cancel", "close", strings.cancel)}</header>
    <section><h2 style="margin-bottom:16px">${strings.database}</h2><label class="check-row"><input id="automatic" type="checkbox" ${draft.database_override === null ? "checked" : ""}>${strings.automatic}</label><p class="help">${strings.automaticHelp}</p>
    <div class="field"><label for="database-path">${strings.override}</label><div class="file-row"><input id="database-path" class="mono" type="text" value="${escape(draft.database_override || "")}" placeholder="/path/to/queue.db" ${draft.database_override === null ? "disabled" : ""}><button id="choose" ${draft.database_override === null ? "disabled" : ""}>${strings.choose}</button></div><p class="help" style="margin-top:8px">${strings.fileHint}</p></div>
    <div class="source-card"><span class="small muted">${strings.effective} · ${sourceLabel()}</span><p class="mono">${escape(state.path)}</p></div></section>
    <section class="settings-section"><h2>${strings.appearance}</h2><div class="settings-controls"><div class="field"><label for="theme">${strings.theme}</label><select id="theme">${options(
      [
        ["system", strings.system],
        ["light", strings.light],
        ["dark", strings.dark],
      ],
      draft.theme,
    )}</select></div>
    <div class="field"><label for="language">${strings.language}</label><select id="language">${options(
      [
        ["en", "English"],
        ["ru", "Русский"],
      ],
      draft.language,
    )}</select></div></div></section>
    ${settingsError ? `<p class="settings-error" role="alert">${escape(settingsError)}</p>` : ""}<footer class="settings-footer"><button id="cancel-footer">${strings.cancel}</button><button class="primary" id="save">${strings.save}</button></footer></main>`;
}

async function command(command, args = {}) {
  if (invoke) return invoke(command, args);
  if (command === "show_main") {
    location.href = `index.html?theme=${state.preferences.theme}&lang=${state.preferences.language}${args.settings ? "&surface=settings" : ""}`;
  }
}

function render(preserveTopicScroll = true) {
  strings = state.preferences.language === "ru" ? ru : en;
  document.documentElement.lang =
    state.preferences.language === "ru" ? "ru" : "en";
  const theme = draft?.theme || state.preferences.theme;
  document.documentElement.dataset.theme =
    theme === "system"
      ? matchMedia("(prefers-color-scheme:dark)").matches
        ? "dark"
        : "light"
      : theme;
  const active = document.activeElement;
  const focusId = active?.id;
  const focusToken = active?.dataset.focus;
  const cursor =
    active instanceof HTMLInputElement ? active.selectionStart : null;
  const top = document.querySelector(".content")?.scrollTop || 0;
  const topics = document.querySelector(".topic-list");
  const topicLeft = preserveTopicScroll ? topics?.scrollLeft || 0 : 0;
  const topicTop = preserveTopicScroll ? topics?.scrollTop || 0 : 0;
  document.getElementById("app").innerHTML = settingsOpen
    ? settingsWindow()
    : surface === "compact"
      ? compactWindow()
      : mainWindow();
  if (top) document.querySelector(".content")?.scrollTo(0, top);
  document.querySelector(".topic-list")?.scrollTo(topicLeft, topicTop);
  if (focusId) {
    const next = document.getElementById(focusId);
    next?.focus({ preventScroll: true });
    if (
      cursor !== null &&
      next instanceof HTMLInputElement &&
      ["text", "search"].includes(next.type)
    )
      next.setSelectionRange(cursor, cursor);
  }
  if (focusToken) {
    [...document.querySelectorAll("[data-focus]")]
      .find((button) => button.dataset.focus === focusToken)
      ?.focus({ preventScroll: true });
  }
  bind();
}

function bind() {
  const on = (id, event, handler) =>
    document.getElementById(id)?.addEventListener(event, handler);
  on("topic-search", "input", (event) => {
    topicFilter = event.target.value;
    render(false);
  });
  on("task-search", "input", (event) => {
    taskFilter = event.target.value;
    render();
  });
  document.querySelectorAll("[data-topic]").forEach((button) =>
    button.addEventListener("click", () => {
      selectedTopic = state.snapshot.topics[Number(button.dataset.topic)].name;
      selectedTask = null;
      taskFilter = "";
      render();
    }),
  );
  document.querySelectorAll("[data-task]").forEach((button) =>
    button.addEventListener("click", () => {
      selectedTask = Number(button.dataset.task);
      render();
    }),
  );
  document.querySelectorAll("[data-open-topic]").forEach((button) =>
    button.addEventListener("click", () =>
      command("show_main", {
        topic: state.snapshot.topics[Number(button.dataset.openTopic)].name,
        settings: false,
      }),
    ),
  );
  on("open-main", "click", () => command("show_main", { settings: false }));
  on("settings", "click", () => {
    if (surface === "compact" && invoke)
      command("show_main", { settings: true });
    else {
      settingsOpen = true;
      draft = { ...state.preferences };
      render();
    }
  });
  const cancel = () => {
    settingsOpen = false;
    draft = null;
    settingsError = "";
    render();
  };
  on("cancel", "click", cancel);
  on("cancel-footer", "click", cancel);
  on("automatic", "change", (event) => {
    draft.database_override = event.target.checked ? null : state.path || "";
    render();
  });
  on("database-path", "input", (event) => {
    draft.database_override = event.target.value;
  });
  on("theme", "change", (event) => {
    draft.theme = event.target.value;
    render();
  });
  on("language", "change", (event) => {
    draft.language = event.target.value;
  });
  on("choose", "click", async () => {
    const path = await command("choose_database");
    if (path) {
      draft.database_override = path;
      render();
    }
  });
  on("save", "click", async () => {
    try {
      if (invoke)
        accept(await command("save_preferences", { preferences: draft }));
      else state.preferences = { ...draft };
      settingsOpen = false;
      draft = null;
      settingsError = "";
      render();
    } catch (error) {
      settingsError = strings.saveFailed;
      render();
    }
  });
  on("refresh", "click", () => command("refresh_panel"));
}

function accept(next) {
  if (next.revision !== undefined && next.revision < (state.revision ?? -1))
    return;
  const displayed = (value) =>
    JSON.stringify([
      value.generation,
      value.status,
      value.snapshot,
      value.preferences,
      value.path,
      value.source,
      value.error_kind,
    ]);
  if (displayed(next) === displayed(state)) {
    state = next;
    const updated = document.getElementById("updated-time");
    if (updated && next.last_success)
      updated.textContent = `${strings.updated} ${new Date(next.last_success).toLocaleTimeString(document.documentElement.lang, { hour: "2-digit", minute: "2-digit" })}`;
    return;
  }
  const changed = next.generation !== generation;
  if (changed) {
    selectedTopic = null;
    selectedTask = null;
    taskFilter = "";
    topicFilter = "";
  }
  generation = next.generation;
  state = next;
  const topics = state.snapshot?.topics || [];
  if (!topics.some((topic) => topic.name === selectedTopic))
    selectedTopic = topics[0]?.name ?? null;
  if (!state.snapshot?.tasks.some((task) => task.sequence === selectedTask))
    selectedTask = null;
  render(!changed);
}

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && surface === "compact" && invoke) {
    command("dismiss_compact");
  }
  if (event.key === "Escape" && settingsOpen) {
    settingsOpen = false;
    draft = null;
    render();
  }
});
matchMedia("(prefers-color-scheme:dark)").addEventListener("change", () =>
  render(),
);
if (invoke) {
  window.__TAURI__.event
    .listen("panel-state", (event) => accept(event.payload))
    .then(() => command("get_panel_state").then(accept));
  window.__TAURI__.event.listen("panel-navigation", ({ payload }) => {
    if (payload.topic !== null) {
      settingsOpen = false;
      draft = null;
      settingsError = "";
      selectedTopic = payload.topic;
      selectedTask = null;
      taskFilter = "";
      topicFilter = "";
    }
    if (payload.settings) {
      settingsOpen = true;
      draft = { ...state.preferences };
    }
    render(false);
  });
} else {
  selectedTopic = state.snapshot?.topics[0]?.name ?? null;
  selectedTask = state.snapshot?.tasks[0]?.sequence ?? null;
  render();
}
