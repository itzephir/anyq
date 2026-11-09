# CI и защита default branch

Полный набор проверок можно также явно запустить через workflow_dispatch для
выбранной ветки; ручной запуск не ослабляет итоговый CI passed.

Workflow [.github/workflows/ci.yml](../.github/workflows/ci.yml) запускается на PR и push в `main`. Изменения ветки открытого PR проверяются через событие synchronize; отдельного дублирующего запуска на push этой ветки нет. Старые запуски одного PR отменяются при новом commit.

| Job | Платформы | Содержание |
| --- | --- | --- |
| Lint | Только Linux | Rustfmt, Clippy --all-features с запретом предупреждений, Rust AST-линтер, Cargo/API boundary checker, Ruff, OpenSpec |
| Build & unit tests | Linux, macOS, Windows | Сборка workspace; unit tests без SQLite/file adapters и CLI integration suites |
| Integration tests | Linux, macOS, Windows | SQLite/file adapter suites; CLI aliases/имена/config/autocreate, конкуренция, demo; отдельный обязательный Native Ctrl+C integration tests step |
| Native distribution | macOS ARM64/Intel, Linux ARM64/x64, Windows x64 | Release-сборка, архив, установка latest/pin и отказ при ошибках загрузки/checksum |
| Website | Linux | Статический контейнер, HTTP, установщики, сохранение работающей версии при ошибке обновления |
| Deploy dev / prod website | Linux, после CI passed | PR своего репозитория обновляет dev.anyq.itzephir.com; push main обновляет anyq.itzephir.com. Отдельные environment keys и forced receiver; HTTPS revision проверяется после публикации |
| CI passed | Итог всех категорий | Проходит только при success Lint, обеих матриц сборки/тестов, distribution и website |

`CI passed` использует `always()` и проверяет конкретные результаты зависимостей. Failure, cancelled и skipped не дают успешного gate. Матрицы используют `fail-fast: false`, чтобы ошибка одной ОС не скрывала результаты остальных. Только этот стабильный итоговый статус связан с правилом branch protection; изменение имени требует одновременного изменения защиты.

Native suite запускается с integration-test-hooks и отправляет реальный SIGINT на Unix, CTRL_C_EVENT в отдельную консоль на Windows. Проверяются commits, промежуточные записи, цепочки, autocreate, config replace, реальное ожидание locks и заполненные stdout/stderr pipes; без skip/ignore. Workflow/file suites отклоняют неизвестные TOML-разделы и таблицы вместо значений без перезаписи файла. Обычный cargo test без feature не заменяет обязательный запуск native target. Таблица исходов — [INTERRUPTIONS.md](INTERRUPTIONS.md).

Default branch `main` защищена на GitHub. Точное тело настройки хранится в [.github/branch-protection.json](../.github/branch-protection.json):

- Изменения только через PR, включая администраторов.
- Обязателен `CI passed` от GitHub Actions app 15368, подтверждённого check runs этого репозитория.
- Ветка PR должна быть актуальна относительно `main` (`strict: true`).
- Force push и удаление запрещены.
- Требуются разрешённые обсуждения и линейная история; squash/rebase доступны.
- Обязательное число сторонних approvals — 0: самостоятельный автор может влить свой проверенный PR.

Защита применена через API и проверяется фактическим GET, а не наличием JSON в репозитории. Для повторного применения с административными правами:

```bash
gh api --method PUT repos/itzephir/anyq/branches/main/protection --input .github/branch-protection.json
gh api repos/itzephir/anyq/branches/main/protection
```

Если default branch меняется, правило нужно перенести на новую default branch и обновить push filter workflow. Workflow не содержит административных токенов и сам не изменяет свою защиту.
