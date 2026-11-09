(() => {
  let theme = document.documentElement.dataset.previewTheme || new URLSearchParams(location.search).get('theme');
  try { theme ||= localStorage.getItem('anyq-theme'); } catch {}
  if (theme !== 'light' && theme !== 'dark') theme = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  document.documentElement.dataset.theme = theme;
})();
