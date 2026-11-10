(() => {
  const translations = {
    en: {
      description:
        "anyq — a local admission queue for builds, agents, and emulators. All the resources you need, at once. Rust, SQLite, no server.",
      title: "anyq — every task has its queue",
      socialDescription:
        "Coordinate local tasks. Acquire all required resources atomically and release them explicitly.",
      home: "anyq — home",
      navigation: "Main navigation",
      theme: "Change theme",
      skip: "Skip to content",
      how: "How it works",
      install: "Install",
      heroTitle: "Every task <br />has its <span>queue.</span>",
      lead: 'Builds, agents, and emulators share resources.<br class="desktop-break" /> anyq helps them take turns.',
      installAnyq: "Install anyq",
      documentation: "Documentation",
      resource: "RESOURCE",
      slots: "SLOTS",
      free: "free",
      queueCaption:
        "build-02 is waiting for both resources. <br /><span>It leaves the free build slot available.</span>",
      usageTitle: "Wait. <br />Work. Complete.",
      usageDescription: "anyq manages admission. <br />You run the work.",
      allCommands: "All commands",
      prepare: "# Set up shared resources",
      wait: "# Wait for admission to all topics",
      work: "# Run your work",
      release: "# Release all task resources",
      codeNote:
        "Use a unique name for each run. <br />All participants share one local database.",
      installTitle: "A few commands. <br />You’re in the queue.",
      copy: "Copy",
      unixInstall:
        "Installs to <code>/usr/local/bin</code>. The script requests sudo if needed.",
      windowsInstall:
        "Installs to <code>%ProgramFiles%\\anyq</code> for all users. Restart your terminal afterward.",
      source: "Source code",
      lightTheme: "Switch to light theme",
      darkTheme: "Switch to dark theme",
      switchLanguage: "Switch to Russian",
      copied: "Copied",
      copyFailed: "Select the command",
    },
    ru: {
      description:
        "anyq — локальная очередь допуска для сборок, агентов и эмуляторов. Все нужные ресурсы одновременно. Rust, SQLite, без сервера.",
      title: "anyq — у каждой задачи своя очередь",
      socialDescription:
        "Координируйте локальные задачи. Получайте все нужные ресурсы атомарно и освобождайте их явно.",
      home: "anyq — главная",
      navigation: "Основная навигация",
      theme: "Сменить тему",
      skip: "К содержимому",
      how: "Как работает",
      install: "Установка",
      heroTitle: "У каждой задачи <br />своя <span>очередь.</span>",
      lead: 'Сборки, агенты и эмуляторы делят ресурсы.<br class="desktop-break" /> anyq помогает им договориться.',
      installAnyq: "Установить anyq",
      documentation: "Документация",
      resource: "РЕСУРС",
      slots: "СЛОТЫ",
      free: "свободен",
      queueCaption:
        "build-02 ждёт оба ресурса. <br /><span>Свободный слот build пока не занимает.</span>",
      usageTitle: "Дождаться. <br />Сделать. Завершить.",
      usageDescription: "anyq управляет допуском. <br />Работу запускаете вы.",
      allCommands: "Все команды",
      prepare: "# Подготовьте общие ресурсы",
      wait: "# Дождитесь допуска ко всем топикам",
      work: "# Выполните свою работу",
      release: "# Освободите все ресурсы задачи",
      codeNote:
        "Задайте уникальное имя для каждого запуска. <br />Все участники используют одну локальную базу.",
      installTitle: "Пара команд. <br />И можно в очередь.",
      copy: "Копировать",
      unixInstall:
        "Установка в <code>/usr/local/bin</code>. При необходимости   скрипт запросит sudo.",
      windowsInstall:
        "Установка в <code>%ProgramFiles%\\anyq</code> для всех пользователей.   После установки перезапустите терминал.",
      source: "Исходный код",
      lightTheme: "Включить светлую тему",
      darkTheme: "Включить тёмную тему",
      switchLanguage: "Переключить на английский",
      copied: "Скопировано",
      copyFailed: "Выделите команду",
    },
  };

  const supportedLanguage = (value) => {
    const primary = String(value || "")
      .toLowerCase()
      .split("-")[0];
    return Object.hasOwn(translations, primary) ? primary : null;
  };

  const getLanguage = () => {
    const override = new URLSearchParams(location.search).get("lang");
    if (override === "en" || override === "ru") return override;
    try {
      const saved = localStorage.getItem("anyq-language");
      if (saved === "en" || saved === "ru") return saved;
    } catch {}
    const preferences = navigator.languages?.length
      ? navigator.languages
      : [navigator.language];
    return preferences.map(supportedLanguage).find(Boolean) || "en";
  };

  let language = getLanguage();
  const themeButton = document.querySelector("#theme");
  const languageButton = document.querySelector("#language");
  const message = (key) => translations[language][key];
  const updateThemeLabel = () =>
    themeButton.setAttribute(
      "aria-label",
      message(
        document.documentElement.dataset.theme === "dark"
          ? "lightTheme"
          : "darkTheme",
      ),
    );

  const renderLanguage = () => {
    document.documentElement.lang = language;
    document.querySelectorAll("[data-i18n]").forEach((element) => {
      element.innerHTML = message(element.dataset.i18n);
    });
    for (const attribute of ["content", "aria-label"]) {
      document
        .querySelectorAll(`[data-i18n-${attribute}]`)
        .forEach((element) => {
          element.setAttribute(
            attribute,
            message(element.getAttribute(`data-i18n-${attribute}`)),
          );
        });
    }
    document.querySelectorAll("[data-copy]").forEach((button) => {
      button.textContent = message("copy");
    });
    languageButton.textContent = language === "en" ? "RU" : "EN";
    languageButton.lang = language;
    languageButton.setAttribute("aria-label", message("switchLanguage"));
    updateThemeLabel();
    document.dispatchEvent(
      new CustomEvent("anyq:languagechange", { detail: { language } }),
    );
  };

  renderLanguage();
  languageButton.hidden = false;
  languageButton.addEventListener("click", () => {
    language = language === "en" ? "ru" : "en";
    try {
      localStorage.setItem("anyq-language", language);
    } catch {}
    const url = new URL(location.href);
    url.searchParams.set("lang", language);
    history.replaceState(null, "", url);
    renderLanguage();
  });

  themeButton.hidden = false;
  themeButton.addEventListener("click", () => {
    const theme =
      document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("anyq-theme", theme);
    } catch {}
    const url = new URL(location.href);
    url.searchParams.delete("theme");
    history.replaceState(null, "", url);
    updateThemeLabel();
  });

  if (navigator.clipboard?.writeText)
    document.querySelectorAll("[data-copy]").forEach((button) => {
      button.hidden = false;
      button.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(
            document.getElementById(button.dataset.copy).textContent,
          );
          button.textContent = message("copied");
        } catch {
          button.textContent = message("copyFailed");
        }
        setTimeout(() => {
          button.textContent = message("copy");
        }, 2400);
      });
    });
})();
