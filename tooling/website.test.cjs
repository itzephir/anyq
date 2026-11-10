const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const root = path.resolve(__dirname, "..");
const html = fs.readFileSync(path.join(root, "website/index.html"), "utf8");
const source = fs.readFileSync(path.join(root, "website/site.js"), "utf8");

// Run the shipped script against its real HTML bindings with browser IO isolated.
function openPage({
  url = "https://anyq.example/",
  saved,
  languages = [],
  blocked = false,
  copyFails = false,
} = {}) {
  const nodes = [...html.matchAll(/<([a-z][\w-]*)(\s[^<>]*?)?>/gi)].map(
    (match) => {
      const attributes = Object.fromEntries(
        [...match[0].matchAll(/([\w:-]+)="([^"]*)"/g)].map((attribute) => [
          attribute[1],
          attribute[2],
        ]),
      );
      return {
        attributes,
        dataset: Object.fromEntries(
          Object.entries(attributes)
            .filter(([name]) => name.startsWith("data-"))
            .map(([name, value]) => [
              name
                .slice(5)
                .replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()),
              value,
            ]),
        ),
        hidden: match[0].includes(" hidden"),
        listeners: {},
        setAttribute(name, value) {
          this.attributes[name] = value;
        },
        getAttribute(name) {
          return this.attributes[name];
        },
        addEventListener(name, listener) {
          this.listeners[name] = listener;
        },
      };
    },
  );
  const byId = (id) => nodes.find((node) => node.attributes.id === id);
  for (const match of html.matchAll(/<code id="([^"]+)">([^<]*)<\/code>/g)) {
    byId(match[1]).textContent = match[2].replaceAll("&amp;", "&");
  }
  const events = [];
  const timers = [];
  const writes = [];
  const storage = new Map(saved ? [["anyq-language", saved]] : []);
  const location = new URL(url);
  const document = {
    documentElement: { dataset: { theme: "dark" }, lang: "en" },
    querySelector: (selector) => byId(selector.slice(1)),
    querySelectorAll: (selector) =>
      nodes.filter((node) =>
        Object.hasOwn(node.attributes, selector.slice(1, -1)),
      ),
    getElementById: byId,
    dispatchEvent: (event) => events.push(event),
  };
  vm.runInNewContext(source, {
    document,
    location,
    URL,
    URLSearchParams,
    navigator: {
      languages,
      language: languages[0],
      clipboard: {
        async writeText(text) {
          if (copyFails) throw Error("clipboard unavailable");
          writes.push(text);
        },
      },
    },
    localStorage: {
      getItem(key) {
        if (blocked) throw Error("storage unavailable");
        return storage.get(key);
      },
      setItem(key, value) {
        if (blocked) throw Error("storage unavailable");
        storage.set(key, value);
      },
    },
    history: {
      replaceState(_state, _title, next) {
        location.href = next.href;
      },
    },
    setTimeout: (callback) => timers.push(callback),
    CustomEvent: class {
      constructor(type, options) {
        this.type = type;
        this.detail = options.detail;
      }
    },
  });
  return { document, byId, nodes, events, timers, storage, location, writes };
}

for (const [name, options, expected] of [
  [
    "URL overrides saved and browser preferences",
    { url: "https://anyq.example/?lang=en", saved: "ru", languages: ["ru-RU"] },
    "en",
  ],
  [
    "Russian URL overrides saved English",
    { url: "https://anyq.example/?lang=ru", saved: "en" },
    "ru",
  ],
  [
    "saved choice overrides browser",
    { saved: "en", languages: ["ru-RU"] },
    "en",
  ],
  [
    "unsupported URL falls through to saved choice",
    { url: "https://anyq.example/?lang=fr", saved: "ru" },
    "ru",
  ],
  [
    "first supported regional browser preference",
    { languages: ["fr-FR", "RU-ru", "en-US"] },
    "ru",
  ],
  [
    "English regional browser preference",
    { languages: ["en-GB", "ru-RU"] },
    "en",
  ],
  [
    "invalid storage falls through",
    { saved: "unknown", languages: ["ru-RU"] },
    "ru",
  ],
  [
    "blocked storage uses browser",
    { blocked: true, languages: ["ru-RU"] },
    "ru",
  ],
  [
    "unsupported language falls back to English",
    { blocked: true, languages: ["fr", "__proto__"] },
    "en",
  ],
  ["missing preferences fall back to English", {}, "en"],
]) {
  test(name, () => {
    const page = openPage(options);
    assert.equal(page.document.documentElement.lang, expected);
    assert.equal(page.events.at(-1).type, "anyq:languagechange");
    assert.equal(page.events.at(-1).detail.language, expected);
    for (const node of page.nodes.filter((node) => node.dataset.i18n))
      assert.equal(typeof node.innerHTML, "string");
  });
}

test("switch persists choice and preserves URL, theme, and installer state", () => {
  const page = openPage({
    url: "https://anyq.example/?lang=en&theme=light&ref=docs#install",
  });
  page.document.documentElement.dataset.theme = "light";
  const installer = page.nodes.find(
    (node) => node.attributes.class === "installer",
  );
  installer.open = true;
  page.byId("language").listeners.click();

  assert.equal(page.storage.get("anyq-language"), "ru");
  assert.equal(page.location.searchParams.get("lang"), "ru");
  assert.equal(page.location.searchParams.get("theme"), "light");
  assert.equal(page.location.searchParams.get("ref"), "docs");
  assert.equal(page.location.hash, "#install");
  assert.equal(installer.open, true);
  assert.equal(page.document.documentElement.dataset.theme, "light");
  assert.match(page.byId("hero-title").innerHTML, /У каждой задачи/);
  assert.match(
    page.nodes.find((node) => node.dataset.i18n === "title").innerHTML,
    /своя очередь/,
  );
  assert.match(
    page.nodes.find((node) => node.dataset.i18nContent === "description")
      .attributes.content,
    /локальная очередь/,
  );
  assert.equal(
    page.byId("theme").attributes["aria-label"],
    "Включить тёмную тему",
  );
  assert.equal(page.events.at(-1).detail.language, "ru");
  assert.equal(
    openPage({ saved: page.storage.get("anyq-language") }).document
      .documentElement.lang,
    "ru",
  );

  page.byId("theme").listeners.click();
  assert.equal(page.document.documentElement.dataset.theme, "dark");
  assert.equal(
    page.byId("theme").attributes["aria-label"],
    "Включить светлую тему",
  );
  assert.equal(page.location.searchParams.get("lang"), "ru");
});

test("switch works without storage and rejects URL content as language", () => {
  const page = openPage({
    blocked: true,
    url: "https://anyq.example/?lang=%3Cscript%3E",
  });
  page.byId("language").listeners.click();
  assert.equal(page.document.documentElement.lang, "ru");
  assert.equal(page.location.searchParams.get("lang"), "ru");
  page.byId("language").listeners.click();
  assert.equal(page.document.documentElement.lang, "en");
  assert.equal(page.byId("language").textContent, "RU");
});

test("clipboard feedback follows current language and commands remain literal", async () => {
  const page = openPage();
  const copy = page.nodes.find((node) => node.dataset.copy === "unix-command");
  await copy.listeners.click();
  assert.equal(copy.textContent, "Copied");
  assert.equal(page.writes[0], page.byId("unix-command").textContent);
  assert.match(page.writes[0], /&& chmod/);

  page.byId("language").listeners.click();
  page.timers[0]();
  assert.equal(copy.textContent, "Копировать");
  await copy.listeners.click();
  assert.equal(copy.textContent, "Скопировано");
  assert.equal(page.writes[0], page.writes[1]);
});

test("clipboard failure uses selected language", async () => {
  const page = openPage({ languages: ["ru"], copyFails: true });
  const copy = page.nodes.find((node) => node.dataset.copy);
  await copy.listeners.click();
  assert.equal(copy.textContent, "Выделите команду");
});

test("static English page keeps both installation disclosures and documentation anchor", () => {
  assert.match(html, /<html lang="en"/);
  assert.doesNotMatch(html, /[А-Яа-яЁё]/);
  assert.match(html, /github\.com\/itzephir\/anyq#commands/);
  assert.equal([...html.matchAll(/<details class="installer"/g)].length, 2);
  assert.match(html, /id="unix-command">curl/);
  assert.match(html, /id="windows-command">curl\.exe/);
});
