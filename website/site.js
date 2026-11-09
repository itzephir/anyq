(() => {
  const themeButton = document.querySelector("#theme");
  const updateLabel = () =>
    themeButton.setAttribute(
      "aria-label",
      document.documentElement.dataset.theme === "dark"
        ? "Включить светлую тему"
        : "Включить тёмную тему",
    );
  themeButton.hidden = false;
  updateLabel();
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
    updateLabel();
  });

  if (navigator.clipboard?.writeText)
    document.querySelectorAll("[data-copy]").forEach((button) => {
      button.hidden = false;
      button.addEventListener("click", async () => {
        try {
          await navigator.clipboard.writeText(
            document.getElementById(button.dataset.copy).textContent,
          );
          button.textContent = "Скопировано";
        } catch {
          button.textContent = "Выделите команду";
        }
        setTimeout(() => {
          button.textContent = "Копировать";
        }, 2400);
      });
    });
})();
